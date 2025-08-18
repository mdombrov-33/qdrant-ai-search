//! Text similarity algorithms for deduplication.

use crate::models::internal::EnhancedResult;
use textdistance::nstr::jaccard;

/// Handles text similarity calculations and deduplication.
pub struct SimilarityCalculator {
    /// Similarity threshold for deduplication
    deduplication_threshold: f64,
}

impl SimilarityCalculator {
    pub fn new() -> Self {
        Self {
            deduplication_threshold: 0.95, // 95% similarity = duplicate
        }
    }

    /// Removes duplicate results using text similarity analysis.
    pub fn remove_duplicates(&self, results: Vec<EnhancedResult>) -> Vec<EnhancedResult> {
        let mut unique_results = Vec::new();

        // Process results in score order
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
