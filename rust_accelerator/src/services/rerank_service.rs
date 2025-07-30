//! This is the main re-ranking service that orchestrates all the algorithms.
//!
//! Think of this as the "controller" in an MVC pattern - it coordinates between
//! different specialized modules to transform raw search results into optimized ones.

use crate::error::AppError;
use crate::models::internal::EnhancedResult;
use crate::models::request::ReRankRequest;
use crate::models::response::{ReRankResponse, ReRankedResult};
use crate::utils::filtering::ResultFilter;
use crate::utils::scoring::ScoreCalculator;
use crate::utils::similarity::SimilarityCalculator;
use crate::utils::text_analysis::TextAnalyzer;
use crate::utils::timing::elapsed_ms;
use std::time::Instant;

/// Main re-ranking service that combines multiple algorithms for optimal results.
///
/// This struct holds all the components needed for re-ranking:
/// - Text analyzer for preprocessing
/// - Score calculator for algorithm application  
/// - Result filter for quality control
/// - Similarity calculator for deduplication
pub struct DocumentReRanker {
    /// Handles all text preprocessing and analysis
    text_analyzer: TextAnalyzer,

    /// Calculates enhanced scores using multiple algorithms
    score_calculator: ScoreCalculator,

    /// Filters out low-quality results early in the pipeline
    result_filter: ResultFilter,

    /// Handles deduplication using text similarity algorithms
    similarity_calculator: SimilarityCalculator,
}

impl DocumentReRanker {
    /// Creates a new instance with all components initialized.
    ///
    /// This is like a constructor in OOP languages. We initialize all our
    /// utility components here so they're ready to use.
    pub fn new(req: &ReRankRequest) -> Self {
        Self {
            text_analyzer: TextAnalyzer::new(),
            score_calculator: ScoreCalculator::new(),
            result_filter: ResultFilter::new(req.threshold),
            similarity_calculator: SimilarityCalculator::new(),
        }
    }

    /// Main entry point for re-ranking documents.
    ///
    /// This function orchestrates the entire re-ranking pipeline:
    /// 1. Validate input
    /// 2. Extract query features (preprocessing)
    /// 3. Filter and enhance results
    /// 4. Sort by enhanced scores
    /// 5. Remove duplicates
    /// 6. Apply final limits
    ///
    /// The `async` keyword means this function can be paused and resumed,
    /// allowing other tasks to run while we wait.
    pub async fn rerank_documents(&self, req: ReRankRequest) -> Result<ReRankResponse, AppError> {
        let start = Instant::now();

        //* */ === STEP 1: INPUT VALIDATION ===
        // We validate early to fail fast - no point processing invalid data
        self.validate_input(&req)?;

        //* */ === STEP 2: QUERY PREPROCESSING ===
        // Extract meaningful features from the search query once, then reuse
        // This is like tokenizing and preprocessing in NLP pipelines
        let query_features = self
            .text_analyzer
            .extract_query_features(&req.query, req.idf_map.clone());

        //* */ === STEP 3: PARALLEL RESULT PROCESSING WITH DOMAIN FILTERING ===
        // Process all results in parallel, filtering and enhancing scores
        // The `into_iter()` takes ownership, `enumerate()` adds position index
        let mut enhanced_results: Vec<EnhancedResult> = req
            .results
            .into_iter()
            .enumerate()
            .filter_map(|(position, result)| {
                // Use domain-aware filtering instead of simple filtering
                if !self
                    .result_filter
                    .should_keep_with_query(&result, &req.query)
                {
                    return None; // `None` means "skip this item"
                }

                // Calculate enhanced score using multiple algorithms
                let enhanced_score = self.score_calculator.calculate_enhanced_score(
                    &result,
                    &query_features,
                    position,
                );

                // Wrap in our internal struct for further processing
                Some(EnhancedResult {
                    original: result,
                    enhanced_score,
                    original_position: position,
                })
            })
            .collect(); // Convert iterator back to Vec

        //* */ === STEP 4: ADVANCED SORTING ===
        // Sort by enhanced score (descending), with original position as tiebreaker
        enhanced_results.sort_by(|a, b| {
            match b.enhanced_score.partial_cmp(&a.enhanced_score) {
                Some(std::cmp::Ordering::Equal) => {
                    // If scores are equal, prefer earlier results
                    a.original_position.cmp(&b.original_position)
                }
                other => other.unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        //* */ === STEP 5: DEDUPLICATION ===
        // Remove results that are too similar to each other
        let deduplicated = self
            .similarity_calculator
            .remove_duplicates(enhanced_results);

        //* */ === STEP 6: FINAL FORMATTING ===
        // Convert to response format and apply limit
        let final_results: Vec<ReRankedResult> = deduplicated
            .into_iter()
            .filter(|enhanced| enhanced.enhanced_score >= req.threshold) // Final threshold check
            .take(req.limit.min(50)) // Cap at 50 for performance
            .map(|enhanced| ReRankedResult {
                id: enhanced.original.id,
                text: enhanced.original.text,
                score: enhanced.enhanced_score,
                metadata: enhanced.original.metadata,
            })
            .collect();

        let processing_time_ms = elapsed_ms(start);

        Ok(ReRankResponse {
            results: final_results,
            processing_time_ms,
        })
    }

    /// Validates the incoming request for basic sanity.
    ///
    /// This is defensive programming - we check inputs early to provide
    /// clear error messages rather than mysterious failures later.
    ///
    /// The `?` operator is Rust's way of early returning on error.
    fn validate_input(&self, req: &ReRankRequest) -> Result<(), AppError> {
        if req.query.trim().is_empty() {
            return Err(AppError::InvalidInput("Query string is empty".into()));
        }

        if req.results.is_empty() {
            return Err(AppError::InvalidInput("Results list is empty".into()));
        }

        if req.limit == 0 {
            return Err(AppError::InvalidInput(
                "Limit must be greater than 0".into(),
            ));
        }

        Ok(()) // Success case - no error
    }
}

/// Public interface function that your HTTP handler will call.
///
/// Creates a new DocumentReRanker instance for each request to ensure
/// the correct threshold is used. The performance impact is minimal since
/// the expensive operations (text analysis, scoring) happen during processing,
/// not during initialization.
pub async fn rerank_documents(req: ReRankRequest) -> Result<ReRankResponse, AppError> {
    let reranker = DocumentReRanker::new(&req);
    reranker.rerank_documents(req).await
}
