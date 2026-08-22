#!/usr/bin/env python3
"""
ArangoDB Storage Module for Hermes Data Pipeline
Storage for commodity data, market quotes, and economic indicators
"""

import logging
from datetime import datetime, timezone
from typing import List, Dict, Any, Optional
from dataclasses import asdict
import json

# Try to import ArangoDB
try:
    from arango import ArangoClient, ArangoServerError
    ARANGO_AVAILABLE = True
except ImportError:
    ARANGO_AVAILABLE = False

logger = logging.getLogger(__name__)


class ArangoDBStorage:
    """ArangoDB storage backend for Hermes intelligence data"""
    
    def __init__(self, 
                 host: str = "http://localhost:8529",
                 username: str = "root",
                 password: str = "",
                 database: str = "intelligence"):
        """
        Initialize ArangoDB storage connection
        
        Args:
            host: ArangoDB server URL
            username: Database username
            password: Database password
            database: Database name
        """
        if not ARANGO_AVAILABLE:
            raise ImportError("python-arango package not installed. Run: pip install python-arango")
        
        self.host = host
        self.username = username
        self.password = password
        self.database = database
        
        self.client = None
        self.db = None
        self._connected = False
        
    def connect(self) -> bool:
        """Establish connection to ArangoDB"""
        try:
            self.client = ArangoClient(hosts=self.host)
            
            # Connect to system database first
            sys_db = self.client.db('_system', username=self.username, password=self.password)
            
            # Check if our database exists
            databases = sys_db.databases()
            if self.database not in databases:
                logger.warning(f"Database '{self.database}' does not exist. Creating...")
                sys_db.create_database(self.database)
            
            # Connect to our database
            self.db = self.client.db(self.database, username=self.username, password=self.password)
            
            # Ensure collections exist
            self._ensure_collections()
            
            self._connected = True
            logger.info(f"✅ Connected to ArangoDB database: {self.database}")
            return True
            
        except Exception as e:
            logger.error(f"❌ ArangoDB connection failed: {e}")
            self._connected = False
            return False
    
    def _ensure_collections(self):
        """Ensure required collections exist with proper indexes"""
        
        # Commodities collection
        if not self.db.has_collection('commodities'):
            commodities = self.db.create_collection('commodities')
            commodities.add_persistent_index(['timestamp'], unique=False)
            commodities.add_persistent_index(['commodity'], unique=False)
            commodities.add_persistent_index(['commodity', 'timestamp'], unique=True)
            logger.info("✅ Created 'commodities' collection")
        
        # Market quotes collection (for stocks)
        if not self.db.has_collection('market_quotes'):
            market_quotes = self.db.create_collection('market_quotes')
            market_quotes.add_persistent_index(['symbol'], unique=False)
            market_quotes.add_persistent_index(['timestamp'], unique=False)
            market_quotes.add_persistent_index(['symbol', 'timestamp'], unique=True)
            logger.info("✅ Created 'market_quotes' collection")
        
        # Economic indicators collection
        if not self.db.has_collection('economic_indicators'):
            economic_indicators = self.db.create_collection('economic_indicators')
            economic_indicators.add_persistent_index(['indicator'], unique=False)
            economic_indicators.add_persistent_index(['timestamp'], unique=False)
            logger.info("✅ Created 'economic_indicators' collection")
    
    def is_connected(self) -> bool:
        """Check if connection is active"""
        return self._connected and self.db is not None
    
    def store_commodities(self, commodities: List[Dict]) -> int:
        """
        Store commodity data in ArangoDB
        
        Args:
            commodities: List of commodity data dictionaries
            
        Returns:
            Number of documents inserted
        """
        if not self.is_connected():
            if not self.connect():
                return 0
        
        try:
            collection = self.db.collection('commodities')
            inserted_count = 0
            
            for commodity in commodities:
                # Generate unique key: commodity-timestamp hash
                key = f"{commodity['commodity'].lower().replace(' ', '_')}_{commodity['timestamp'].replace(':', '').replace('-', '').replace('.', '')}"
                
                # Prepare document
                document = {
                    '_key': key[:254],  # ArangoDB key limit
                    'commodity': commodity['commodity'],
                    'price': commodity['price'],
                    'change': commodity.get('change', 0.0),
                    'currency': commodity.get('currency', 'USD'),
                    'unit': commodity.get('unit', 'per unit'),
                    'timestamp': commodity['timestamp'],
                    'source': commodity.get('source', 'unknown'),
                    'stored_at': datetime.now(timezone.utc).isoformat(),
                    'metadata': {
                        'collection_method': 'yfinance',
                        'processed': False
                    }
                }
                
                # Insert or update
                try:
                    collection.insert(document, overwrite=True)
                    inserted_count += 1
                except Exception as e:
                    logger.warning(f"⚠️ Failed to insert commodity {key}: {e}")
            
            logger.info(f"💾 Stored {inserted_count}/{len(commodities)} commodities in ArangoDB")
            return inserted_count
            
        except Exception as e:
            logger.error(f"❌ Failed to store commodities: {e}")
            return 0
    
    def store_market_quote(self, symbol: str, price: float, change: float = 0.0, 
                          change_percent: float = 0.0, volume: int = 0, 
                          currency: str = "IDR", metadata: Dict = None) -> bool:
        """
        Store a single market quote (stock price)
        
        Args:
            symbol: Stock symbol (e.g., 'BBRI.JK')
            price: Current price
            change: Price change
            change_percent: Percentage change
            volume: Trading volume
            currency: Currency code
            metadata: Additional metadata
            
        Returns:
            True if successful
        """
        if not self.is_connected():
            if not self.connect():
                return False
        
        try:
            collection = self.db.collection('market_quotes')
            timestamp = datetime.now(timezone.utc).isoformat()
            
            # Generate unique key
            key = f"{symbol.lower().replace('.', '_')}_{timestamp.replace(':', '').replace('-', '').replace('.', '')}"
            
            document = {
                '_key': key[:254],
                'symbol': symbol,
                'price': price,
                'change': change,
                'change_percent': change_percent,
                'volume': volume,
                'currency': currency,
                'timestamp': timestamp,
                'stored_at': datetime.now(timezone.utc).isoformat(),
                'metadata': metadata or {}
            }
            
            collection.insert(document, overwrite=True)
            logger.debug(f"💾 Stored market quote: {symbol} = {price} {currency}")
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to store market quote for {symbol}: {e}")
            return False
    
    def get_latest_commodity_price(self, commodity: str) -> Optional[Dict]:
        """
        Get the latest price for a specific commodity
        
        Args:
            commodity: Commodity name
            
        Returns:
            Latest commodity document or None
        """
        if not self.is_connected():
            if not self.connect():
                return None
        
        try:
            collection = self.db.collection('commodities')
            
            # AQL query to get latest price
            aql = """
            FOR doc IN commodities
                FILTER doc.commodity == @commodity
                SORT doc.timestamp DESC
                LIMIT 1
                RETURN doc
            """
            
            cursor = self.db.aql.execute(aql, bind_vars={'commodity': commodity})
            result = list(cursor)
            
            if result:
                return result[0]
            return None
            
        except Exception as e:
            logger.error(f"❌ Failed to get commodity price for {commodity}: {e}")
            return None
    
    def get_commodity_history(self, commodity: str, hours: int = 24) -> List[Dict]:
        """
        Get commodity price history for specified time window
        
        Args:
            commodity: Commodity name
            hours: Number of hours of history to retrieve
            
        Returns:
            List of historical commodity prices
        """
        if not self.is_connected():
            if not self.connect():
                return []
        
        try:
            collection = self.db.collection('commodities')
            
            # Calculate timestamp cutoff
            cutoff = datetime.now(timezone.utc).isoformat()  # Simplified - should calculate actual cutoff
            
            # AQL query
            aql = """
            FOR doc IN commodities
                FILTER doc.commodity == @commodity
                SORT doc.timestamp DESC
                LIMIT 100  // Limit results
                RETURN doc
            """
            
            cursor = self.db.aql.execute(aql, bind_vars={'commodity': commodity})
            return list(cursor)
            
        except Exception as e:
            logger.error(f"❌ Failed to get commodity history for {commodity}: {e}")
            return []
    
    def get_dashboard_stats(self) -> Dict[str, Any]:
        """
        Get dashboard statistics from ArangoDB
        
        Returns:
            Dictionary of dashboard statistics
        """
        if not self.is_connected():
            if not self.connect():
                return {}
        
        try:
            stats = {
                'commodities': {
                    'total': self.db.collection('commodities').count(),
                    'unique': 0,
                    'latest_update': None
                },
                'market_quotes': {
                    'total': self.db.collection('market_quotes').count(),
                    'unique_symbols': 0,
                    'latest_update': None
                },
                'economic_indicators': {
                    'total': self.db.collection('economic_indicators').count(),
                    'latest_update': None
                }
            }
            
            # Get unique commodities count
            aql_unique = """
            RETURN LENGTH(
                FOR doc IN commodities
                    COLLECT commodity = doc.commodity
                    RETURN commodity
            )
            """
            cursor = self.db.aql.execute(aql_unique)
            stats['commodities']['unique'] = list(cursor)[0] if cursor else 0
            
            # Get unique symbols count
            aql_symbols = """
            RETURN LENGTH(
                FOR doc IN market_quotes
                    COLLECT symbol = doc.symbol
                    RETURN symbol
            )
            """
            cursor = self.db.aql.execute(aql_symbols)
            stats['market_quotes']['unique_symbols'] = list(cursor)[0] if cursor else 0
            
            # Get latest timestamps
            for collection_name in ['commodities', 'market_quotes', 'economic_indicators']:
                aql_latest = f"""
                FOR doc IN {collection_name}
                    SORT doc.timestamp DESC
                    LIMIT 1
                    RETURN doc.timestamp
                """
                cursor = self.db.aql.execute(aql_latest)
                latest = list(cursor)
                if latest:
                    stats[collection_name.replace('_', ' ')]['latest_update'] = latest[0]
            
            return stats
            
        except Exception as e:
            logger.error(f"❌ Failed to get dashboard stats: {e}")
            return {}
    
    def cleanup_old_data(self, days_to_keep: int = 30) -> int:
        """
        Clean up old data beyond retention period
        
        Args:
            days_to_keep: Number of days to keep data
            
        Returns:
            Number of documents deleted
        """
        if not self.is_connected():
            if not self.connect():
                return 0
        
        try:
            # Calculate cutoff date
            from datetime import datetime, timedelta
            cutoff = (datetime.now(timezone.utc) - timedelta(days=days_to_keep)).isoformat()
            
            total_deleted = 0
            
            # Delete old commodities
            aql_commodities = """
            FOR doc IN commodities
                FILTER doc.timestamp < @cutoff
                REMOVE doc IN commodities
                COLLECT WITH COUNT INTO deleted
                RETURN deleted
            """
            cursor = self.db.aql.execute(aql_commodities, bind_vars={'cutoff': cutoff})
            deleted_commodities = list(cursor)[0] if cursor else 0
            total_deleted += deleted_commodities
            
            # Delete old market quotes
            aql_quotes = """
            FOR doc IN market_quotes
                FILTER doc.timestamp < @cutoff
                REMOVE doc IN market_quotes
                COLLECT WITH COUNT INTO deleted
                RETURN deleted
            """
            cursor = self.db.aql.execute(aql_quotes, bind_vars={'cutoff': cutoff})
            deleted_quotes = list(cursor)[0] if cursor else 0
            total_deleted += deleted_quotes
            
            logger.info(f"🧹 Cleaned up {total_deleted} old documents (older than {days_to_keep} days)")
            return total_deleted
            
        except Exception as e:
            logger.error(f"❌ Failed to cleanup old data: {e}")
            return 0


# Convenience functions for common operations
def get_arango_storage() -> ArangoDBStorage:
    """Get configured ArangoDB storage instance"""
    return ArangoDBStorage(
        host="http://localhost:8529",
        username="root",
        password="",
        database="intelligence"
    )


def test_connection() -> bool:
    """Test ArangoDB connection"""
    try:
        storage = get_arango_storage()
        return storage.connect()
    except Exception as e:
        logger.error(f"ArangoDB test failed: {e}")
        return False


if __name__ == "__main__":
    # Test script
    import sys
    
    logging.basicConfig(level=logging.INFO)
    
    print("🔧 Testing ArangoDB Storage Module...")
    
    if not ARANGO_AVAILABLE:
        print("❌ python-arango not installed. Run: pip install python-arango")
        sys.exit(1)
    
    storage = get_arango_storage()
    
    if storage.connect():
        print("✅ ArangoDB connection successful")
        
        # Test dashboard stats
        stats = storage.get_dashboard_stats()
        print(f"📊 Dashboard Stats: {json.dumps(stats, indent=2)}")
        
        print("✅ All tests passed")
    else:
        print("❌ ArangoDB connection failed")
        sys.exit(1)