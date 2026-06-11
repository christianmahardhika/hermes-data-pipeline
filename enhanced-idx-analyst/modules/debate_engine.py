"""
Debate Engine Module — Researcher Team with 5 Personas
Bull vs Bear debate with persona-specific reasoning
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
    """Single debate round state with persona info"""
    round_num: int
    bull_persona: str
    bear_persona: str
    bull_argument: str
    bear_argument: str
    bull_confidence: str
    bear_confidence: str
    bull_reasoning: str  # NEW: Why this persona thinks so
    bear_reasoning: str  # NEW: Why this persona thinks so


class PersonaStyle:
    """Persona investment philosophy"""
    def __init__(self, name: str, emoji: str, philosophy: str, focus_metrics: List[str]):
        self.name = name
        self.emoji = emoji
        self.philosophy = philosophy
        self.focus_metrics = focus_metrics
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        """Build bullish case from this persona's perspective"""
        raise NotImplementedError
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        """Build bearish case from this persona's perspective"""
        raise NotImplementedError


class BuffettPersona(PersonaStyle):
    """Warren Buffett — Long-term moats, dividend quality, sustainable advantages"""
    def __init__(self):
        super().__init__(
            name="Warren Buffett",
            emoji="🦉",
            philosophy="Long-term moats, dividend quality, sustainable advantages",
            focus_metrics=["dividend_yield", "roe", "der", "per"]
        )
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🦉 **Buffett Bull Case for {ticker}**

This company exhibits classic moat characteristics:

1. **Quality Moat (ROE {roe:.1f}%)**: High return on equity signals competitive advantage
   - Ability to reinvest earnings at high rates indicates sustainable moat
   - Economic advantages allow premium pricing power

2. **Dividend Quality (Yield {dy:.2f}%)**:
   - Sustainable dividend demonstrates cash generation capability
   - Income provides downside cushion for patient shareholders
   - Payout policy indicates management confidence in business durability

3. **Conservative Leverage (D/E {der:.2f}x)**:
   - Low debt provides financial flexibility
   - Room to increase leverage or return capital if needed
   - Safety-first approach protects downside in downturns

4. **Reasonable Valuation (P/E {per:.1f}x)**:
   - Margin of safety exists at current levels
   - Quality doesn't command premium price
   - Long-term compounding potential attractive

**Buffett Verdict**: BUY — Hold for long term, reinvest dividends, let compounding work
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🦉 **Buffett Bear Case for {ticker}**

This investment fails moat quality criteria:

1. **ROE Concerns (ROE {roe:.1f}%)**:
   - ROE insufficient to justify premium holding
   - Business may lack true competitive advantage
   - Returns may not justify long-term capital deployment

2. **Dividend Sustainability Risk (Yield {dy:.2f}%)**:
   - Payout ratio unsustainable in downturns
   - High yield may signal desperation, not strength
   - Dividend cuts possible if business faces pressure

3. **Leverage Approaching Limits (D/E {der:.2f}x)**:
   - Limited flexibility for challenges ahead
   - Restricted ability to invest in moat expansion
   - Vulnerable to recession

4. **Valuation Premium Not Justified (P/E {per:.1f}x)**:
   - Quality premium cannot be supported by fundamentals
   - Better opportunities exist at lower risk/reward

**Buffett Verdict**: PASS — Wait for better entry or find superior moat
"""
        return case.strip()


class GrahamPersona(PersonaStyle):
    """Benjamin Graham — Margin of safety, deep value, intrinsic value"""
    def __init__(self):
        super().__init__(
            name="Benjamin Graham",
            emoji="📚",
            philosophy="Margin of safety, deep value, intrinsic value",
            focus_metrics=["per", "pbv", "der", "dividend_yield"]
        )
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        
        case = f"""
📚 **Graham Bull Case for {ticker}**

Adequate margin of safety exists:

1. **Deep Value Metrics (P/E {per:.1f}x, P/BV {pbv:.2f}x)**:
   - Trading below intrinsic value with safety margin
   - Downside protection substantial at current prices
   - Mathematical edge favors investment

2. **Balance Sheet Safety (D/E {der:.2f}x)**:
   - Conservative debt levels provide bankruptcy protection
   - Asset coverage strong relative to debt
   - Company can weather extended downturn

3. **Dividend Yield ({dy:.2f}%)**:
   - Income component provides return cushion
   - Dividend provides evidence of real earnings
   - Cash generation validated by shareholder distributions

4. **Price vs Intrinsic Value**:
   - Sufficient margin of safety to overcome analysis errors
   - Graham's cigar-butt approach: buy dollar for 50 cents

**Graham Verdict**: BUY — Margin of safety adequate, downside protected
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        
        case = f"""
📚 **Graham Bear Case for {ticker}**

Inadequate margin of safety:

1. **Valuation Concerns (P/E {per:.1f}x, P/BV {pbv:.2f}x)**:
   - Price reflects fair value already, not discount
   - Insufficient margin of safety for Graham's requirements
   - Risk/reward unfavorable at current levels

2. **Balance Sheet Red Flags (D/E {der:.2f}x)**:
   - Leverage approaching concerning levels
   - Debt service burden may pressure dividends
   - Limited room for negative surprise

3. **Dividend Sustainability Questions (Yield {dy:.2f}%)**:
   - Payout ratio leaves little margin for earnings decline
   - Dividend cut risk material if earnings fall

4. **Intrinsic Value Below Market Price**:
   - No margin of safety exists
   - Violates Graham's foundational principle
   - Better opportunities available with greater discounts

**Graham Verdict**: PASS — Insufficient margin of safety, wait for lower price
"""
        return case.strip()


class LynchPersona(PersonaStyle):
    """Peter Lynch — Business simplicity, growth, understandable companies"""
    def __init__(self):
        super().__init__(
            name="Peter Lynch",
            emoji="🎯",
            philosophy="Business simplicity, growth, understandable companies",
            focus_metrics=["roe", "pbv", "per", "dividend_yield"]
        )
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        
        case = f"""
🎯 **Lynch Bull Case for {ticker}**

This is a simple, understandable business with growth:

1. **Strong ROE (ROE {roe:.1f}%)**:
   - Company reinvests profitably into growth
   - Unit economics sound and sustainable
   - Growth from operational excellence, not financial engineering

2. **Reasonable Growth Valuation (P/E {per:.1f}x)**:
   - P/E/Growth ratio attractive
   - Growth opportunities priced reasonably
   - Lynch's "tenbagger" candidates start here

3. **Book Value Discount (P/BV {pbv:.2f}x)**:
   - Trading below intrinsic book value
   - Conservative balance sheet supporting growth
   - Room for multiple expansion

4. **Income Plus Growth (DY {dy:.2f}%)**:
   - Dividend provides income while waiting for growth
   - Best of both worlds: income + appreciation potential
   - Understandable business model

**Lynch Verdict**: BUY — Simple, understandable, good growth/value balance
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        
        case = f"""
🎯 **Lynch Bear Case for {ticker}**

Growth story doesn't hold up:

1. **ROE Insufficient for Growth (ROE {roe:.1f}%)**:
   - Return on equity insufficient to fund growth
   - Organic growth hampered by profitability constraints
   - May resort to dilutive financing

2. **Valuation Doesn't Reflect Growth (P/E {per:.1f}x)**:
   - P/E/Growth ratio unfavorable
   - Growth prospects already priced in
   - Limited upside to multiple expansion

3. **Book Value Premium (P/BV {pbv:.2f}x)**:
   - Trading above book suggests market pricing in troubles
   - Growth not materializing as expected
   - Reversion risk to lower multiples

4. **Dividend a Warning Sign (DY {dy:.2f}%)**:
   - High yield may indicate slow growth ahead
   - Management allocating to dividends instead of growth reinvestment
   - Better growth candidates elsewhere

**Lynch Verdict**: PASS — Growth story deteriorating, limited upside
"""
        return case.strip()


class MungerPersona(PersonaStyle):
    """Charlie Munger — Risk avoidance, simplicity, predictable business"""
    def __init__(self):
        super().__init__(
            name="Charlie Munger",
            emoji="🧠",
            philosophy="Risk avoidance, simplicity, predictable business",
            focus_metrics=["der", "roe", "pbv", "per"]
        )
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        pbv = metrics.get("pbv", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🧠 **Munger Bull Case for {ticker}**

Predictable business with acceptable risk:

1. **Conservative Leverage (D/E {der:.2f}x)**:
   - Risk profile minimal due to low debt
   - Predictable cash flows support business
   - No financial distress risk for a decade

2. **Proven Profitability (ROE {roe:.1f}%)**:
   - Consistent return on capital demonstrates business predictability
   - Simple business model easy to understand
   - Avoid complexity — embrace predictability

3. **Reasonable Valuation (P/BV {pbv:.2f}x, P/E {per:.1f}x)**:
   - No speculation required, just simple compounding
   - Margin of safety exists
   - Prices paid for simplicity and predictability

4. **Risk/Reward Asymmetric**:
   - Downside protected by fundamentals
   - Upside from dividend growth and compounding
   - Munger's "sit on your ass" investing approach

**Munger Verdict**: BUY — Simple, predictable, low risk, let it compound
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        pbv = metrics.get("pbv", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🧠 **Munger Bear Case for {ticker}**

Risk profile unacceptable:

1. **Leverage Approaching Limits (D/E {der:.2f}x)**:
   - Debt levels limit business predictability
   - Financial stress in downturn possible
   - Less margin for operational mistakes

2. **Profitability Deteriorating (ROE {roe:.1f}%)**:
   - Return on capital insufficient
   - Business lacking pricing power
   - Competitive position weakening

3. **Valuation Premium for Risk (P/BV {pbv:.2f}x, P/E {per:.1f}x)**:
   - Paying too much for unpredictable outcome
   - Simplicity no longer applies
   - Risk/reward unfavorable

4. **Better Opportunities Exist**:
   - Munger principle: "If it's too hard to understand, don't buy it"
   - Business complexity high, predictability low
   - Capital better deployed elsewhere

**Munger Verdict**: PASS — Too complex, too risky, insufficient returns
"""
        return case.strip()


class IDXValueGuruPersona(PersonaStyle):
    """Indonesia Value Guru — BUMN policy, regulation, seasonality, macro"""
    def __init__(self):
        super().__init__(
            name="Indonesia Value Guru",
            emoji="🇮🇩",
            philosophy="BUMN policy, regulation, seasonality, macro",
            focus_metrics=["per", "dividend_yield", "der", "roe"]
        )
    
    def build_bull_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        
        case = f"""
🇮🇩 **Indonesia Value Guru Bull Case for {ticker}**

Tailwinds from policy and macro environment:

1. **BUMN Policy Tailwinds (P/E {per:.1f}x)**:
   - Government supporting strategic sectors
   - State-owned or strategic partner advantage
   - Policy favorability improving sector dynamics

2. **Dividend Stability (DY {dy:.2f}%)**:
   - Government dividend requirements ensure distributions
   - BUMN dividend policy supports shareholder returns
   - Income stability better than private sector

3. **Macro Momentum (D/E {der:.2f}x, ROE {roe:.1f}%)**:
   - Strong commodity prices (coal, nickel, CPO, oil)
   - Rupiah supported by BI rate defense
   - Economic growth above trend
   - Mining/energy/agriculture benefiting

4. **Seasonal Patterns**:
   - Dividend payout peaks Q4/Q1
   - Earnings seasonality favorable
   - Historically strong during this period

**Guru Verdict**: BUY — Government support, macro tailwinds, dividend security
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        
        case = f"""
🇮🇩 **Indonesia Value Guru Bear Case for {ticker}**

Policy risks and macro headwinds emerging:

1. **BUMN Policy Risk (P/E {per:.1f}x)**:
   - Government policy shifts can reverse fortunes
   - Export restrictions (Prabowo BUMN mandate) pressure commodities
   - Regulatory uncertainty increasing

2. **Dividend Cuts Risk (DY {dy:.2f}%)**:
   - Government may reduce dividend to fund debt reduction
   - Fiscal consolidation underway
   - Dividend policy subject to political pressure

3. **Macro Deteriorating (D/E {der:.2f}x, ROE {roe:.1f}%)**:
   - Commodity supercycle ending, price weakness ahead
   - BI tightening cycle limiting growth
   - Rupiah depreciation pressuring importers
   - Economic slowdown evident

4. **Seasonal Weakness**:
   - Post-dividend decline typical
   - Earnings likely to disappoint next quarter
   - Seasonal low approaching

**Guru Verdict**: PASS — Policy headwinds, macro deteriorating, dividend at risk
"""
        return case.strip()


class PersonaDebateEngine:
    """Debate engine powered by 5 personas"""
    
    def __init__(self, ticker: str, max_rounds: int = 2):
        self.ticker = ticker
        self.max_rounds = max_rounds
        
        # Initialize personas
        self.personas = {
            "buffett": BuffettPersona(),
            "graham": GrahamPersona(),
            "lynch": LynchPersona(),
            "munger": MungerPersona(),
            "guru_id": IDXValueGuruPersona(),
        }
        
        # Bull side: Buffett & Lynch
        self.bull_personas = ["buffett", "lynch"]
        # Bear side: Graham & Munger
        self.bear_personas = ["graham", "munger"]
        # Wildcard: Indonesia Value Guru (alternates)
        self.guru_persona = "guru_id"
        
        self.debate_history: List[DebateRound] = []
        self.final_signal = Signal.HOLD
        self.consensus_confidence = "MEDIUM"
        self.bull_win_rate = 0.5
    
    def run_debate(self, metrics: Dict[str, float]) -> Dict[str, Any]:
        """Run full debate cycle with personas"""
        
        for round_num in range(1, self.max_rounds + 1):
            # Select personas for this round (rotate or use fixed pair)
            if round_num == 1:
                bull_persona_key = self.bull_personas[0]  # Buffett
                bear_persona_key = self.bear_personas[0]  # Graham
            else:
                bull_persona_key = self.bull_personas[1]  # Lynch
                bear_persona_key = self.bear_personas[1]  # Munger
            
            bull_persona = self.personas[bull_persona_key]
            bear_persona = self.personas[bear_persona_key]
            
            # Generate arguments
            bull_arg = bull_persona.build_bull_case(self.ticker, metrics)
            bear_arg = bear_persona.build_bear_case(self.ticker, metrics)
            
            # Extract confidence (Buffett/Lynch: HIGH round 1, MEDIUM round 2+)
            # (Graham/Munger: same pattern)
            bull_conf = "HIGH" if round_num == 1 else "MEDIUM"
            bear_conf = "HIGH" if round_num == 1 else "MEDIUM"
            
            # Log round
            self.debate_history.append(DebateRound(
                round_num=round_num,
                bull_persona=bull_persona_key,
                bear_persona=bear_persona_key,
                bull_argument=bull_arg,
                bear_argument=bear_arg,
                bull_confidence=bull_conf,
                bear_confidence=bear_conf,
                bull_reasoning=f"{bull_persona.name}: {bull_persona.philosophy}",
                bear_reasoning=f"{bear_persona.name}: {bear_persona.philosophy}",
            ))
        
        # Calculate consensus
        self._calculate_consensus()
        
        return self.get_debate_summary()
    
    def _calculate_consensus(self):
        """Determine consensus signal from debate"""
        bull_high_count = sum(1 for r in self.debate_history if r.bull_confidence == "HIGH")
        bear_high_count = sum(1 for r in self.debate_history if r.bear_confidence == "HIGH")
        
        total_rounds = len(self.debate_history)
        self.bull_win_rate = bull_high_count / total_rounds if total_rounds > 0 else 0.5
        
        if self.bull_win_rate >= 0.75:
            self.final_signal = Signal.STRONG_BUY
            self.consensus_confidence = "HIGH"
        elif self.bull_win_rate >= 0.5:
            self.final_signal = Signal.BUY
            self.consensus_confidence = "MEDIUM"
        elif self.bull_win_rate >= 0.25:
            self.final_signal = Signal.HOLD
            self.consensus_confidence = "MEDIUM"
        else:
            self.final_signal = Signal.PASS
            self.consensus_confidence = "MEDIUM"
    
    def get_debate_summary(self) -> Dict[str, Any]:
        """Return debate summary with persona info"""
        
        debate_transcript = ""
        for r in self.debate_history:
            persona_obj_bull = self.personas[r.bull_persona]
            persona_obj_bear = self.personas[r.bear_persona]
            
            debate_transcript += f"""
**Round {r.round_num}**

{persona_obj_bull.emoji} **{persona_obj_bull.name} (Bull)**
{r.bull_argument}

{persona_obj_bear.emoji} **{persona_obj_bear.name} (Bear)**
{r.bear_argument}
"""
        
        # Consensus reasoning
        consensus = ""
        if self.bull_win_rate >= 0.75:
            consensus = "Strong consensus among personalities that stock is attractive. Multiple perspectives (quality moat, margin of safety, predictability) align on BUY."
        elif self.bull_win_rate >= 0.5:
            consensus = "Modest bull consensus. Quality and value arguments outweigh risk concerns, but debate is meaningful."
        elif self.bull_win_rate >= 0.25:
            consensus = "No clear consensus. Bear arguments about risk and valuation offset bull case. Balanced debate."
        else:
            consensus = "Bear consensus. Multiple perspectives (margin of safety, predictability, macro risks) suggest waiting for better entry."
        
        return {
            "ticker": self.ticker,
            "rounds": len(self.debate_history),
            "final_signal": self.final_signal.value,
            "confidence": self.consensus_confidence,
            "bull_win_rate": self.bull_win_rate,
            "debate_transcript": debate_transcript.strip(),
            "debate_rounds": self.debate_history,
            "personas_pair": [(r.bull_persona, r.bear_persona) for r in self.debate_history],
            "consensus_summary": consensus,
        }


def debate_stock(ticker: str, metrics: Dict[str, float], max_rounds: int = 2) -> Dict[str, Any]:
    """Run persona-powered debate for a single stock"""
    engine = PersonaDebateEngine(ticker, max_rounds)
    return engine.run_debate(metrics)
