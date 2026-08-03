//! News Collector CLI - IDX Analyst Focus
//! 
//! Minimal main for running idx-analyst command
//! 
//! Usage:
//!   news-collector idx-analyst --portfolio --full

use anyhow::Result;
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("idx-analyst");

    match command {
        "idx-analyst" => run_idx_analyst(&args).await?,
        "digest" => run_idx_digest(&args).await?,
        _ => {
            println!("News Collector - IDX Analyst");
            println!();
            println!("Usage: news-collector idx-analyst [options] [TICKERS...]");
            println!("       news-collector digest");
            println!();
            println!("Options:");
            println!("  --portfolio  Analyze all portfolio stocks");
            println!("  --full       Full RTI Business format (vs compact)");
            println!("  --mock       Use mock data (no network)");
            println!();
            println!("Examples:");
            println!("  news-collector idx-analyst --portfolio --full");
            println!("  news-collector digest");
        }
    }

    Ok(())
}

async fn run_idx_analyst(args: &[String]) -> Result<()> {
    use news_collector::idx_analyst::{
        IdxAnalyst, IdxConfig, PORTFOLIO_STOCKS,
        data_source::{mock_stock_data, YahooFinanceSource},
        formatter::RTIFormatter,
    };

    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");

    // Subcommand: digest — unified fetch + analyze for cron
    if sub == "digest" {
        return run_idx_digest(args).await;
    }

    info!("📊 Starting IDX Analyst (5-persona debate engine)...");

    let config = IdxConfig::default();

    // Parse args
    let mut tickers: Vec<String> = Vec::new();
    let mut mock_mode = false;
    let mut portfolio_mode = false;
    let mut full_mode = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--mock" => { mock_mode = true; i += 1; }
            "--portfolio" => { portfolio_mode = true; i += 1; }
            "--full" => { full_mode = true; i += 1; }
            "--help" | "-h" => {
                println!("IDX AI Analyst — 5-Persona Debate Engine (Rust)");
                println!();
                println!("Usage: news-collector idx-analyst [subcommand] [options] [TICKERS...]");
                println!();
                println!("Subcommands:");
                println!("  digest     Fetch + analyze all portfolio (cron mode)");
                println!();
                println!("Options:");
                println!("  --mock       Use mock data (no network)");
                println!("  --portfolio  Analyze all portfolio stocks");
                println!("  --full       Full RTI Business format (vs compact)");
                println!();
                println!("Examples:");
                println!("  news-collector idx-analyst BMRI BBRI --mock");
                println!("  news-collector idx-analyst --portfolio --mock --full");
                println!("  news-collector idx-analyst digest");
                println!("  news-collector idx-analyst digest --mock");
                return Ok(());
            }
            other => {
                if !other.starts_with('-') {
                    tickers.push(other.to_uppercase());
                }
                i += 1;
            }
        }
    }

    // Determine tickers
    if portfolio_mode || tickers.is_empty() {
        tickers = PORTFOLIO_STOCKS.iter().map(|s| s.to_string()).collect();
    }

    let analyst = IdxAnalyst::new(config)?;

    for ticker in &tickers {
        let stock_data = if mock_mode {
            mock_stock_data(ticker)
        } else {
            let source = YahooFinanceSource::new()?;
            match source.fetch_fundamentals(ticker).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::warn!("⚠️ Yahoo failed for {}: {}, using mock", ticker, e);
                    mock_stock_data(ticker)
                }
            }
        };

        match analyst.analyze_stock(ticker, &stock_data).await {
            Ok(result) => {
                if full_mode {
                    println!("{}", RTIFormatter::format_full(
                        &result.stock_data, &result.debate, &result.proposal, &result.risk
                    ));
                } else {
                    println!("{}", RTIFormatter::format_telegram(
                        &result.stock_data, &result.debate.final_signal
                    ));
                }
                println!();
            }
            Err(e) => {
                error!("❌ Failed to analyze {}: {}", ticker, e);
            }
        }
    }

    info!("✅ IDX Analyst complete!");
    Ok(())
}

async fn run_idx_digest(args: &[String]) -> Result<()> {
    use news_collector::idx_analyst::{
        IdxAnalyst, IdxConfig, PORTFOLIO_STOCKS,
        data_source::{mock_stock_data, YahooFinanceSource},
        formatter::RTIFormatter,
    };

    info!("📊 IDX Digest (cron mode)...");

    let config = IdxConfig::default();
    let analyst = IdxAnalyst::new(config)?;

    // Check for --mock flag
    let mock_mode = args.iter().any(|a| a == "--mock");

    let mut digest = String::from("📈 **IDX Daily Digest**\n\n");

    for ticker in PORTFOLIO_STOCKS {
        let stock_data = if mock_mode {
            mock_stock_data(ticker)
        } else {
            let source = YahooFinanceSource::new()?;
            match source.fetch_fundamentals(ticker).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::warn!("⚠️ Yahoo failed for {}: {}, using mock", ticker, e);
                    mock_stock_data(ticker)
                }
            }
        };

        match analyst.analyze_stock(ticker, &stock_data).await {
            Ok(result) => {
                let telegram_msg = RTIFormatter::format_telegram(
                    &result.stock_data, &result.debate.final_signal
                );
                digest.push_str(&telegram_msg);
                digest.push_str("\n");
            }
            Err(e) => {
                error!("❌ Failed to analyze {}: {}", ticker, e);
                digest.push_str(&format!("❌ {} - Error: {}\n", ticker, e));
            }
        }
    }

    println!("{}", digest);
    info!("✅ IDX Digest complete!");
    Ok(())
}
