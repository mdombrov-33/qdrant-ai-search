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

/// Re-ranking service combining multiple algorithms for optimal results.
pub struct DocumentReRanker {
    /// Text preprocessing and analysis
    text_analyzer: TextAnalyzer,

    /// Enhanced scoring
    score_calculator: ScoreCalculator,

    /// Early filtering
    result_filter: ResultFilter,

    /// Deduplication
    similarity_calculator: SimilarityCalculator,
}

impl DocumentReRanker {
    /// Creates a new instance with all components.
    pub fn new(req: &ReRankRequest) -> Self {
        Self {
            text_analyzer: TextAnalyzer::new(),
            score_calculator: ScoreCalculator::new(),
            result_filter: ResultFilter::new(req.threshold),
            similarity_calculator: SimilarityCalculator::new(),
        }
    }

    /// Main entry point for re-ranking documents.
    pub async fn rerank_documents(&self, req: ReRankRequest) -> Result<ReRankResponse, AppError> {
        let start = Instant::now();

        // Step 1: Input validation
        self.validate_input(&req)?;

        // Step 2: Query preprocessing
        let query_features = self
            .text_analyzer
            .extract_query_features(&req.query, req.idf_map.clone());

        // Step 3: Parallel result processing with domain filtering
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

/// Public interface function that HTTP handler will call.
///
/// Creates a new DocumentReRanker instance for each request to ensure
/// the correct threshold is used. The performance impact is minimal since
/// the expensive operations (text analysis, scoring) happen during processing,
/// not during initialization.
pub async fn rerank_documents(req: ReRankRequest) -> Result<ReRankResponse, AppError> {
    let reranker = DocumentReRanker::new(&req);
    reranker.rerank_documents(req).await
}
