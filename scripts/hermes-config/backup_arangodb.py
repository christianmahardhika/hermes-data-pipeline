#!/usr/bin/env python3
"""
ArangoDB Backup Script - ADDITIVE
Exports all collections in the 'intelligence' database to timestamped JSON
+ creates a tar.gz archive. Never touches running services (read-only export).
Usage: python backup_arangodb.py [output_dir] [--keep N]  (default: ./backups, keep 7)
"""

import gzip
import json
import os
import shutil
import sys
import tarfile
from datetime import datetime, timezone

BACKUP_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "backups")
KEEP = 7


def export_database(db, out_dir: str):
    """Export every collection to gzipped JSONL."""
    collections = db.collections()
    exported = []
    for col in collections:
        name = col["name"]
        if name.startswith("_"):
            continue  # skip system collections
        docs = list(db.collection(name).all())
        if not docs:
            continue
        path = os.path.join(out_dir, f"{name}.json.gz")
        with gzip.open(path, "wt", encoding="utf-8") as f:
            for doc in docs:
                f.write(json.dumps(doc, default=str) + "\n")
        exported.append((name, len(docs)))
    return exported


def cleanup_old(backup_root: str, keep: int):
    """Remove oldest backup dirs beyond keep count."""
    if not os.path.isdir(backup_root):
        return
    dirs = sorted(
        d for d in os.listdir(backup_root)
        if os.path.isdir(os.path.join(backup_root, d))
    )
    for old in dirs[:-keep] if len(dirs) > keep else []:
        shutil.rmtree(os.path.join(backup_root, old))
        print(f"  🗑️  removed old backup: {old}")


def main():
    global BACKUP_DIR, KEEP
    if len(sys.argv) > 1:
        BACKUP_DIR = sys.argv[1]
    if len(sys.argv) > 2:
        KEEP = int(sys.argv[2])

    try:
        from arango import ArangoClient
    except ImportError:
        print("❌ python-arango not installed")
        sys.exit(1)

    ts = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    out_dir = os.path.join(BACKUP_DIR, ts)
    os.makedirs(out_dir, exist_ok=True)

    print(f"💾 Backing up ArangoDB intelligence DB → {out_dir}")

    client = ArangoClient(hosts="http://localhost:8529")
    db = client.db("intelligence", username="root", password="")

    exported = export_database(db, out_dir)
    if not exported:
        print("⚠️  No collections exported")
        sys.exit(1)

    # Manifest
    manifest = {
        "timestamp": ts,
        "collections": {name: count for name, count in exported},
        "total_docs": sum(c for _, c in exported),
    }
    with open(os.path.join(out_dir, "manifest.json"), "w") as f:
        json.dump(manifest, f, indent=2)

    # Archive
    archive_path = os.path.join(BACKUP_DIR, f"backup_{ts}.tar.gz")
    with tarfile.open(archive_path, "w:gz") as tar:
        tar.add(out_dir, arcname=ts)
    shutil.rmtree(out_dir)  # keep only the tarball

    print(f"  ✅ {len(exported)} collections, {manifest['total_docs']} docs")
    print(f"  📦 {archive_path} ({os.path.getsize(archive_path)/1024:.0f} KB)")
    cleanup_old(BACKUP_DIR, KEEP)
    print("✅ Backup complete")


if __name__ == "__main__":
    main()
