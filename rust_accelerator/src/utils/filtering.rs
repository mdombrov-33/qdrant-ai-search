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
    pub fn new() -> Self {
        Self {
            min_text_length: 10,       // At least 10 characters
            max_text_length: 5000,     // No more than 5000 characters
            min_similarity_score: 0.1, // At least 10% similarity
        }
    }

    /// Determines if a result should be kept for further processing.
    ///
    /// This function applies multiple quick filters to catch obvious problems:
    /// - Empty or nearly empty text
    /// - Extremely long text (probably not relevant excerpts)
    /// - Very low similarity scores (probably noise)
    /// - Non-meaningful content (only symbols, etc.)
    ///
    /// Returns true if the result should be processed further.
    pub fn should_keep(&self, result: &SearchResult) -> bool {
        // Filter 1: Check text length bounds
        let text_len = result.text.len();
        if text_len < self.min_text_length || text_len > self.max_text_length {
            return false;
        }

        // Filter 2: Check similarity score threshold
        if result.score < self.min_similarity_score {
            return false;
        }

        // Filter 3: Check for meaningful content
        if !self.has_meaningful_content(&result.text) {
            return false;
        }

        // Filter 4: Check for obvious spam or low-quality patterns
        if self.looks_like_spam(&result.text) {
            return false;
        }

        true // Passed all filters
    }

    /// Checks if text contains meaningful alphabetic content.
    ///
    /// This catches results that are mostly symbols, numbers, or whitespace.
    /// We need a reasonable amount of actual text content.
    fn has_meaningful_content(&self, text: &str) -> bool {
        let alphabetic_chars = text.chars().filter(|c| c.is_alphabetic()).count();
        let total_chars = text.chars().count();

        // At least 30% of characters should be alphabetic letters
        // and we need at least 5 alphabetic characters total
        alphabetic_chars >= 5 && (alphabetic_chars as f64 / total_chars as f64) >= 0.3
    }

    /// Checks for patterns that indicate spam or very low quality content.
    ///
    /// This is a simple heuristic-based spam detector. In a production system,
    /// you might use more sophisticated ML-based approaches.
    fn looks_like_spam(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();

        // Pattern 1: Excessive repetition of characters
        if self.has_excessive_repetition(&text_lower) {
            return true;
        }

        // Pattern 2: Too many special characters
        let special_char_count = text
            .chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
            .count();
        let special_char_ratio = special_char_count as f64 / text.len() as f64;
        if special_char_ratio > 0.3 {
            return true; // More than 30% special characters
        }

        // Pattern 3: All caps (often indicates shouting/spam)
        let alpha_chars: String = text.chars().filter(|c| c.is_alphabetic()).collect();
        if alpha_chars.len() > 20 && alpha_chars == alpha_chars.to_uppercase() {
            return true;
        }

        false // Doesn't look like spam
    }

    /// Detects excessive character or word repetition.
    ///
    /// Spam often contains repeated characters or words like "aaaaa" or
    /// "buy now buy now buy now". This function catches such patterns.
    fn has_excessive_repetition(&self, text: &str) -> bool {
        // Check for repeated characters (like "aaaaaa")
        let chars: Vec<char> = text.chars().collect();
        let mut consecutive_count = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                consecutive_count += 1;
                if consecutive_count > 5 {
                    return true; // More than 5 consecutive same characters
                }
            } else {
                consecutive_count = 1;
            }
        }

        // Check for repeated words
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() >= 4 {
            for i in 0..words.len() - 3 {
                if words[i] == words[i + 1]
                    && words[i + 1] == words[i + 2]
                    && words[i + 2] == words[i + 3]
                {
                    return true; // Same word repeated 4 times in a row
                }
            }
        }

        false
    }
}
