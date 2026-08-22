#!/usr/bin/env python3
"""
Simple verification of ArangoDB storage
"""

import sys
import os

# Add current directory to path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

try:
    from arango_storage import get_arango_storage
    print("✅ arango_storage imported successfully")
    
    storage = get_arango_storage()
    if storage.connect():
        print("✅ Connected to ArangoDB")
        
        # Get simple stats
        stats = storage.get_dashboard_stats()
        print(f"📊 Commodities stored: {stats.get('commodities', {}).get('total', 0)}")
        print(f"📊 Unique commodities: {stats.get('commodities', {}).get('unique', 0)}")
        
        # Check if we have data
        commodities = ['Gold', 'Crude Oil']
        for commodity in commodities:
            latest = storage.get_latest_commodity_price(commodity)
            if latest:
                print(f"💰 {commodity}: ${latest.get('price', 0):,.2f}")
            else:
                print(f"⚠️  {commodity}: No data yet")
                
        print("\n✅ Verification complete - ArangoDB is working!")
    else:
        print("❌ Failed to connect to ArangoDB")
        
except ImportError as e:
    print(f"❌ Import error: {e}")
    print("Make sure python-arango is installed: pip install python-arango")
except Exception as e:
    print(f"❌ Error: {e}")