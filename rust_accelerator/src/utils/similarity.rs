//! Text similarity algorithms for deduplication.
//!
//! This module implements various text similarity measures used to identify
//! and remove duplicate or near-duplicate results from the search output.

use crate::models::internal::EnhancedResult;
use textdistance::nstr::jaccard;

/// Handles text similarity calculations and deduplication.
///
/// This component uses various algorithms to measure how similar two pieces
/// of text are, then removes results that are too similar to each other.
pub struct SimilarityCalculator {
    /// Similarity threshold above which we consider results duplicates
    deduplication_threshold: f64,
}

impl SimilarityCalculator {
    pub fn new() -> Self {
        Self {
            deduplication_threshold: 0.95, // 95% similarity = duplicate
        }
    }

    /// Removes duplicate results using text similarity analysis.
    ///
    /// This function takes a list of results and removes any that are too
    /// similar to results with higher scores. It preserves the highest-scoring
    /// version of any group of similar results.
    ///
    /// The algorithm is O(n²) in the worst case, but with early termination
    /// and optimizations it's usually much faster in practice.
    pub fn remove_duplicates(&self, results: Vec<EnhancedResult>) -> Vec<EnhancedResult> {
        let mut unique_results = Vec::new();

        // Process results in score order (already sorted by caller)
        for candidate in results {
            let is_duplicate = unique_results.iter().any(|existing: &EnhancedResult| {
                // Use Jaccard similarity to compare texts
                self.jaccard_similarity(&candidate.original.text, &existing.original.text)
                    > self.deduplication_threshold
            });

            // Only keep non-duplicates
            if !is_duplicate {
                unique_results.push(candidate);
            }
            // If it's a duplicate, we discard it (keeping the higher-scored one)
        }

        unique_results
    }

    /// Calculates Jaccard similarity between two texts.
    ///
    /// Jaccard similarity measures the overlap between two sets:
    /// similarity = |intersection| / |union|
    ///
    /// For text, we treat each text as a set of words, then calculate:
    /// - intersection = words that appear in both texts
    /// - union = all unique words from both texts
    ///
    /// Example:
    /// Text1: "the quick brown fox"
    /// Text2: "the brown fox jumps"
    /// Words1: {the, quick, brown, fox}
    /// Words2: {the, brown, fox, jumps}
    /// Intersection: {the, brown, fox} = 3 words
    /// Union: {the, quick, brown, fox, jumps} = 5 words
    /// Similarity: 3/5 = 0.6 (60% similar)
    pub fn jaccard_similarity(&self, text1: &str, text2: &str) -> f64 {
        jaccard(text1, text2)
    }
}
