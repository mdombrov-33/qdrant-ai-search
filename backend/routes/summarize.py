from fastapi import APIRouter, HTTPException
from models.summarize import SummarizeRequest, SummarizeResponse
from openai_chat_service import get_chat_completion, create_summary_messages
from config import settings
from utils.logging_config import logger
import time

router = APIRouter()


@router.post("/summarize", response_model=SummarizeResponse)
async def summarize_search_results(request: SummarizeRequest):
    if not settings.OPENAI_API_KEY:
        raise HTTPException(status_code=500, detail="OpenAI API key is not configured")

    if not request.query.strip():
        raise HTTPException(status_code=400, detail="Query cannot be empty")

    if not request.search_results:
        raise HTTPException(
            status_code=400, detail="No search results provided for summarization"
        )

    start_time = time.time()

    try:
        # Extract text chunks from SearchResult objects
        chunks = [result.text for result in request.search_results]

        logger.info(
            f"Summarizing {len(chunks)} chunks for query: '{request.query[:100]}...'"
        )

        # Create properly formatted messages for OpenAI
        messages = create_summary_messages(
            query=request.query, chunks=chunks, style=request.style
        )

        # Get summary from OpenAI
        summary = await get_chat_completion(messages)

        end_time = time.time()
        query_time_ms = int((end_time - start_time) * 1000)

        logger.info(f"Summary completed in {query_time_ms}ms for {len(chunks)} chunks")

        return SummarizeResponse(
            summary=summary,
            query_time_ms=query_time_ms,
            chunks_processed=len(chunks),
        )

    except Exception as e:
        logger.error(f"Summarization failed: {e}")
        raise HTTPException(
            status_code=500, detail="Internal server error during summarization"
        )
