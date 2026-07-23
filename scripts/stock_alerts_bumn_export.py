#!/usr/bin/env python3
"""
Stock price alerts for BUMN Export policy beneficiaries and affected stocks.
Monitors PTBA, ANTM (potential winners) and ADRO, BYAN, INCO (affected).
Uses MCP tools for reliable IDX data.
"""

import json
import os
from datetime import datetime
import yfinance as yf

STATE_FILE = os.path.expanduser("~/.hermes/stock_alerts_state.json")

# Stocks to monitor with thresholds
MONITORED_STOCKS = {
    # Potential BUMN beneficiaries
    "PTBA": {
        "name": "Bukit Asam",
        "type": "BUMN_COAL",
        "thesis": "Potential export hub for coal exports",
        "buy_below": 2500,
        "sell_above": 3500,
        "watch_levels": [2200, 2400, 2600]
    },
    "ANTM": {
        "name": "Aneka Tambang",
        "type": "BUMN_MINING",
        "thesis": "Diversified BUMN, could be export hub for nickel",
        "buy_below": 1200,
        "sell_above": 1600,
        "watch_levels": [1000, 1100, 1300]
    },
    
    # Affected stocks (short candidates or avoid)
    "ADRO": {
        "name": "Adaro Energy",
        "type": "PRIVATE_COAL",
        "thesis": "Margin risk from BUMN export policy",
        "avoid_above": 2800,
        "watch_levels": [2500, 2700, 2900]
    },
    "BYAN": {
        "name": "Bayan Resources",
        "type": "PRIVATE_COAL",
        "thesis": "High margin compressed by BUMN policy",
        "avoid_above": 22000,
        "watch_levels": [18000, 20000, 24000]
    },
    "INCO": {
        "name": "Vale Indonesia",
        "type": "FOREIGN_NICKEL",
        "thesis": "Regulatory uncertainty - foreign company treatment",
        "avoid_above": 3500,
        "watch_levels": [3000, 3300, 3700]
    },
    "AALI": {
        "name": "Astra Agro Lestari",
        "type": "SAWIT",
        "thesis": "Margin pressure from BUMN export",
        "avoid_above": 22000,
        "watch_levels": [18000, 20000, 24000]
    }
}


def load_state():
    if os.path.exists(STATE_FILE):
        with open(STATE_FILE) as f:
            return json.load(f)
    return {"last_prices": {}, "alerts_sent": []}


def save_state(state):
    os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
    with open(STATE_FILE, "w") as f:
        json.dump(state, f, indent=2)


def get_stock_prices_via_yfinance(symbols):
    """Fetch current prices using yfinance."""
    prices = {}
    
    for symbol in symbols:
        try:
            ticker = symbol if symbol.endswith('.JK') else f"{symbol}.JK"
            stock = yf.Ticker(ticker)
            info = stock.info
            
            current_price = info.get("currentPrice")
            change_pct = info.get("regularMarketChangePercent", 0)
            volume = info.get("volume", 0)
            
            if current_price:
                prices[symbol] = {
                    "price": current_price,
                    "change_pct": change_pct,
                    "volume": volume
                }
        except Exception as e:
            print(f"⚠️ Error fetching {symbol}: {e}")
    
    return prices


def check_alerts(prices, state):
    """Check if any price thresholds are hit."""
    alerts = []
    
    for symbol, data in prices.items():
        if symbol not in MONITORED_STOCKS:
            continue
        
        config = MONITORED_STOCKS[symbol]
        price = data["price"]
        
        # Check watch levels
        for level in config.get("watch_levels", []):
            alert_key = f"{symbol}_{level}"
            
            if abs(price - level) / level < 0.02:  # Within 2% of level
                if alert_key not in state.get("alerts_sent", []):
                    alerts.append({
                        "symbol": symbol,
                        "name": config["name"],
                        "type": config["type"],
                        "price": price,
                        "level": level,
                        "thesis": config["thesis"],
                        "message": f"{config['name']} ({symbol}) mendekati level {level:,}. Harga: {price:,.0f}"
                    })
                    state["alerts_sent"].append(alert_key)
        
        # Check buy below (for BUMN)
        if config.get("buy_below") and price < config["buy_below"]:
            alerts.append({
                "symbol": symbol,
                "name": config["name"],
                "type": "BUY_SIGNAL",
                "price": price,
                "message": f"🟢 BUY ALERT: {config['name']} ({symbol}) di bawah target beli {config['buy_below']:,}. Harga: {price:,.0f}"
            })
        
        # Check avoid above (for affected stocks)
        if config.get("avoid_above") and price > config["avoid_above"]:
            alerts.append({
                "symbol": symbol,
                "name": config["name"],
                "type": "AVOID_SIGNAL",
                "price": price,
                "message": f"🔴 AVOID ALERT: {config['name']} ({symbol}) di atas level hindari {config['avoid_above']:,}. Harga: {price:,.0f}"
            })
    
    return alerts


def main():
    state = load_state()
    
    # Fetch prices using yfinance
    symbols = list(MONITORED_STOCKS.keys())
    prices = get_stock_prices_via_yfinance(symbols)
    
    if not prices:
        print("⚠️ No price data available")
        return
    
    # Check for alerts
    alerts = check_alerts(prices, state)
    
    # Update state
    state["last_prices"] = {k: v["price"] for k, v in prices.items()}
    state["last_check"] = datetime.now().isoformat()
    save_state(state)
    
    if alerts:
        digest = []
        digest.append("📊 **STOCK ALERT - BUMN Export Policy Impact**")
        digest.append(f"📅 {datetime.now().strftime('%d %b %Y %H:%M')}")
        digest.append("")
        
        for alert in alerts:
            digest.append(f"**{alert['name']}** ({alert['symbol']})")
            digest.append(f"{alert['message']}")
            if 'thesis' in alert:
                digest.append(f"💡 {alert['thesis']}")
            digest.append("")
        
        digest.append("---")
        digest.append("📌 **Monitoring:**")
        digest.append("• PTBA, ANTM: BUMN beneficiaries")
        digest.append("• ADRO, BYAN: Private coal margin risk")
        digest.append("• INCO: Foreign nickel regulatory risk")
        digest.append("• AALI: Sawit margin pressure")
        
        print("\n".join(digest))
    else:
        # Print current prices silently
        pass


if __name__ == "__main__":
    main()
