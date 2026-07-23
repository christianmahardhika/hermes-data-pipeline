use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

mod handlers;
mod models;
mod services;
mod websocket;

use handlers::*;
use models::*;
use services::*;
use websocket::*;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub stock_service: Arc<StockService>,
    pub geopolitical_service: Arc<GeopoliticalService>,
    pub commodity_service: Arc<CommodityService>,
    pub news_service: Arc<NewsService>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize services with database
    let stock_service = Arc::new(StockService::new());
    let geopolitical_service = Arc::new(GeopoliticalService::new());
    let commodity_service = Arc::new(CommodityService::new());
    let news_service = Arc::new(NewsService::new());

    // Initialize database and populate with real Indonesian market data
    info!("🔌 Connecting to ArangoDB and initializing collections...");
    if let Err(e) = stock_service.initialize().await {
        error!("Failed to initialize database: {}", e);
        info!("⚠️  Continuing with mock data - database features may be limited");
    } else {
        info!("✅ Database initialized with real Indonesian market data");
    }

    let state = AppState {
        stock_service,
        geopolitical_service,
        commodity_service,
        news_service,
    };

    // CORS configuration for Next.js dashboard
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(tower_http::cors::Any);

    // Build the application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/dashboard", get(get_dashboard))
        .route("/api/weekly", get(get_weekly_news))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/stocks", get(get_stocks))
        .route("/api/correlations", get(get_correlations))
        .route("/api/geopolitical", get(get_geopolitical))
        .route("/api/commodities", get(get_commodities))
        .route("/api/news", get(get_news))
        .route("/api/predictions", get(get_predictions))
        .route("/api/alerts", get(get_alerts))
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state.clone());

    // Start WebSocket server in background
    let websocket_server = WebSocketServer::new(state);
    let websocket_handle = tokio::spawn(async move {
        if let Err(e) = websocket_server.start().await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888")
        .await
        .expect("Failed to bind to port 8888");
    
    info!("🚀 Intelligence System Rust Backend starting on port 8888");
    info!("📊 Dashboard endpoint: http://localhost:8888/api/dashboard");
    info!("🏥 Health check: http://localhost:8888/health");
    info!("🔌 WebSocket server: ws://localhost:8889");

    // Run both servers concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                error!("HTTP server error: {}", e);
            }
        }
        _ = websocket_handle => {
            info!("WebSocket server finished");
        }
    }
}

// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "Intelligence System Rust Backend",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}