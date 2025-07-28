# API Routes Documentation

This document describes all available API endpoints in the Qdrant AI Search backend. All routes use JSON for request/response bodies unless otherwise specified.

## Base URL

All API routes are prefixed with `/api` except for health and configuration endpoints.

- **Development**: `http://localhost:8000`
- **Production**: Your deployed URL

## Error Handling

All endpoints follow consistent error response patterns:

````json
{
  "detail": "Error description"
}

Common HTTP status codes:
- `200` - Success
- `400` - Bad Request (invalid input)
- `500` - Internal Server Error (API failures, configuration issues)

---

## Health Check

### GET /health

Simple health check endpoint to verify the service is running.

**Request:**
```bash
curl -X GET http://localhost:8000/health
````

**Response:**

```json
{
  "status": "ok"
}
```

---

## Document Upload

### POST /api/upload

Upload and process documents for semantic search. Supports PDF, DOCX, and TXT files. Documents are automatically chunked using semantic boundaries and embedded using OpenAI's text-embedding-ada-002 model.

**Request:**

```bash
curl -X POST \
  -F "file=@document.pdf" \
  http://localhost:8000/api/upload
```

**File Constraints:**

- Maximum size: 50MB
- Supported formats: PDF, DOCX, TXT
- Files are processed with advanced semantic chunking for optimal search results

**Response:**

```json
{
  "filename": "document.pdf",
  "content_type": "application/pdf",
  "size": 1024576,
  "chunks_count": 45,
  "message": "File processed and embedded successfully."
}
```

**Error Examples:**

```bash
# File too large
{
  "detail": "File too large. Maximum size is 50MB"
}

# Unsupported format
{
  "detail": "Unsupported file type. Please upload a PDF, TXT, or DOCX file."
}
```

---

## Semantic Search

### POST /api/search

Perform semantic search across uploaded documents using natural language queries. Results are ranked using both vector similarity and a Rust-powered re-ranking algorithm for optimal relevance.

**Request:**

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "machine learning algorithms for text analysis",
    "limit": 5,
    "threshold": 0.7
  }' \
  http://localhost:8000/api/search
```

**Request Body:**

```json
{
  "query": "string", // Required: Search query in natural language
  "limit": 10, // Optional: Maximum results to return (default: 10)
  "threshold": 0.7, // Optional: Minimum similarity score (default: 0.7)
  "idf_map": {} // Optional: IDF boosting map (advanced usage)
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "uuid-string",
      "text": "Machine learning algorithms, particularly neural networks...",
      "score": 0.95,
      "metadata": {
        "file_name": "ml_guide.pdf",
        "content_type": "application/pdf",
        "chunk_index": 12,
        "word_count": 245,
        "chunk_type": "heading_section"
      }
    }
  ],
  "query_time_ms": 45, // Total processing time including Rust re-ranking
  "total_found": 23 // Total matches before limit applied
}
```

**Search Process:**

1. Query is embedded using OpenAI's embedding model
2. Vector similarity search performed against Qdrant database
3. Results sent to Rust microservice for re-ranking and filtering
4. Top results returned with enhanced scoring

**Error Examples:**

```bash
# Empty query
{
  "detail": "Query cannot be empty"
}

# Qdrant not configured
{
  "detail": "Qdrant URL is not configured"
}
```

---

## Document Summarization

### POST /api/summarize

Generate intelligent summaries from search results. Takes search results and the original query to create contextual summaries that directly answer the user's question.

**Request:**

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the main machine learning algorithms?",
    "search_results": [
      {
        "id": "uuid-1",
        "text": "Neural networks are a class of machine learning algorithms...",
        "score": 0.95,
        "metadata": {}
      },
      {
        "id": "uuid-2",
        "text": "Decision trees provide interpretable classification...",
        "score": 0.88,
        "metadata": {}
      }
    ],
    "style": "comprehensive"
  }' \
  http://localhost:8000/api/summarize
```

**Request Body:**

```json
{
  "query": "string", // Required: Original search query for context
  "search_results": [
    // Required: Array of SearchResult objects
    {
      "id": "string",
      "text": "string",
      "score": 0.95,
      "metadata": {}
    }
  ],
  "style": "comprehensive" // Optional: Summary style (see below)
}
```

**Summary Styles:**

- `comprehensive` (default) - Detailed, thorough summary that fully answers the query
- `brief` - Concise summary focusing on key points
- `bullet_points` - Structured bullet-point format highlighting main findings

**Response:**

```json
{
  "summary": "Based on the provided documents, the main machine learning algorithms include neural networks, which are powerful for pattern recognition and complex data analysis, and decision trees, which offer interpretable classification with clear decision paths. Neural networks excel at handling non-linear relationships and large datasets, while decision trees provide transparency in decision-making processes...",
  "query_time_ms": 1250, // Time taken for OpenAI processing
  "chunks_processed": 2 // Number of search results processed
}
```

**Typical Workflow:**

1. User searches: `POST /api/search` with query
2. User summarizes results: `POST /api/summarize` with same query + results
3. Get contextual summary that answers the original question

**Error Examples:**

```bash
# No search results provided
{
  "detail": "No search results provided for summarization"
}

# Empty query
{
  "detail": "Query cannot be empty"
}

# OpenAI API issues
{
  "detail": "OpenAI API error: 429 - Rate limit exceeded"
}
```

---

## Complete Usage Example

Here's a complete workflow from document upload to summarized results:

### 1. Upload a document

```bash
curl -X POST \
  -F "file=@research_paper.pdf" \
  http://localhost:8000/api/upload
```

### 2. Search for information

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the limitations of current AI systems?",
    "limit": 3
  }' \
  http://localhost:8000/api/search > search_results.json
```

### 3. Generate summary from results

```bash
# Extract results from search_results.json and use in summarize request
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What are the limitations of current AI systems?",
    "search_results": [/* results from step 2 */],
    "style": "bullet_points"
  }' \
  http://localhost:8000/api/search
```

## Performance Notes

- **Search**: Typically 20-100ms depending on corpus size and Rust re-ranking
- **Upload**: Time varies with document size and content complexity
- **Summarization**: 1-5 seconds depending on content length and OpenAI API latency
- **Concurrent requests**: Upload and search support multiple concurrent requests
- **Rate limiting**: OpenAI API calls include automatic retry with exponential backoff

## Configuration

The API behavior can be controlled via environment variables:

- `OPENAI_API_KEY` - Required for embeddings and summarization
- `QDRANT_URL` - Vector database connection
- `RUST_SERVICE_URL` - Re-ranking microservice endpoint
- `OPENAI_CHAT_MODEL` - Model for summarization (default: gpt-4o)
- `MAX_SUMMARY_TOKENS` - Maximum tokens in summaries (default: 500)
- `SUMMARY_TEMPERATURE` - Creativity level for summaries (default: 0.3)
