# Enhanced IDX AI Analyst with Debate Mechanism

## Overview

This is an **enhanced version** of the 5-persona IDX analyst that adds:

1. **Debate Mechanism** — Bull vs Bear researchers actively debate, instead of independent scoring
2. **Trader Execution** — Converts signals into concrete entry/stop/take-profit levels
3. **Risk Management** — Validates positions against portfolio constraints
4. **Decision Memory** — Logs all decisions and reflects on realized returns

## Architecture

```
idx_ai_analyst_enhanced.py (main orchestrator)
├── Step 1: Data Gathering (analyst_reports)
│   ├─ MarketAnalyst: technicals, price action
│   ├─ SentimentAnalyst: social sentiment
│   ├─ NewsAnalyst: company + macro news
│   └─ FundamentalsAnalyst: PER/PBV/ROE/DER/DY
│
├── Step 2: Debate (debate_result)
│   ├─ BullResearcher: builds bullish case
│   ├─ BearResearcher: builds bearish case
│   └─ Loop N rounds, calculate consensus signal
│
├── Step 3: Trader Execution (trader_proposal)
│   ├─ Action: BUY/HOLD/SELL
│   ├─ Entry price, stop-loss, take-profit
│   └─ Position sizing
│
├── Step 4: Risk Assessment (risk_assessment)
│   ├─ Check position limits
│   ├─ Validate liquidity, valuation, debt
│   └─ Approve/Flag position
│
└── Step 5: Memory Log (decision logged)
    ├─ Record to ~/.hermes/.../memory/trading_decisions.md
    └─ Reflect on realized returns on next run
```

## Usage

### Test with Mock Data
```bash
cd ~/projects/hermes-data-pipeline/enhanced-idx-analyst/
python idx_ai_analyst_enhanced.py --all --mock --debug
```

### Analyze Specific Stocks
```bash
python idx_ai_analyst_enhanced.py BMRI KLBF --mock
```

### Portfolio Only
```bash
python idx_ai_analyst_enhanced.py --portfolio --mock
```

### Real Data (requires Notion integration)
```bash
python idx_ai_analyst_enhanced.py --all
```

## Configuration

Edit `config.py` to adjust:

- **Portfolio/Watchlist**: PORTFOLIO_STOCKS, WATCHLIST_STOCKS
- **Screening Criteria**: CRITERIA (PER, PBV, ROE, DER, DY)
- **Personas**: 5-persona styles and focus areas
- **Debate**: max_rounds, consensus thresholds
- **Risk**: position limits, sector concentration, drawdown
- **Memory**: logging directory, reflection config
- **Execution**: entry offset, stop-loss %, position sizing

## Integration with Pagupon Cron

To run at 09:00 and 15:00 WIB:

```bash
# Create shell wrapper
cat > ~/.hermes/profiles/pagupon-finance/scripts/idx_ai_analyst_enhanced.sh << 'EOF'
#!/bin/bash
cd ~/projects/hermes-data-pipeline/enhanced-idx-analyst/
python idx_ai_analyst_enhanced.py --all --save
EOF

chmod +x ~/.hermes/profiles/pagupon-finance/scripts/idx_ai_analyst_enhanced.sh

# Update cron job
cronjob create \
  --name "idx-ai-personas-enhanced-daily" \
  --schedule "0 9,15 * * 1-5" \
  --deliver "origin" \
  --script "idx_ai_analyst_enhanced.sh"
```

## Modules

### config.py
- Stock lists, criteria, personas, thresholds

### modules/data_gathering.py
- Analyst classes: Market, Sentiment, News, Fundamentals
- Generates 4 analyst reports per stock

### modules/debate_engine.py
- Bull/Bear researcher classes
- DebateEngine orchestrates multi-round debate
- Generates final consensus signal

### modules/trader_executor.py
- TraderExecutor converts signal to TraderProposal
- Generates entry, stop-loss, take-profit, sizing

### modules/risk_manager.py
- RiskManager validates against portfolio constraints
- Checks position size, liquidity, valuation, debt
- Returns approval/rejection with risk score

### modules/memory_logger.py
- MemoryLogger persists decisions to markdown
- Reflects on realized returns vs predictions
- Maintains decision history for learning

## Example Output

```
📊 **Enhanced IDX Analyst** | 09:00 WIB
============================================================

📊 **BMRI — Enhanced Analysis**
**Signal:** BUY | **Confidence:** MEDIUM
⏰ 09:05 WIB

**Debate Summary (2 Rounds):**
Bull Argument: Valuation attractive, dividend solid, macro supportive...
Bear Rebuttal: Risk/reward not compelling, better opportunities elsewhere...

**Trader Proposal:**
Action: BUY
Entry: Rp 3,950
Stop: Rp 3,634
Size: 3.0%

**Risk Status:** ✅ Approved
Risk Score: 0.25/1.0

---
...
```

## Next Steps

1. ✅ Module structure complete
2. ⏳ Test with mock data
3. ⏳ Integrate with Notion portfolio
4. ⏳ Add to cron at 09:00 & 15:00 WIB
5. ⏳ Compare vs current 5-persona output
6. ⏳ Iterate based on Christian's feedback

## Known Limitations

- Mock analyst reports (real integration needs Notion + yfinance)
- Debate uses simplified logic (production would use LLM)
- Risk assessment is basic (production needs portfolio API)
- Memory logging is placeholder (production needs file I/O)

For production use, integrate:
- Notion API for stock data
- yfinance for market data
- LLM (Claude/GPT/Kiro) for realistic debate
- Real portfolio tracking API
