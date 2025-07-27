//! Internal data structures used during processing.
//!
//! These are "private" models that help us organize data during computation
//! but aren't exposed in our API. Think of them as implementation details.

use crate::models::request::SearchResult;
use std::collections::HashMap;

/// Features extracted from a search query for reuse across all results.
///
/// Instead of re-analyzing the query for every result, we do it once upfront.
/// This is a common optimization pattern in information retrieval systems.
///
/// In Python, this would be a dataclass:
/// ```python
/// @dataclass
/// class QueryFeatures:
///     original: str
///     meaningful_words: List[str]
///     word_frequencies: Dict[str, int]
///     total_words: int
/// ```
#[derive(Debug, Clone)]
pub struct QueryFeatures {
    /// Frequency count for TF-IDF style weighting
    /// Example: {"AI": 1, "risks": 1}
    pub word_frequencies: HashMap<String, usize>,

    pub idf_map: HashMap<String, f64>,
}

/// Internal wrapper for results during processing.
///
/// We need extra fields during computation that aren't in the final response.
/// This struct holds both the original data and our computed enhancements.
#[derive(Debug)]
pub struct EnhancedResult {
    /// Original search result from Qdrant
    pub original: SearchResult,

    /// New score after applying all our algorithms
    pub enhanced_score: f64,

    /// Original position in the result list (for tie-breaking)
    pub original_position: usize,
}
