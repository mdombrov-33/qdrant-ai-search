from pydantic import BaseModel
from typing import List
from .search import SearchResult


class SummarizeRequest(BaseModel):
    query: str
    search_results: List[SearchResult]
    style: str = "comprehensive"


class SummarizeResponse(BaseModel):
    summary: str
    query_time_ms: int
    chunks_processed: int
