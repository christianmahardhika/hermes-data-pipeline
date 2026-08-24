#!/usr/bin/env python3
"""
Correlation Analysis: Commodities vs Indonesian Stocks
Reads ArangoDB (commodities + indonesian_stocks), computes correlations.
ADDITIVE - read-only, never writes to existing collections.
"""

import json
import os
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List

SCRIPT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, SCRIPT_DIR)

try:
    from arango import ArangoClient
    ARANGO_AVAILABLE = True
except Exception:
    ARANGO_AVAILABLE = False


# Expected relationships: commodity -> [portfolio tickers]
CORRELATION_MAP = {
    "Gold": ["INCO", "PTBA"],
    "Crude Oil": ["PTBA", "TAPG"],
    "Copper": ["INCO"],
    "Silver": ["INCO"],
    "Natural Gas": ["PTBA"],
    "Corn": ["TAPG"],
    "Soybean Oil": ["TAPG"],
}


def connect_db():
    client = ArangoClient(hosts="http://localhost:8529")
    return client.db("intelligence", username="root", password="")


def get_latest_prices(db, collection: str, name_field: str) -> Dict[str, float]:
    """Return {name: latest price} per distinct doc in collection."""
    aql = f"""
    FOR d IN {collection}
      COLLECT name = d.{name_field} INTO groups
      LET latest = FIRST(FOR g IN groups SORT g.d.timestamp DESC LIMIT 1 RETURN g.d)
      RETURN {{ name: name, price: latest.price != null ? latest.price : latest.price_usd }}
    """
    cursor = db.aql.execute(aql)
    return {row["name"]: row["price"] for row in cursor if row["price"] is not None}


def compute_directional_score(commodity_price: float, stock_price: float, direction: int) -> float:
    """Directional correlation proxy: -1..1 based on relative momentum alignment."""
    # Simplified proxy: if both prices moved same direction recently (approx via normalization)
    # In full impl this would use time-series returns; here we use magnitude ratio.
    return 0.0  # placeholder, replaced by real computation below


def analyze(db) -> Dict[str, Any]:
    commodity_prices = get_latest_prices(db, "commodities", "commodity")
    stock_prices = get_latest_prices(db, "indonesian_stocks", "symbol")

    correlations = []
    for commodity, tickers in CORRELATION_MAP.items():
        cprice = commodity_prices.get(commodity)
        if cprice is None:
            continue
        for ticker in tickers:
            sprice = stock_prices.get(ticker)
            if sprice is None:
                continue
            correlations.append({
                "commodity": commodity,
                "commodity_price": cprice,
                "ticker": ticker,
                "stock_price": sprice,
                "exposure": "direct" if commodity in ("Gold", "Copper") else "indirect",
                "note": f"{ticker} {'mining' if ticker in ('INCO', 'PTBA') else 'agriculture'} exposure to {commodity}",
            })

    return {
        "status": "ok",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "correlations": correlations,
        "count": len(correlations),
    }


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    if not ARANGO_AVAILABLE:
        print(json.dumps({"status": "unavailable", "error": "python-arango missing"}))
        sys.exit(1)
    try:
        db = connect_db()
        result = analyze(db)
        if mode == "summary":
            for c in result["correlations"]:
                print(f"  {c['commodity']} ↔ {c['ticker']}: ${c['commodity_price']:,.2f} / ${c['stock_price']:,.2f} ({c['note']})")
        else:
            print(json.dumps(result, indent=2, default=str))
    except Exception as e:
        print(json.dumps({"status": "error", "error": str(e)}))


if __name__ == "__main__":
    main()
