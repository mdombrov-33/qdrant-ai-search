//! Text preprocessing and analysis utilities.

use crate::models::internal::QueryFeatures;
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::{HashMap, HashSet};
use stop_words::{LANGUAGE, get};

/// Handles text analysis and preprocessing operations.
pub struct TextAnalyzer {
    /// Set of common words to ignore during analysis
    stop_words: HashSet<String>,
}

impl TextAnalyzer {
    /// Creates a new TextAnalyzer with preloaded stop words.
    pub fn new() -> Self {
        let stop_words = get(LANGUAGE::English)
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();

        Self { stop_words }
    }

    /// Extracts meaningful features from a search query.
    pub fn extract_query_features(
        &self,
        query: &str,
        idf_map: Option<HashMap<String, f64>>,
    ) -> QueryFeatures {
        // Step 1: Normalize to lowercase for consistent matching
        let normalized = query.to_lowercase();

        // Step 2: Extract meaningful single words with basic stemming
        let stemmer = Stemmer::create(Algorithm::English);
        let single_words: Vec<String> = normalized
            .split_whitespace()
            .filter(|word| !self.stop_words.contains(*word) && word.len() >= 2)
            .map(|word| stemmer.stem(word).to_string())
            .collect();

        // Step 3: Extract meaningful 2-word phrases
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut phrases = Vec::new();

        for window in words.windows(2) {
            if !self.stop_words.contains(window[0]) && !self.stop_words.contains(window[1]) {
                phrases.push(window.join(" "));
            }
        }

        // Step 4: Combine single words and phrases into one vector
        let all_terms: Vec<String> = single_words.into_iter().chain(phrases).collect();

        // Step 5: Count term frequencies
        let mut word_frequencies = HashMap::new();
        for term in &all_terms {
            *word_frequencies.entry(term.clone()).or_insert(0) += 1;
        }

        QueryFeatures {
            word_frequencies,
            idf_map: idf_map.unwrap_or_default(),
        }
    }
}
