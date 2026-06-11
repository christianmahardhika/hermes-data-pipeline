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

Owner mentality: This is a business worth holding forever, not a stock to trade.

1. **Quality Moat (ROE {roe:.1f}%)**:
   📖 "The investor should ask: does the business have a durable competitive advantage?"
   - ROE demonstrates ability to reinvest at high rates (sustainable moat marker)
   - Premium pricing power indicates economic advantages
   - Owner return on incremental capital validates long-term viability

2. **Dividend Power (Yield {dy:.2f}%)**:
   📖 "Reinvestment of dividends is where compounding truly accelerates"
   - Sustainable distributions prove real cash generation
   - Reinvestment compounds wealth exponentially over decades
   - Management confidence signaled through payout consistency

3. **Balance Sheet Fortress (D/E {der:.2f}x)**:
   - Low leverage = flexibility for opportunities & downturns
   - Never financial distress = never forced selling
   - Can increase return to shareholders if needed

4. **Valuation Offers Margin of Safety (P/E {per:.1f}x)**:
   - Quality businesses need not be expensive
   - Multiple compression unlikely if moat persists
   - Decades of compounding ahead

**Buffett Verdict**: BUY — Own for 30+ years, reinvest dividends, avoid selling
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🦉 **Buffett Bear Case for {ticker}**

No durable moat visible—this is a commodity business, not a franchise.

1. **Weak ROE (ROE {roe:.1f}%)**:
   📖 "If a business earns 5% ROE, it destroys shareholder value at 10% cost of capital"
   - Insufficient returns on reinvested capital
   - No competitive advantage—business is replaceable
   - Capital allocation destroying value, not creating it

2. **Dividend as Warning Signal (Yield {dy:.2f}%)**:
   - High yield may indicate deteriorating business
   - Management returning capital because reinvestment unprofitable
   - Dividend cut likely if business weakens further

3. **Leverage Limits Flexibility (D/E {der:.2f}x)**:
   - Debt restricts ability to invest through downturns
   - Vulnerable to recession = forced selling
   - Cannot compound capital long-term

4. **Valuation Does Not Compensate (P/E {per:.1f}x)**:
   - Price not cheap enough to offset mediocre business quality
   - Better moats available at similar or lower prices
   - Opportunity cost of capital deployment is real

**Buffett Verdict**: PASS — Seek superior moats, avoid mediocrity at any price
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

Deep value opportunity with adequate margin of safety.

1. **Trading Below Intrinsic Value (P/E {per:.1f}x, P/BV {pbv:.2f}x)**:
   📖 "The margin of safety is the foundation of sound investing"
   - Price substantially discounted from calculated intrinsic value
   - Buffer against analysis errors and future disappointments
   - Graham's cigar-butt: buy dollar for 50 cents, collect yield

2. **Strong Balance Sheet (D/E {der:.2f}x)**:
   📖 "Evaluate the balance sheet first—avoid financial distress risk"
   - Conservative leverage = ability to weather any downturn
   - Asset coverage exceeds debt obligations
   - Can service obligations from operations indefinitely

3. **Dividend Validation (Yield {dy:.2f}%)**:
   📖 "Dividends prove real earnings exist; they cannot be faked"
   - Distributions demonstrate actual cash flow generation
   - Income provides return while waiting for reversion
   - Sustainable payout from strong cash generation

4. **Mathematical Edge**:
   - Downside protected by valuation floor
   - Upside from reversion to mean or intrinsic value
   - Risk/reward asymmetry favorable to buyer

**Graham Verdict**: BUY — Margin of safety adequate, mechanical investing advantage
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        
        case = f"""
📚 **Graham Bear Case for {ticker}**

Margin of safety insufficient—valuation not discounted enough.

1. **Insufficient Discount (P/E {per:.1f}x, P/BV {pbv:.2f}x)**:
   📖 "If you cannot calculate a margin of safety, do not buy the security"
   - Current price approaches intrinsic value estimate
   - Insufficient buffer for estimation errors
   - Downside protection evaporates if fundamentals deteriorate slightly

2. **Balance Sheet Deteriorating (D/E {der:.2f}x)**:
   📖 "Avoid companies where debt service is uncertain"
   - Leverage limiting financial flexibility
   - Debt service burden may pressure distributions
   - Vulnerability in recession unacceptable

3. **Dividend Sustainability Risk (Yield {dy:.2f}%)**:
   📖 "Real earnings support dividends; unsustainable payouts are red flags"
   - Payout ratio leaves minimal margin for earnings decline
   - Dividend cut likely if business faces headwinds
   - Income return illusory if cut is imminent

4. **No Margin of Safety**:
   - Violates Graham's foundational principle
   - Risk/reward unfavorable at current price
   - Better opportunities exist with greater discounts

**Graham Verdict**: PASS — Wait for lower price, margin of safety non-existent
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

Simple, understandable business with sustainable growth—a tenbagger candidate.

1. **Strong ROE (ROE {roe:.1f}%)**:
   📖 "A company with high ROE reinvests those earnings at high rates—that's where real value comes from"
   - ROE proves unit economics are sound
   - Reinvestment compound at high rates over decades
   - Growth funded organically, not through dilution

2. **Attractive GARP Valuation (P/E {per:.1f}x)**:
   📖 "Growth at a reasonable price is where the real money is made"
   - P/E to growth ratio favorable
   - Stock priced for reasonable growth, not pie-in-sky
   - Room for multiple expansion if growth accelerates

3. **Book Value Discount (P/BV {pbv:.2f}x)**:
   - Trading below tangible book value
   - Conservative balance sheet supports growth
   - Upside from multiple expansion as quality recognized

4. **Income + Growth (Dividend {dy:.2f}%)**:
   📖 "The best investments are simple businesses you can understand that also pay you to wait"
   - Dividend provides income cushion during volatility
   - Growth funds both appreciation and distribution growth
   - Understandable business model = predictable returns

**Lynch Verdict**: BUY — Tenbagger potential, simple story, reinvestment quality
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        roe = metrics.get("roe", 0)
        per = metrics.get("per", 0)
        pbv = metrics.get("pbv", 0)
        dy = metrics.get("dy", 0)
        
        case = f"""
🎯 **Lynch Bear Case for {ticker}**

Growth story breaking down—this is a value trap, not a tenbagger.

1. **ROE Insufficient for Growth (ROE {roe:.1f}%)**:
   📖 "If ROE doesn't support the expected growth rate, the company will need to raise capital and dilute shareholders"
   - ROE inadequate to fund organic growth
   - Company will resort to dilutive equity issuance
   - Growth funded by shareholder dilution, not profitability

2. **Valuation Doesn't Justify Growth (P/E {per:.1f}x)**:
   📖 "When P/E to growth ratio exceeds 2.0, the market is pricing in too much perfection"
   - P/E/Growth ratio unattractive
   - Growth expectations already priced in
   - Multiple compression risk if growth disappoints

3. **Book Value Premium (P/BV {pbv:.2f}x)**:
   - Trading above book value
   - Market skeptical of ability to deploy capital profitably
   - Reversion risk to lower multiples

4. **Dividend Warning Signal (DY {dy:.2f}%)**:
   📖 "High dividend yield while growth is expected? That's a red flag—management paying to quiet shareholders"
   - High payout indicates weak reinvestment opportunities
   - Management shifting to distributions instead of investing
   - Better growth opportunities elsewhere

**Lynch Verdict**: PASS — Growth slowing, valuation extended, better candidates exist
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

Predictable, simple business with minimal risk—perfect for sit-and-wait compounding.

1. **Conservative Leverage (D/E {der:.2f}x)**:
   📖 "We want to buy only where the business has durable competitive advantages"
   - Low debt = predictable cash flows
   - No financial distress risk for indefinite horizon
   - Financial flexibility to capitalize on opportunities

2. **Proven Profitability (ROE {roe:.1f}%)**:
   📖 "The best business to own is one that produces steady returns on capital with minimal capital requirements"
   - Consistent ROE demonstrates business predictability
   - Simple model easy to understand and forecast
   - Avoid complexity that breeds mistakes

3. **Reasonable Valuation (P/BV {pbv:.2f}x, P/E {per:.1f}x)**:
   📖 "If something is too hard to understand, we don't do it. We don't worry about stocks we don't understand"
   - No speculation required, only simple compounding
   - Margin of safety exists in valuations
   - Price reasonable for quality and certainty

4. **Risk/Reward Asymmetric**:
   📖 "The three most important words in investing are 'I don't know'"
   - Downside protected by conservative fundamentals
   - Upside from dividend growth and compounding
   - Munger's "sit on your ass" investing pays off

**Munger Verdict**: BUY — Simple, predictable, low risk, compound for decades
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        pbv = metrics.get("pbv", 0)
        per = metrics.get("per", 0)
        
        case = f"""
🧠 **Munger Bear Case for {ticker}**

Too risky and too complex—this violates Munger's core principles.

1. **Leverage Approaching Limits (D/E {der:.2f}x)**:
   📖 "Debt is dangerous. We want to own only simple, predictable businesses with minimal financial risk"
   - Debt levels threaten cash flow predictability
   - Financial stress in downturn highly probable
   - Less margin for operational mistakes

2. **Profitability Deteriorating (ROE {roe:.1f}%)**:
   📖 "When ROE declines, management is destroying capital, not creating it"
   - Return on capital insufficient
   - Business lacking competitive advantages
   - Competitive position weakening = future pressure

3. **Valuation Premium for Uncertainty (P/BV {pbv:.2f}x, P/E {per:.1f}x)**:
   📖 "We only buy when there's a significant margin of safety"
   - Paying premium for unpredictable outcome
   - Business complexity high, clarity low
   - Risk/reward unfavorable at current price

4. **Better Opportunities Exist**:
   📖 "If it's too hard to understand, don't buy it. We simply don't buy what we don't understand"
   - Business complexity violates investment principle
   - Predictability insufficient for Munger discipline
   - Capital better deployed in simpler, safer situations

**Munger Verdict**: PASS — Too risky, too complex, insufficient predictability
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

BUMN tailwinds + macro sweetspot = textbook value opportunity.

1. **BUMN Policy Support (P/E {per:.1f}x)**:
   📖 Indonesian government prioritizes BUMN dividend income for fiscal budget
   - Strategic sector support through policy (Prabowo infrastructure mandate)
   - State guarantee of dividend continuity
   - Export volume targets supporting commodity BUMN

2. **Dividend Security (DY {dy:.2f}%)**:
   📖 BUMN dividend is politically locked-in source for government revenue
   - Government cannot cut dividends without political backlash
   - Payout more stable than private sector
   - Q4/Q1 seasonal peaks historically reliable

3. **Macro Sweet Spot (D/E {der:.2f}x, ROE {roe:.1f}%)**:
   - Commodity supercycle still strong: coal (PTBA), nickel (INCO), CPO (TAPG)
   - BI rate elevated but stabilizing—doesn't hurt earnings
   - Rupiah resilience from SBN flows + BI defense
   - Economic growth buffering demand
   - Mining/energy/agriculture exporters benefiting from global recovery

4. **Seasonal Seasonality Tailwind**:
   - Dividend payout Q4/Q1 historically supports prices
   - Earnings momentum typically peaks in Q1 reporting
   - Historically strong: December-February entry window

**Guru Verdict**: BUY — Policy locked, dividends secure, macro window open
"""
        return case.strip()
    
    def build_bear_case(self, ticker: str, metrics: Dict[str, float]) -> str:
        per = metrics.get("per", 0)
        dy = metrics.get("dy", 0)
        der = metrics.get("der", 0)
        roe = metrics.get("roe", 0)
        
        case = f"""
🇮🇩 **Indonesia Value Guru Bear Case for {ticker}**

Policy risks materializing + macro cycle deteriorating = exit window closing.

1. **BUMN Policy Reversal Risk (P/E {per:.1f}x)**:
   📖 Prabowo administration shifting focus from dividend extraction to debt reduction
   - Export restrictions on commodities (coal ban, nickel moratoria discussions) pressuring BUMN earnings
   - Dividend cuts likely if fiscal consolidation accelerates
   - Policy volatility = unpredictable earnings
   - Strategic sector no longer guaranteed winner

2. **Dividend Cut Imminent (DY {dy:.2f}%)**:
   📖 Government fiscal position tightening—dividend policy is variable, not fixed
   - Fiscal deficit widening forces dividend reduction decisions
   - SBN issuance costs rising = less willingness to finance from dividends
   - BUMN dividend typically first casualty in fiscal consolidation
   - High yield today = dividend cut announcement tomorrow

3. **Macro Cycle Peak Passing (D/E {der:.2f}x, ROE {roe:.1f}%)**:
   - Commodity supercycle already pricing in recovery expectations
   - Coal prices peak: weather-dependent demand weak Q2-Q3
   - Nickel inventory building: price weakness ahead
   - Rupiah depreciation risk if BI loosens (growth vs stability trade-off)
   - Economic slowdown evident in slowing credit growth

4. **Seasonal Weakness Approaching**:
   📖 Post-dividend Q2-Q3 historically weak for Indonesian stocks
   - Dividend distributions complete by Feb—no buying support thereafter
   - Earnings announcements likely to disappoint (beat expectations difficult)
   - Seasonal low typically Feb-May for commodity exporters
   - Better entry points likely in 3-4 months

**Guru Verdict**: PASS — Policy headwinds emerging, macro peak passing, dividend at risk
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
