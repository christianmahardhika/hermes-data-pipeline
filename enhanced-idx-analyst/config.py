"""
Configuration for Enhanced IDX AI Analyst with Debate Mechanism
Integrates TradingAgents concepts (debate + risk + memory) into 5-persona system
"""

# Portfolio & Watchlist
PORTFOLIO_STOCKS = ["KLBF", "TLKM", "BBRI", "PTBA", "BJTM", "ADMF", "TAPG", "JPFA", "TSPC", "BMRI", "ASII"]
WATCHLIST_STOCKS = ["INCO", "ANTM", "MDKA"]
ALL_STOCKS = PORTFOLIO_STOCKS + WATCHLIST_STOCKS

# Investment Screening Criteria (Christian's standard value investing filter)
CRITERIA = {
    "per_max": 15,      # Price-to-Earnings ratio
    "pbv_max": 2,       # Price-to-Book value ratio
    "roe_min": 10,      # Return on Equity %
    "der_max": 1,       # Debt-to-Equity ratio
    "dy_min": 3         # Dividend Yield %
}

# 5 Personas (Buffett, Graham, Lynch, Munger, Indonesia Value Guru)
PERSONAS = {
    "buffett": {
        "emoji": "🦉",
        "name": "Warren Buffett",
        "style": "Long-term moats, dividend quality, sustainable advantages",
        "focus": ["dividend_yield", "roe", "der", "per"]
    },
    "graham": {
        "emoji": "📚",
        "name": "Benjamin Graham",
        "style": "Margin of safety, deep value, intrinsic value",
        "focus": ["per", "pbv", "der", "dividend_yield"]
    },
    "lynch": {
        "emoji": "🎯",
        "name": "Peter Lynch",
        "style": "Business simplicity, growth, understandable companies",
        "focus": ["roe", "pbv", "per", "dividend_yield"]
    },
    "munger": {
        "emoji": "🧠",
        "name": "Charlie Munger",
        "style": "Risk avoidance, simplicity, predictable business",
        "focus": ["der", "roe", "pbv", "per"]
    },
    "guru_id": {
        "emoji": "🇮🇩",
        "name": "Indonesia Value Guru",
        "style": "BUMN policy, regulation, seasonality, macro",
        "focus": ["per", "dividend_yield", "der", "roe"]
    }
}

# Debate Settings
DEBATE_CONFIG = {
    "max_rounds": 2,                    # Bull/Bear rounds
    "personas_pair": [                  # Which personas debate
        ("buffett", "graham"),          # Quality vs Safety
        ("lynch", "munger"),            # Growth vs Simplicity
    ],
    "consensus_threshold": 0.6,         # 60% agreement for strong signal
}

# Risk Management
RISK_CONFIG = {
    "max_position_size_pct": 0.05,      # 5% of portfolio per position
    "max_sector_concentration": 0.25,   # 25% max in one sector
    "max_portfolio_drawdown": 0.10,     # 10% max drawdown tolerance
    "min_liquidity_score": 0.3,         # Avoid illiquid stocks
}

# Signal Thresholds
SIGNAL_CONFIG = {
    "strong_buy_min_agreement": 0.8,    # 80% personas agree BUY
    "buy_min_agreement": 0.6,           # 60% personas agree BUY
    "hold_min_agreement": 0.4,          # 40%+ personas agree HOLD
    "pass_threshold": 0.3,              # < 30% agree = PASS
}

# Notion Integration
NOTION_CONFIG = {
    "portfolio_db_id": "362cd5f2-e4f1-80a3-86fa-000bfbb0fa2d",
    "transaction_db_id": "362cd5f2-e4f1-8039-b027-000bd51f4a33",
}

# Memory & Logging
MEMORY_CONFIG = {
    "memory_dir": "~/.hermes/profiles/pagupon-finance/memory",
    "memory_log_file": "trading_decisions.md",
    "reflection_enabled": True,         # Log realized returns vs predictions
    "holding_days": 5,                  # Default holding period for return calc
    "benchmark_ticker": "SPY",          # Alpha baseline
}

# Execution Defaults
EXECUTION_CONFIG = {
    "default_position_size_pct": 0.03,  # 3% of portfolio
    "entry_price_offset_pct": 0.02,     # Entry 2% below technical level
    "stop_loss_pct": 0.08,              # Stop loss at 8% below entry
    "take_profit_pct": 0.15,            # Take profit at 15% above entry
}

# Output Formatting
OUTPUT_CONFIG = {
    "telegram_max_length": 4000,        # Telegram message limit
    "include_persona_scores": True,
    "include_debate_summary": True,
    "include_risk_assessment": True,
    "include_decision_confidence": True,
}
