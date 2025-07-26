import asyncio
import os
import httpx
from fastapi import HTTPException
import random

OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
if not OPENAI_API_KEY:
    raise ValueError("OPENAI_API_KEY environment variable is not set")

OPENAI_EMBEDDING_URL = "https://api.openai.com/v1/embeddings"


def chunk_text(text, max_words: int = 1000) -> list[str]:
    """
    Splits the input text into chunks of a specified maximum number of words.

    Args:
        text (str): The input text to be chunked.
        max_words (int): The maximum number of words per chunk. Default is 1000.

    Returns:
        list[str]: A list of text chunks.
    """
    words = text.split()
    # Split the list of words into chunks of size max_words,
    # then join each chunk back into a single string with spaces.
    # The result is a list of strings, each string is a chunk of the original text.
    # For example, if max_words=1000, it creates chunks of 1000 words each.
    return [" ".join(words[i : i + max_words]) for i in range(0, len(words), max_words)]


def _extract_error_message(response: httpx.Response) -> str:
    """Extract error message from OpenAI API response."""
    try:
        error_response = response.json()
        if "error" in error_response:
            return error_response["error"].get("message", "Unknown error")
    except Exception:
        pass
    return response.text


async def _make_embedding_request(text: str) -> list[float]:
    """Make a single embedding request to OpenAI API."""
    headers = {
        "Authorization": f"Bearer {OPENAI_API_KEY}",
        "Content-Type": "application/json",
    }

    json_data = {
        "model": "text-embedding-ada-002",
        "input": text,
    }

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            OPENAI_EMBEDDING_URL, headers=headers, json=json_data
        )
        response.raise_for_status()
        data = response.json()
        return data["data"][0]["embedding"]


async def get_embedding(text: str, max_retries: int = 3) -> list[float]:
    """
    Get embedding for text using OpenAI's API.

    Args:
        text (str): The text to embed
        max_retries (int): Maximum number of retry attempts for rate limiting

    Returns:
        list[float]: The embedding vector

    Raises:
        HTTPException: On API errors, timeouts, or other failures
    """
    last_exception = None

    for attempt in range(max_retries):
        try:
            return await _make_embedding_request(text)

        except httpx.HTTPStatusError as e:
            last_exception = e
            # Handle rate limiting with exponential backoff
            if e.response.status_code == 429 and attempt < max_retries - 1:
                wait_time = (2**attempt) + random.uniform(0, 1)
                await asyncio.sleep(wait_time)
                continue

            # Handle other HTTP errors - don't retry
            error_message = _extract_error_message(e.response)
            raise HTTPException(
                status_code=500,
                detail=f"OpenAI API error: {e.response.status_code} - {error_message}",
            )

        except (httpx.TimeoutException, httpx.RequestError) as e:
            last_exception = e
            if attempt < max_retries - 1:
                wait_time = 2**attempt
                await asyncio.sleep(wait_time)
                continue

        except Exception as e:
            # Unexpected errors - don't retry these
            raise HTTPException(
                status_code=500, detail=f"Embedding generation failed: {str(e)}"
            )

    # If we get here, all retries failed
    if isinstance(last_exception, httpx.TimeoutException):
        raise HTTPException(
            status_code=500, detail="OpenAI API timeout - please try again"
        )
    elif isinstance(last_exception, httpx.RequestError):
        raise HTTPException(
            status_code=500,
            detail=f"Network error connecting to OpenAI API: {str(last_exception)}",
        )
    else:
        raise HTTPException(
            status_code=500, detail="Max retries exceeded for embedding generation"
        )


async def embed_chunks(chunks: list[str]) -> list[dict]:
    """
    Embeds a list of text chunks using OpenAI's embedding API with concurrency control.

    Args:
        chunks (list[str]): A list of text chunks to be embedded.

    Returns:
        list[dict]: A list of dictionaries containing the chunk and its embedding.
    """
    # Process chunks concurrently (but with rate limiting)
    semaphore = asyncio.Semaphore(5)  # Max 5 concurrent requests

    async def embed_single(chunk):
        async with semaphore:
            embedding = await get_embedding(chunk)
            return {"chunk": chunk, "embedding": embedding}

    tasks = [embed_single(chunk) for chunk in chunks]
    return await asyncio.gather(*tasks)
