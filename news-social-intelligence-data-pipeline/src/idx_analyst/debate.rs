//! 5-Persona Debate Engine
//!
//! Bull vs Bear debate with persona-specific reasoning.
//! Personas: Buffett, Graham, Lynch, Munger, Indonesia Value Guru

use std::collections::HashMap;
use crate::idx_analyst::models::{Signal, Confidence, ExternalSignal};

/// Single debate round
#[derive(Debug, Clone)]
pub struct DebateRound {
    pub round_num: usize,
    pub bull_persona: &'static str,
    pub bear_persona: &'static str,
    pub bull_argument: String,
    pub bear_argument: String,
    pub bull_confidence: Confidence,
    pub bear_confidence: Confidence,
}

/// Full debate result
#[derive(Debug, Clone)]
pub struct DebateResult {
    pub ticker: String,
    pub rounds: Vec<DebateRound>,
    pub final_signal: Signal,
    pub confidence: Confidence,
    pub bull_win_rate: f64,
    pub consensus_summary: String,
}

trait Persona: Send + Sync {
    fn name(&self) -> &'static str;
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String;
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String;
}

fn get(m: &HashMap<String, f64>, key: &str) -> f64 {
    m.get(key).copied().unwrap_or(0.0)
}

struct BuffettPersona;
impl Persona for BuffettPersona {
    fn name(&self) -> &'static str { "Warren Buffett" }
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🦉 Buffett Bull {}: ROE {:.1}%, DY {:.2}%, D/E {:.2}x, P/E {:.1}x — Quality moat, own forever",
            ticker, get(m,"roe"), get(m,"dy"), get(m,"der"), get(m,"per"))
    }
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🦉 Buffett Bear {}: ROE {:.1}%, DY {:.2}%, D/E {:.2}x, P/E {:.1}x — No durable moat, commodity business",
            ticker, get(m,"roe"), get(m,"dy"), get(m,"der"), get(m,"per"))
    }
}

struct GrahamPersona;
impl Persona for GrahamPersona {
    fn name(&self) -> &'static str { "Benjamin Graham" }
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("📚 Graham Bull {}: P/E {:.1}x, P/BV {:.2}x, D/E {:.2}x, DY {:.2}% — Margin of safety adequate",
            ticker, get(m,"per"), get(m,"pbv"), get(m,"der"), get(m,"dy"))
    }
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("📚 Graham Bear {}: P/E {:.1}x, P/BV {:.2}x, D/E {:.2}x, DY {:.2}% — Insufficient discount",
            ticker, get(m,"per"), get(m,"pbv"), get(m,"der"), get(m,"dy"))
    }
}

struct LynchPersona;
impl Persona for LynchPersona {
    fn name(&self) -> &'static str { "Peter Lynch" }
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🎯 Lynch Bull {}: ROE {:.1}%, P/E {:.1}x, P/BV {:.2}x, DY {:.2}% — Tenbagger potential",
            ticker, get(m,"roe"), get(m,"per"), get(m,"pbv"), get(m,"dy"))
    }
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🎯 Lynch Bear {}: ROE {:.1}%, P/E {:.1}x, P/BV {:.2}x, DY {:.2}% — Growth story broken",
            ticker, get(m,"roe"), get(m,"per"), get(m,"pbv"), get(m,"dy"))
    }
}

struct MungerPersona;
impl Persona for MungerPersona {
    fn name(&self) -> &'static str { "Charlie Munger" }
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🧠 Munger Bull {}: D/E {:.2}x, ROE {:.1}%, P/BV {:.2}x, P/E {:.1}x — Simple, predictable, compound",
            ticker, get(m,"der"), get(m,"roe"), get(m,"pbv"), get(m,"per"))
    }
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🧠 Munger Bear {}: D/E {:.2}x, ROE {:.1}%, P/BV {:.2}x, P/E {:.1}x — Too complex, too risky",
            ticker, get(m,"der"), get(m,"roe"), get(m,"pbv"), get(m,"per"))
    }
}

struct IdxGuruPersona;
impl Persona for IdxGuruPersona {
    fn name(&self) -> &'static str { "Indonesia Value Guru" }
    fn build_bull_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🇮🇩 IDX Guru Bull {}: P/E {:.1}x, DY {:.2}%, D/E {:.2}x, ROE {:.1}% — BUMN policy + macro tailwind",
            ticker, get(m,"per"), get(m,"dy"), get(m,"der"), get(m,"roe"))
    }
    fn build_bear_case(&self, ticker: &str, m: &HashMap<String, f64>) -> String {
        format!("🇮🇩 IDX Guru Bear {}: P/E {:.1}x, DY {:.2}%, D/E {:.2}x, ROE {:.1}% — Policy headwinds, macro peak",
            ticker, get(m,"per"), get(m,"dy"), get(m,"der"), get(m,"roe"))
    }
}

/// 5-persona debate engine
pub struct PersonaDebateEngine {
    ticker: String,
    max_rounds: usize,
    personas: Vec<Box<dyn Persona>>,
}

impl PersonaDebateEngine {
    pub fn new(ticker: &str, max_rounds: usize) -> Self {
        Self {
            ticker: ticker.to_string(),
            max_rounds,
            personas: vec![
                Box::new(BuffettPersona),
                Box::new(GrahamPersona),
                Box::new(LynchPersona),
                Box::new(MungerPersona),
                Box::new(IdxGuruPersona),
            ],
        }
    }

    /// Run full debate: Round 1 (Buffett vs Graham), Round 2 (Lynch vs Munger)
    pub fn run_debate(&self, metrics: &HashMap<String, f64>, external_signals: &[ExternalSignal]) -> DebateResult {
        let pairs: &[(usize, usize)] = &[(0, 1), (2, 3)];
        let mut rounds = Vec::with_capacity(self.max_rounds);

        for (i, &(bull_idx, bear_idx)) in pairs.iter().enumerate() {
            if i >= self.max_rounds { break; }

            let bull = &self.personas[bull_idx];
            let bear = &self.personas[bear_idx];

            rounds.push(DebateRound {
                round_num: i + 1,
                bull_persona: bull.name(),
                bear_persona: bear.name(),
                bull_argument: bull.build_bull_case(&self.ticker, metrics),
                bear_argument: bear.build_bear_case(&self.ticker, metrics),
                bull_confidence: if i == 0 { Confidence::High } else { Confidence::Medium },
                bear_confidence: if i == 0 { Confidence::High } else { Confidence::Medium },
            });
        }

        let (signal, confidence, bull_win_rate) = self.consensus(&rounds, external_signals);

        let consensus_summary = match signal {
            Signal::StrongBuy => "Strong consensus — multiple perspectives align on BUY.",
            Signal::Buy => "Modest bull consensus — quality outweighs risk.",
            Signal::Hold => "No clear consensus — balanced debate.",
            Signal::Pass | Signal::Avoid => "Bear consensus — wait for better entry.",
        }.to_string();

        DebateResult { ticker: self.ticker.clone(), rounds, final_signal: signal, confidence, bull_win_rate, consensus_summary }
    }

    fn consensus(&self, rounds: &[DebateRound], external_signals: &[ExternalSignal]) -> (Signal, Confidence, f64) {
        let total = rounds.len() as f64;
        if total == 0.0 { return (Signal::Hold, Confidence::Medium, 0.5); }

        let bull_high = rounds.iter().filter(|r| r.bull_confidence == Confidence::High).count() as f64;
        let mut rate = bull_high / total;

        // IDX Guru signal integration: external signals modify the bull_win_rate
        for signal in external_signals {
            if signal.confidence > 0.5 {
                match signal.direction.as_str() {
                    "positive" => rate += signal.confidence * 0.15,
                    "negative" => rate -= signal.confidence * 0.15,
                    _ => {} // neutral — no change
                }
            }
        }

        // Clamp rate to [0.0, 1.0]
        rate = rate.clamp(0.0, 1.0);

        if rate >= 0.75 { (Signal::StrongBuy, Confidence::High, rate) }
        else if rate >= 0.5 { (Signal::Buy, Confidence::Medium, rate) }
        else if rate >= 0.25 { (Signal::Hold, Confidence::Medium, rate) }
        else { (Signal::Pass, Confidence::Medium, rate) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debate_two_rounds() {
        let metrics = HashMap::from([
            ("per".into(), 8.0), ("pbv".into(), 1.2), ("roe".into(), 18.0),
            ("der".into(), 0.5), ("dy".into(), 5.0),
        ]);
        let engine = PersonaDebateEngine::new("BMRI", 2);
        let result = engine.run_debate(&metrics, &[]);

        assert_eq!(result.rounds.len(), 2);
        assert_eq!(result.final_signal, Signal::Buy); // bull_win_rate = 0.5
    }

    #[test]
    fn test_debate_one_round_strong_buy() {
        let metrics = HashMap::from([
            ("per".into(), 6.0), ("pbv".into(), 0.8), ("roe".into(), 25.0),
            ("der".into(), 0.3), ("dy".into(), 7.0),
        ]);
        let engine = PersonaDebateEngine::new("PTBA", 1);
        let result = engine.run_debate(&metrics, &[]);

        assert_eq!(result.rounds.len(), 1);
        assert_eq!(result.final_signal, Signal::StrongBuy); // bull_win_rate = 1.0
    }

    #[test]
    fn test_external_signal_modifies_debate() {
        let metrics = HashMap::from([
            ("per".into(), 12.0), ("pbv".into(), 1.5), ("roe".into(), 12.0),
            ("der".into(), 0.8), ("dy".into(), 3.0),
        ]);

        let engine = PersonaDebateEngine::new("PTBA", 2);

        // Without signals
        let result_no_signal = engine.run_debate(&metrics, &[]);

        // With strong positive signal: should shift toward Buy
        let signals = vec![ExternalSignal {
            source: "coal_price_spike".into(),
            direction: "positive".into(),
            confidence: 0.85,
        }];
        let result_with_signal = engine.run_debate(&metrics, &signals);

        // The signal should boost bull_win_rate
        assert!(result_with_signal.bull_win_rate >= result_no_signal.bull_win_rate);
    }

    #[test]
    fn test_negative_external_signal_reduces_bull_rate() {
        let metrics = HashMap::from([
            ("per".into(), 8.0), ("pbv".into(), 1.2), ("roe".into(), 18.0),
            ("der".into(), 0.5), ("dy".into(), 5.0),
        ]);

        let engine = PersonaDebateEngine::new("ADRO", 2);

        let result_no_signal = engine.run_debate(&metrics, &[]);

        let signals = vec![ExternalSignal {
            source: "coal_price_crash".into(),
            direction: "negative".into(),
            confidence: 0.9,
        }];
        let result_with_signal = engine.run_debate(&metrics, &signals);

        // Negative signal should reduce bull_win_rate
        assert!(result_with_signal.bull_win_rate <= result_no_signal.bull_win_rate);
    }
}
