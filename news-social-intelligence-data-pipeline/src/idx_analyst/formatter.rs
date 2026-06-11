//! Output Formatter — RTI Business Style & Telegram Compact

use crate::idx_analyst::models::{Signal, StockData};
use crate::idx_analyst::debate::DebateResult;
use crate::idx_analyst::trader::TraderProposal;
use crate::idx_analyst::risk::RiskAssessment;
use crate::idx_analyst::config::CRITERIA;

/// Format number with thousand separator
fn fmt_rp(val: f64) -> String {
    let s = format!("{:.0}", val);
    let bytes = s.as_bytes();
    let mut result = String::new();
    let len = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 && b != b'-' {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// RTI Business formatter
pub struct RTIFormatter;

impl RTIFormatter {
    /// Full RTI Business format
    pub fn format_full(
        stock: &StockData,
        debate: &DebateResult,
        proposal: &TraderProposal,
        risk: &RiskAssessment,
    ) -> String {
        let risk_label = if risk.risk_score < 0.3 { "LOW" }
            else if risk.risk_score < 0.7 { "MEDIUM" }
            else { "HIGH" };
        let risk_status = if risk.is_approved { "✅" } else { "⚠️" };

        let mut out = format!(
            "{}\n{}\n📈 Price: Rp{} | 52W: Rp{} - Rp{}\n\
             P/E: {:.2} | P/BV: {:.2} | ROE: {:.2}% | D/E: {:.2} | DY: {:.2}%\n\n\
             {} SIGNAL: {} ({} confidence) | Bull: {:.0}%\n",
            stock.ticker, "=".repeat(50),
            fmt_rp(stock.current_price), fmt_rp(stock.low_52w), fmt_rp(stock.high_52w),
            stock.per, stock.pbv, stock.roe, stock.der, stock.dy,
            debate.final_signal.emoji(), debate.final_signal,
            debate.confidence, debate.bull_win_rate * 100.0,
        );

        out.push_str(&format!("Consensus: {}\n", debate.consensus_summary));

        if let (Some(e), Some(s), Some(t)) = (proposal.entry_price, proposal.stop_loss, proposal.take_profit) {
            let pos = proposal.position_size_pct.unwrap_or(0.0);
            out.push_str(&format!(
                "\n💼 Entry: Rp{} | Stop: Rp{} | Target: Rp{}\n\
                 Position: {:.1}% | Risk: {:.1}/10 ({}) {}\n",
                fmt_rp(e), fmt_rp(s), fmt_rp(t),
                pos * 100.0, risk.risk_score * 10.0, risk_label, risk_status
            ));
        }

        out
    }

    /// Compact Telegram format
    pub fn format_telegram(stock: &StockData, signal: &Signal) -> String {
        let mut criteria = Vec::new();
        if stock.per > 0.0 && stock.per < CRITERIA.per_max { criteria.push("✅P/E"); }
        if stock.dy > CRITERIA.dy_min { criteria.push("✅DY"); }
        if stock.roe > CRITERIA.roe_min { criteria.push("✅ROE"); }
        if stock.der > 0.0 && stock.der < CRITERIA.der_max { criteria.push("✅D/E"); }

        let c = if criteria.is_empty() { "❌ Criteria fail".into() } else { criteria.join(" ") };

        format!("{} **{}** — {}\nRp{} | P/E: {:.2} | DY: {:.2}% | ROE: {:.2}% | D/E: {:.2}\n{}",
            signal.emoji(), stock.ticker, signal, fmt_rp(stock.current_price),
            stock.per, stock.dy, stock.roe, stock.der, c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_rp() {
        assert_eq!(fmt_rp(6250.0), "6,250");
        assert_eq!(fmt_rp(100000.0), "100,000");
        assert_eq!(fmt_rp(42.0), "42");
    }

    #[test]
    fn test_telegram_format() {
        let stock = StockData {
            ticker: "BMRI".into(), current_price: 6000.0,
            per: 8.0, dy: 5.0, roe: 20.0, der: 0.5, ..Default::default()
        };
        let output = RTIFormatter::format_telegram(&stock, &Signal::Buy);
        assert!(output.contains("BMRI"));
        assert!(output.contains("✅P/E"));
    }
}
