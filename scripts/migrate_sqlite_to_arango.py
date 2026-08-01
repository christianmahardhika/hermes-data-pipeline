#!/usr/bin/env python3
"""
SQLite to ArangoDB Migration Script
Phase 4: Pipeline Re-Architecture

Migrates data from SQLite staging tables to ArangoDB collections:
- raw_feeds → articles (status: raw)  
- cleaned → articles (status: cleaned)
- labeled → articles (status: labeled)
- ingested → articles (status: ingested)
- feed_health → feed_health collection
- adapters → adapters collection

Usage:
    python scripts/migrate_sqlite_to_arango.py --sqlite-path news_staging.db --dry-run
    python scripts/migrate_sqlite_to_arango.py --sqlite-path news_staging.db --execute
"""

import sqlite3
import json
import sys
import argparse
import hashlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass

# ArangoDB client
try:
    from arango.client import ArangoClient
    from arango.database import StandardDatabase
    from arango.collection import StandardCollection
except ImportError:
    print("ERROR: python-arango not installed. Run: pip install python-arango")
    sys.exit(1)

@dataclass
class MigrationStats:
    """Track migration statistics"""
    total_articles: int = 0
    migrated_articles: int = 0
    skipped_duplicates: int = 0
    errors: int = 0
    feed_health_migrated: int = 0
    adapters_migrated: int = 0
    
    def print_summary(self):
        print(f"\n=== Migration Summary ===")
        print(f"Total articles processed: {self.total_articles}")
        print(f"Successfully migrated: {self.migrated_articles}")
        print(f"Skipped duplicates: {self.skipped_duplicates}")
        print(f"Errors: {self.errors}")
        print(f"Feed health records: {self.feed_health_migrated}")
        print(f"Adapters migrated: {self.adapters_migrated}")
        print(f"Success rate: {(self.migrated_articles/max(self.total_articles,1)*100):.1f}%")

class SQLiteToArangoMigrator:
    """Migrates SQLite staging data to ArangoDB production"""
    
    def __init__(self, sqlite_path: str, arango_url: str, arango_db: str, 
                 arango_user: str, arango_pass: str, dry_run: bool = False):
        self.sqlite_path = sqlite_path
        self.dry_run = dry_run
        self.stats = MigrationStats()
        
        # Connect to SQLite
        self.sqlite_conn = sqlite3.connect(sqlite_path)
        self.sqlite_conn.row_factory = sqlite3.Row
        
        # Connect to ArangoDB
        if not dry_run:
            self.arango_client = ArangoClient(hosts=arango_url)
            self.arango_db: StandardDatabase = self.arango_client.db(
                arango_db, username=arango_user, password=arango_pass
            )
            
            # Ensure collections exist
            self._ensure_collections()
        
        print(f"📚 SQLite source: {sqlite_path}")
        print(f"🗄️  ArangoDB target: {arango_url}/{arango_db}")
        print(f"🔍 Mode: {'DRY RUN' if dry_run else 'EXECUTE'}")
    
    def _ensure_collections(self):
        """Ensure ArangoDB collections exist"""
        collections = ['articles', 'feed_health', 'adapters', 'source_freshness']
        
        for collection_name in collections:
            if not self.arango_db.has_collection(collection_name):
                print(f"Creating collection: {collection_name}")
                self.arango_db.create_collection(collection_name)
    
    def _generate_article_key(self, content: str, url: str) -> str:
        """Generate consistent article _key using SHA256 hash"""
        # Use content + URL for deduplication
        combined = f"{content}{url}".encode('utf-8')
        return hashlib.sha256(combined).hexdigest()[:16]
    
    def _parse_datetime(self, dt_str: Optional[str]) -> Optional[str]:
        """Parse SQLite datetime string to ISO format"""
        if not dt_str:
            return None
        
        try:
            # Try RFC3339 first
            dt = datetime.fromisoformat(dt_str.replace('Z', '+00:00'))
            return dt.isoformat()
        except:
            try:
                # Try SQLite CURRENT_TIMESTAMP format
                dt = datetime.strptime(dt_str, '%Y-%m-%d %H:%M:%S')
                return dt.replace(tzinfo=timezone.utc).isoformat()
            except:
                print(f"⚠️  Could not parse datetime: {dt_str}")
                return datetime.now(timezone.utc).isoformat()
    
    def migrate_raw_feeds(self):
        """Migrate raw_feeds table to articles collection (status: raw)"""
        print("\n🔄 Migrating raw_feeds...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT id, feed_name, raw_content, content_type, fetched_at, 
                   status, retry_count 
            FROM raw_feeds 
            ORDER BY fetched_at
        """)
        
        for row in cursor:
            self.stats.total_articles += 1
            
            # Generate article document
            article_key = f"raw_{row['id']}"
            
            article = {
                '_key': article_key,
                'title': f"Raw feed from {row['feed_name']}",
                'content': row['raw_content'].decode('utf-8', errors='ignore') if row['raw_content'] else '',
                'url': f"feed://{row['feed_name']}/{row['id']}",
                'source': row['feed_name'],
                'status': 'raw',
                'created_at': self._parse_datetime(row['fetched_at']),
                'updated_at': self._parse_datetime(row['fetched_at']),
                'category': 'raw_feed',
                'sqlite_metadata': {
                    'original_id': row['id'],
                    'content_type': row['content_type'],
                    'retry_count': row['retry_count'],
                    'original_status': row['status']
                }
            }
            
            if not self.dry_run:
                try:
                    # Check if already exists
                    if self.arango_db.collection('articles').has(article_key):
                        self.stats.skipped_duplicates += 1
                        continue
                    
                    self.arango_db.collection('articles').insert(article)
                    self.stats.migrated_articles += 1
                    
                except Exception as e:
                    print(f"❌ Error migrating raw feed {row['id']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would migrate: {article_key} ({row['feed_name']})")
                self.stats.migrated_articles += 1
    
    def migrate_cleaned_articles(self):
        """Migrate cleaned table to articles collection (status: cleaned)"""
        print("\n🔄 Migrating cleaned articles...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT c.id, c.raw_id, c.title, c.content, c.published_at,
                   c.source, c.url, c.content_hash, c.cleaned_at,
                   r.feed_name
            FROM cleaned c
            LEFT JOIN raw_feeds r ON c.raw_id = r.id
            ORDER BY c.cleaned_at
        """)
        
        for row in cursor:
            self.stats.total_articles += 1
            
            # Use content hash as key for deduplication
            article_key = row['content_hash'] or self._generate_article_key(
                row['content'] or '', row['url'] or ''
            )
            
            article = {
                '_key': article_key,
                'title': row['title'] or '',
                'content': row['content'] or '',
                'url': row['url'] or '',
                'source': row['source'] or row['feed_name'] or 'unknown',
                'status': 'cleaned',
                'created_at': self._parse_datetime(row['published_at'] or row['cleaned_at']),
                'updated_at': self._parse_datetime(row['cleaned_at']),
                'category': 'news',
                'sqlite_metadata': {
                    'original_cleaned_id': row['id'],
                    'original_raw_id': row['raw_id'],
                    'content_hash': row['content_hash']
                }
            }
            
            if not self.dry_run:
                try:
                    # Use upsert to handle duplicates gracefully
                    self.arango_db.collection('articles').update(
                        {'_key': article_key}, article, upsert=True
                    )
                    self.stats.migrated_articles += 1
                    
                except Exception as e:
                    print(f"❌ Error migrating cleaned article {row['id']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would migrate: {article_key} ({row['title'][:50]}...)")
                self.stats.migrated_articles += 1
    
    def migrate_labeled_articles(self):
        """Migrate labeled table to articles collection (status: labeled)"""
        print("\n🔄 Migrating labeled articles...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT l.id, l.cleaned_id, l.sentiment, l.sentiment_score,
                   l.news_type, l.news_subtype, l.events, l.actors,
                   l.relations, l.context, l.pattern_match, l.investment_signal,
                   l.labeled_at, l.labeled_by,
                   c.content_hash, c.title, c.content, c.url, c.source
            FROM labeled l
            JOIN cleaned c ON l.cleaned_id = c.id
            ORDER BY l.labeled_at
        """)
        
        for row in cursor:
            # Use cleaned article's content hash as key
            article_key = row['content_hash'] or self._generate_article_key(
                row['content'] or '', row['url'] or ''
            )
            
            # Parse JSON fields safely
            def safe_json_parse(json_str):
                if not json_str:
                    return {}
                try:
                    return json.loads(json_str)
                except:
                    return {}
            
            # Update existing article with labels
            labels = {
                'prof_jiang_analysis': {
                    'sentiment': row['sentiment'],
                    'sentiment_score': float(row['sentiment_score'] or 0.0),
                    'news_type': row['news_type'],
                    'news_subtype': row['news_subtype'],
                    'events': safe_json_parse(row['events']),
                    'actors': safe_json_parse(row['actors']),
                    'relations': safe_json_parse(row['relations']),
                    'context': safe_json_parse(row['context']),
                    'pattern_match': safe_json_parse(row['pattern_match']),
                    'investment_signal': safe_json_parse(row['investment_signal']),
                    'labeled_at': self._parse_datetime(row['labeled_at']),
                    'labeled_by': row['labeled_by']
                }
            }
            
            update_doc = {
                'status': 'labeled',
                'labels': labels,
                'updated_at': self._parse_datetime(row['labeled_at'])
            }
            
            if not self.dry_run:
                try:
                    # Update existing article with labels
                    result = self.arango_db.collection('articles').update(
                        article_key, update_doc
                    )
                    if result['_id']:
                        self.stats.migrated_articles += 1
                    else:
                        print(f"⚠️  Article not found for labeling: {article_key}")
                        self.stats.errors += 1
                    
                except Exception as e:
                    print(f"❌ Error updating labeled article {row['id']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would label: {article_key} ({row['sentiment']})")
                self.stats.migrated_articles += 1
    
    def migrate_ingested_articles(self):
        """Update articles to status: ingested"""
        print("\n🔄 Migrating ingested tracking...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT i.id, i.labeled_id, i.qdrant_id, i.ingested_at,
                   c.content_hash
            FROM ingested i
            JOIN labeled l ON i.labeled_id = l.id
            JOIN cleaned c ON l.cleaned_id = c.id
            ORDER BY i.ingested_at
        """)
        
        for row in cursor:
            article_key = row['content_hash'] or f"labeled_{row['labeled_id']}"
            
            update_doc = {
                'status': 'ingested',
                'embedding': {'qdrant_id': row['qdrant_id']},
                'updated_at': self._parse_datetime(row['ingested_at'])
            }
            
            if not self.dry_run:
                try:
                    result = self.arango_db.collection('articles').update(
                        article_key, update_doc
                    )
                    if result['_id']:
                        self.stats.migrated_articles += 1
                    
                except Exception as e:
                    print(f"❌ Error updating ingested status {row['id']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would mark ingested: {article_key}")
                self.stats.migrated_articles += 1
    
    def migrate_feed_health(self):
        """Migrate feed_health table"""
        print("\n🔄 Migrating feed health...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT feed_name, last_success, consecutive_failures, last_error,
                   circuit_open_until, backoff_secs, category
            FROM feed_health
        """)
        
        for row in cursor:
            health_doc = {
                '_key': row['feed_name'].replace('/', '_').replace('.', '_'),
                'feed_name': row['feed_name'],
                'last_success': self._parse_datetime(row['last_success']),
                'consecutive_failures': row['consecutive_failures'] or 0,
                'last_error': row['last_error'],
                'circuit_state': {
                    'open_until': self._parse_datetime(row['circuit_open_until']),
                    'backoff_seconds': row['backoff_secs'] or 3600
                },
                'category': row['category'],
                'updated_at': datetime.now(timezone.utc).isoformat()
            }
            
            if not self.dry_run:
                try:
                    self.arango_db.collection('feed_health').upsert(health_doc)
                    self.stats.feed_health_migrated += 1
                    
                except Exception as e:
                    print(f"❌ Error migrating feed health {row['feed_name']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would migrate feed health: {row['feed_name']}")
                self.stats.feed_health_migrated += 1
    
    def migrate_adapters(self):
        """Migrate adapters table"""
        print("\n🔄 Migrating adapters...")
        
        cursor = self.sqlite_conn.execute("""
            SELECT feed_name, transform_rules, updated_at, updated_by
            FROM adapters
        """)
        
        for row in cursor:
            adapter_doc = {
                '_key': row['feed_name'].replace('/', '_').replace('.', '_'),
                'feed_name': row['feed_name'],
                'transform_rules': json.loads(row['transform_rules'] or '{}'),
                'updated_at': self._parse_datetime(row['updated_at']),
                'updated_by': row['updated_by']
            }
            
            if not self.dry_run:
                try:
                    self.arango_db.collection('adapters').upsert(adapter_doc)
                    self.stats.adapters_migrated += 1
                    
                except Exception as e:
                    print(f"❌ Error migrating adapter {row['feed_name']}: {e}")
                    self.stats.errors += 1
            else:
                print(f"  🔍 Would migrate adapter: {row['feed_name']}")
                self.stats.adapters_migrated += 1
    
    def verify_migration(self):
        """Verify migration completeness"""
        print("\n🔍 Verifying migration...")
        
        if self.dry_run:
            print("  ⚠️  Skipping verification (dry run mode)")
            return
        
        # Count articles by status
        status_counts = {}
        for status in ['raw', 'cleaned', 'labeled', 'ingested']:
            count = self.arango_db.aql.execute(
                'FOR article IN articles FILTER article.status == @status RETURN 1',
                bind_vars={'status': status}
            ).statistics()['scanned_full']
            status_counts[status] = count
        
        print(f"  📊 Articles in ArangoDB:")
        for status, count in status_counts.items():
            print(f"    {status}: {count}")
        
        # Verify no data loss
        sqlite_counts = {}
        sqlite_counts['raw'] = self.sqlite_conn.execute('SELECT COUNT(*) FROM raw_feeds').fetchone()[0]
        sqlite_counts['cleaned'] = self.sqlite_conn.execute('SELECT COUNT(*) FROM cleaned').fetchone()[0]
        sqlite_counts['labeled'] = self.sqlite_conn.execute('SELECT COUNT(*) FROM labeled').fetchone()[0]
        sqlite_counts['ingested'] = self.sqlite_conn.execute('SELECT COUNT(*) FROM ingested').fetchone()[0]
        
        print(f"  📊 Original SQLite counts:")
        for status, count in sqlite_counts.items():
            print(f"    {status}: {count}")
        
        # Check for data loss
        total_sqlite = sum(sqlite_counts.values())
        total_arango = sum(status_counts.values())
        
        if total_arango >= total_sqlite * 0.95:  # Allow 5% variance for deduplication
            print(f"  ✅ Migration verification PASSED")
        else:
            print(f"  ❌ Migration verification FAILED - possible data loss")
            print(f"     SQLite total: {total_sqlite}, ArangoDB total: {total_arango}")
    
    def run_migration(self):
        """Execute complete migration"""
        print(f"🚀 Starting SQLite → ArangoDB migration...")
        
        try:
            # Migrate in dependency order
            self.migrate_raw_feeds()
            self.migrate_cleaned_articles()
            self.migrate_labeled_articles()
            self.migrate_ingested_articles()
            self.migrate_feed_health()
            self.migrate_adapters()
            
            # Verify results
            self.verify_migration()
            
            # Print summary
            self.stats.print_summary()
            
            if self.stats.errors == 0:
                print(f"\n✅ Migration completed successfully!")
            else:
                print(f"\n⚠️  Migration completed with {self.stats.errors} errors")
            
        except Exception as e:
            print(f"\n❌ Migration failed: {e}")
            raise
        
        finally:
            self.sqlite_conn.close()

def main():
    parser = argparse.ArgumentParser(description='Migrate SQLite staging data to ArangoDB')
    parser.add_argument('--sqlite-path', required=True, help='Path to SQLite database')
    parser.add_argument('--arango-url', default='http://localhost:8529', help='ArangoDB URL')
    parser.add_argument('--arango-db', default='hermes', help='ArangoDB database name')
    parser.add_argument('--arango-user', default='root', help='ArangoDB username')
    parser.add_argument('--arango-pass', default='', help='ArangoDB password')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be migrated without executing')
    parser.add_argument('--execute', action='store_true', help='Execute migration (required if not dry-run)')
    
    args = parser.parse_args()
    
    # Safety check
    if not args.dry_run and not args.execute:
        print("ERROR: Must specify either --dry-run or --execute")
        sys.exit(1)
    
    # Check SQLite file exists
    if not Path(args.sqlite_path).exists():
        print(f"ERROR: SQLite file not found: {args.sqlite_path}")
        sys.exit(1)
    
    # Load ArangoDB credentials from environment if not provided
    import os
    arango_url = os.getenv('ARANGO_URL', args.arango_url)
    arango_db = os.getenv('ARANGO_DATABASE', args.arango_db)
    arango_user = os.getenv('ARANGO_USERNAME', args.arango_user)
    arango_pass = os.getenv('ARANGO_PASSWORD', args.arango_pass)
    
    # Execute migration
    migrator = SQLiteToArangoMigrator(
        sqlite_path=args.sqlite_path,
        arango_url=arango_url,
        arango_db=arango_db,
        arango_user=arango_user,
        arango_pass=arango_pass,
        dry_run=args.dry_run
    )
    
    migrator.run_migration()

if __name__ == '__main__':
    main()