from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List
from embedding import get_embedding
from qdrant_service import search_vectors, client
from rust_bridge import re_rank_results
from config import settings
from utils.logging_config import logger
import time

router = APIRouter()


class SearchRequest(BaseModel):
    query: str
    limit: int = 10
    threshold: float = 0.7


class SearchResult(BaseModel):
    id: str
    text: str
    score: float
    metadata: dict


class SearchResponse(BaseModel):
    results: List[SearchResult]
    query_time_ms: int
    total_found: int


@router.post("/search", response_model=SearchResponse)
async def search_documents(request: SearchRequest):
    if not settings.QDRANT_URL:
        raise HTTPException(status_code=500, detail="Qdrant URL is not configured")

    if not request.query.strip():
        raise HTTPException(status_code=400, detail="Query cannot be empty")

    start_time = time.time()

    try:
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

        try:
            ranked_response = await re_rank_results(
                query=request.query, results=raw_results, limit=request.limit
            )

            ranked_results = ranked_response["results"]
            processing_time_ms = ranked_response.get("processing_time_ms")

        except Exception as e:
            logger.error(f"Re-ranking failed: {e}")
            ranked_results = raw_results[: request.limit]
            processing_time_ms = None  # fallback to Python timer

        search_results = []
        for result in ranked_results:
            search_results.append(
                SearchResult(
                    id=str(result["id"]),
                    text=(
                        result["payload"]["text"]
                        if "payload" in result
                        else result["text"]
                    ),
                    score=result["score"],
                    metadata=(
                        result["payload"]["metadata"]
                        if "payload" in result and "metadata" in result["payload"]
                        else result.get("metadata", {})
                    ),
                )
            )

        end_time = time.time()
        fallback_ms = int((end_time - start_time) * 1000)

        return SearchResponse(
            results=search_results,
            query_time_ms=processing_time_ms or fallback_ms,
            total_found=len(raw_results),
        )

    except Exception as e:
        logger.error(f"Search failed: {e}")
        raise HTTPException(
            status_code=500, detail="Internal server error during search"
        )
