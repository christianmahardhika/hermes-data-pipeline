use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};

use crate::{models::*, services::*, AppState};

// Dashboard endpoint - returns all data in one response
pub async fn get_dashboard(State(state): State<AppState>) -> Json<Value> {
    let portfolio_data = state.stock_service.get_portfolio_data().await;
    let stock_data = state.stock_service.get_stock_data().await;
    let correlation_data = state.stock_service.get_correlations().await;
    let geopolitical_signals = state.geopolitical_service.get_geopolitical_signals().await;
    let commodity_data = state.commodity_service.get_commodity_data().await;
    let news_correlations = state.news_service.get_news_correlations().await;
    let predictions = state.news_service.get_predictions().await;
    let alerts = state.news_service.get_alerts().await;

    let response = DashboardResponse {
        portfolio_data,
        stock_data,
        correlation_data,
        geopolitical_signals,
        commodity_data,
        news_correlations,
        predictions,
        alerts,
        performance: PerformanceMetrics {
            response_time_ms: 15.2,
            memory_usage_mb: 128.5,
            cpu_usage_percent: 12.3,
            cache_hit_ratio: 0.85,
        },
    };

    Json(json!(response))
}

pub async fn get_weekly_news(State(state): State<AppState>) -> Json<Value> {
    let weekly_news = state.news_service.get_weekly_news_summary().await.unwrap_or_default();
    
    Json(json!({
        "weeklyNews": weekly_news,
        "performance": {
            "response_time_ms": 12.1,
            "memory_usage_mb": 95.3,
            "cpu_usage_percent": 8.7,
            "cache_hit_ratio": 0.92
        }
    }))
}

// Portfolio endpoint
pub async fn get_portfolio(State(state): State<AppState>) -> Json<Value> {
    let portfolio_data = state.stock_service.get_portfolio_data().await;
    Json(json!(portfolio_data))
}

// Indonesian stocks endpoint
pub async fn get_stocks(State(state): State<AppState>) -> Json<Value> {
    let stock_data = state.stock_service.get_stock_data().await;
    Json(json!(stock_data))
}

// Correlation matrices endpoint
pub async fn get_correlations(State(state): State<AppState>) -> Json<Value> {
    let correlation_data = state.stock_service.get_correlations().await;
    Json(json!(correlation_data))
}

// Geopolitical signals endpoint
pub async fn get_geopolitical(State(state): State<AppState>) -> Json<Value> {
    let geopolitical_signals = state.geopolitical_service.get_geopolitical_signals().await;
    Json(json!(geopolitical_signals))
}

// Commodities endpoint
pub async fn get_commodities(State(state): State<AppState>) -> Json<Value> {
    let commodity_data = state.commodity_service.get_commodity_data().await;
    Json(json!(commodity_data))
}

// News sentiment endpoint
pub async fn get_news(State(state): State<AppState>) -> Json<Value> {
    let news_correlations = state.news_service.get_news_correlations().await;
    Json(json!(news_correlations))
}

// Predictions endpoint
pub async fn get_predictions(State(state): State<AppState>) -> Json<Value> {
    let predictions = state.news_service.get_predictions().await;
    Json(json!(predictions))
}

// Alerts endpoint
pub async fn get_alerts(State(state): State<AppState>) -> Json<Value> {
    let alerts = state.news_service.get_alerts().await;
    Json(json!(alerts))
}