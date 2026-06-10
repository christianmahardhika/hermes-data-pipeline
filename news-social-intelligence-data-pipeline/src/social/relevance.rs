//! Token overlap relevance scoring for social intelligence queries.
//!
//! Measures how well a result matches a query using simple token overlap,
//! without requiring embeddings or ML models.

use std::collections::HashSet;

/// Tokenize text into lowercase tokens, splitting on non-alphanumeric chars.
/// Filters out tokens shorter than 3 characters.
pub fn tokenize(text: &str) -> HashSet<String> {
    if text.is_empty() {
        return HashSet::new();
    }
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_string())
        .collect()
}

/// Calculate relevance score based on token overlap between query and text.
/// Returns a float between 0.0 and 1.0.
pub fn token_overlap_relevance(query: &str, text: &str) -> f32 {
    let query_tokens = tokenize(query);
    let text_tokens = tokenize(text);

    if query_tokens.is_empty() {
        return 0.0;
    }

    let overlap = query_tokens.intersection(&text_tokens).count();
    overlap as f32 / query_tokens.len() as f32
}

/// Calculate weighted relevance combining title and description scores.
/// `title_weight` controls how much title contributes vs description.
pub fn combined_relevance(query: &str, title: &str, description: &str, title_weight: f32) -> f32 {
    let title_score = token_overlap_relevance(query, title);
    let desc_score = if description.is_empty() {
        0.0
    } else {
        token_overlap_relevance(query, description)
    };

    let desc_weight = 1.0 - title_weight;
    (title_score * title_weight) + (desc_score * desc_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("Hello, World! This is Rust.");
        assert!(tokens.contains("hello"));
        assert!(tokens.contains("world"));
        assert!(tokens.contains("this"));
        assert!(tokens.contains("rust"));
        // "is" is < 3 chars, should be filtered
        assert!(!tokens.contains("is"));
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_relevance_exact_match() {
        let score = token_overlap_relevance("artificial intelligence", "artificial intelligence is the future");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_relevance_no_match() {
        let score = token_overlap_relevance("quantum computing", "the cat sat on the mat");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_combined_relevance_title_weighted() {
        // Query matches title fully, description not at all
        let score = combined_relevance(
            "rust programming",
            "rust programming language",
            "the weather is nice today",
            0.7,
        );
        // title_score = 1.0, desc_score = 0.0
        // combined = 1.0 * 0.7 + 0.0 * 0.3 = 0.7
        assert!((score - 0.7).abs() < f32::EPSILON);
    }
}
