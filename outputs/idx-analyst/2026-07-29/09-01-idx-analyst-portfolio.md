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
    Finished `release` profile [optimized] target(s) in 0.98s
     Running `target/release/news-collector idx-analyst --portfolio --full`
[2m2026-07-29T02:01:41.677729Z[0m [32m INFO[0m 📊 Starting IDX Analyst (5-persona debate engine)...
[2m2026-07-29T02:01:41.760114Z[0m [33m WARN[0m quoteSummary failed for KLBF, fallback to chart
[2m2026-07-29T02:01:41.869409Z[0m [32m INFO[0m 📊 Analyzing KLBF through debate pipeline...
KLBF
==================================================
📈 Price: Rp700 | 52W: Rp630 - Rp1,510
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp686 | Stop: Rp631 | Target: Rp789
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:41.953458Z[0m [33m WARN[0m quoteSummary failed for TLKM, fallback to chart
[2m2026-07-29T02:01:42.061708Z[0m [32m INFO[0m 📊 Analyzing TLKM through debate pipeline...
TLKM
==================================================
📈 Price: Rp2,560 | 52W: Rp2,350 - Rp3,990
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,509 | Stop: Rp2,308 | Target: Rp2,885
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:42.164840Z[0m [33m WARN[0m quoteSummary failed for BBRI, fallback to chart
[2m2026-07-29T02:01:42.259253Z[0m [32m INFO[0m 📊 Analyzing BBRI through debate pipeline...
BBRI
==================================================
📈 Price: Rp2,930 | 52W: Rp2,540 - Rp4,270
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,871 | Stop: Rp2,642 | Target: Rp3,302
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:42.334134Z[0m [33m WARN[0m quoteSummary failed for PTBA, fallback to chart
[2m2026-07-29T02:01:42.431847Z[0m [32m INFO[0m 📊 Analyzing PTBA through debate pipeline...
PTBA
==================================================
📈 Price: Rp2,330 | 52W: Rp2,170 - Rp3,220
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,283 | Stop: Rp2,101 | Target: Rp2,626
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:42.495960Z[0m [33m WARN[0m quoteSummary failed for BJTM, fallback to chart
[2m2026-07-29T02:01:42.576795Z[0m [32m INFO[0m 📊 Analyzing BJTM through debate pipeline...
BJTM
==================================================
📈 Price: Rp515 | 52W: Rp488 - Rp605
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp505 | Stop: Rp464 | Target: Rp580
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:42.801125Z[0m [33m WARN[0m quoteSummary failed for ADMF, fallback to chart
[2m2026-07-29T02:01:42.893719Z[0m [32m INFO[0m 📊 Analyzing ADMF through debate pipeline...
ADMF
==================================================
📈 Price: Rp8,250 | 52W: Rp7,675 - Rp9,200
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp8,085 | Stop: Rp7,438 | Target: Rp9,298
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:42.971471Z[0m [33m WARN[0m quoteSummary failed for TAPG, fallback to chart
[2m2026-07-29T02:01:43.044846Z[0m [32m INFO[0m 📊 Analyzing TAPG through debate pipeline...
TAPG
==================================================
📈 Price: Rp1,755 | 52W: Rp1,255 - Rp2,300
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp1,720 | Stop: Rp1,582 | Target: Rp1,978
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:43.116263Z[0m [33m WARN[0m quoteSummary failed for JPFA, fallback to chart
[2m2026-07-29T02:01:43.269184Z[0m [32m INFO[0m 📊 Analyzing JPFA through debate pipeline...
JPFA
==================================================
📈 Price: Rp2,100 | 52W: Rp1,510 - Rp2,970
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,058 | Stop: Rp1,893 | Target: Rp2,367
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:43.392564Z[0m [33m WARN[0m quoteSummary failed for TSPC, fallback to chart
[2m2026-07-29T02:01:43.489743Z[0m [32m INFO[0m 📊 Analyzing TSPC through debate pipeline...
TSPC
==================================================
📈 Price: Rp2,720 | 52W: Rp2,120 - Rp3,220
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp2,666 | Stop: Rp2,452 | Target: Rp3,065
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:43.560719Z[0m [33m WARN[0m quoteSummary failed for BMRI, fallback to chart
[2m2026-07-29T02:01:43.656201Z[0m [32m INFO[0m 📊 Analyzing BMRI through debate pipeline...
BMRI
==================================================
📈 Price: Rp4,100 | 52W: Rp3,650 - Rp5,375
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp4,018 | Stop: Rp3,697 | Target: Rp4,621
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:43.743358Z[0m [33m WARN[0m quoteSummary failed for ASII, fallback to chart
[2m2026-07-29T02:01:43.847018Z[0m [32m INFO[0m 📊 Analyzing ASII through debate pipeline...
ASII
==================================================
📈 Price: Rp4,970 | 52W: Rp4,350 - Rp7,475
P/E: 0.00 | P/BV: 0.00 | ROE: 0.00% | D/E: 0.00 | DY: 0.00%

📈 SIGNAL: BUY (MEDIUM confidence) | Bull: 50%
Consensus: Modest bull consensus — quality outweighs risk.

💼 Entry: Rp4,871 | Stop: Rp4,481 | Target: Rp5,601
Position: 3.0% | Risk: 3.0/10 (MEDIUM) ✅


[2m2026-07-29T02:01:43.847370Z[0m [32m INFO[0m ✅ IDX Analyst complete!
