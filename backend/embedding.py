import os
import httpx

OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
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


def get_embedding(text: str) -> list[float]:
    headers = {
        "Authorization": f"Bearer {OPENAI_API_KEY}",
        "Content-Type": "application/json",
    }

    json_data = {
        "model": "text-embedding-ada-002",
        "input": text,
    }

    with httpx.Client(timeout=10.0) as client:
        response = client.post(OPENAI_EMBEDDING_URL, headers=headers, json=json_data)
        response.raise_for_status()  # Raise an error for bad responses
        data = response.json()
        return data["data"][0]["embedding"]


def embed_chunks(chunks: list[str]) -> list[dict]:
    """
    Embeds a list of text chunks using OpenAI's embedding API.

    Args:
        chunks (list[str]): A list of text chunks to be embedded.

    Returns:
        list[dict]: A list of dictionaries containing the chunk and its embedding.
    """
    embeddings = [
        {"chunk": chunk, "embedding": get_embedding(chunk)} for chunk in chunks
    ]
    return embeddings
