use news_intelligence::{StorageClient, Article};

#[tokio::test]
async fn test_live_arango_storage() {
    std::env::set_var("ARANGO_URL", "http://localhost:8529");
    std::env::set_var("ARANGO_DATABASE", "news_analysis");
    std::env::set_var("ARANGO_USERNAME", "root");
    std::env::set_var("ARANGO_PASSWORD", "");

    let client = StorageClient::from_env().expect("Failed to create client");
    
    let article = Article {
        title: "Test News Article Rust Integration".to_string(),
        source: "Test Source Rust".to_string(),
        content: "Test Content from Rust test harness.".to_string(),
        timestamp: "2026-09-02T10:00:00Z".to_string(),
        category: "Test".to_string(),
    };

    let result = client.store_article(&article).await;
    assert!(result.is_ok(), "Storage failed: {:?}", result.err());
}
