# IDX Analyst Output Archive

Automated output repository for IDX AI Analyst (Rust binary) — 5-persona debate system for portfolio stock analysis.

## Structure

```
idx-analyst/
├── YYYY-MM-DD/
│   ├── HH-MM-idx-analyst-portfolio.md  (full RTI Business format)
│   └── HH-MM-idx-analyst-portfolio.md
├── README.md (this file)
└── ...
```

## Output Format

Each file contains RTI Business-formatted analysis with:
- **Fundamentals**: P/E, P/BV, ROE, Dividend Yield, Target Price
- **5-Persona Debate**: Buffett, Graham, Lynch, Munger, Indonesia Value Guru
- **Final Signal**: STRONG_BUY / BUY / HOLD / SELL with consensus confidence
- **Risk Assessment**: 30% standard for IDX stocks

## Generation

**Command:**
```bash
cd ~/projects/hermes-data-pipeline/news-social-intelligence-data-pipeline && \
cargo run --release -- idx-analyst --portfolio --full
```

**Schedule:**
- ⏰ 09:00 WIB, weekdays (job: 1bd66ab761ff, 1f750203839e)
- ⏰ 15:00 WIB, weekdays (job: 1bd66ab761ff, 90a964acf8cd)

**Stocks Analyzed:**
KLBF, TLKM, BBRI, PTBA, BJTM, ADMF, TAPG, JPFA, TSPC, BMRI, ASII

## File Naming

- Date: `YYYY-MM-DD` (e.g., `2026-07-23`)
- Time: `HH-MM` in 24h format (e.g., `09-00` for 09:00 WIB)
- Format: RTI Business (full mode)
- Example: `2026-07-23/09-00-idx-analyst-portfolio.md`

## Archival & Tracking

- **Retention**: All files kept (no pruning)
- **Version Control**: Auto-committed + pushed to GitHub main branch daily
- **Delivery**: Outputs also sent to Telegram (Pagupon Fuk Chi Sia, thread 3)
- **Analysis Tracking**: Used to monitor signal accuracy + debate evolution over time

## Notes

- Output captures live market data (via Yahoo Finance, with mock fallback)
- Persona signals are deterministic given the same fundamentals
- Use historical files to backtest portfolio signals or audit decision quality
