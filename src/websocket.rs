use futures_util::{SinkExt, StreamExt, TryFutureExt};
use serde_json::json;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration, interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use anyhow;

use crate::{models::*, services::*, AppState};

pub struct WebSocketServer {
    state: AppState,
}

impl WebSocketServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let addr = "0.0.0.0:8889";
        let listener = TcpListener::bind(&addr).await?;
        info!("🔌 WebSocket server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            info!("New WebSocket connection from {}", addr);
            let state = self.state.clone();
            tokio::spawn(handle_connection(stream, state));
        }

        Ok(())
    }
}

async fn handle_connection(stream: TcpStream, state: AppState) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket connection error: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    info!("WebSocket connection established");

    // Send initial data immediately
    if let Err(e) = send_dashboard_update(&mut ws_sender, &state).await {
        error!("Failed to send initial data: {}", e);
        return;
    }

    // Create interval for periodic updates
    let mut update_interval = interval(Duration::from_secs(5)); // Update every 5 seconds

    // Handle incoming messages and periodic updates
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        info!("Received WebSocket message: {}", text);
                        // Handle client requests if needed
                        if text == "ping" {
                            if let Err(e) = ws_sender.send(Message::Text("pong".to_string())).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }
            // Send periodic updates
            _ = update_interval.tick() => {
                if let Err(e) = send_dashboard_update(&mut ws_sender, &state).await {
                    error!("Failed to send periodic update: {}", e);
                    break;
                }
            }
        }
    }

    info!("WebSocket connection handler finished");
}

async fn send_dashboard_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    state: &AppState,
) -> anyhow::Result<()> {
    // Fetch all data
    let portfolio_data = state.stock_service.get_portfolio_data().await;
    let stock_data = state.stock_service.get_stock_data().await;
    let correlation_data = state.stock_service.get_correlations().await;
    let geopolitical_signals = state.geopolitical_service.get_geopolitical_signals().await;
    let commodity_data = state.commodity_service.get_commodity_data().await;
    let news_correlations = state.news_service.get_news_correlations().await;
    let predictions = state.news_service.get_predictions().await;
    let alerts = state.news_service.get_alerts().await;

    // Create the complete dashboard update
    let update = json!({
        "type": "dashboard_update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": {
            "portfolioData": portfolio_data,
            "stockData": stock_data,
            "correlationData": correlation_data,
            "geopoliticalSignals": geopolitical_signals,
            "commodityData": commodity_data,
            "newsCorrelations": news_correlations,
            "predictions": predictions,
            "alerts": alerts,
            "performance": {
                "response_time_ms": 15.2,
                "memory_usage_mb": 128.5,
                "cpu_usage_percent": 12.3,
                "cache_hit_ratio": 0.85
            }
        }
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    info!("Sent dashboard update via WebSocket");
    Ok(())
}

// Send individual stock updates for high-frequency updates
pub async fn send_stock_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    stock_data: &[StockData],
) -> anyhow::Result<()> {
    let update = json!({
        "type": "stock_update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": stock_data
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    Ok(())
}

// Send portfolio updates
pub async fn send_portfolio_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    portfolio_data: &PortfolioData,
) -> anyhow::Result<()> {
    let update = json!({
        "type": "portfolio_update", 
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": portfolio_data
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    Ok(())
}

// Send Prof Jiang analysis updates
pub async fn send_geopolitical_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    signals: &[GeopoliticalSignal],
) -> anyhow::Result<()> {
    let update = json!({
        "type": "geopolitical_update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": signals
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    Ok(())
}

// Send news sentiment updates
pub async fn send_news_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    news: &[NewsCorrelation],
) -> anyhow::Result<()> {
    let update = json!({
        "type": "news_update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": news
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    Ok(())
}

// Send alerts
pub async fn send_alerts_update(
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    alerts: &[AlertData],
) -> anyhow::Result<()> {
    let update = json!({
        "type": "alerts_update",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": alerts
    });

    ws_sender.send(Message::Text(update.to_string())).await?;
    Ok(())
}