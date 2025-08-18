//! Scoring algorithms for result enhancement.

use crate::models::internal::QueryFeatures;
use crate::models::request::SearchResult;

/// Configuration for score weighting.
pub struct ScoreWeights {
    pub text_quality: f64,        // Content quality weight
    pub keyword_matching: f64,    // Keyword overlap weight
    pub length_optimization: f64, // Ideal length weight
    pub position_decay: f64,      // Position weight
    pub completeness: f64,        // Completeness weight
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            text_quality: 0.3,
            keyword_matching: 0.4,
            length_optimization: 0.2,
            position_decay: 0.02,
            completeness: 0.05,
        }
    }
}

/// Calculates enhanced scores using multiple algorithms.
pub struct ScoreCalculator {
    weights: ScoreWeights,
}

impl ScoreCalculator {
    pub fn new() -> Self {
        Self {
            weights: ScoreWeights::default(),
        }
    }

    /// Calculates an enhanced score using multiple strategies.
    pub fn calculate_enhanced_score(
        &self,
        result: &SearchResult,
        query_features: &QueryFeatures,
        position: usize,
    ) -> f64 {
        // Start with the original Qdrant similarity score
        let base_score = result.score;

        //* */ === ALGORITHM 1: TEXT QUALITY ADJUSTMENT ===
        // Apply as a smaller multiplier to avoid too much penalty
        let quality_factor = self.calculate_text_quality_factor(&result.text);
        let quality_adjustment = (quality_factor - 1.0) * self.weights.text_quality;

        //* */ === ALGORITHM 2: KEYWORD MATCHING BOOST ===
        //* */ This should be additive to reward exact matches
        let keyword_boost = self.calculate_keyword_boost(&result.text, query_features);

        //* */ === ALGORITHM 3: LENGTH OPTIMIZATION ===
        //* */ Apply as smaller adjustment, not harsh multiplier
        let length_factor = self.calculate_length_factor(result.text.len());
        let length_adjustment = (length_factor - 1.0) * self.weights.length_optimization;

        //* */ === ALGORITHM 4: POSITION DECAY ===
        //* */ Keep as small multiplicative penalty
        let position_factor = self.calculate_position_factor(position);
        let position_penalty = position_factor * self.weights.position_decay;

        //* */ === ALGORITHM 5: COMPLETENESS BONUS ===
        //* */ Small bonus for complete sentences and well-formed text
        let completeness_bonus = self.calculate_completeness_bonus(&result.text);

        //* */ Combine all factors more conservatively
        let mut final_score = base_score;
        final_score += quality_adjustment; // Add/subtract quality
        final_score += keyword_boost * self.weights.keyword_matching; // Add keyword boost
        final_score += length_adjustment; // Add/subtract length
        final_score *= 1.0 - position_penalty; // Small position penalty
        final_score += completeness_bonus * self.weights.completeness; // Add completeness

        //* */ Ensure score stays in valid range but allow enhancement above 1.0
        final_score.clamp(0.0, 2.0)
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

        // 1. Length check - minimum 20 chars
        if text_len < 20 {
            // Scale penalty linearly from 10-20 chars - less punitive
            return 0.7 + (text_len as f64 / 20.0 * 0.3);
        }

        // 2. Sentence structure analysis
        let ends_with_punctuation = text.trim().ends_with(['.', '!', '?']);
        let sentence_count = text.matches(['.', '!', '?']).count();

        // 3. Quality scoring - much less punitive
        match (ends_with_punctuation, sentence_count) {
            // Ideal case: Properly terminated with 1-3 sentences
            (true, 1..=3) => 1.1, // Good quality - 10% boost

            // Good but could be better - less penalty
            (true, 0) => 1.0,  // Neutral - has punctuation
            (true, _) => 1.05, // Many sentences but properly terminated

            // Problem cases - much less punitive
            (false, 0) => 0.95, // Very slight penalty
            (false, _) => 0.98, // Tiny penalty for fragments
        }
    }

    /// Calculates keyword matching boost using phrase-aware scoring.
    ///
    /// This algorithm rewards results that contain words and phrases from the search query.
    /// It provides stronger boosts for:
    /// 1. Exact phrase matches (e.g. "neural networks")
    /// 2. Multiple occurrences of query terms
    /// 3. Rare terms in the query
    ///
    /// Example: Query "convolutional layers"
    /// - Exact phrase match: +0.3 boost
    /// - Individual word matches: +0.1 each
    /// - Total possible boost: 0.5 (50%)
    fn calculate_keyword_boost(&self, text: &str, query_features: &QueryFeatures) -> f64 {
        let text_lower = text.to_lowercase();
        let mut total_boost = 0.0;

        // Check each term from the query (both words and phrases)
        for (term, query_frequency) in &query_features.word_frequencies {
            // Case 1: Exact phrase match (e.g. "neural networks")
            if term.contains(' ') && text_lower.contains(term) {
                // Strong boost for exact phrase matches
                total_boost += 0.4 * (*query_frequency as f64);
                continue;
            }

            // Case 2: Individual word matches
            let text_frequency = text_lower.matches(term).count();
            if text_frequency > 0 {
                // Calculate boost based on:
                // - Term frequency in query (sqrt-weighted)
                // - Term frequency in text
                let idf = query_features.idf_map.get(term).copied().unwrap_or(1.0);
                let weight = ((*query_frequency as f64).sqrt()) * idf;
                let boost = (text_frequency as f64) * weight * 0.1;
                total_boost += boost;
            }
        }

        // Cap the total boost to prevent over-domination
        total_boost.min(0.25) // Maximum 25% boost
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
            0..=50 => 0.9,      // Short - small penalty instead of major
            51..=150 => 1.0,    // Short but complete - neutral
            151..=500 => 1.05,  // Sweet spot - small bonus
            501..=1000 => 1.02, // Still good - tiny bonus
            1001..=2000 => 1.0, // Getting long - neutral
            _ => 0.98,          // Too long - very small penalty
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
