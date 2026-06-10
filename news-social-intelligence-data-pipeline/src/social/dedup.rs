//! Near-duplicate detection via Qdrant vector similarity search.

use anyhow::Result;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;

/// Similarity thresholds
pub const EXACT_DUPLICATE_THRESHOLD: f32 = 0.98;
pub const NEAR_DUPLICATE_THRESHOLD: f32 = 0.92;
pub const SAME_EVENT_THRESHOLD: f32 = 0.85;
pub const RELATED_THRESHOLD: f32 = 0.75;

/// Check if an embedding is a near-duplicate in the given collection.
/// Returns true if a point with similarity >= threshold exists.
pub async fn is_near_duplicate(
    client: &Qdrant,
    collection: &str,
    embedding: &[f32],
    threshold: f32,
) -> Result<bool> {
    let results = client
        .search_points(
            SearchPointsBuilder::new(collection, embedding.to_vec(), 1)
                .score_threshold(threshold),
        )
        .await?;

    Ok(!results.result.is_empty())
}

/// Classify a similarity score into a human-readable category.
pub fn classify_similarity(score: f32) -> &'static str {
    if score >= EXACT_DUPLICATE_THRESHOLD {
        "exact_duplicate"
    } else if score >= NEAR_DUPLICATE_THRESHOLD {
        "near_duplicate"
    } else if score >= SAME_EVENT_THRESHOLD {
        "same_event"
    } else if score >= RELATED_THRESHOLD {
        "related"
    } else {
        "different"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_exact_duplicate() {
        assert_eq!(classify_similarity(0.99), "exact_duplicate");
    }

    #[test]
    fn test_classify_near_duplicate() {
        assert_eq!(classify_similarity(0.94), "near_duplicate");
    }

    #[test]
    fn test_classify_same_event() {
        assert_eq!(classify_similarity(0.87), "same_event");
    }

    #[test]
    fn test_classify_related() {
        assert_eq!(classify_similarity(0.78), "related");
    }

    #[test]
    fn test_classify_different() {
        assert_eq!(classify_similarity(0.50), "different");
    }
}
