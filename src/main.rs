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
    Config, RssCollector, ArticleCleaner, KiroLabeler, TeiEmbedder,
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
