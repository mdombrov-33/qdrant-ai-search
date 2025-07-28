from pydantic import BaseModel
from typing import List


class SearchRequest(BaseModel):
    query: str
    limit: int = 10
    threshold: float = 0.7
    idf_map: dict = {}


class SearchResult(BaseModel):
    id: str
    text: str
    score: float
    metadata: dict


class SearchResponse(BaseModel):
    results: List[SearchResult]
    query_time_ms: int
    total_found: int
