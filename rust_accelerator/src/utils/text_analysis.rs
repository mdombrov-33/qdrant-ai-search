//! Text preprocessing and analysis utilities.
//!
//! This module handles all the text-related operations like tokenization,
//! stop word removal, and feature extraction. It's like the NLP preprocessing
//! pipeline in a machine learning project.

use crate::models::internal::QueryFeatures;
use std::collections::{HashMap, HashSet};

/// Handles text analysis and preprocessing operations.
///
/// This struct encapsulates all text-related functionality and holds
/// precomputed data structures (like stop words) for efficiency.
pub struct TextAnalyzer {
    /// Set of common words to ignore during analysis
    /// Using HashSet for O(1) lookup time vs Vec which would be O(n)
    stop_words: HashSet<String>,
}

impl TextAnalyzer {
    /// Creates a new TextAnalyzer with preloaded stop words.
    ///
    /// Stop words are common words like "the", "and", "is" that don't carry
    /// much meaning for search relevance. We filter these out to focus on
    /// meaningful terms.
    pub fn new() -> Self {
        // In a real system, you'd load this from a file or configuration
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "can", "this", "that",
            "these", "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her",
            "us", "them",
        ]
        .iter()
        .map(|&s| s.to_lowercase()) // Convert to lowercase for case-insensitive matching
        .collect();

        Self { stop_words }
    }

    /// Extracts meaningful features from a search query.
    ///
    /// This is like the preprocessing step in NLP pipelines:
    /// 1. Normalize text (lowercase)
    /// 2. Tokenize (split into words)
    /// 3. Filter stop words
    /// 4. Count frequencies
    ///
    /// Example: "What are the AI risks in 2024?"
    /// → meaningful_words: ["AI", "risks", "2024"]
    /// → word_frequencies: {"AI": 1, "risks": 1, "2024": 1}
    pub fn extract_query_features(&self, query: &str) -> QueryFeatures {
        // Step 1: Normalize to lowercase for consistent matching
        let normalized = query.to_lowercase();

        // Step 2: Split into words and filter
        let meaningful_words: Vec<String> = normalized
            .split_whitespace() // Split on any whitespace
            .filter(|word| {
                // Keep words that are:
                // - Not stop words (using HashSet for O(1) lookup)
                // - At least 2 characters long
                !self.stop_words.contains(*word) && word.len() >= 2
            })
            .map(|s| s.to_string()) // Convert &str to owned String
            .collect();

        // Step 3: Count word frequencies (like Python's Counter)
        let mut word_frequencies = HashMap::new();
        for word in &meaningful_words {
            // Get current count (or 0 if not exists) and increment
            *word_frequencies.entry(word.clone()).or_insert(0) += 1;
        }

        QueryFeatures { word_frequencies }
    }
}
