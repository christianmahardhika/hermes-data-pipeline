#!/usr/bin/env python3
"""
Lightweight script runner with SQLite log aggregation and DLQ.

Zero-daemon approach — just a wrapper that captures output and stores in SQLite.
Designed for Hermes cron integration.

Usage:
    script-runner run <script> [args...]     Run script with logging
    script-runner logs [script] [--failed]   View logs
    script-runner dlq                        List failed jobs in DLQ
    script-runner retry <run_id>             Retry a failed job
    script-runner retry --all                Retry all DLQ jobs
    script-runner tail <script>              Tail recent logs
    script-runner stats                      Show execution statistics
"""

import argparse
import json
import os
import re
import shlex
import sqlite3
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Optional


# === Configuration ===
DEFAULT_DB_PATH = Path.home() / ".hermes" / "script-logs.db"
MAX_LOG_LINES = 10000  # Per run, truncate if exceeded
TRANSIENT_ERROR_PATTERNS = [
    r"token.*expired",
    r"401.*unauthorized",
    r"connection.*refused",
    r"timeout",
    r"rate.?limit",
    r"503.*service.*unavailable",
    r"ECONNRESET",
    r"ETIMEDOUT",
]


class RunStatus(Enum):
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    DLQ = "dlq"  # Failed with transient error, retryable


@dataclass
class RunRecord:
    id: int
    script: str
    args: str
    started: datetime
    ended: Optional[datetime]
    exit_code: Optional[int]
    status: RunStatus
    error_type: Optional[str]
    retry_count: int
    parent_run_id: Optional[int]


def get_db(db_path: Path = DEFAULT_DB_PATH) -> sqlite3.Connection:
    """Get database connection, creating schema if needed."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            script TEXT NOT NULL,
            args TEXT DEFAULT '',
            started TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            ended TIMESTAMP,
            exit_code INTEGER,
            status TEXT DEFAULT 'running',
            error_type TEXT,
            retry_count INTEGER DEFAULT 0,
            parent_run_id INTEGER REFERENCES runs(id)
        );
        
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id INTEGER NOT NULL REFERENCES runs(id),
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            stream TEXT DEFAULT 'stdout',  -- stdout, stderr
            line TEXT NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_runs_script ON runs(script);
        CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status);
        CREATE INDEX IF NOT EXISTS idx_runs_started ON runs(started);
        CREATE INDEX IF NOT EXISTS idx_logs_run_id ON logs(run_id);
    """)
    conn.commit()
    return conn


def detect_transient_error(output: str) -> Optional[str]:
    """Check if output contains transient error patterns."""
    output_lower = output.lower()
    for pattern in TRANSIENT_ERROR_PATTERNS:
        if re.search(pattern, output_lower, re.IGNORECASE):
            return pattern
    return None


def run_script(
    script: str,
    args: list[str],
    db_path: Path = DEFAULT_DB_PATH,
    timeout: Optional[int] = None,
    workdir: Optional[Path] = None,
    existing_run_id: Optional[int] = None,
    passthrough: bool = True,
) -> int:
    """Run a script and capture output to SQLite.
    
    If passthrough=True (default), also prints stdout to terminal.
    This is important for Hermes cron no-agent jobs where stdout IS the delivery.
    """
    conn = get_db(db_path)
    
    # Resolve script path
    script_path = Path(script)
    if not script_path.is_absolute():
        # Check common locations
        candidates = [
            Path.cwd() / script,
            Path.home() / ".hermes" / "scripts" / script,
            Path.home() / "hermes-data-pipeline" / script,
        ]
        for candidate in candidates:
            if candidate.exists():
                script_path = candidate
                break
    
    # Determine interpreter
    suffix = script_path.suffix.lower()
    if suffix in (".sh", ".bash"):
        cmd = ["bash", str(script_path)] + args
    elif suffix == ".py":
        cmd = [sys.executable, str(script_path)] + args
    else:
        cmd = [str(script_path)] + args
    
    # Create or reuse run record
    if existing_run_id:
        run_id = existing_run_id
        conn.execute(
            "UPDATE runs SET started = CURRENT_TIMESTAMP, status = ? WHERE id = ?",
            (RunStatus.RUNNING.value, run_id)
        )
    else:
        cursor = conn.execute(
            "INSERT INTO runs (script, args, status) VALUES (?, ?, ?)",
            (script, json.dumps(args), RunStatus.RUNNING.value)
        )
        run_id = cursor.lastrowid
    conn.commit()
    
    # Execute with output capture
    all_output = []
    exit_code = None
    process = None
    
    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            cwd=workdir,
            bufsize=1,  # Line buffered
        )
        
        line_count = 0
        if process.stdout:
            for line in process.stdout:
                line = line.rstrip('\n')
                all_output.append(line)
                
                # Passthrough to stdout for Hermes cron integration
                if passthrough:
                    print(line)
                
                # Store log line (with truncation)
                if line_count < MAX_LOG_LINES:
                    conn.execute(
                        "INSERT INTO logs (run_id, stream, line) VALUES (?, 'stdout', ?)",
                        (run_id, line)
                    )
                    if line_count % 100 == 0:  # Batch commits
                        conn.commit()
                line_count += 1
        
        process.wait(timeout=timeout)
        exit_code = process.returncode
        
    except subprocess.TimeoutExpired:
        if process:
            process.kill()
        exit_code = -1
        conn.execute(
            "INSERT INTO logs (run_id, stream, line) VALUES (?, 'stderr', ?)",
            (run_id, f"[script-runner] Timeout after {timeout}s")
        )
    except Exception as e:
        exit_code = -1
        conn.execute(
            "INSERT INTO logs (run_id, stream, line) VALUES (?, 'stderr', ?)",
            (run_id, f"[script-runner] Error: {e}")
        )
    
    # Determine final status
    combined_output = "\n".join(all_output)
    error_type = None
    
    if exit_code == 0:
        status = RunStatus.SUCCESS
    else:
        error_type = detect_transient_error(combined_output)
        if error_type:
            status = RunStatus.DLQ  # Retryable
        else:
            status = RunStatus.FAILED
    
    # Update run record
    conn.execute(
        """UPDATE runs 
           SET ended = CURRENT_TIMESTAMP, exit_code = ?, status = ?, error_type = ?
           WHERE id = ?""",
        (exit_code, status.value, error_type, run_id)
    )
    conn.commit()
    conn.close()
    
    return exit_code


def list_logs(
    script: Optional[str] = None,
    failed_only: bool = False,
    limit: int = 20,
    db_path: Path = DEFAULT_DB_PATH,
) -> list[dict]:
    """List recent runs with summary."""
    conn = get_db(db_path)
    
    query = "SELECT * FROM runs WHERE 1=1"
    params = []
    
    if script:
        query += " AND script LIKE ?"
        params.append(f"%{script}%")
    
    if failed_only:
        query += " AND status IN ('failed', 'dlq')"
    
    query += " ORDER BY started DESC LIMIT ?"
    params.append(limit)
    
    rows = conn.execute(query, params).fetchall()
    conn.close()
    
    return [dict(row) for row in rows]


def get_run_logs(run_id: int, db_path: Path = DEFAULT_DB_PATH) -> list[str]:
    """Get all log lines for a run."""
    conn = get_db(db_path)
    rows = conn.execute(
        "SELECT timestamp, stream, line FROM logs WHERE run_id = ? ORDER BY id",
        (run_id,)
    ).fetchall()
    conn.close()
    return [f"[{row['timestamp']}] {row['line']}" for row in rows]


def list_dlq(db_path: Path = DEFAULT_DB_PATH) -> list[dict]:
    """List all jobs in DLQ (retryable failures)."""
    conn = get_db(db_path)
    rows = conn.execute(
        """SELECT * FROM runs 
           WHERE status = 'dlq' 
           ORDER BY started DESC"""
    ).fetchall()
    conn.close()
    return [dict(row) for row in rows]


def retry_run(
    run_id: int,
    db_path: Path = DEFAULT_DB_PATH,
) -> int:
    """Retry a failed/DLQ run."""
    conn = get_db(db_path)
    row = conn.execute("SELECT * FROM runs WHERE id = ?", (run_id,)).fetchone()
    
    if not row:
        print(f"Run {run_id} not found", file=sys.stderr)
        return 1
    
    script = row['script']
    args = json.loads(row['args']) if row['args'] else []
    retry_count = row['retry_count'] + 1
    
    # Update retry count
    conn.execute(
        "UPDATE runs SET retry_count = ? WHERE id = ?",
        (retry_count, run_id)
    )
    conn.commit()
    conn.close()
    
    # Run the script, reusing the same run_id
    return run_script(script, args, db_path, existing_run_id=run_id)


def retry_all_dlq(db_path: Path = DEFAULT_DB_PATH) -> dict:
    """Retry all DLQ jobs."""
    dlq_jobs = list_dlq(db_path)
    results = {"success": 0, "failed": 0}
    
    for job in dlq_jobs:
        exit_code = retry_run(job['id'], db_path)
        if exit_code == 0:
            results["success"] += 1
        else:
            results["failed"] += 1
    
    return results


def get_stats(db_path: Path = DEFAULT_DB_PATH) -> dict:
    """Get execution statistics."""
    conn = get_db(db_path)
    
    stats = {}
    
    # Overall counts
    row = conn.execute("""
        SELECT 
            COUNT(*) as total,
            SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success,
            SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed,
            SUM(CASE WHEN status = 'dlq' THEN 1 ELSE 0 END) as dlq
        FROM runs
    """).fetchone()
    stats['total'] = dict(row)
    
    # Per-script stats (last 7 days)
    rows = conn.execute("""
        SELECT 
            script,
            COUNT(*) as runs,
            SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success,
            AVG(CASE WHEN ended IS NOT NULL 
                THEN (julianday(ended) - julianday(started)) * 86400 
                ELSE NULL END) as avg_duration_sec
        FROM runs
        WHERE started > datetime('now', '-7 days')
        GROUP BY script
        ORDER BY runs DESC
        LIMIT 20
    """).fetchall()
    stats['by_script'] = [dict(row) for row in rows]
    
    conn.close()
    return stats


def format_duration(seconds: Optional[float]) -> str:
    """Format duration in human readable form."""
    if seconds is None:
        return "-"
    if seconds < 60:
        return f"{seconds:.1f}s"
    if seconds < 3600:
        return f"{seconds/60:.1f}m"
    return f"{seconds/3600:.1f}h"


def format_status(status: str) -> str:
    """Format status with emoji."""
    return {
        "running": "🔄 running",
        "success": "✅ success",
        "failed": "❌ failed",
        "dlq": "🔁 dlq (retryable)",
        "retried": "↩️ retried",
    }.get(status, status)


def main():
    parser = argparse.ArgumentParser(
        description="Lightweight script runner with SQLite logging and DLQ",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        "--db", type=Path, default=DEFAULT_DB_PATH,
        help=f"Database path (default: {DEFAULT_DB_PATH})"
    )
    
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    # run
    run_parser = subparsers.add_parser("run", help="Run a script")
    run_parser.add_argument("script", help="Script to run")
    run_parser.add_argument("args", nargs="*", help="Script arguments")
    run_parser.add_argument("--timeout", type=int, help="Timeout in seconds")
    run_parser.add_argument("--workdir", type=Path, help="Working directory")
    run_parser.add_argument("-q", "--quiet", action="store_true", 
                           help="Don't pass through stdout (log only)")
    
    # logs
    logs_parser = subparsers.add_parser("logs", help="View logs")
    logs_parser.add_argument("script", nargs="?", help="Filter by script name")
    logs_parser.add_argument("--failed", action="store_true", help="Show only failed runs")
    logs_parser.add_argument("-n", "--limit", type=int, default=20, help="Number of runs")
    logs_parser.add_argument("--run-id", type=int, help="Show logs for specific run")
    
    # dlq
    dlq_parser = subparsers.add_parser("dlq", help="List DLQ (retryable failures)")
    
    # retry
    retry_parser = subparsers.add_parser("retry", help="Retry a failed run")
    retry_parser.add_argument("run_id", nargs="?", type=int, help="Run ID to retry")
    retry_parser.add_argument("--all", action="store_true", help="Retry all DLQ jobs")
    
    # tail
    tail_parser = subparsers.add_parser("tail", help="Tail recent logs for a script")
    tail_parser.add_argument("script", help="Script name")
    tail_parser.add_argument("-n", "--lines", type=int, default=50, help="Number of lines")
    
    # stats
    stats_parser = subparsers.add_parser("stats", help="Show execution statistics")
    
    args = parser.parse_args()
    
    if args.command == "run":
        exit_code = run_script(
            args.script, args.args, args.db,
            timeout=args.timeout, workdir=args.workdir,
            passthrough=not args.quiet
        )
        sys.exit(exit_code)
    
    elif args.command == "logs":
        if args.run_id:
            logs = get_run_logs(args.run_id, args.db)
            for line in logs:
                print(line)
        else:
            runs = list_logs(args.script, args.failed, args.limit, args.db)
            print(f"{'ID':>6} {'Script':<30} {'Status':<18} {'Exit':>4} {'Started':<20}")
            print("-" * 85)
            for run in runs:
                print(f"{run['id']:>6} {run['script'][:30]:<30} {format_status(run['status']):<18} "
                      f"{run['exit_code'] or '-':>4} {run['started']:<20}")
    
    elif args.command == "dlq":
        jobs = list_dlq(args.db)
        if not jobs:
            print("DLQ is empty ✨")
        else:
            print(f"{'ID':>6} {'Script':<30} {'Error Type':<20} {'Retries':>7}")
            print("-" * 70)
            for job in jobs:
                print(f"{job['id']:>6} {job['script'][:30]:<30} "
                      f"{(job['error_type'] or '-')[:20]:<20} {job['retry_count']:>7}")
            print(f"\n{len(jobs)} jobs in DLQ. Use 'retry <id>' or 'retry --all'")
    
    elif args.command == "retry":
        if args.all:
            results = retry_all_dlq(args.db)
            print(f"Retried all DLQ jobs: {results['success']} success, {results['failed']} failed")
        elif args.run_id:
            exit_code = retry_run(args.run_id, args.db)
            sys.exit(exit_code)
        else:
            parser.error("Specify run_id or --all")
    
    elif args.command == "tail":
        runs = list_logs(args.script, limit=1, db_path=args.db)
        if not runs:
            print(f"No runs found for '{args.script}'")
            sys.exit(1)
        
        logs = get_run_logs(runs[0]['id'], args.db)
        for line in logs[-args.lines:]:
            print(line)
    
    elif args.command == "stats":
        stats = get_stats(args.db)
        total = stats['total']
        
        print("=== Overall Stats ===")
        print(f"Total runs: {total['total']}")
        print(f"  ✅ Success: {total['success']}")
        print(f"  ❌ Failed:  {total['failed']}")
        print(f"  🔁 DLQ:     {total['dlq']}")
        
        if stats['by_script']:
            print("\n=== Last 7 Days (by script) ===")
            print(f"{'Script':<35} {'Runs':>6} {'Success':>8} {'Avg Time':>10}")
            print("-" * 65)
            for s in stats['by_script']:
                success_rate = f"{s['success']}/{s['runs']}"
                print(f"{s['script'][:35]:<35} {s['runs']:>6} {success_rate:>8} "
                      f"{format_duration(s['avg_duration_sec']):>10}")


if __name__ == "__main__":
    main()
