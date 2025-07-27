//! Advanced scoring algorithms for result enhancement.
//!
//! This module contains all the mathematical algorithms we use to improve
//! upon the basic vector similarity scores from Qdrant. Each algorithm
//! targets a different aspect of result quality.

use crate::models::internal::QueryFeatures;
use crate::models::request::SearchResult;

/// Configuration for score weighting.
///
/// These constants control how much each algorithm contributes to the final score.
/// In a production system, these would be tunable parameters that you could
/// A/B test or optimize based on user feedback.
pub struct ScoreWeights {
    pub text_quality: f64,        // How much to weight content quality
    pub keyword_matching: f64,    // How much to weight keyword overlap
    pub length_optimization: f64, // How much to weight ideal length
    pub position_decay: f64,      // How much to weight original position
    pub completeness: f64,        // How much to weight sentence completeness
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            text_quality: 1.0,        // Full weight - quality is crucial
            keyword_matching: 0.3,    // 30% boost max - helps but don't overdo
            length_optimization: 1.0, // Full weight - length matters a lot
            position_decay: 0.05,     // Small weight - slight preference for earlier results
            completeness: 0.1,        // Small boost - nice to have complete sentences
        }
    }
}

/// Calculates enhanced scores using multiple algorithms.
///
/// This is the heart of our re-ranking system. It takes the basic similarity
/// score from Qdrant and enhances it using domain knowledge about what makes
/// a good search result.
pub struct ScoreCalculator {
    weights: ScoreWeights,
}

impl ScoreCalculator {
    pub fn new() -> Self {
        Self {
            weights: ScoreWeights::default(),
        }
    }

    /// Calculates an enhanced score using multiple algorithms.
    ///
    /// This function combines several scoring strategies:
    /// 1. Text quality assessment (penalize fragments, boost complete text)
    /// 2. Keyword matching (TF-IDF inspired scoring)
    /// 3. Length optimization (sweet spot scoring)
    /// 4. Position-based decay (slight preference for earlier results)
    /// 5. Completeness bonus (reward complete sentences)
    ///
    /// The final score is a weighted combination of all these factors.
    pub fn calculate_enhanced_score(
        &self,
        result: &SearchResult,
        query_features: &QueryFeatures,
        position: usize,
    ) -> f64 {
        // Start with the original Qdrant similarity score
        let mut score = result.score;

        // === ALGORITHM 1: TEXT QUALITY MULTIPLIER ===
        // Penalize low-quality text, boost high-quality text
        let quality_factor = self.calculate_text_quality_factor(&result.text);
        score *= quality_factor * self.weights.text_quality;

        // === ALGORITHM 2: KEYWORD MATCHING BOOST ===
        // Boost results that contain query keywords
        let keyword_boost = self.calculate_keyword_boost(&result.text, query_features);
        score += keyword_boost * self.weights.keyword_matching;

        // === ALGORITHM 3: LENGTH OPTIMIZATION ===
        // Apply sweet-spot scoring based on text length
        let length_factor = self.calculate_length_factor(result.text.len());
        score *= length_factor * self.weights.length_optimization;

        // === ALGORITHM 4: POSITION DECAY ===
        // Slight preference for results that were originally ranked higher
        let position_factor = self.calculate_position_factor(position);
        score *= 1.0 - (position_factor * self.weights.position_decay);

        // === ALGORITHM 5: COMPLETENESS BONUS ===
        // Small bonus for complete sentences and well-formed text
        let completeness_bonus = self.calculate_completeness_bonus(&result.text);
        score += completeness_bonus * self.weights.completeness;

        // Ensure score stays in valid range [0.0, 1.0]
        score.clamp(0.0, 1.0)
    }

    /// Assesses text quality and returns a multiplier.
    ///
    /// This algorithm looks for indicators of high-quality, useful text:
    /// - Adequate length (not too short, not too long)
    /// - Proper sentence structure
    /// - Meaningful content
    ///
    /// Returns a multiplier between 0.1 and 1.5
    fn calculate_text_quality_factor(&self, text: &str) -> f64 {
        let text_len = text.len();

        // Severely penalize very short text (likely fragments or noise)
        if text_len < 20 {
            return 0.1; // 90% penalty
        }

        // Penalize text without proper sentence endings (likely fragments)
        if !text.contains('.') && !text.contains('!') && !text.contains('?') {
            return 0.7; // 30% penalty
        }

        // Bonus for well-formed text with multiple sentences
        let sentence_count = text.matches(['.', '!', '?']).count();
        match sentence_count {
            0 => 0.8,     // No sentences - probably a fragment
            1 => 1.0,     // One sentence - neutral
            2..=5 => 1.2, // Multiple sentences - good content
            _ => 1.1,     // Too many sentences - might be verbose
        }
    }

    /// Calculates keyword matching boost using TF-IDF inspired scoring.
    ///
    /// This algorithm rewards results that contain words from the search query.
    /// It uses frequency weighting: if a query word appears multiple times in
    /// the query, matches for that word get higher scores.
    ///
    /// Example: Query "machine learning machine"
    /// - "machine" appears twice, so gets higher weight
    /// - Results containing "machine" get bigger boost than those with "learning"
    fn calculate_keyword_boost(&self, text: &str, query_features: &QueryFeatures) -> f64 {
        let text_lower = text.to_lowercase();
        let mut total_boost = 0.0;

        // Check each meaningful word from the query
        for (word, query_frequency) in &query_features.word_frequencies {
            // Count how many times this word appears in the result text
            let text_frequency = text_lower.matches(word).count();

            if text_frequency > 0 {
                // Calculate boost using TF-IDF inspired weighting:
                // - More frequent in query = higher weight
                // - More frequent in text = higher boost
                // - Square root to prevent dominance of very frequent words
                let weight = (*query_frequency as f64).sqrt();
                let boost = (text_frequency as f64) * weight * 0.05; // 5% per match
                total_boost += boost;
            }
        }

        // Cap the total boost to prevent one result from dominating
        total_boost.min(0.3) // Maximum 30% boost
    }

    /// Calculates length optimization factor.
    ///
    /// Different text lengths are optimal for different purposes:
    /// - Very short: Usually fragments or titles (not helpful for content)
    /// - Short-medium: Good for quick answers and summaries
    /// - Medium: Sweet spot for most search results
    /// - Long: Can be comprehensive but might be too verbose
    /// - Very long: Often contains irrelevant information
    fn calculate_length_factor(&self, length: usize) -> f64 {
        match length {
            0..=50 => 0.5,      // Too short - major penalty
            51..=150 => 1.0,    // Short but complete - neutral
            151..=500 => 1.1,   // Sweet spot - small bonus
            501..=1000 => 1.05, // Still good - tiny bonus
            1001..=2000 => 1.0, // Getting long - neutral
            _ => 0.9,           // Too long - small penalty
        }
    }

    /// Calculates position-based decay factor.
    ///
    /// Results that were originally ranked higher by Qdrant probably have
    /// better semantic similarity. We give them a tiny preference to maintain
    /// some of that signal while still allowing our algorithms to reorder.
    fn calculate_position_factor(&self, position: usize) -> f64 {
        // Very gentle decay: positions 0,1,2,3... get factors 0.00, 0.01, 0.02, 0.03...
        // This means position 0 gets no penalty, position 10 gets 1% penalty, etc.
        (position as f64 * 0.01).min(0.2) // Cap at 20% penalty for very late results
    }

    /// Calculates completeness bonus for well-formed text.
    ///
    /// Complete sentences and paragraphs are generally more useful than
    /// fragments. This algorithm rewards proper sentence structure.
    fn calculate_completeness_bonus(&self, text: &str) -> f64 {
        let mut bonus = 0.0;

        // Small bonus for proper sentence endings
        if text.trim().ends_with(['.', '!', '?']) {
            bonus += 0.02; // 2% bonus
        }

        // Bonus for having multiple complete sentences
        let sentence_count = text.matches(['.', '!', '?']).count();
        if sentence_count >= 3 {
            bonus += 0.03; // Additional 3% bonus
        }

        // Bonus for having both uppercase and lowercase (indicates proper text)
        let has_upper = text.chars().any(|c| c.is_uppercase());
        let has_lower = text.chars().any(|c| c.is_lowercase());
        if has_upper && has_lower {
            bonus += 0.01; // 1% bonus for proper capitalization
        }

        bonus
    }
}
