#!/usr/bin/env python3
"""
Process Staged Data - Transfer from SQLite to ArangoDB

This script:
1. Reads pending records from SQLite staging
2. Transforms and validates data
3. Inserts into appropriate ArangoDB collections
4. Updates staging status

Usage:
    python3 scripts/process_staged_data.py [--limit N]
"""

import sqlite3
import json
import time
import logging
from datetime import datetime
from typing import Dict, List, Optional
from arangodb_client import ArangoDBClient, ArangoConfig

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class DataProcessor:
    """Process staged data and insert into ArangoDB"""
    
    def __init__(self, db_path: str = 'staging/staging.db', arango_config: Optional[ArangoConfig] = None):
        self.db_path = db_path
        self.arango = ArangoDBClient(arango_config or ArangoConfig())
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
    
    def process_economic_indicators(self, limit: int = 100) -> int:
        """Process economic indicators from staging"""
        logger.info("Processing economic indicators...")
        
        # Get pending records
        cursor = self.conn.execute('''
            SELECT * FROM raw_economic_indicators
            WHERE processed_at IS NULL
            ORDER BY fetched_at ASC
            LIMIT ?
        ''', (limit,))
        
        records = cursor.fetchall()
        logger.info(f"Found {len(records)} pending economic indicators")
        
        inserted = 0
        for row in records:
            try:
                # Convert row to dict for easier access
                record = dict(row)
                
                # Transform for ArangoDB
                doc = {
                    'indicator_name': record.get('indicator_name'),
                    'value': float(record.get('value')) if record.get('value') else None,
                    'unit': record.get('unit'),
                    'country': record.get('country'),
                    'source': record.get('source'),
                    'frequency': record.get('frequency'),
                    'fetched_at': record.get('fetched_at'),
                    'processed_at': datetime.utcnow().isoformat(),
                    'raw_json': json.loads(record.get('raw_json')) if record.get('raw_json') else None,
                }
                
                # Insert into ArangoDB
                doc_key = self.arango.insert_document('economic_indicators', doc)
                
                # Update staging status
                self.conn.execute('''
                    UPDATE raw_economic_indicators
                    SET processed_at = ?
                    WHERE id = ?
                ''', (datetime.utcnow().isoformat(), record.get('id')))
                
                inserted += 1
                logger.info(f"✅ Processed: {record.get('indicator_name')} = {record.get('value')} {record.get('unit')}")
                
            except Exception as e:
                logger.error(f"❌ Failed to process record: {e}")
                import traceback
                traceback.print_exc()
        
        self.conn.commit()
        return inserted
    
    def process_all(self, limit: int = 100) -> Dict[str, int]:
        """Process all pending data"""
        logger.info("=== PROCESSING ALL STAGED DATA ===")
        
        results = {}
        
        # Process economic indicators
        results['economic_indicators'] = self.process_economic_indicators(limit)
        
        # TODO: Add other processors as needed
        # results['finance'] = self.process_finance_data(limit)
        # results['news'] = self.process_news_data(limit)
        # results['commodity'] = self.process_commodity_data(limit)
        
        logger.info(f"\n=== PROCESSING COMPLETE ===")
        for collection, count in results.items():
            logger.info(f"{collection}: {count} records processed")
        
        return results
    
    def get_status(self) -> Dict[str, int]:
        """Get current staging status"""
        cursor = self.conn.execute('SELECT * FROM staging_status')
        
        status = {}
        for row in cursor.fetchall():
            # Row is a tuple: (table_name, total_records, pending_records, processed_records, error_records)
            table = row[0]
            status[table] = {
                'total': row[1] or 0,
                'pending': row[2] or 0,
                'processed': row[3] or 0,
                'errors': row[4] or 0,
            }
        
        return status
    
    def close(self):
        """Close database connection"""
        self.conn.close()


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Process staged data into ArangoDB')
    parser.add_argument('--db-path', default='staging/staging.db', help='Path to SQLite database')
    parser.add_argument('--limit', type=int, default=100, help='Max records to process per table')
    parser.add_argument('--status', action='store_true', help='Show staging status only')
    
    args = parser.parse_args()
    
    # Initialize processor
    processor = DataProcessor(db_path=args.db_path)
    
    try:
        # Show status if requested
        if args.status:
            status = processor.get_status()
            print("\n=== STAGING STATUS ===")
            for table, counts in status.items():
                print(f"\n{table}:")
                print(f"  Total: {counts['total']}")
                print(f"  Pending: {counts['pending']}")
                print(f"  Processed: {counts['processed']}")
                print(f"  Errors: {counts['errors']}")
            return
        
        # Process data
        results = processor.process_all(limit=args.limit)
        
        # Show final status
        status = processor.get_status()
        print("\n=== FINAL STATUS ===")
        for table, counts in status.items():
            print(f"{table}: {counts['pending']} pending, {counts['processed']} processed")
        
    finally:
        processor.close()


if __name__ == '__main__':
    main()
