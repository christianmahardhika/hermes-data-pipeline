warning: unused import: `DateTime`
 --> src/cleaners/mod.rs:6:14
  |
6 | use chrono::{DateTime, Utc};
  |              ^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> src/unlimited/mod.rs:110:13
    |
110 |         let mut collector = Self {
    |             ----^^^^^^^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: associated function `url_hash` is never used
  --> src/social/collector.rs:96:8
   |
38 | impl SocialCollector {
   | -------------------- associated function in this implementation
...
96 |     fn url_hash(url: &str) -> String {
   |        ^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `news-collector` (lib) generated 3 warnings (run `cargo fix --lib -p news-collector` to apply 2 suggestions)
    Finished `release` profile [optimized] target(s) in 0.19s
     Running `target/release/news-collector idx-analyst --portfolio --full`
[2m2026-07-27T02:01:51.111848Z[0m [32m INFO[0m 📊 Starting IDX Analyst (5-persona debate engine)...
[2m2026-07-27T02:01:51.176436Z[0m [33m WARN[0m quoteSummary failed for KLBF, fallback to chart
[2m2026-07-27T02:01:51.267268Z[0m [32m INFO[0m 📊 Analyzing KLBF through debate pipeline...
KLBF
==================================================
📈 Price: Rp720 | 52W: Rp630 - Rp1,510
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp706 | Stop: Rp649 | Target: Rp811
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:51.344698Z[0m [33m WARN[0m quoteSummary failed for TLKM, fallback to chart
[2m2026-07-27T02:01:51.439903Z[0m [32m INFO[0m 📊 Analyzing TLKM through debate pipeline...
TLKM
==================================================
📈 Price: Rp2,630 | 52W: Rp2,350 - Rp3,990
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,577 | Stop: Rp2,371 | Target: Rp2,964
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:51.554108Z[0m [33m WARN[0m quoteSummary failed for BBRI, fallback to chart
[2m2026-07-27T02:01:51.683021Z[0m [32m INFO[0m 📊 Analyzing BBRI through debate pipeline...
BBRI
==================================================
📈 Price: Rp2,960 | 52W: Rp2,540 - Rp4,270
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,901 | Stop: Rp2,669 | Target: Rp3,336
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:51.753372Z[0m [33m WARN[0m quoteSummary failed for PTBA, fallback to chart
[2m2026-07-27T02:01:51.846956Z[0m [32m INFO[0m 📊 Analyzing PTBA through debate pipeline...
PTBA
==================================================
📈 Price: Rp2,390 | 52W: Rp2,170 - Rp3,220
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,342 | Stop: Rp2,155 | Target: Rp2,694
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:51.914048Z[0m [33m WARN[0m quoteSummary failed for BJTM, fallback to chart
[2m2026-07-27T02:01:51.999334Z[0m [32m INFO[0m 📊 Analyzing BJTM through debate pipeline...
BJTM
==================================================
📈 Price: Rp520 | 52W: Rp488 - Rp605
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp510 | Stop: Rp469 | Target: Rp586
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.070185Z[0m [33m WARN[0m quoteSummary failed for ADMF, fallback to chart
[2m2026-07-27T02:01:52.157409Z[0m [32m INFO[0m 📊 Analyzing ADMF through debate pipeline...
ADMF
==================================================
📈 Price: Rp8,250 | 52W: Rp7,675 - Rp9,200
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp8,085 | Stop: Rp7,438 | Target: Rp9,298
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.227416Z[0m [33m WARN[0m quoteSummary failed for TAPG, fallback to chart
[2m2026-07-27T02:01:52.306657Z[0m [32m INFO[0m 📊 Analyzing TAPG through debate pipeline...
TAPG
==================================================
📈 Price: Rp1,725 | 52W: Rp1,255 - Rp2,300
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp1,690 | Stop: Rp1,555 | Target: Rp1,944
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.378228Z[0m [33m WARN[0m quoteSummary failed for JPFA, fallback to chart
[2m2026-07-27T02:01:52.469047Z[0m [32m INFO[0m 📊 Analyzing JPFA through debate pipeline...
JPFA
==================================================
📈 Price: Rp2,130 | 52W: Rp1,510 - Rp2,970
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,087 | Stop: Rp1,920 | Target: Rp2,401
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.539522Z[0m [33m WARN[0m quoteSummary failed for TSPC, fallback to chart
[2m2026-07-27T02:01:52.643876Z[0m [32m INFO[0m 📊 Analyzing TSPC through debate pipeline...
TSPC
==================================================
📈 Price: Rp2,700 | 52W: Rp2,120 - Rp3,220
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,646 | Stop: Rp2,434 | Target: Rp3,043
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.710365Z[0m [33m WARN[0m quoteSummary failed for BMRI, fallback to chart
[2m2026-07-27T02:01:52.809136Z[0m [32m INFO[0m 📊 Analyzing BMRI through debate pipeline...
BMRI
==================================================
📈 Price: Rp4,130 | 52W: Rp3,650 - Rp5,375
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp4,047 | Stop: Rp3,724 | Target: Rp4,655
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:52.876316Z[0m [33m WARN[0m quoteSummary failed for ASII, fallback to chart
[2m2026-07-27T02:01:53.578635Z[0m [32m INFO[0m 📊 Analyzing ASII through debate pipeline...
ASII
==================================================
📈 Price: Rp4,980 | 52W: Rp4,350 - Rp7,475
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp4,880 | Stop: Rp4,490 | Target: Rp5,612
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-27T02:01:53.578774Z[0m [32m INFO[0m ✅ IDX Analyst complete!
