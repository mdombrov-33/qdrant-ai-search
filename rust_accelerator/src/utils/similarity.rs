//! Text similarity algorithms for deduplication.
//!
//! This module implements various text similarity measures used to identify
//! and remove duplicate or near-duplicate results from the search output.

use crate::models::internal::EnhancedResult;
use std::collections::HashSet;

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
            deduplication_threshold: 0.8, // 80% similarity = duplicate
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
        // Store lowercase versions in variables to keep them alive for &str slices
        let lowered1 = text1.to_lowercase();
        let lowered2 = text2.to_lowercase();

        // Convert texts to sets of words (case-insensitive)
        let words1: HashSet<&str> = lowered1.split_whitespace().collect();
        let words2: HashSet<&str> = lowered2.split_whitespace().collect();

        // Handle edge case of empty texts
        if words1.is_empty() && words2.is_empty() {
            return 1.0; // Two empty texts are identical
        }

        // Calculate intersection and union sizes
        let intersection_size = words1.intersection(&words2).count();
        let union_size = words1.union(&words2).count();

        // Calculate Jaccard coefficient
        if union_size == 0 {
            0.0 // Avoid division by zero
        } else {
            intersection_size as f64 / union_size as f64
        }
    }

    /// Calculates cosine similarity between two texts (alternative algorithm).
    ///
    /// Cosine similarity treats texts as vectors in high-dimensional space
    /// and measures the angle between them. It's often better than Jaccard
    /// for texts of very different lengths.
    ///
    /// This implementation uses simple word frequency vectors.
    /// In production, you might use TF-IDF weights or embeddings.
    #[allow(dead_code)] // Mark as unused for now
    pub fn cosine_similarity(&self, text1: &str, text2: &str) -> f64 {
        use std::collections::HashMap;

        // Store lowercase versions to keep alive while borrowing slices
        let lowered1 = text1.to_lowercase();
        let lowered2 = text2.to_lowercase();

        // Build word frequency vectors
        let mut freq1 = HashMap::new();
        let mut freq2 = HashMap::new();

        // Count words in text1
        for word in lowered1.split_whitespace() {
            *freq1.entry(word).or_insert(0) += 1;
        }

        // Count words in text2
        for word in lowered2.split_whitespace() {
            *freq2.entry(word).or_insert(0) += 1;
        }

        // Get all unique words from both texts
        let all_words: HashSet<&str> = freq1.keys().chain(freq2.keys()).copied().collect();

        if all_words.is_empty() {
            return 0.0;
        }

        // Calculate dot product and magnitudes
        let mut dot_product = 0.0;
        let mut magnitude1 = 0.0;
        let mut magnitude2 = 0.0;

        for word in all_words {
            let f1 = *freq1.get(word).unwrap_or(&0) as f64;
            let f2 = *freq2.get(word).unwrap_or(&0) as f64;

            dot_product += f1 * f2;
            magnitude1 += f1 * f1;
            magnitude2 += f2 * f2;
        }

        // Calculate cosine similarity
        let magnitude_product = magnitude1.sqrt() * magnitude2.sqrt();
        if magnitude_product == 0.0 {
            0.0
        } else {
            dot_product / magnitude_product
        }
    }
}
