//! Early filtering utilities to remove low-quality results.
//!
//! This module implements filters that catch obviously bad results before
//! we spend CPU time on expensive scoring algorithms. It's like a first-pass
//! quality gate.

use crate::models::request::SearchResult;

/// Handles early filtering of search results.
///
/// This component decides which results are worth processing and which
/// should be discarded immediately. It's designed to be very fast since
/// it runs on every result.
pub struct ResultFilter {
    /// Minimum text length to be considered valid
    min_text_length: usize,

    /// Maximum text length before we consider it too verbose
    max_text_length: usize,

    /// Minimum similarity score from Qdrant to be worth considering
    min_similarity_score: f64,
}

impl ResultFilter {
    pub fn new(min_similarity_score: f64) -> Self {
        Self {
            min_text_length: 10,   // At least 10 characters
            max_text_length: 5000, // No more than 5000 characters
            min_similarity_score,
        }
    }
    /// Determines if a result should be kept for further processing.
    ///
    /// Applies efficient quality checks in order of computational cost:
    /// 1. Fast length/similarity checks
    /// 2. Content quality evaluation
    /// 3. Spam pattern detection
    pub fn should_keep(&self, result: &SearchResult) -> bool {
        // Fast path: Reject immediately if length or score fails
        let text = result.text.trim();
        if text.len() < self.min_text_length
            || text.len() > self.max_text_length
            || result.score < self.min_similarity_score
        {
            return false;
        }

        // Content quality check
        let alpha_count = text.chars().filter(|c| c.is_alphabetic()).count();
        if alpha_count < 5 || (alpha_count * 3) < text.len() {
            return false; // Reject if <5 letters or <33% alphabetic
        }

        // Spam detection
        let lower_text = text.to_lowercase();
        if lower_text.chars().all(|c| c.is_uppercase()) && text.len() > 20 {
            return false; // All-caps check
        }

        // Check for character repetition (like "aaaaa")
        if lower_text
            .chars()
            .collect::<Vec<_>>()
            .windows(6)
            .any(|w| w.iter().all(|&c| c == w[0]))
        {
            return false;
        }

        // Check for word repetition (like "spam spam spam")
        let words: Vec<_> = lower_text.split_whitespace().collect();
        if words.len() >= 4
            && words
                .windows(4)
                .any(|w| w[0] == w[1] && w[1] == w[2] && w[2] == w[3])
        {
            return false;
        }

        true
    }
}
