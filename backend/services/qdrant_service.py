import os
import uuid
from typing import List
from utils.logging_config import logger
from qdrant_client import QdrantClient
from qdrant_client.http.models import Distance, VectorParams, PointStruct
from qdrant_client.http.exceptions import UnexpectedResponse
from exceptions import QdrantServiceError, VectorSearchError

QDRANT_URL = os.getenv("QDRANT_URL", "http://localhost:6333")

client = QdrantClient(url=QDRANT_URL)


def create_collection(
    client: QdrantClient, collection_name: str, vector_size: int
) -> None:
    try:
        if client.collection_exists(collection_name=collection_name):
            logger.info(
                f"Collection '{collection_name}' already exists. Skipping creation."
            )
            return
    except UnexpectedResponse as e:
        logger.warning(f"Error checking collection: {e}. Proceeding with creation.")

    try:
        client.create_collection(
            collection_name=collection_name,
            vectors_config=VectorParams(
                size=vector_size,
                distance=Distance.COSINE,
            ),
        )
        logger.info(
            f"Collection '{collection_name}' created with vector size {vector_size}."
        )
    except UnexpectedResponse as e:
        # Handle the 'already exists' error gracefully
        if "already exists" in str(e):
            logger.info(
                f"Collection '{collection_name}' already exists (caught on create)."
            )
        else:
            raise


def insert_vectors_batch(
    client: QdrantClient,
    collection_name: str,
    items: List[dict],
    batch_size: int = 500,
    generate_ids: bool = False,
) -> None:
    """
    Insert vectors and their metadata into a Qdrant collection in batches.

    Args:
        client (QdrantClient): The Qdrant client.
        collection_name (str): Target collection name.
        items (List[dict]): List of dictionaries each with:
            - 'id' (optional): unique string id for the vector point
            - 'embedding': List[float], the vector itself
            - 'payload': dict with metadata
        batch_size (int): How many points to send per batch.
        generate_ids (bool): If True, generate UUIDs as IDs if 'id' is missing.

    Returns:
        None
    """
    batch = []
    for i, item in enumerate(items, 1):
        point_id = item.get("id")
        if generate_ids and not point_id:
            # Generate a UUIDv4 string if no id provided and generate_ids=True
            point_id = str(uuid.uuid4())

        if not point_id:
            raise QdrantServiceError(
                "Each item must have an 'id' or enable 'generate_ids=True'."
            )

        point = PointStruct(
            id=point_id,
            vector=item["embedding"],
            payload=item.get("payload", {}),
        )
        batch.append(point)

        if len(batch) == batch_size or i == len(items):
            client.upsert(collection_name=collection_name, points=batch)
            logger.info(
                f"Inserted batch of {len(batch)} vectors into '{collection_name}'."
            )
            batch.clear()


def search_vectors(
    client: QdrantClient,
    collection_name: str,
    query_vector: List[float],
    limit: int = 10,
    score_threshold: float = 0.7,
) -> List[dict]:
    """
    Search for vectors in a Qdrant collection.

    Args:
        client (QdrantClient): The Qdrant client.
        collection_name (str): Target collection name.
        query_vector (List[float]): The vector to search for.
        limit (int): Maximum number of results to return.
        score_threshold (float): Minimum score threshold for results.

    Returns:
        List[dict]: List of search results with 'id', 'score', and 'payload'.
    """
    try:
        search_results = client.search(
            collection_name=collection_name,
            query_vector=query_vector,
            limit=limit,
            score_threshold=score_threshold,
            with_payload=True,
            with_vectors=False,  # Don't return vectors to save bandwidth
        )

        # Convert to dict for easier handling
        results = []
        for result in search_results:
            results.append(
                {
                    "id": str(result.id),
                    "score": result.score,
                    "payload": result.payload,
                }
            )

        logger.info(f"Found {len(results)} results for search query")
        return results

    except Exception as e:
        logger.error(f"Error during search: {e}")
        raise VectorSearchError(f"Vector search failed: {str(e)}")
