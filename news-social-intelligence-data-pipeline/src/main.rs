//! News Collector CLI
//! 
//! Rust ETL Pipeline for Indonesian news with Prof Jiang event extraction
//! 
//! Usage:
//!   news-collector run           # Run full pipeline once
//!   news-collector collect       # Collect RSS feeds only
//!   news-collector clean         # Clean pending raw feeds
//!   news-collector label         # Label cleaned articles
//!   news-collector embed         # Embed and store in Qdrant
//!   news-collector health        # Check Kiro health
//!   news-collector daemon        # Run as scheduled daemon

use anyhow::Result;
use news_collector::{
    Config, RssCollector, ArticleCleaner, KiroLabeler, TeiEmbedder, UnlimitedCollector,
    storage::Database,
    health::{KiroHealth, SelfHealingMonitor},
};
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

    let config = Config::default();
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("run");

    match command {
        "run" => run_pipeline(&config).await?,
        "collect" => run_collect(&config).await?,
        "clean" => run_clean(&config).await?,
        "label" => run_label(&config).await?,
        "embed" => run_embed(&config).await?,
        "health" => run_health(&config).await?,
        "daemon" => run_daemon(&config).await?,
        "prune" => run_prune(&config).await?,
        "social" => run_social(&config, &args).await?,
        "unlimited" => run_unlimited(&config, &args).await?,
        "idx-analyst" => run_idx_analyst(&args).await?,
        "economic" => run_economic(&args).await?,
        _ => {
            println!("News Collector - Rust ETL Pipeline");
            println!();
            println!("Usage: news-collector <command>");
            println!();
            println!("Commands:");
            println!("  run      Run full pipeline once");
            println!("  collect  Collect RSS feeds only");
            println!("  clean    Clean pending raw feeds");
            println!("  label    Label cleaned articles (Kiromania)");
            println!("  embed    Embed and store in Qdrant");
            println!("  health   Check Kiro health");
            println!("  daemon   Run as scheduled daemon (15min)");
            println!("  prune    Prune ingested records from SQLite");
            println!("  social   Run social intelligence collector");
            println!("  unlimited Run unlimited news daemon (Rust + TEI 768-dim)");
            println!("  idx-analyst Run IDX stock analyst (5-persona debate)");
            println!("  economic Run economic data collector (commodities, crypto, FRED, BI)");
            println!();
            println!("Social subcommands:");
            println!("  social --query \"AI\" --depth quick");
            println!("  social --front-page");
        }
    }

    Ok(())
}

/// Run full pipeline: collect → clean → label → embed
async fn run_pipeline(config: &Config) -> Result<()> {
    info!("🚀 Starting full pipeline...");
    
    let db = Database::open(&config.db_path)?;

    // Phase 1: Collect
    info!("📥 Phase 1: Collecting RSS feeds...");
    let collector = RssCollector::new();
    let collect_stats = collector.collect_all(&db).await?;
    info!("{}", collect_stats);

    // Phase 2: Clean
    info!("🧹 Phase 2: Cleaning articles...");
    let cleaner = ArticleCleaner::new();
    let clean_stats = cleaner.process_pending(&db, 1000).await?;
    info!("{}", clean_stats);

    // Phase 3: Label
    info!("🏷️ Phase 3: Labeling with Prof Jiang model...");
    let labeler = KiroLabeler::new(&config.kiro_url, &config.kiro_api_key);
    let label_stats = labeler.process_pending(&db, config.label_batch_size as i64).await?;
    info!("{}", label_stats);

    // Phase 4: Embed
    info!("📊 Phase 4: Embedding and storing in Qdrant...");
    let embedder = TeiEmbedder::new(
        &config.tei_url,
        &config.qdrant_url,
        &config.collection_name,
        config.similarity_threshold,
    ).await?;
    
    // Get pending labeled articles
    let pending = db.get_pending_embed(100)?;
    if !pending.is_empty() {
        let embed_stats = embedder.process_batch(pending).await?;
        info!("{}", embed_stats);
    }

    // Phase 5: Self-healing check
    info!("🔧 Phase 5: Self-healing check...");
    let monitor = SelfHealingMonitor::new(&config.kiro_url, &config.kiro_api_key, 5);
    monitor.check_and_heal(&db).await?;

    info!("✅ Pipeline complete!");
    Ok(())
}

/// Collect RSS feeds only
async fn run_collect(config: &Config) -> Result<()> {
    info!("📥 Collecting RSS feeds...");
    let db = Database::open(&config.db_path)?;
    let collector = RssCollector::new();
    let stats = collector.collect_all(&db).await?;
    info!("{}", stats);
    Ok(())
}

/// Clean pending raw feeds
async fn run_clean(config: &Config) -> Result<()> {
    info!("🧹 Cleaning articles...");
    let db = Database::open(&config.db_path)?;
    let cleaner = ArticleCleaner::new();
    let stats = cleaner.process_pending(&db, 1000).await?;
    info!("{}", stats);
    Ok(())
}

/// Label cleaned articles
async fn run_label(config: &Config) -> Result<()> {
    info!("🏷️ Labeling articles...");
    let db = Database::open(&config.db_path)?;
    let labeler = KiroLabeler::new(&config.kiro_url, &config.kiro_api_key);
    let stats = labeler.process_pending(&db, config.label_batch_size as i64).await?;
    info!("{}", stats);
    Ok(())
}

/// Embed and store in Qdrant
async fn run_embed(config: &Config) -> Result<()> {
    info!("📊 Embedding and storing...");
    let db = Database::open(&config.db_path)?;
    let embedder = TeiEmbedder::new(
        &config.tei_url,
        &config.qdrant_url,
        &config.collection_name,
        config.similarity_threshold,
    ).await?;
    
    let pending = db.get_pending_embed(100)?;
    if pending.is_empty() {
        info!("No pending articles to embed");
        return Ok(());
    }
    
    let stats = embedder.process_batch(pending).await?;
    
    // Mark as ingested in SQLite
    if !stats.ingested_ids.is_empty() {
        db.mark_ingested(&stats.ingested_ids, None)?;
        info!("Marked {} articles as ingested", stats.ingested_ids.len());
    }
    
    info!("{}", stats);
    Ok(())
}

/// Check Kiro health
async fn run_health(config: &Config) -> Result<()> {
    info!("🏥 Checking Kiro health...");
    let health = KiroHealth::new(&config.kiro_url, &config.kiro_api_key);
    
    match health.check().await {
        Ok(true) => {
            info!("✅ Kiro is healthy");
        }
        Ok(false) => {
            error!("❌ Kiro is unhealthy");
            info!("Attempting reauthentication...");
            health.reauthenticate().await?;
            
            if health.check().await? {
                info!("✅ Kiro is now healthy after reauth");
            } else {
                error!("❌ Kiro still unhealthy after reauth");
            }
        }
        Err(e) => {
            error!("❌ Health check error: {}", e);
        }
    }
    
    Ok(())
}

/// Prune ingested records
async fn run_prune(config: &Config) -> Result<()> {
    info!("🗑️ Pruning ingested records...");
    let db = Database::open(&config.db_path)?;
    let count = db.prune_ingested()?;
    info!("Pruned {} records", count);
    Ok(())
}

/// Run as scheduled daemon
async fn run_daemon(config: &Config) -> Result<()> {
    info!("🔄 Starting daemon mode (every 15 minutes)...");
    
    let interval = tokio::time::Duration::from_secs(15 * 60); // 15 minutes
    
    loop {
        info!("⏰ Scheduled run starting...");
        if let Err(e) = run_pipeline(config).await {
            error!("Pipeline error: {}", e);
        }
        
        info!("💤 Sleeping for 15 minutes...");
        tokio::time::sleep(interval).await;
    }
}

/// Run social intelligence collector
async fn run_social(config: &Config, args: &[String]) -> Result<()> {
    use news_collector::social::{collector::SocialCollector, Depth};

    info!("🌐 Starting social intelligence collector...");

    // Parse args: --query "..." --depth quick/default/deep --front-page --no-store
    let mut query: Option<String> = None;
    let mut depth = Depth::Default;
    let mut front_page = false;
    let mut store = true;

    let mut i = 2; // skip "news-collector" and "social"
    while i < args.len() {
        match args[i].as_str() {
            "--query" | "-q" => {
                if i + 1 < args.len() {
                    query = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--depth" => {
                if i + 1 < args.len() {
                    depth = args[i + 1].parse().unwrap_or(Depth::Default);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--front-page" => {
                front_page = true;
                i += 1;
            }
            "--no-store" => {
                store = false;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    if query.is_none() && !front_page {
        println!("Usage: news-collector social [options]");
        println!();
        println!("Options:");
        println!("  --query, -q <text>   Search query (required unless --front-page)");
        println!("  --depth <level>      quick, default, or deep (default: default)");
        println!("  --front-page         Get HackerNews front page instead of searching");
        println!("  --no-store           Don't store in Qdrant, just print results");
        return Ok(());
    }

    let collector = SocialCollector::new(&config.qdrant_url, &config.tei_url).await?;

    if front_page {
        info!("📥 Collecting HackerNews front page...");
        let (items, stats) = collector.collect_hackernews(None, depth, store).await?;
        println!("\n=== HackerNews Front Page ({} items) ===", items.len());
        for item in items.iter().take(10) {
            println!("  [{}] {}", item.score, &item.title[..item.title.len().min(70)]);
        }
        println!("\n📊 Stats: {}", stats);
    } else if let Some(ref q) = query {
        info!("📥 Collecting all sources for '{}'...", q);
        let stats = collector.collect_all(q, depth, None, store).await?;
        println!("\n📊 Social collection complete: {}", stats);
    }

    Ok(())
}

/// Run unlimited news collector daemon (Rust + TEI 768-dim)
async fn run_unlimited(config: &Config, args: &[String]) -> Result<()> {
    info!("🚀 Starting unlimited news collector (Rust + TEI 768-dim)...");

    // Parse args: --interval <minutes>
    let mut interval_minutes: u64 = 15; // default 15 minutes

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--interval" | "-i" => {
                if i + 1 < args.len() {
                    interval_minutes = args[i + 1].parse().unwrap_or(15);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Unlimited News Collector - Rust + TEI 768-dim");
                println!();
                println!("Usage: news-collector unlimited [options]");
                println!();
                println!("Options:");
                println!("  --interval, -i <minutes>  Collection interval (default: 15)");
                println!();
                println!("Indonesian feeds: Tempo, CNN Indonesia, Antara, Republika, Merdeka, Tribunnews, Jpnn");
                println!("International feeds: BBC Business/World, Reuters, Google News Business");
                println!();
                println!("Collections created:");
                println!("  - indonesian_news_768 (768-dim vectors)");
                println!("  - international_news_768 (768-dim vectors)");
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    let mut collector = UnlimitedCollector::new(
        &config.tei_url,
        &config.qdrant_url,
        config.similarity_threshold,
    ).await?;

    collector.run_daemon(interval_minutes).await?;

    Ok(())
}

/// Run IDX stock analyst (5-persona debate engine)
async fn run_idx_analyst(args: &[String]) -> Result<()> {
    use news_collector::idx_analyst::{
        IdxAnalyst, IdxConfig, data_source::mock_stock_data, formatter::RTIFormatter,
    };

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
                println!("Usage: news-collector idx-analyst [options] [TICKERS...]");
                println!();
                println!("Options:");
                println!("  --mock       Use mock data (no network)");
                println!("  --portfolio  Analyze all portfolio stocks");
                println!("  --full       Full RTI Business format (vs compact)");
                println!();
                println!("Examples:");
                println!("  news-collector idx-analyst BMRI BBRI --mock");
                println!("  news-collector idx-analyst --portfolio --mock --full");
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
        tickers = news_collector::idx_analyst::PORTFOLIO_STOCKS
            .iter().map(|s| s.to_string()).collect();
    }

    let analyst = IdxAnalyst::new(config)?;

    for ticker in &tickers {
        let stock_data = if mock_mode {
            mock_stock_data(ticker)
        } else {
            // Try Yahoo Finance, fallback to mock
            let source = news_collector::idx_analyst::data_source::YahooFinanceSource::new()?;
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

/// Run economic data collector (commodities, crypto, FRED, Bank Indonesia)
async fn run_economic(args: &[String]) -> Result<()> {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("all");
    let arango = news_collector::arangodb::ArangoClient::new()?;

    match sub {
        "commodity" => {
            info!("📊 Fetching commodity prices...");
            let collector = news_collector::economic::yahoo_commodities::YahooCommodityCollector::new()?;
            let stats = collector.collect_all(&arango).await?;
            info!("{}", stats);
        }
        "crypto" => {
            info!("📊 Fetching crypto prices...");
            let collector = news_collector::economic::coingecko::CoinGeckoCollector::new()?;
            let stats = collector.collect_all(&arango).await?;
            info!("{}", stats);
        }
        "fred" => {
            info!("📊 Fetching FRED data...");
            let collector = news_collector::economic::fred::FredCollector::new();
            let stats = collector.collect_all(&arango).await?;
            info!("{}", stats);
        }
        "bi" => {
            info!("📊 Fetching Bank Indonesia data...");
            let collector = news_collector::economic::bank_indonesia::BankIndonesiaCollector::new();
            let stats = collector.collect_all(&arango).await?;
            info!("{}", stats);
        }
        "all" | _ => {
            info!("📊 Collecting all economic data...");

            let collector = news_collector::economic::yahoo_commodities::YahooCommodityCollector::new()?;
            let stats = collector.collect_all(&arango).await?;
            info!("  Commodities: {}", stats);

            let collector = news_collector::economic::coingecko::CoinGeckoCollector::new()?;
            let stats = collector.collect_all(&arango).await?;
            info!("  Crypto: {}", stats);

            let collector = news_collector::economic::fred::FredCollector::new();
            let stats = collector.collect_all(&arango).await?;
            info!("  FRED: {}", stats);

            let collector = news_collector::economic::bank_indonesia::BankIndonesiaCollector::new();
            let stats = collector.collect_all(&arango).await?;
            info!("  Bank Indonesia: {}", stats);
        }
    }

    info!("✅ Economic data collection complete!");
    Ok(())
}
