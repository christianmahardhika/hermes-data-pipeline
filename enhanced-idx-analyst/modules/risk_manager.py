"""
Risk Manager Module — Portfolio Constraint Validation
Validates trader proposal against portfolio limits before execution
"""

from typing import Dict, Any, Optional
from dataclasses import dataclass

@dataclass
class RiskAssessment:
    """Risk assessment result"""
    is_approved: bool
    approval_reason: str
    suggested_position_size: Optional[float] = None
    risk_score: float = 0.0  # 0-1
    warnings: list = None
    
    def __post_init__(self):
        if self.warnings is None:
            self.warnings = []


class RiskManager:
    """Validates proposals against portfolio constraints"""
    def __init__(self, portfolio_value: float, risk_config: Dict[str, Any]):
        self.portfolio_value = portfolio_value
        self.config = risk_config
    
    def assess_proposal(self,
                       ticker: str,
                       proposed_position_size_pct: Optional[float],
                       stock_data: Dict[str, Any],
                       existing_positions: Dict[str, float] = None) -> RiskAssessment:
        """Assess risk of proposed position"""
        
        if existing_positions is None:
            existing_positions = {}
        
        warnings = []
        risk_score = 0.0
        
        # Skip if action is HOLD
        if proposed_position_size_pct is None:
            return RiskAssessment(
                is_approved=True,
                approval_reason="Action is HOLD, no position risk to assess"
            )
        
        # Check 1: Position size limit
        max_position_pct = self.config.get("max_position_size_pct", 0.05)
        if proposed_position_size_pct > max_position_pct:
            warnings.append(
                f"Position size {proposed_position_size_pct*100:.1f}% exceeds limit {max_position_pct*100:.1f}%"
            )
            # Auto-cap to limit
            proposed_position_size_pct = max_position_pct
            risk_score += 0.3
        
        # Check 2: Liquidity
        dy = stock_data.get("dividend_yield", 0)
        roe = stock_data.get("roe", 0)
        
        if dy < 1:
            warnings.append(f"Low dividend yield ({dy:.1f}%) — liquidity concern")
            risk_score += 0.1
        
        if roe < 5:
            warnings.append(f"Low ROE ({roe:.1f}%) — poor quality")
            risk_score += 0.2
        
        # Check 3: Valuation extremes
        per = stock_data.get("per", 0)
        if per > 30:
            warnings.append(f"High P/E ({per:.1f}) — valuation risk")
            risk_score += 0.2
        
        # Check 4: Debt level
        der = stock_data.get("der", 0)
        if der > 1.5:
            warnings.append(f"High debt ({der:.2f}) — leverage risk")
            risk_score += 0.15
        
        # Decision
        is_approved = risk_score < 0.5
        
        if is_approved:
            approval_reason = f"Position approved at {proposed_position_size_pct*100:.1f}% with manageable risk"
        else:
            approval_reason = f"Position risk too high (score {risk_score:.2f}). Consider reducing size or waiting."
        
        return RiskAssessment(
            is_approved=is_approved,
            approval_reason=approval_reason,
            suggested_position_size=proposed_position_size_pct if is_approved else None,
            risk_score=risk_score,
            warnings=warnings,
        )
    
    def format_assessment(self, assessment: RiskAssessment, ticker: str) -> str:
        """Format risk assessment for reporting"""
        status = "✅ APPROVED" if assessment.is_approved else "⚠️ FLAGGED"
        
        output = f"""
**RISK ASSESSMENT — {ticker}**

Status: {status}
Risk Score: {assessment.risk_score:.2f}/1.0

{assessment.approval_reason}
"""
        
        if assessment.warnings:
            output += "\nWarnings:\n"
            for w in assessment.warnings:
                output += f"• {w}\n"
        
        if assessment.suggested_position_size:
            output += f"\nSuggested Position Size: {assessment.suggested_position_size*100:.1f}%"
        
        return output.strip()


def assess_risk(ticker: str,
               proposed_position_size_pct: Optional[float],
               stock_data: Dict[str, Any],
               portfolio_value: float,
               risk_config: Dict[str, Any],
               existing_positions: Dict[str, float] = None) -> RiskAssessment:
    """Convenience function"""
    manager = RiskManager(portfolio_value, risk_config)
    return manager.assess_proposal(ticker, proposed_position_size_pct, stock_data, existing_positions)
