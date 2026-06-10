# Rust Development Patterns

Idiomatic Rust patterns for the Hermes data pipeline.

## Core Principles

1. **Ownership and borrowing** — prefer references over owned types in function params
2. **Error propagation** — use `?` operator, wrap with context via `anyhow`
3. **Zero-cost abstractions** — iterators, traits, generics over runtime dispatch
4. **Fearless concurrency** — leverage tokio for async, Send/Sync for safety
5. **Make illegal states unrepresentable** — use enums and type system

## Error Handling

### Application Errors (anyhow)
```rust
use anyhow::{Result, Context};

pub async fn process(&self) -> Result<Stats> {
    let data = self.fetch()
        .await
        .context("fetching RSS feed")?;
    Ok(stats)
}
```

### Custom Domain Errors (thiserror)
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("feed {name} unreachable: {source}")]
    FeedUnreachable { name: String, #[source] source: reqwest::Error },
    #[error("LLM API returned {status}: {body}")]
    LlmError { status: u16, body: String },
    #[error("duplicate content: {hash}")]
    Duplicate { hash: String },
}
```

## Async Patterns

### Retry with Backoff (project pattern)
```rust
use backoff::{ExponentialBackoff, future::retry};

let backoff = ExponentialBackoff {
    max_elapsed_time: Some(Duration::from_secs(60)),
    ..Default::default()
};

let result = retry(backoff, || async {
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => Ok(resp),
        Ok(resp) if resp.status().is_server_error() =>
            Err(backoff::Error::transient(anyhow!("server error"))),
        Ok(resp) => Err(backoff::Error::permanent(anyhow!("client error"))),
        Err(e) => Err(backoff::Error::transient(anyhow!(e))),
    }
}).await?;
```

### Daemon Loop
```rust
let interval = tokio::time::Duration::from_secs(15 * 60);
loop {
    if let Err(e) = run_pipeline(config).await {
        error!("Pipeline error: {}", e);
    }
    tokio::time::sleep(interval).await;
}
```

## Data Processing Patterns

### Pipeline Phase Pattern (project standard)
```rust
pub struct PhaseName;

impl PhaseName {
    pub fn new() -> Self { Self }

    pub async fn process_pending(&self, db: &Database, limit: i64) -> Result<Stats> {
        let pending = db.get_pending(limit)?;
        let mut stats = Stats::default();

        for item in pending {
            match self.process_one(&item, db) {
                Ok(count) => {
                    db.update_status(item.id.unwrap(), "processed")?;
                    stats.success += count;
                }
                Err(e) => {
                    db.update_status(item.id.unwrap(), "failed")?;
                    stats.errors += 1;
                    warn!("Failed {}: {}", item.name, e);
                }
            }
        }
        Ok(stats)
    }
}
```

### Stats Pattern
```rust
#[derive(Debug, Default)]
pub struct Stats { pub success: usize, pub errors: usize }

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} success, {} errors", self.success, self.errors)
    }
}
```

## SQLite Patterns

### Parameterized Queries (ALWAYS)
```rust
self.conn.execute(
    "INSERT INTO table (col1, col2) VALUES (?1, ?2)",
    params![value1, value2],
)?;
```

### DateTime as RFC3339
```rust
let stored = dt.to_rfc3339();
let parsed = DateTime::parse_from_rfc3339(&stored)
    .map(|dt| dt.with_timezone(&Utc))
    .unwrap_or_else(|_| Utc::now());
```

## Qdrant Patterns

### Collection Setup
```rust
let exists = client.collection_exists(&name).await?;
if !exists {
    client.create_collection(
        CreateCollectionBuilder::new(&name)
            .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
    ).await?;
}
```

### Near-Duplicate Detection
```rust
let results = client.search_points(
    SearchPointsBuilder::new(&collection, vector, 1)
        .score_threshold(0.95)
).await?;
let is_duplicate = !results.result.is_empty();
```

## Logging Convention
```rust
// Emoji prefix convention used in this project:
info!("📥 Phase 1: Collecting...");  // collect
info!("🧹 Phase 2: Cleaning...");   // clean
info!("🏷️ Phase 3: Labeling...");   // label
info!("📊 Phase 4: Embedding...");  // embed
info!("✅ Done: {}", item);         // success
warn!("⚠️ Retry: {}", reason);      // warning
error!("❌ Failed: {}", e);          // error
```

## Testing (when adding tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_removes_tags() {
        let cleaner = ArticleCleaner::new();
        assert_eq!(cleaner.strip_html("<p>Hello</p>"), "Hello");
    }

    #[tokio::test]
    async fn test_embed_returns_vector() {
        // Mock TEI endpoint or use integration test
    }
}
```
