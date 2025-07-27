use serde::Deserialize;

/// Metadata about the source of the text (filename, content type, etc.)
#[derive(Debug, Deserialize)]
pub struct Metadata {
    /// Original file name, e.g. "ai_safety.pdf"
    pub file_name: String,

    /// MIME type of the document, e.g. "application/pdf"
    pub content_type: String,
}

/// Single result returned from FastAPI (before reranking)
#[derive(Debug, Deserialize)]
pub struct SearchResultInput {
    /// Unique identifier for the chunk (UUID or other string)
    pub id: String,

    /// Text content of the document chunk
    pub text: String,

    /// Original similarity score from Qdrant (0.0 to 1.0)
    pub score: f32,

    /// Additional metadata like filename/type
    pub metadata: Metadata,
}

/// Full incoming POST body to /re-rank endpoint
#[derive(Debug, Deserialize)]
pub struct ReRankRequest {
    /// Natural language query string
    pub query: String,

    /// List of document chunks retrieved from Qdrant
    pub results: Vec<SearchResultInput>,

    /// Optional limit for number of results to return
    pub limit: Option<usize>,
}
