import asyncio
import httpx
import random
from typing import Dict, List
from utils.logging_config import logger
from config import settings
from exceptions import OpenAIServiceError

OPENAI_CHAT_URL = "https://api.openai.com/v1/chat/completions"


def _extract_error_message(response: httpx.Response) -> str:
    """Extract error message from OpenAI API response."""
    try:
        error_response = response.json()
        if "error" in error_response:
            return error_response["error"].get("message", "Unknown error")
    except Exception:
        pass
    return response.text


async def _make_chat_request(
    messages: List[Dict[str, str]],
    model: str | None = None,
    temperature: float | None = None,
    max_tokens: int | None = None,
) -> str:
    """Make a single chat completion request to OpenAI API."""
    headers = {
        "Authorization": f"Bearer {settings.OPENAI_API_KEY}",
        "Content-Type": "application/json",
    }

    json_data = {
        "model": model or settings.OPENAI_CHAT_MODEL,
        "messages": messages,
        "temperature": temperature or settings.SUMMARY_TEMPERATURE,
        "max_tokens": max_tokens or settings.MAX_SUMMARY_TOKENS,
    }

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(OPENAI_CHAT_URL, headers=headers, json=json_data)
        response.raise_for_status()
        data = response.json()
        return data["choices"][0]["message"]["content"]


async def get_chat_completion(
    messages: List[Dict[str, str]],
    model: str | None = None,
    temperature: float | None = None,
    max_tokens: int | None = None,
    max_retries: int = 3,
) -> str:
    """
    Get chat completion using OpenAI's API.

    Args:
        messages: List of message objects [{"role": "user", "content": "..."}]
        model: OpenAI model to use (defaults to config setting)
        temperature: Creativity level (defaults to config setting)
        max_tokens: Maximum tokens in response (defaults to config setting)
        max_retries: Maximum retry attempts for rate limiting

    Returns:
        str: The completion text

    Raises:
        HTTPException: On API errors, timeouts, or other failures
    """
    last_exception = None

    for attempt in range(max_retries):
        try:
            return await _make_chat_request(messages, model, temperature, max_tokens)

        except httpx.HTTPStatusError as e:
            last_exception = e
            # Handle rate limiting with exponential backoff
            if e.response.status_code == 429 and attempt < max_retries - 1:
                wait_time = (2**attempt) + random.uniform(0, 1)
                logger.warning(f"Rate limited, retrying in {wait_time:.1f}s")
                await asyncio.sleep(wait_time)
                continue

            # Handle other HTTP errors - don't retry
            error_message = _extract_error_message(e.response)
            raise OpenAIServiceError(
                f"OpenAI API error: {e.response.status_code} - {error_message}"
            )

        except (httpx.TimeoutException, httpx.RequestError) as e:
            last_exception = e
            if attempt < max_retries - 1:
                wait_time = 2**attempt
                logger.warning(f"Request failed, retrying in {wait_time}s")
                await asyncio.sleep(wait_time)
                continue

        except Exception as e:
            # Unexpected errors - don't retry these
            raise OpenAIServiceError(f"Chat completion failed: {str(e)}")

    # If we get here, all retries failed
    if isinstance(last_exception, httpx.TimeoutException):
        raise OpenAIServiceError("OpenAI API timeout - please try again")
    elif isinstance(last_exception, httpx.RequestError):
        raise OpenAIServiceError(
            f"Network error connecting to OpenAI API: {str(last_exception)}"
        )
    else:
        raise OpenAIServiceError("Max retries exceeded for chat completion")


def create_summary_messages(
    query: str, chunks: List[str], style: str = "comprehensive"
) -> List[Dict[str, str]]:
    """
    Helper function to create properly formatted messages for summarization.

    Args:
        query: Original search query
        chunks: List of text chunks to summarize
        style: Summary style (comprehensive/brief/bullet_points)

    Returns:
        List of message objects for OpenAI chat API
    """
    # Combine all chunks into context
    context = "\n\n".join([f"Chunk {i + 1}: {chunk}" for i, chunk in enumerate(chunks)])

    # Style-specific prompts
    style_prompts = {
        "comprehensive": (
            "Provide a detailed, comprehensive summary that thoroughly "
            "answers the query."
        ),
        "brief": "Provide a concise, brief summary focusing on the key points.",
        "bullet_points": (
            "Provide a summary in bullet points highlighting the main findings."
        ),
    }

    style_instruction = style_prompts.get(style, style_prompts["comprehensive"])

    user_message = (
        f'Based on the following document chunks related to the query "{query}", '
        f"{style_instruction}\n\n"
        f"Context from documents:\n{context}\n\n"
        f"Please synthesize this information to provide a coherent answer "
        f'to the original query: "{query}"'
    )

    return [
        {
            "role": "system",
            "content": (
                "You are a helpful AI assistant that summarizes document "
                "search results. Provide accurate, well-structured summaries "
                "based only on the provided context."
            ),
        },
        {"role": "user", "content": user_message},
    ]
