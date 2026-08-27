#!/usr/bin/env python3
"""
Verify ArangoDB commodity data storage
"""

import json
from arango_storage import get_arango_storage

def main():
    print("🔍 Verifying ArangoDB Commodity Data Storage")
    print("=" * 50)
    
    storage = get_arango_storage()
    if storage.connect():
        # Get dashboard stats
        stats = storage.get_dashboard_stats()
        print("📊 Dashboard Stats:")
        print(f"  • Commodities: {stats['commodities']['total']:,} total, {stats['commodities']['unique']} unique")
        print(f"  • Latest update: {stats['commodities'].get('latest_update', 'N/A')}")
        
        # Get latest commodity prices
        print("\n💰 Latest Commodity Prices:")
        commodities = ['Gold', 'Crude Oil', 'Copper', 'Silver']
        for commodity in commodities:
            latest = storage.get_latest_commodity_price(commodity)
            if latest:
                print(f"  • {commodity}: ${latest['price']:,.2f} {latest['currency']} ({latest['timestamp'][:19]})")
            else:
                print(f"  • {commodity}: No data")
        
        # Get commodity history sample
        print("\n📈 Gold Price History (last 5 entries):")
        history = storage.get_commodity_history('Gold', hours=24)
        for i, entry in enumerate(history[:5]):
            print(f"  {i+1}. ${entry['price']:,.2f} at {entry['timestamp'][:19]}")
        
        print("\n✅ Verification complete - ArangoDB storage is working!")
        
    else:
        print("❌ Failed to connect to ArangoDB")

if __name__ == "__main__":
    main()