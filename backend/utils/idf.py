import math
from collections import defaultdict


def compute_idf(documents):
    """
    IDF (Inverse Document Frequency) is a classic information retrieval metric.
    It measures how important a word is across a set of documents:
    rare words get higher scores, while common words get lower scores.
    This helps boost the impact of unique, meaningful terms
    in search and ranking algorithms, and is a key part of TF-IDF and BM25 scoring.
    """
    N = len(documents)
    df = defaultdict(int)
    for doc in documents:
        words = set(doc.lower().split())
        for w in words:
            df[w] += 1
    idf = {}
    for term, freq in df.items():
        idf[term] = math.log((N + 1) / (freq + 1)) + 1
    return idf
