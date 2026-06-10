"""
Trader Executor Module — Transaction Proposal Generation
Converts debate signal into concrete trade plan with entry/stop/sizing
"""

from typing import Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum


class TraderAction(Enum):
    """Transaction direction"""
    BUY = "BUY"
    HOLD = "HOLD"
    SELL = "SELL"


@dataclass
class TraderProposal:
    """Structured transaction proposal"""
    ticker: str
    action: TraderAction
    reasoning: str
    entry_price: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    position_size_pct: Optional[float] = None
    holding_days: int = 5
    confidence: str = "MEDIUM"


class TraderExecutor:
    """Generates trade proposals from debate signals"""
    def __init__(self, ticker: str):
        self.ticker = ticker
    
    def execute(self, 
                debate_signal: str,
                stock_data: Dict[str, Any],
                analyst_reports: Dict[str, str],
                portfolio_config: Dict[str, Any] = None) -> TraderProposal:
        """Generate trade proposal"""
        
        if portfolio_config is None:
            portfolio_config = {
                "default_position_size_pct": 0.03,
                "entry_price_offset_pct": 0.02,
                "stop_loss_pct": 0.08,
                "take_profit_pct": 0.15,
            }
        
        current_price = stock_data.get("current_price", 0)
        
        # Convert signal to action
        if debate_signal in ["STRONG BUY", "BUY"]:
            action = TraderAction.BUY
            position_size = portfolio_config.get("default_position_size_pct", 0.03)
            
            # Calculate entry/stop/tp
            entry_offset = portfolio_config.get("entry_price_offset_pct", 0.02)
            entry_price = current_price * (1 - entry_offset)  # Buy dip
            
            stop_loss = entry_price * (1 - portfolio_config.get("stop_loss_pct", 0.08))
            take_profit = entry_price * (1 + portfolio_config.get("take_profit_pct", 0.15))
            
            reasoning = f"""
Debate reached {debate_signal} consensus after multi-round debate.
Bull researcher's arguments on quality fundamentals and valuation support initiation.
Entry at support level ({entry_price:.0f}) provides margin of safety.
Risk/reward attractive (stop at {stop_loss:.0f}, target {take_profit:.0f}).
"""
            
        elif debate_signal == "HOLD":
            action = TraderAction.HOLD
            position_size = None
            entry_price = None
            stop_loss = None
            take_profit = None
            
            reasoning = """
Debate inconclusive — bull and bear arguments balanced.
HOLD existing positions if owned; avoid new entry until conviction builds.
Monitor for catalyst that breaks tie toward BUY or PASS.
"""
            
        else:  # PASS, AVOID
            action = TraderAction.HOLD  # Don't transact
            position_size = None
            entry_price = None
            stop_loss = None
            take_profit = None
            
            reasoning = f"""
Debate favored bear arguments ({debate_signal}).
Risk/reward unfavorable at current price.
PASS on new entry; wait for material repricing or fundamental improvement.
"""
        
        confidence_map = {
            "STRONG BUY": "HIGH",
            "BUY": "MEDIUM",
            "HOLD": "MEDIUM",
            "PASS": "MEDIUM",
            "AVOID": "HIGH"
        }
        
        proposal = TraderProposal(
            ticker=self.ticker,
            action=action,
            reasoning=reasoning.strip(),
            entry_price=entry_price,
            stop_loss=stop_loss,
            take_profit=take_profit,
            position_size_pct=position_size,
            holding_days=5,
            confidence=confidence_map.get(debate_signal, "MEDIUM")
        )
        
        return proposal
    
    def format_proposal(self, proposal: TraderProposal) -> str:
        """Format proposal for human reading"""
        output = f"""
**TRADER PROPOSAL — {proposal.ticker}**

Action: **{proposal.action.value}**
Confidence: {proposal.confidence}

Reasoning:
{proposal.reasoning}
"""
        
        if proposal.action == TraderAction.BUY:
            output += f"""
**Execution Plan:**
• Entry Price: Rp {proposal.entry_price:,.0f}
• Stop Loss: Rp {proposal.stop_loss:,.0f}
• Take Profit: Rp {proposal.take_profit:,.0f}
• Position Size: {proposal.position_size_pct * 100:.1f}% of portfolio
• Holding Period: {proposal.holding_days} trading days
"""
        
        return output.strip()


def generate_trader_proposal(ticker: str,
                            debate_signal: str,
                            stock_data: Dict[str, Any],
                            analyst_reports: Dict[str, str],
                            config: Dict[str, Any] = None) -> TraderProposal:
    """Convenience function"""
    trader = TraderExecutor(ticker)
    return trader.execute(debate_signal, stock_data, analyst_reports, config)
