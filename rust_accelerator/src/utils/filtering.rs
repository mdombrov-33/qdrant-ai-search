//! Utilities for early filtering of low-quality results.

use crate::models::request::SearchResult;
use std::collections::HashSet;

/// Handles early filtering of search results.
pub struct ResultFilter {
    /// Minimum text length to be considered valid
    min_text_length: usize,

    /// Maximum text length before we consider it too verbose
    max_text_length: usize,

    /// Minimum similarity score from Qdrant to be worth considering
    min_similarity_score: f64,

    /// Domain keywords for different subject areas
    domain_keywords: DomainKeywords,
}

/// Domain-specific keywords for relevance filtering
struct DomainKeywords {
    tech_programming: HashSet<&'static str>,
    biology_nature: HashSet<&'static str>,
    science_research: HashSet<&'static str>,
    business_finance: HashSet<&'static str>,
}

impl ResultFilter {
    pub fn new(min_similarity_score: f64) -> Self {
        Self {
            min_text_length: 10,   // At least 10 characters
            max_text_length: 5000, // No more than 5000 characters
            min_similarity_score,
            domain_keywords: DomainKeywords::new(),
        }
    }

    /// Determines if a result should be kept for further processing.
    ///
    /// Applies efficient quality checks in order of computational cost:
    /// 1. Fast length/similarity checks
    /// 2. Content quality evaluation
    /// 3. Spam pattern detection
    /// 4. Domain relevance check (NEW)
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

    /// Check domain relevance between query and result
    ///
    /// This method detects when query and result are from completely different domains
    /// and applies a stricter threshold to prevent irrelevant matches.
    pub fn should_keep_with_query(&self, result: &SearchResult, query: &str) -> bool {
        // First apply standard filtering
        if !self.should_keep(result) {
            return false;
        }

        // More aggressive domain mismatch detection
        let query_lower = query.to_lowercase();
        let result_lower = result.text.to_lowercase();

        // Strong domain indicators check
        let query_is_tech = self
            .domain_keywords
            .has_strong_tech_indicators(&query_lower);
        let result_is_bio = self
            .domain_keywords
            .has_strong_bio_indicators(&result_lower);

        // If query is clearly tech and result is clearly bio, reject completely
        if query_is_tech && result_is_bio {
            return false;
        }

        // Additional check: if query has "javascript" or "programming" but result has "panda" or "species"
        if (query_lower.contains("javascript") || query_lower.contains("programming"))
            && (result_lower.contains("panda")
                || result_lower.contains("species")
                || result_lower.contains("animal"))
        {
            return false;
        }

        // Use regular domain relevance as fallback
        let domain_relevance = self.calculate_domain_relevance(&query_lower, &result_lower);

        // Get domain classifications to check if either is general
        let query_domains = self.domain_keywords.classify_domains(&query_lower);
        let result_domains = self.domain_keywords.classify_domains(&result_lower);

        // If either query or result is classified as "general", be more lenient
        let is_general_query = query_domains.contains("general");
        let is_general_result = result_domains.contains("general");

        // Be very strict about domain mismatches, but lenient for general content
        if domain_relevance < 0.4 && !is_general_query && !is_general_result {
            return result.score >= 0.98; // Extremely high bar for cross-domain matches
        }

        true
    }

    /// Calculate domain relevance between query and result text
    fn calculate_domain_relevance(&self, query: &str, result_text: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let result_lower = result_text.to_lowercase();

        let query_domains = self.domain_keywords.classify_domains(&query_lower);
        let result_domains = self.domain_keywords.classify_domains(&result_lower);

        // Calculate overlap between domain classifications
        let overlap_count = query_domains.intersection(&result_domains).count();
        let total_domains = query_domains.union(&result_domains).count();

        if total_domains == 0 {
            return 1.0; // No domain classification, assume compatible
        }

        overlap_count as f64 / total_domains as f64
    }
}

impl DomainKeywords {
    fn new() -> Self {
        let mut tech_programming = HashSet::new();
        tech_programming.extend([
            "javascript",
            "python",
            "programming",
            "code",
            "function",
            "variable",
            "computer",
            "technology",
            "api",
            "database",
            "web",
            "app",
            "development",
            "framework",
            "library",
            "script",
            "debug",
            "compile",
            "syntax",
            "class",
            "method",
            "object",
            "array",
            "loop",
            "frontend",
            "backend",
            "html",
            "css",
            "node",
            "react",
            "angular",
            "vue",
            "typescript",
            "coding",
            "developer",
            "programming",
        ]);

        let mut biology_nature = HashSet::new();
        biology_nature.extend([
            "panda",
            "animal",
            "species",
            "habitat",
            "conservation",
            "wildlife",
            "forest",
            "bamboo",
            "ecology",
            "biodiversity",
            "endangered",
            "mammal",
            "genetics",
            "population",
            "ecosystem",
            "natural",
            "environment",
            "biological",
            "organism",
            "zoological",
            "fauna",
            "flora",
            "conservation",
            "mitochondrial",
            "dna",
            "genetic",
            "phylogenetic",
            "taxonomic",
            "morphological",
        ]);

        let mut science_research = HashSet::new();
        science_research.extend([
            "hypothesis",
            "methodology",
            "statistical",
            "survey",
            "sample",
            "scientific",
            "publication",
            "peer",
            "review",
            "findings",
            "conclusion",
            "evidence",
            "theory",
            "model",
            "observation",
            "measurement",
            "laboratory",
            "experiment",
            "clinical",
            "empirical",
        ]);

        let mut business_finance = HashSet::new();
        business_finance.extend([
            "business",
            "finance",
            "market",
            "investment",
            "revenue",
            "profit",
            "company",
            "corporate",
            "management",
            "strategy",
            "economics",
            "financial",
            "banking",
            "trade",
            "commerce",
            "industry",
            "sales",
        ]);

        Self {
            tech_programming,
            biology_nature,
            science_research,
            business_finance,
        }
    }

    /// Classify which domains a text belongs to based on keyword presence
    fn classify_domains(&self, text: &str) -> HashSet<&'static str> {
        let mut domains = HashSet::new();

        // Count matches in each domain
        let tech_matches = self.count_matches(text, &self.tech_programming);
        let bio_matches = self.count_matches(text, &self.biology_nature);
        let science_matches = self.count_matches(text, &self.science_research);
        let business_matches = self.count_matches(text, &self.business_finance);

        // Classify based on strong keyword presence (at least 3 matches for specificity)
        // Also check for primary domain indicators
        if tech_matches >= 3 || self.has_strong_tech_indicators(text) {
            domains.insert("technology");
        }
        if bio_matches >= 3 || self.has_strong_bio_indicators(text) {
            domains.insert("biology");
        }
        if science_matches >= 3 {
            domains.insert("science");
        }
        if business_matches >= 3 {
            domains.insert("business");
        }

        // If no strong domain signals, classify as general
        if domains.is_empty() {
            domains.insert("general");
        }

        domains
    }

    /// Check for strong technology domain indicators
    fn has_strong_tech_indicators(&self, text: &str) -> bool {
        let strong_tech_terms = [
            "javascript",
            "python",
            "programming",
            "coding",
            "html",
            "css",
            "react",
            "angular",
            "vue",
            "typescript",
            "nodejs",
            "frontend",
            "backend",
        ];
        strong_tech_terms.iter().any(|&term| text.contains(term))
    }

    /// Check for strong biology domain indicators  
    fn has_strong_bio_indicators(&self, text: &str) -> bool {
        let strong_bio_terms = [
            "panda",
            "animal",
            "species",
            "wildlife",
            "conservation",
            "genetics",
            "dna",
            "mitochondrial",
            "population",
            "ecosystem",
            "habitat",
        ];
        strong_bio_terms.iter().any(|&term| text.contains(term))
    }

    fn count_matches(&self, text: &str, keywords: &HashSet<&'static str>) -> usize {
        keywords
            .iter()
            .filter(|&&keyword| text.contains(keyword))
            .count()
    }
}
