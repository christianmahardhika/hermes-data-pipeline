# Market Data Pipeline

Collects and stores financial market data for analysis.

## Data Sources

### IDX (Indonesia Stock Exchange)
- Stock prices (IHSG, LQ45, individual stocks)
- Trading volume and market cap
- Corporate actions (dividends, stock splits)

### Forex
- USD/IDR exchange rate
- Major pairs (EUR/USD, GBP/USD, JPY/USD)

### Commodities
- Gold (XAU/USD)
- Oil (Brent, WTI)
- Coal, Palm Oil (CPO)

## Architecture

```
┌─────────────────┐
│  Data Sources   │
│  (Yahoo, IDX)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Collector     │  Scheduled fetch (market hours)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Processor     │  OHLCV normalization, indicators
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Storage      │  TimescaleDB / SQLite
└─────────────────┘
```

## Features (Planned)

- [ ] IDX stock price collector (yfinance)
- [ ] Forex rate collector
- [ ] Commodity price collector
- [ ] Technical indicators (MA, RSI, MACD)
- [ ] Price alerts
- [ ] Historical data backfill

## Requirements

- Python 3.10+
- yfinance
- pandas
- TimescaleDB or SQLite

## Usage

```bash
# Install deps
pip install -r requirements.txt

# Run collector
python collector.py --source idx --symbols BBRI.JK,BMRI.JK

# Run daemon
python collector.py daemon --interval 5m
```

## Environment Variables

```bash
# Optional: for premium data sources
ALPHA_VANTAGE_API_KEY=
POLYGON_API_KEY=
```
