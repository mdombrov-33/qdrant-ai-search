import httpx
from typing import List, Dict, Any
from config import settings
from utils.logging_config import logger
from typing import Optional
from exceptions import RustServiceError


async def re_rank_results(
    query: str,
    results: List[Dict[str, Any]],
    limit: int = 10,
    idf_map: Optional[dict[str, float]] = None,
    threshold: float = 0.7,
    timeout: float = 5.0,
) -> Dict[str, Any]:
    """
    Send search results to Rust service for re-ranking and filtering.

    Args:
        query (str): Original search query
        results (List[Dict]): Raw results from Qdrant
        limit (int): Maximum number of results to return
        timeout (float): Request timeout in seconds

    Returns:
        Dict[str, Any]: Re-ranked and filtered results

    Raises:
        RustServiceError: If Rust service fails or times out
    """
    logger.info(
        f"Re-ranking {len(results)} results for query: '{query[:100]}...' "
        f"(limit: {limit})"
    )

    if not results:
        logger.debug("No results to re-rank, returning empty list")
        return {"results": [], "query": query, "limit": limit}

    # Prepare payload for Rust service
    payload = {
        "query": query,
        "results": [
            {
                "id": str(result["id"]),
                "text": result["payload"]["text"],
                "score": result["score"],
                "metadata": {
                    "file_name": result["payload"].get("file_name", ""),
                    "content_type": result["payload"].get("content_type", ""),
                },
            }
            for result in results
        ],
        "limit": limit,
        "idf_map": idf_map or {},
        "threshold": threshold,
    }

    try:
        logger.debug(
            f"Sending re-rank request to Rust service: "
            f"{settings.RUST_SERVICE_URL}/re-rank"
        )
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                f"{settings.RUST_SERVICE_URL}/re-rank",
                json=payload,
                headers={"Content-Type": "application/json"},
            )

        logger.debug(f"Rust service responded with status: {response.status_code}")

        if response.status_code == 200:
            rust_response = response.json()
            results_count = len(rust_response.get("results", []))
            logger.info(
                f"Successfully re-ranked results, returning {results_count} items"
            )

            return rust_response

        else:
            error_msg = f"Rust service returned {response.status_code}: {response.text}"
            logger.error(error_msg)
            raise RustServiceError(error_msg)

    except httpx.TimeoutException:
        error_msg = "Rust service timeout"
        logger.error(error_msg)
        raise RustServiceError(error_msg)
    except httpx.RequestError as e:
        error_msg = f"Failed to connect to Rust service: {str(e)}"
        logger.error(error_msg)
        raise RustServiceError(error_msg)
    except Exception as e:
        error_msg = f"Unexpected error during re-ranking: {str(e)}"
        logger.error(error_msg)
        raise RustServiceError(error_msg)


async def health_check_rust_service() -> bool:
    """
    Check if Rust service is healthy and responding.

    Returns:
        bool: True if service is healthy, False otherwise
    """
    try:
        logger.debug("Performing health check on Rust service")
        async with httpx.AsyncClient(timeout=2.0) as client:
            response = await client.get(f"{settings.RUST_SERVICE_URL}/health")
            is_healthy = response.status_code == 200

            if is_healthy:
                logger.debug("Rust service health check passed")
            else:
                logger.warning(
                    f"Rust service health check failed with status: "
                    f"{response.status_code}"
                )

            return is_healthy
    except Exception as e:
        logger.error(f"Rust service health check failed: {str(e)}")
        return False
