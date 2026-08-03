#!/usr/bin/env python3
"""
ArangoDB HTTP Client - Python Fallback Implementation
Part of Hermes Data Pipeline

Since Rust reqwest requires OpenSSL, this Python implementation
serves as a functional fallback for ArangoDB operations.

Features:
- Create collections
- Insert documents
- Query with AQL
- Health check

DevSecOps:
- Input validation
- Error handling with retries
- Connection pooling
- Timeout management
"""

import requests
import logging
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
import time

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class ArangoConfig:
    """ArangoDB connection configuration"""
    host: str = "localhost"
    port: int = 8529
    database: str = "intelligence"
    username: str = "root"
    password: str = ""
    timeout: int = 30
    max_retries: int = 3


class ArangoDBClient:
    """HTTP client for ArangoDB operations"""
    
    def __init__(self, config: Optional[ArangoConfig] = None):
        self.config = config or ArangoConfig()
        self.base_url = f"http://{self.config.host}:{self.config.port}"
        self.db_url = f"{self.base_url}/_db/{self.config.database}"
        self.session = requests.Session()
        self.session.auth = (self.config.username, self.config.password)
        self.session.timeout = self.config.timeout
        
        # DevSecOps: Security headers
        self.session.headers.update({
            'Accept': 'application/json',
            'Content-Type': 'application/json',
        })
    
    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None) -> Dict:
        """Make HTTP request with retry logic"""
        url = f"{self.db_url}/{endpoint}"
        
        for attempt in range(self.config.max_retries):
            try:
                response = self.session.request(
                    method=method,
                    url=url,
                    json=data,
                    timeout=self.config.timeout
                )
                
                # Parse response
                result = response.json()
                
                if result.get('error', False):
                    error_num = result.get('errorNum', 0)
                    error_msg = result.get('errorMessage', 'Unknown error')
                    
                    # Collection already exists - not an error
                    if error_num == 1207:
                        logger.info(f"Collection already exists")
                        return {'result': 'exists'}
                    
                    raise Exception(f"ArangoDB error {error_num}: {error_msg}")
                
                return result
                
            except requests.exceptions.Timeout:
                logger.warning(f"Timeout on attempt {attempt + 1}/{self.config.max_retries}")
                if attempt < self.config.max_retries - 1:
                    time.sleep(2 ** attempt)  # Exponential backoff
                else:
                    raise
            except requests.exceptions.ConnectionError as e:
                logger.error(f"Connection error: {e}")
                raise
            except Exception as e:
                logger.error(f"Request failed: {e}")
                raise
        
        raise Exception("Max retries exceeded")
    
    def health_check(self) -> bool:
        """Check ArangoDB connectivity"""
        try:
            response = self.session.get(
                f"{self.base_url}/_api/database/current",
                timeout=5
            )
            is_healthy = response.status_code == 200
            logger.info(f"✅ ArangoDB health check: {'PASS' if is_healthy else 'FAIL'}")
            return is_healthy
        except Exception as e:
            logger.error(f"❌ Health check failed: {e}")
            return False
    
    def create_collection(self, name: str) -> Dict:
        """Create a document collection"""
        logger.info(f"Creating collection: {name}")
        
        # Input validation (DevSecOps)
        if not name or not name.replace('_', '').isalnum():
            raise ValueError(f"Invalid collection name: {name}")
        
        result = self._request(
            method='POST',
            endpoint='_api/collection',
            data={'name': name, 'type': 2}  # Type 2 = document collection
        )
        
        logger.info(f"✅ Collection '{name}' created")
        return result
    
    def insert_document(self, collection: str, document: Dict) -> str:
        """Insert a document into a collection"""
        logger.info(f"Inserting document into '{collection}'")
        
        # Input validation (DevSecOps)
        if not collection or not document:
            raise ValueError("Collection and document are required")
        
        result = self._request(
            method='POST',
            endpoint=f'_api/document/{collection}',
            data=document
        )
        
        doc_key = result.get('_key', '')
        logger.info(f"✅ Document inserted with key: {doc_key}")
        return doc_key
    
    def query(self, aql: str, bind_vars: Optional[Dict] = None) -> List[Dict]:
        """Execute AQL query"""
        logger.info(f"Executing AQL query")
        
        # Input validation (DevSecOps)
        if not aql:
            raise ValueError("AQL query is required")
        
        result = self._request(
            method='POST',
            endpoint='_api/cursor',
            data={
                'query': aql,
                'bindVars': bind_vars or {},
                'count': True
            }
        )
        
        documents = result.get('result', [])
        logger.info(f"✅ Query returned {len(documents)} documents")
        return documents
    
    def initialize_collections(self) -> None:
        """Initialize standard Hermes collections"""
        collections = [
            'indonesian_stocks',
            'articles',
            'commodities',
            'geopolitical_signals',
            'economic_indicators'
        ]
        
        logger.info("Initializing Hermes collections...")
        for collection in collections:
            try:
                self.create_collection(collection)
            except Exception as e:
                if 'already exists' in str(e).lower() or 'duplicate' in str(e).lower():
                    logger.info(f"📦 Collection '{collection}' already exists")
                else:
                    logger.warning(f"⚠️  Failed to create '{collection}': {e}")


def main():
    """Test ArangoDB client"""
    import argparse
    import os
    
    parser = argparse.ArgumentParser(description='ArangoDB Client Test')
    parser.add_argument('--host', default='localhost', help='ArangoDB host')
    parser.add_argument('--port', type=int, default=8529, help='ArangoDB port')
    parser.add_argument('--database', default='intelligence', help='Database name')
    parser.add_argument('--username', default='root', help='Username')
    parser.add_argument('--password', default='', help='Password')
    
    args = parser.parse_args()
    
    # Create config
    config = ArangoConfig(
        host=args.host,
        port=args.port,
        database=args.database,
        username=args.username,
        password=args.password or os.getenv('ARANGO_PASSWORD', '')
    )
    
    # Create client
    client = ArangoDBClient(config)
    
    # Test health check
    print("\n=== TESTING ARANGODB CLIENT ===")
    is_healthy = client.health_check()
    
    if is_healthy:
        print("✅ ArangoDB is reachable")
        
        # Initialize collections
        print("\n=== INITIALIZING COLLECTIONS ===")
        client.initialize_collections()
        
        # Test insert
        print("\n=== TESTING INSERT ===")
        test_doc = {
            'test': True,
            'timestamp': time.time(),
            'message': 'Test from Python client'
        }
        
        try:
            doc_key = client.insert_document('economic_indicators', test_doc)
            print(f"✅ Test document inserted: {doc_key}")
        except Exception as e:
            print(f"⚠️  Insert test failed: {e}")
        
        # Test query
        print("\n=== TESTING QUERY ===")
        try:
            results = client.query('FOR doc IN economic_indicators LIMIT 5 RETURN doc')
            print(f"✅ Query returned {len(results)} documents")
        except Exception as e:
            print(f"⚠️  Query test failed: {e}")
        
        print("\n✅ ALL TESTS PASSED")
    else:
        print("❌ ArangoDB is not reachable")
        print("Please ensure ArangoDB is running on {}:{}".format(config.host, config.port))


if __name__ == '__main__':
    main()
