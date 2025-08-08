from fastapi import APIRouter, HTTPException
from utils.idf import compute_idf
from models.search import SearchRequest, SearchResult, SearchResponse
from services.embedding import get_embedding
from services.qdrant_service import search_vectors, client
from services.rust_bridge import re_rank_results
from config import settings
from utils.logging_config import logger
from utils.exception_handler import handle_custom_exception
from exceptions import QdrantSearchError
import time

router = APIRouter()


@router.post("/search", response_model=SearchResponse)
async def search_documents(request: SearchRequest):
    """
    Search documents by semantic similarity and IDF-aware re-ranking.

    Request body (SearchRequest):
    - query: User query text to embed and search against.
    - limit: Max results to return (fetches up to limit*5 candidates,
        capped at 100).
    - threshold: Minimum vector similarity score for candidate retrieval
        and filtering.

    Internals:
    - idf_map: Computed from candidates via utils.idf.compute_idf and passed
        to the re-ranker to boost rare terms.

    Returns:
    - SearchResponse with:
        - results: list of {id, text, score, metadata}
        - query_time_ms: processing time in ms
        - total_found: number of raw candidates found
    """
    try:
        if not settings.QDRANT_URL:
            raise HTTPException(status_code=500, detail="Qdrant URL is not configured")

        if not request.query.strip():
            raise HTTPException(status_code=400, detail="Query cannot be empty")

        start_time = time.time()

        query_embedding = await get_embedding(request.query)

        raw_results = search_vectors(
            client=client,
            collection_name=settings.QDRANT_COLLECTION_NAME,
            query_vector=query_embedding,
            limit=min(request.limit * 5, 100),
            score_threshold=request.threshold,
        )

        if not raw_results:
            return SearchResponse(results=[], query_time_ms=0, total_found=0)

        # Map original payloads by id for later metadata enrichment
        payload_by_id = {str(r["id"]): r.get("payload", {}) for r in raw_results}

        documents = [result["payload"]["text"] for result in raw_results]
        idf_map = compute_idf(documents)

        try:
            ranked_response = await re_rank_results(
                query=request.query,
                results=raw_results,
                limit=request.limit,
                idf_map=idf_map,
                threshold=request.threshold,
            )

            ranked_results = ranked_response["results"]
            processing_time_ms = ranked_response.get("processing_time_ms")

        except Exception as e:
            logger.error(f"Re-ranking failed: {e}")
            ranked_results = raw_results[: request.limit]
            processing_time_ms = None  # fallback to Python timer

        # Minimal, safe metadata fields to expose by default
        METADATA_WHITELIST = {
            "file_name",
            "content_type",
            "chunk_index",
            "word_count",
            "chunk_type",
        }

        search_results = []
        for result in ranked_results:
            # Start with metadata from Rust (if present)
            base_metadata = result.get("metadata", {})
            # Merge with selected fields from the original Qdrant payload
            original_payload = payload_by_id.get(str(result.get("id"))) or {}
            enriched = {
                k: v for k, v in original_payload.items() if k in METADATA_WHITELIST
            }
            # Rust-provided keys should win if overlaps
            merged_metadata = {**enriched, **base_metadata}
            # Prefer "text" field from Rust response.
            # Fallback to payload.text for raw Qdrant items when Rust isn't used.
            text_value = result.get("text")
            if not text_value and "payload" in result:
                text_value = result["payload"].get("text")
            search_results.append(
                SearchResult(
                    id=str(result["id"]),
                    text=text_value or "",
                    score=result["score"],
                    metadata=merged_metadata,
                )
            )

        end_time = time.time()
        fallback_ms = int((end_time - start_time) * 1000)

        return SearchResponse(
            results=search_results,
            query_time_ms=processing_time_ms or fallback_ms,
            total_found=len(raw_results),
        )

    except QdrantSearchError as e:
        raise handle_custom_exception(e)
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Search failed: {e}")
        raise HTTPException(
            status_code=500, detail="Internal server error during search"
        )
