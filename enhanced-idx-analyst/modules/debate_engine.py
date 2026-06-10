"""
Debate Engine Module — Researcher Team
Bull vs Bear debate to build consensus signal
Implements TradingAgents debate mechanism on top of 5-persona system
"""

import sys
from typing import Dict, List, Tuple, Any
from enum import Enum
from dataclasses import dataclass

class Signal(Enum):
    """Investment signal"""
    STRONG_BUY = "STRONG BUY"
    BUY = "BUY"
    HOLD = "HOLD"
    PASS = "PASS"
    AVOID = "AVOID"


@dataclass
class DebateRound:
    """Single debate round state"""
    round_num: int
    bull_argument: str
    bear_argument: str
    bull_confidence: str
    bear_confidence: str


class Researcher:
    """Base researcher class"""
    def __init__(self, name: str, ticker: str, style: str):
        self.name = name
        self.ticker = ticker
        self.style = style
    
    def argue(self, analyst_reports: Dict[str, str], opponent_argument: str = "", round_num: int = 1) -> Tuple[str, str]:
        """Generate argument for this research round"""
        raise NotImplementedError


class BullResearcher(Researcher):
    """Bullish researcher — builds case FOR investment"""
    def __init__(self, ticker: str):
        super().__init__("Bull Researcher", ticker, "Bullish")
    
    def argue(self, analyst_reports: Dict[str, str], opponent_argument: str = "", round_num: int = 1) -> Tuple[str, str]:
        """Build bullish argument"""
        market_report = analyst_reports.get("market_analyst", "")
        fundamentals_report = analyst_reports.get("fundamentals_analyst", "")
        sentiment_report = analyst_reports.get("sentiment_analyst", "")
        news_report = analyst_reports.get("news_analyst", "")
        
        if round_num == 1:
            argument = f"""
**Bull Argument (Round {round_num})**

Key Points:
1. **Valuation**: Stock trades at reasonable valuation with margin of safety
2. **Fundamentals**: Quality business with stable cash generation
3. **Dividend**: Sustainable dividend yield above market average
4. **Sentiment**: Positive sentiment from retail and institutional investors
5. **Catalysts**: Sector tailwinds and policy support

Supporting Evidence:
- Fundamentals report shows strong quality metrics
- Market analyst notes healthy technical setup
- News analyst sees positive macro environment
- Sentiment shows bullish bias building

Recommendation: Position is attractive for patient capital
"""
        else:
            # Subsequent rounds: refute bear argument
            argument = f"""
**Bull Rebuttal (Round {round_num})**

Addressing bear concerns:

The bear raises valid points about {opponent_argument[:100]}...

However:
1. **Risk is manageable** — position sizing and stop-loss mitigate downside
2. **Quality competes** — company fundamentals will drive returns despite macro noise
3. **Dividend floor** — high yield provides income cushion
4. **Entry is right** — waiting for perfection means missing gains

Bottom line: Risk/reward heavily favors BUY at current levels
"""
        
        confidence = "HIGH" if round_num == 1 else "MEDIUM"
        return argument.strip(), confidence


class BearResearcher(Researcher):
    """Bearish researcher — builds case AGAINST investment"""
    def __init__(self, ticker: str):
        super().__init__("Bear Researcher", ticker, "Bearish")
    
    def argue(self, analyst_reports: Dict[str, str], opponent_argument: str = "", round_num: int = 1) -> Tuple[str, str]:
        """Build bearish argument"""
        market_report = analyst_reports.get("market_analyst", "")
        fundamentals_report = analyst_reports.get("fundamentals_analyst", "")
        sentiment_report = analyst_reports.get("sentiment_analyst", "")
        news_report = analyst_reports.get("news_analyst", "")
        
        if round_num == 1:
            argument = f"""
**Bear Argument (Round {round_num})**

Key Concerns:
1. **Valuation**: Not cheap relative to peers despite quality
2. **Growth**: Profit growth limited by sector maturity
3. **Macro**: Interest rates and policy uncertainty pose risks
4. **Technicals**: Price action shows resistance, risk/reward unfavorable
5. **Alternatives**: Better opportunities exist elsewhere

Supporting Evidence:
- Market shows distribution at resistance levels
- News hints at policy changes affecting sector
- Fundamentals solid but not exceptional
- Sentiment deteriorating on rate concerns

Recommendation: WAIT for better entry or avoid entirely
"""
        else:
            # Subsequent rounds: refute bull argument
            argument = f"""
**Bear Rebuttal (Round {round_num})**

Contesting bull's position:

Bull's optimism on {opponent_argument[:100]}... overlooks key risks:

1. **Valuation not a bargain** — quality premium already priced in
2. **Macro headwinds** — central bank tightening will pressure earnings
3. **Dividend unsustainable** — payout ratio limits flexibility
4. **Better opportunities** — capital better deployed elsewhere

Bottom line: Risk/reward favors PASS until inflection points clear
"""
        
        confidence = "HIGH" if round_num == 1 else "MEDIUM"
        return argument.strip(), confidence


class DebateEngine:
    """Orchestrates bull/bear debate"""
    def __init__(self, ticker: str, max_rounds: int = 2):
        self.ticker = ticker
        self.max_rounds = max_rounds
        self.bull = BullResearcher(ticker)
        self.bear = BearResearcher(ticker)
        self.debate_history: List[DebateRound] = []
        self.final_signal = Signal.HOLD
        self.consensus_confidence = "MEDIUM"
    
    def run_debate(self, analyst_reports: Dict[str, str]) -> Dict[str, Any]:
        """Run full debate cycle"""
        bull_last_arg = ""
        bear_last_arg = ""
        
        for round_num in range(1, self.max_rounds + 1):
            # Bull argues
            bull_arg, bull_conf = self.bull.argue(analyst_reports, bear_last_arg, round_num)
            
            # Bear responds
            bear_arg, bear_conf = self.bear.argue(analyst_reports, bull_arg, round_num)
            
            # Log round
            self.debate_history.append(DebateRound(
                round_num=round_num,
                bull_argument=bull_arg,
                bear_argument=bear_arg,
                bull_confidence=bull_conf,
                bear_confidence=bear_conf,
            ))
            
            bull_last_arg = bull_arg
            bear_last_arg = bear_arg
        
        # Calculate consensus
        self._calculate_consensus()
        
        return self.get_debate_summary()
    
    def _calculate_consensus(self):
        """Determine consensus signal from debate"""
        # Simplified: count confidence levels
        bull_high_count = sum(1 for r in self.debate_history if r.bull_confidence == "HIGH")
        bear_high_count = sum(1 for r in self.debate_history if r.bear_confidence == "HIGH")
        
        total_rounds = len(self.debate_history)
        bull_win_rate = bull_high_count / total_rounds if total_rounds > 0 else 0.5
        
        if bull_win_rate >= 0.75:
            self.final_signal = Signal.STRONG_BUY
            self.consensus_confidence = "HIGH"
        elif bull_win_rate >= 0.5:
            self.final_signal = Signal.BUY
            self.consensus_confidence = "MEDIUM"
        elif bull_win_rate >= 0.25:
            self.final_signal = Signal.HOLD
            self.consensus_confidence = "MEDIUM"
        else:
            self.final_signal = Signal.PASS
            self.consensus_confidence = "MEDIUM"
    
    def get_debate_summary(self) -> Dict[str, Any]:
        """Return debate summary"""
        return {
            "ticker": self.ticker,
            "rounds": len(self.debate_history),
            "final_signal": self.final_signal.value,
            "confidence": self.consensus_confidence,
            "debate_transcript": "\n\n".join([
                f"**Round {r.round_num}**\n{r.bull_argument}\n\n{r.bear_argument}"
                for r in self.debate_history
            ]),
            "debate_history": self.debate_history,
        }


def debate_stock(ticker: str, analyst_reports: Dict[str, str], max_rounds: int = 2) -> Dict[str, Any]:
    """Run debate for a single stock"""
    engine = DebateEngine(ticker, max_rounds)
    return engine.run_debate(analyst_reports)
