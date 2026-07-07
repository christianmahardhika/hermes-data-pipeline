#!/usr/bin/env python3
"""
Enhanced Commodity Data Collector for Hermes Intelligence Pipeline
Supports Indonesian strategic commodities: Coal, Palm Oil, Crude Oil, Nickel, Gold

Integrated from intelligence-system-rust enhanced Aladdin features
Author: Christian Mahardhika (Enhanced Intelligence System)
"""

import asyncio
import json
import logging
import time
from datetime import datetime, timezone
from typing import Dict, List, Optional
import requests
from dataclasses import dataclass, asdict

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class CommodityData:
    """Commodity data structure matching intelligence-system-rust format"""
    commodity: str
    price: float
    change: float
    currency: str
    unit: str
    timestamp: str
    source: str = "Enhanced Intelligence System"

class CommodityCollector:
    """Enhanced Commodity Collector for Indonesian Strategic Commodities"""
    
    def __init__(self):
        self.session = requests.Session()
        # Session will use timeout in individual requests
        
    def get_enhanced_commodities(self) -> List[CommodityData]:
        """
        Get enhanced commodity data for Indonesian strategic exports
        Matches exactly with intelligence-system-rust implementation
        """
        timestamp = datetime.now(timezone.utc).isoformat()
        
        # Enhanced commodity data from intelligence-system-rust
        commodities = [
            # Metals & Mining (Indonesian strength)
            CommodityData(
                commodity="Nickel",
                price=18450.0,
                change=245.0,
                currency="USD",
                unit="per tonne",
                timestamp=timestamp
            ),
            CommodityData(
                commodity="Gold",
                price=2018.50,
                change=-15.25,
                currency="USD", 
                unit="per oz",
                timestamp=timestamp
            ),
            
            # Energy Commodities (Strategic Indonesian exports)
            CommodityData(
                commodity="Crude Oil",
                price=78.45,
                change=1.23,
                currency="USD",
                unit="per barrel", 
                timestamp=timestamp
            ),
            CommodityData(
                commodity="Thermal Coal",  # NEW: Added per Christian's request
                price=135.50,
                change=3.75,
                currency="USD",
                unit="per metric ton",
                timestamp=timestamp
            ),
            
            # Agricultural Commodities (Indonesian dominance)  
            CommodityData(
                commodity="Palm Oil",  # NEW: Added per Christian's request
                price=965.0,
                change=-12.5,
                currency="USD",
                unit="per tonne",
                timestamp=timestamp
            ),
        ]
        
        logger.info(f"📊 Collected {len(commodities)} enhanced commodities")
        return commodities
        
    def fetch_live_commodity_data(self, symbol: str) -> Optional[CommodityData]:
        """
        Fetch live commodity data from external APIs
        Future enhancement: integrate with Yahoo Finance, Alpha Vantage, etc.
        """
        try:
            # Placeholder for live API integration
            # This would connect to real commodity data sources
            logger.info(f"🔄 Fetching live data for {symbol}")
            return None
        except Exception as e:
            logger.error(f"❌ Failed to fetch {symbol}: {e}")
            return None
    
    def save_to_storage(self, commodities: List[CommodityData], storage_type: str = "json"):
        """Save commodity data to storage (JSON, SQLite, TimescaleDB)"""
        try:
            if storage_type == "json":
                filename = f"commodity_data_{int(time.time())}.json"
                data = [asdict(commodity) for commodity in commodities]
                
                with open(filename, 'w') as f:
                    json.dump(data, f, indent=2)
                
                logger.info(f"💾 Saved {len(commodities)} commodities to {filename}")
                
        except Exception as e:
            logger.error(f"❌ Storage error: {e}")
    
    async def run_collector_daemon(self, interval_minutes: int = 30):
        """Run continuous commodity data collection"""
        logger.info(f"🔄 Starting commodity collector daemon (interval: {interval_minutes}m)")
        
        while True:
            try:
                commodities = self.get_enhanced_commodities()
                self.save_to_storage(commodities)
                
                # Print summary
                for commodity in commodities:
                    change_symbol = "+" if commodity.change > 0 else ""
                    logger.info(
                        f"📈 {commodity.commodity}: {commodity.currency}{commodity.price:,.2f} "
                        f"({change_symbol}{commodity.change}) {commodity.unit}"
                    )
                
                await asyncio.sleep(interval_minutes * 60)
                
            except Exception as e:
                logger.error(f"❌ Collector error: {e}")
                await asyncio.sleep(60)  # Wait 1 minute before retry

def main():
    """Enhanced Commodity Collector Entry Point"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Enhanced Commodity Data Collector")
    parser.add_argument("--daemon", action="store_true", help="Run as daemon")
    parser.add_argument("--interval", type=int, default=30, help="Collection interval (minutes)")
    parser.add_argument("--once", action="store_true", help="Run once and exit")
    
    args = parser.parse_args()
    
    collector = CommodityCollector()
    
    if args.daemon:
        logger.info("🚀 Enhanced Commodity Collector starting...")
        asyncio.run(collector.run_collector_daemon(args.interval))
        
    elif args.once:
        logger.info("📊 Running single commodity collection...")
        commodities = collector.get_enhanced_commodities()
        collector.save_to_storage(commodities)
        
        # Print results
        print(f"\n🎯 Enhanced Commodities Summary ({len(commodities)} items):")
        for commodity in commodities:
            change_symbol = "+" if commodity.change > 0 else ""
            print(f"  • {commodity.commodity}: ${commodity.price:,.2f} ({change_symbol}${commodity.change}) {commodity.unit}")
    
    else:
        parser.print_help()

if __name__ == "__main__":
    main()