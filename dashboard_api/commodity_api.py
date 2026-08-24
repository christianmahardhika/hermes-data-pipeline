#!/usr/bin/env python3
"""
Commodity Dashboard API - lightweight Flask/FastAPI bridge
Serves commodity data from ArangoDB to the Next.js dashboard.

Read-only additive endpoint. Does NOT touch existing endpoints.
"""

import json
import os
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

# Allow import of arango_storage from scripts/hermes-config
SCRIPT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "scripts", "hermes-config")
sys.path.insert(0, SCRIPT_DIR)

try:
    from arango_storage import get_arango_storage
    ARANGO_AVAILABLE = True
except Exception:
    ARANGO_AVAILABLE = False


def get_commodity_prices() -> Dict[str, Any]:
    """Return latest commodity prices from ArangoDB."""
    if not ARANGO_AVAILABLE:
        return {"status": "unavailable", "error": "arango_storage not importable", "data": []}

    storage = get_arango_storage()
    if not storage.connect():
        return {"status": "error", "error": "ArangoDB connection failed", "data": []}

    commodities = ["Gold", "Crude Oil", "Copper", "Silver", "Natural Gas", "Corn", "Soybean Oil"]
    out = []
    for name in commodities:
        latest = storage.get_latest_commodity_price(name)
        if latest:
            out.append({
                "commodity": name,
                "price": latest.get("price"),
                "change": latest.get("change"),
                "currency": latest.get("currency", "USD"),
                "unit": latest.get("unit", ""),
                "timestamp": latest.get("timestamp"),
            })
    return {
        "status": "ok",
        "count": len(out),
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "data": out,
    }


def get_commodity_stats() -> Dict[str, Any]:
    """Return dashboard stats from ArangoDB."""
    if not ARANGO_AVAILABLE:
        return {"status": "unavailable", "error": "arango_storage not importable"}

    storage = get_arango_storage()
    if not storage.connect():
        return {"status": "error", "error": "ArangoDB connection failed"}

    stats = storage.get_dashboard_stats()
    return {"status": "ok", "stats": stats}


def main():
    """CLI entrypoint for direct invocation / cron."""
    mode = sys.argv[1] if len(sys.argv) > 1 else "prices"
    if mode == "prices":
        print(json.dumps(get_commodity_prices(), indent=2, default=str))
    elif mode == "stats":
        print(json.dumps(get_commodity_stats(), indent=2, default=str))
    else:
        print(json.dumps({"error": f"unknown mode: {mode}"}))


if __name__ == "__main__":
    main()
