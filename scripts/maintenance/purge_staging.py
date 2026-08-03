#!/usr/bin/env python3
"""
Hermes Data Pipeline - SQLite Staging Purge Script

Purges processed records older than TTL days.
Runs weekly via cron to reclaim disk space.

Usage:
    python purge_staging.py [--ttl DAYS] [--dry-run] [--vacuum]
"""

import sqlite3
import argparse
import logging
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class StagingPurger:
    """Manages SQLite staging database purge operations."""
    
    def __init__(self, db_path: str, ttl_days: int = 7):
        self.db_path = Path(db_path)
        self.ttl_days = ttl_days
        self.conn = sqlite3.connect(str(self.db_path))
        self.conn.row_factory = sqlite3.Row
        
    def get_cutoff_date(self) -> datetime:
        """Calculate cutoff date based on TTL."""
        return datetime.now() - timedelta(days=self.ttl_days)
    
    def count_records_to_purge(self) -> Dict[str, int]:
        """Count records eligible for purge by table."""
        cutoff = self.get_cutoff_date()
        cutoff_str = cutoff.isoformat()
        
        counts = {}
        tables = [
            'raw_finance_data',
            'raw_news_data',
            'raw_economic_indicators',
            'raw_commodity_prices'
        ]
        
        for table in tables:
            cursor = self.conn.execute(f"""
                SELECT COUNT(*) as count
                FROM {table}
                WHERE processed_at < ?
                AND status = 'processed'
            """, (cutoff_str,))
            counts[table] = cursor.fetchone()['count']
            
        return counts
    
    def purge_table(self, table: str, dry_run: bool = False) -> int:
        """Purge processed records from a single table."""
        cutoff = self.get_cutoff_date()
        cutoff_str = cutoff.isoformat()
        
        if dry_run:
            cursor = self.conn.execute(f"""
                SELECT COUNT(*) as count
                FROM {table}
                WHERE processed_at < ?
                AND status = 'processed'
            """, (cutoff_str,))
            count = cursor.fetchone()['count']
            logger.info(f"[DRY-RUN] Would purge {count} records from {table}")
            return count
        
        cursor = self.conn.execute(f"""
            DELETE FROM {table}
            WHERE processed_at < ?
            AND status = 'processed'
        """, (cutoff_str,))
        
        deleted = cursor.rowcount
        self.conn.commit()
        logger.info(f"✅ Purged {deleted} records from {table}")
        return deleted
    
    def purge_all(self, dry_run: bool = False) -> Dict[str, int]:
        """Purge all eligible records from all tables."""
        results = {}
        tables = [
            'raw_finance_data',
            'raw_news_data',
            'raw_economic_indicators',
            'raw_commodity_prices'
        ]
        
        for table in tables:
            results[table] = self.purge_table(table, dry_run)
            
        return results
    
    def vacuum(self):
        """Reclaim disk space by running VACUUM."""
        logger.info("🔄 Running VACUUM to reclaim disk space...")
        self.conn.execute("VACUUM")
        logger.info("✅ VACUUM complete")
    
    def get_db_size(self) -> int:
        """Get database file size in bytes."""
        return self.db_path.stat().st_size
    
    def close(self):
        """Close database connection."""
        self.conn.close()


def main():
    parser = argparse.ArgumentParser(
        description='Purge old processed records from staging database'
    )
    parser.add_argument(
        '--db-path',
        default='staging/staging.db',
        help='Path to SQLite staging database'
    )
    parser.add_argument(
        '--ttl',
        type=int,
        default=7,
        help='Days to retain processed records (default: 7)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show what would be purged without actually deleting'
    )
    parser.add_argument(
        '--vacuum',
        action='store_true',
        help='Run VACUUM after purge to reclaim disk space'
    )
    
    args = parser.parse_args()
    
    # Initialize purger
    purger = StagingPurger(args.db_path, args.ttl)
    
    try:
        # Show database size before
        size_before = purger.get_db_size()
        logger.info(f"📊 Database size before: {size_before / 1024 / 1024:.2f} MB")
        
        # Count records to purge
        counts = purger.count_records_to_purge()
        total = sum(counts.values())
        logger.info(f"📋 Found {total} records eligible for purge:")
        for table, count in counts.items():
            logger.info(f"   - {table}: {count} records")
        
        if total == 0:
            logger.info("✨ No records to purge")
            return
        
        # Purge records
        results = purger.purge_all(dry_run=args.dry_run)
        total_purged = sum(results.values())
        
        # Vacuum if requested
        if args.vacuum and not args.dry_run and total_purged > 0:
            purger.vacuum()
        
        # Show database size after
        if not args.dry_run:
            size_after = purger.get_db_size()
            size_saved = size_before - size_after
            logger.info(f"📊 Database size after: {size_after / 1024 / 1024:.2f} MB")
            logger.info(f"💾 Disk space reclaimed: {size_saved / 1024 / 1024:.2f} MB")
        
    finally:
        purger.close()


if __name__ == '__main__':
    main()
