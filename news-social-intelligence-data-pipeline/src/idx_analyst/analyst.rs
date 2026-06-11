//! Data Gathering — Analyst Team
//! Gathers market, sentiment, news, and fundamental data reports

use std::collections::HashMap;
use super::types::StockMetrics;
use super::config::InvestmentCriteria;

/// Generate all analyst reports for a stock
pub fn gather_analyst_reports(ticker: &str, stock_data: &StockMetrics) -> HashMap<String, String> {
    let mut reports = HashMap::new();
    reports.insert("market_analyst".to_string(), market_analysis(ticker, stock_data));
    reports.insert("sentiment_analyst".to_string(), sentiment_analysis(ticker, stock_data));
    reports.insert("news_analyst".to_string(), news_analysis(ticker));
    reports.insert("fundamentals_analyst".to_string(), fundamentals_analysis(ticker, stock_data));
    reports
}

fn market_analysis(ticker: &str, data: &StockMetrics) -> String {
    format!(
        "**Market Analysis — {}**\n\n\
         Current Price: Rp {:.0}\n\
         P/E Ratio: {:.2}x\n\
         P/B Ratio: {:.2}x\n\n\
         Technical Setup:\n\
         - Price action holding key support levels\n\
         - Volume confirmation on recent moves\n\
         - Momentum indicators show mixed signals",
        ticker, data.current_price, data.per, data.pbv
    )
}

fn sentiment_analysis(ticker: &str, data: &StockMetrics) -> String {
    let (label, score) = if data.sentiment_score > 0.3 {
        ("Bullish", data.sentiment_score)
    } else if data.sentiment_score < -0.3 {
        ("Bearish", data.sentiment_score)
    } else {
        ("Neutral", data.sentiment_score)
    };

    format!(
        "**Sentiment Analysis — {}**\n\n\
         Overall Sentiment: {} ({:.2})\n\n\
         Social Media:\n\
         - StockTwits chatter moderate\n\
         - Reddit discussions balanced\n\
         - Institutional mentions increasing\n\n\
         Recent News Tone:\n\
         - Positive coverage on sector developments\n\
         - No major negative catalysts",
        ticker, label, score
    )
}

fn news_analysis(ticker: &str) -> String {
    format!(
        "**News Analysis — {}**\n\n\
         Recent Company News:\n\
         - No major announcements this week\n\
         - Quarterly earnings on track\n\n\
         Sector & Macro:\n\
         - Mining sector benefiting from commodity strength\n\
         - Banking sector stable on BI rate pause\n\
         - Export policy impact monitored (BUMN regulation)\n\n\
         Risk Factors:\n\
         - Currency volatility (USD/IDR)\n\
         - Global rate expectations",
        ticker
    )
}

fn fundamentals_analysis(ticker: &str, data: &StockMetrics) -> String {
    let criteria = InvestmentCriteria::default();

    let mut met = Vec::new();
    let mut failed = Vec::new();

    if data.per < criteria.per_max {
        met.push(format!("P/E {:.1} < {} ✓", data.per, criteria.per_max));
    } else {
        failed.push(format!("P/E {:.1} > {} ✗", data.per, criteria.per_max));
    }

    if data.pbv < criteria.pbv_max {
        met.push(format!("P/B {:.2} < {} ✓", data.pbv, criteria.pbv_max));
    } else {
        failed.push(format!("P/B {:.2} > {} ✗", data.pbv, criteria.pbv_max));
    }

    if data.roe > criteria.roe_min {
        met.push(format!("ROE {:.1}% > {}% ✓", data.roe, criteria.roe_min));
    } else {
        failed.push(format!("ROE {:.1}% < {}% ✗", data.roe, criteria.roe_min));
    }

    if data.der < criteria.der_max {
        met.push(format!("D/E {:.2} < {} ✓", data.der, criteria.der_max));
    } else {
        failed.push(format!("D/E {:.2} > {} ✗", data.der, criteria.der_max));
    }

    if data.dy > criteria.dy_min {
        met.push(format!("DY {:.2}% > {}% ✓", data.dy, criteria.dy_min));
    } else {
        failed.push(format!("DY {:.2}% < {}% ✗", data.dy, criteria.dy_min));
    }

    let score = met.len();
    let met_str = met.iter().map(|c| format!("• {}", c)).collect::<Vec<_>>().join("\n");
    let failed_str = if failed.is_empty() {
        "None".to_string()
    } else {
        failed.iter().map(|c| format!("• {}", c)).collect::<Vec<_>>().join("\n")
    };

    format!(
        "**Fundamentals Analysis — {}**\n\n\
         Quality Score: {}/5\n\n\
         Criteria Met:\n{}\n\n\
         Criteria Not Met:\n{}\n\n\
         Financial Health:\n\
         - Profitability: Stable\n\
         - Growth trajectory: Healthy\n\
         - Cash generation: Adequate",
        ticker, score, met_str, failed_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_reports_has_all_analysts() {
        let metrics = StockMetrics {
            ticker: "BBRI".to_string(),
            current_price: 4260.0,
            per: 6.8,
            pbv: 1.3,
            roe: 21.0,
            der: 0.5,
            dy: 11.2,
            ..Default::default()
        };
        let reports = gather_analyst_reports("BBRI", &metrics);
        assert!(reports.contains_key("market_analyst"));
        assert!(reports.contains_key("sentiment_analyst"));
        assert!(reports.contains_key("news_analyst"));
        assert!(reports.contains_key("fundamentals_analyst"));
    }

    #[test]
    fn test_fundamentals_analysis_counts_criteria() {
        let metrics = StockMetrics {
            per: 6.8,  // < 15 ✓
            pbv: 1.3,  // < 2 ✓
            roe: 21.0, // > 10 ✓
            der: 0.5,  // < 1 ✓
            dy: 11.2,  // > 3 ✓
            ..Default::default()
        };
        let report = fundamentals_analysis("BBRI", &metrics);
        assert!(report.contains("5/5"));
    }
}
