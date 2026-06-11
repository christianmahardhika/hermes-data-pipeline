"""
Output Formatter — RTI Business Style
Format stock analysis results sebagai readable RTI-style output
dengan emoji, structure, dan semua metrics
"""

from typing import Dict, Any, List, Optional
from dataclasses import dataclass

@dataclass
class RTIBusinessOutput:
    """RTI Business style output structure"""
    ticker: str
    name: str
    price_section: str
    market_section: str
    fundamentals_section: str
    signal_section: str
    debate_section: str
    extras_section: str
    
    def __str__(self):
        """Full formatted output"""
        return f"""{self.ticker} - {self.name}
{'='*60}
{self.price_section}

{self.market_section}

{self.fundamentals_section}

{self.signal_section}

{self.debate_section}

{self.extras_section}
"""

class RTIBusinessFormatter:
    """Format analysis output ke RTI Business style"""
    
    def __init__(self):
        self.emoji_map = {
            "STRONG_BUY": "🚀",
            "BUY": "📈",
            "HOLD": "⏸️",
            "PASS": "⏭️",
            "AVOID": "🛑",
        }
    
    def format_analysis(
        self,
        ticker: str,
        stock_data: Dict[str, Any],
        debate_result: Dict[str, Any],
        trader_proposal: Dict[str, Any],
        risk_assessment: Dict[str, Any],
        idx_profile: Optional[Dict[str, Any]] = None,
    ) -> RTIBusinessOutput:
        """Format complete analysis to RTI Business style"""
        
        if idx_profile is None:
            idx_profile = {}
        
        # Extract data
        price = stock_data.get("current_price", 0)
        name = idx_profile.get("name", ticker)
        signal = debate_result.get("final_signal", "HOLD")
        confidence = debate_result.get("confidence", "MEDIUM")
        
        # Build sections
        price_section = self._format_price_section(ticker, stock_data, idx_profile)
        market_section = self._format_market_section(stock_data, idx_profile)
        fundamentals_section = self._format_fundamentals_section(stock_data, idx_profile)
        signal_section = self._format_signal_section(signal, confidence, debate_result)
        debate_section = self._format_debate_section(debate_result)
        extras_section = self._format_extras_section(trader_proposal, risk_assessment, idx_profile)
        
        return RTIBusinessOutput(
            ticker=ticker,
            name=name,
            price_section=price_section,
            market_section=market_section,
            fundamentals_section=fundamentals_section,
            signal_section=signal_section,
            debate_section=debate_section,
            extras_section=extras_section,
        )
    
    def _format_price_section(
        self,
        ticker: str,
        stock_data: Dict[str, Any],
        idx_profile: Dict[str, Any]
    ) -> str:
        """
        📈 Price: Rp4,260 | 52W: Rp3,650 - Rp5,375
        """
        price = stock_data.get("current_price", 0)
        h52w = idx_profile.get("52w_high", 0)
        l52w = idx_profile.get("52w_low", 0)
        mcap = idx_profile.get("market_cap_trn", 0)
        
        return f"""📈 Price: Rp{price:,.0f} | 52W: Rp{l52w:,.0f} - Rp{h52w:,.0f}
💰 MCap: Rp{mcap}T"""
    
    def _format_market_section(
        self,
        stock_data: Dict[str, Any],
        idx_profile: Dict[str, Any]
    ) -> str:
        """
        P/E: 6.80 | P/BV: 1.30
        """
        per = stock_data.get("per", 0)
        pbv = stock_data.get("pbv", 0)
        
        return f"""P/E: {per:.2f} | P/BV: {pbv:.2f}"""
    
    def _format_fundamentals_section(
        self,
        stock_data: Dict[str, Any],
        idx_profile: Dict[str, Any]
    ) -> str:
        """
        📊 EPS: Rp626.65 | BV: Rp3,270.59 | DY: 11.2%
        📈 ROE: 21.04% | ROA: 2.57% | NPM: 40.24%
        💵 Revenue: Rp145.3T | Net Inc: Rp58.5T | D/E: 0.50
        """
        eps = stock_data.get("eps", 0)
        bv = stock_data.get("bv_per_share", 0)
        dy = stock_data.get("dy", 0)
        roe = stock_data.get("roe", 0)
        roa = stock_data.get("roa", 0)
        npm = stock_data.get("npm", 0)
        revenue = stock_data.get("revenue_trn", 0)
        net_inc = stock_data.get("net_income_trn", 0)
        der = stock_data.get("der", 0)
        
        return f"""📊 EPS: Rp{eps:,.2f} | BV: Rp{bv:,.2f} | DY: {dy:.2f}%
📈 ROE: {roe:.2f}% | ROA: {roa:.2f}% | NPM: {npm:.2f}%
💵 Revenue: Rp{revenue}T | Net Inc: Rp{net_inc}T | D/E: {der:.2f}"""
    
    def _format_signal_section(
        self,
        signal: str,
        confidence: str,
        debate_result: Dict[str, Any]
    ) -> str:
        """
        🚀 SIGNAL: STRONG BUY (HIGH confidence)
        Threshold: 80% personas agree
        """
        emoji = self.emoji_map.get(signal, "❓")
        bull_win = debate_result.get("bull_win_rate", 0.5)
        
        return f"""{emoji} SIGNAL: {signal} ({confidence} confidence)
Bull vs Bear agreement: {bull_win*100:.0f}% confidence in this direction"""
    
    def _format_debate_section(self, debate_result: Dict[str, Any]) -> str:
        """
        🎯 DEBATE REASONING:
        
        Round 1:
        🦉 Bull (Buffett): [argument]
        🐻 Bear (Graham): [counter-argument]
        
        Round 2:
        🦉 Bull Rebuttal: [response]
        🐻 Bear Rebuttal: [response]
        
        Consensus: Bull argues moat strength, Bear counters margin safety concern
        """
        rounds = debate_result.get("debate_rounds", [])
        personas_mentioned = debate_result.get("personas_pair", [])
        
        debate_text = "🎯 DEBATE REASONING:\n"
        
        for i, round_data in enumerate(rounds, 1):
            debate_text += f"\n**Round {i}:**\n"
            
            if isinstance(round_data, dict):
                bull_arg = round_data.get("bull_argument", "")
                bear_arg = round_data.get("bear_argument", "")
                bull_conf = round_data.get("bull_confidence", "MEDIUM")
                bear_conf = round_data.get("bear_confidence", "MEDIUM")
                
                if i <= len(personas_mentioned):
                    bull_persona, bear_persona = personas_mentioned[i-1]
                    debug_txt = f"🦉 Bull ({bull_persona.title()}) [{bull_conf}]:\n{bull_arg[:200]}...\n\n"
                    debug_txt += f"🐻 Bear ({bear_persona.title()}) [{bear_conf}]:\n{bear_arg[:200]}...\n"
                    debate_text += debug_txt
        
        consensus = debate_result.get("consensus_summary", "Debate concluded")
        debate_text += f"\n**Consensus:** {consensus}"
        
        return debate_text
    
    def _format_extras_section(
        self,
        trader_proposal: Any,
        risk_assessment: Dict[str, Any],
        idx_profile: Dict[str, Any]
    ) -> str:
        """
        💼 EXECUTION PLAN:
        Entry: Rp4,100 | Stop: Rp3,770 | Target: Rp4,715
        Position: 3% of portfolio | Risk score: 2.5/10 (LOW)
        """
        # Handle both dict and object
        if hasattr(trader_proposal, 'entry_price'):
            entry = trader_proposal.entry_price
            stop = trader_proposal.stop_loss
            target = trader_proposal.take_profit
            position_pct = trader_proposal.position_size_pct
        else:
            entry = trader_proposal.get("entry_price", 0) if isinstance(trader_proposal, dict) else 0
            stop = trader_proposal.get("stop_loss", 0) if isinstance(trader_proposal, dict) else 0
            target = trader_proposal.get("take_profit", 0) if isinstance(trader_proposal, dict) else 0
            position_pct = trader_proposal.get("position_size_pct", 0.03) if isinstance(trader_proposal, dict) else 0.03
        
        risk_approved = risk_assessment.get("is_approved", True)
        risk_score = risk_assessment.get("risk_score", 5)
        risk_label = "LOW" if risk_score < 3 else "MEDIUM" if risk_score < 7 else "HIGH"
        
        free_float = idx_profile.get("free_float_pct", 0)
        ownership = idx_profile.get("ownership", {})
        liquidity = idx_profile.get("liquidity", {})
        
        institutional = ownership.get("institutional", 40)
        foreign = ownership.get("foreign", 20)
        retail = 100 - institutional - foreign
        
        bid_ask = liquidity.get("bid_ask_spread_pct", 0.3)
        volume = liquidity.get("volume_trend_30d_avg", 0)
        
        status = "✅ APPROVED" if risk_approved else "⚠️ FLAGGED"
        
        return f"""💼 EXECUTION PLAN:
Entry: Rp{entry:,.0f} | Stop: Rp{stop:,.0f} | Target: Rp{target:,.0f}
Position: {position_pct*100:.1f}% of portfolio | Risk: {risk_score:.1f}/10 ({risk_label}) {status}

📊 IDX EXTRAS:
Free Float: {free_float:.0f}% | Institutional: {institutional:.0f}% | Retail: {retail:.0f}% | Foreign: {foreign:.0f}%
Liquidity: {liquidity.get('liquidity_rating', 'Unknown')} (bid-ask {bid_ask:.2f}%) | Volume (30d): {volume/1_000_000:.1f}M shares"""
    
    def format_telegram_compact(
        self,
        ticker: str,
        signal: str,
        price: float,
        per: float,
        dy: float,
        roe: float,
        der: float,
    ) -> str:
        """Compact Telegram format untuk cepat scanning"""
        emoji = self.emoji_map.get(signal, "❓")
        
        criteria_met = []
        if per < 15:
            criteria_met.append("✅ P/E")
        if dy > 3:
            criteria_met.append("✅ DY")
        if roe > 10:
            criteria_met.append("✅ ROE")
        if der < 1:
            criteria_met.append("✅ D/E")
        
        criteria_str = " ".join(criteria_met) if criteria_met else "❌ Criteria fail"
        
        return f"""{emoji} **{ticker}** - {signal}
Rp{price:,.0f} | P/E: {per:.2f} | DY: {dy:.2f}% | ROE: {roe:.2f}% | D/E: {der:.2f}
{criteria_str}"""

def format_multiple_stocks(
    analysis_results: List[Dict[str, Any]]
) -> str:
    """Format multiple stock analyses untuk Telegram bulk output"""
    
    formatter = RTIBusinessFormatter()
    output = "📊 **IDX AI ANALYST — ENHANCED DEBATE SYSTEM**\n"
    output += "=" * 60 + "\n\n"
    
    for result in analysis_results:
        try:
            ticker = result.get("ticker", "UNKNOWN")
            stock_data = result.get("stock_data", {})
            debate_result = result.get("debate_result", {})
            trader_proposal = result.get("trader_proposal", {})
            risk_assessment = result.get("risk_assessment", {})
            idx_profile = result.get("idx_profile", {})
            
            # Use compact format
            signal = debate_result.get("final_signal", "HOLD")
            price = stock_data.get("current_price", 0)
            per = stock_data.get("per", 0)
            dy = stock_data.get("dy", 0)
            roe = stock_data.get("roe", 0)
            der = stock_data.get("der", 0)
            
            compact = formatter.format_telegram_compact(ticker, signal, price, per, dy, roe, der)
            output += compact + "\n\n"
        except Exception as e:
            output += f"❌ Error formatting {result.get('ticker', 'UNKNOWN')}: {e}\n\n"
    
    return output
