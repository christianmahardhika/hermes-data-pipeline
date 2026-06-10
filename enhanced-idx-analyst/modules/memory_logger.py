"""
Memory Logger Module — Decision Log & Reflection
Logs all decisions and reflects on realized returns vs predictions
Implements TradingAgents decision memory concept
"""

import os
from datetime import datetime
from typing import Dict, Any, Optional
from pathlib import Path


class MemoryLogger:
    """Logs trading decisions and reflects on outcomes"""
    def __init__(self, memory_dir: str = "~/.hermes/profiles/pagupon-finance/memory"):
        self.memory_dir = Path(memory_dir).expanduser()
        self.memory_dir.mkdir(parents=True, exist_ok=True)
        self.log_file = self.memory_dir / "trading_decisions.md"
    
    def log_decision(self,
                    ticker: str,
                    trade_date: str,
                    signal: str,
                    confidence: str,
                    entry_price: Optional[float],
                    stop_loss: Optional[float],
                    take_profit: Optional[float],
                    position_size_pct: Optional[float],
                    debate_summary: str,
                    trader_reasoning: str) -> None:
        """Log a trading decision"""
        
        timestamp = datetime.now().isoformat()
        
        entry = f"""
## {ticker} — {trade_date}
**Signal:** {signal} | **Confidence:** {confidence}
**Timestamp:** {timestamp}

### Debate Summary
{debate_summary}

### Trader Proposal
Entry: {f"Rp {entry_price:,.0f}" if entry_price else "N/A"}
Stop Loss: {f"Rp {stop_loss:,.0f}" if stop_loss else "N/A"}
Take Profit: {f"Rp {take_profit:,.0f}" if take_profit else "N/A"}
Position Size: {f"{position_size_pct*100:.1f}%" if position_size_pct else "N/A"}

Reasoning:
{trader_reasoning}

### Status
**Decision Status:** PENDING (awaiting return data)
**Raw Return:** TBD
**Alpha Return:** TBD
**Holding Days:** 5
**Reflection:** Pending

---
"""
        
        # Append to log
        with open(self.log_file, "a") as f:
            f.write(entry)
    
    def reflect_on_decision(self,
                           ticker: str,
                           trade_date: str,
                           raw_return: float,
                           alpha_return: float,
                           holding_days: int,
                           benchmark: str = "SPY") -> str:
        """Generate reflection on realized decision"""
        
        # Generate reflection text
        if raw_return > 0.05:
            sentiment = "✅ **Positive outcome**"
            assessment = "Trade thesis played out as expected."
        elif raw_return > 0:
            sentiment = "⚠️ **Marginal gain**"
            assessment = "Trade worked but with lower-than-expected return."
        elif raw_return > -0.05:
            sentiment = "⚠️ **Small loss**"
            assessment = "Trade had small adverse move but stayed within risk tolerance."
        else:
            sentiment = "❌ **Loss realized**"
            assessment = "Stop-loss or thesis break triggered."
        
        alpha_assess = f"Alpha vs {benchmark}: {alpha_return:+.2%}"
        
        reflection = f"""
{sentiment}

Raw Return: {raw_return:+.2%} over {holding_days} days
{alpha_assess}

{assessment}

Key Lessons:
- Debate thesis validity: confirmed with positive alpha
- Position sizing: appropriate for outcome
- Entry/exit levels: worked as planned

---
"""
        
        return reflection.strip()
    
    def update_decision_with_reflection(self,
                                       ticker: str,
                                       trade_date: str,
                                       raw_return: float,
                                       alpha_return: float,
                                       holding_days: int,
                                       benchmark: str = "SPY") -> None:
        """Update a pending decision with reflection"""
        
        reflection = self.reflect_on_decision(ticker, trade_date, raw_return, alpha_return, holding_days, benchmark)
        
        # Read entire file
        if self.log_file.exists():
            with open(self.log_file, "r") as f:
                content = f.read()
        else:
            content = ""
        
        # Find and update the decision entry
        marker = f"## {ticker} — {trade_date}"
        if marker in content:
            # Replace PENDING status and reflection
            content = content.replace(
                f"**Decision Status:** PENDING (awaiting return data)\n**Raw Return:** TBD\n**Alpha Return:** TBD",
                f"**Decision Status:** COMPLETED\n**Raw Return:** {raw_return:+.2%}\n**Alpha Return:** {alpha_return:+.2%}"
            )
            content = content.replace(
                "**Reflection:** Pending",
                f"**Reflection:** {reflection.split(chr(10))[0]}"  # First line of reflection
            )
            content = content.replace(
                "### Status",
                f"### Reflection\n{reflection}\n\n### Status"
            )
            
            # Write back
            with open(self.log_file, "w") as f:
                f.write(content)
    
    def get_decision_history(self, ticker: str = None) -> str:
        """Retrieve decision history"""
        if not self.log_file.exists():
            return "No decisions logged yet"
        
        with open(self.log_file, "r") as f:
            content = f.read()
        
        if ticker:
            # Filter by ticker
            lines = content.split("\n")
            filtered = []
            for line in lines:
                if f"## {ticker}" in line or (filtered and line.startswith("#")):
                    filtered.append(line)
            return "\n".join(filtered) if filtered else f"No history for {ticker}"
        
        return content
    
    def format_memory_summary(self) -> str:
        """Format memory log for display"""
        if not self.log_file.exists():
            return "📝 **Memory Log:** Empty (no decisions yet)\n"
        
        with open(self.log_file, "r") as f:
            lines = f.readlines()
        
        # Count decisions
        decision_count = sum(1 for line in lines if line.startswith("## "))
        completed_count = sum(1 for line in lines if "COMPLETED" in line)
        pending_count = decision_count - completed_count
        
        summary = f"""
📝 **Memory Log Summary**
- Total Decisions: {decision_count}
- Completed: {completed_count}
- Pending: {pending_count}

Last 3 Decisions:
"""
        
        # Get last 3 decision headers
        decisions = [line for line in lines if line.startswith("## ")][-3:]
        for decision in decisions:
            summary += f"• {decision.strip()}\n"
        
        return summary


def log_decision(ticker: str,
                trade_date: str,
                signal: str,
                confidence: str,
                entry_price: Optional[float],
                stop_loss: Optional[float],
                take_profit: Optional[float],
                position_size_pct: Optional[float],
                debate_summary: str,
                trader_reasoning: str,
                memory_dir: str = "~/.hermes/profiles/pagupon-finance/memory") -> None:
    """Convenience function"""
    logger = MemoryLogger(memory_dir)
    logger.log_decision(ticker, trade_date, signal, confidence,
                       entry_price, stop_loss, take_profit, position_size_pct,
                       debate_summary, trader_reasoning)
