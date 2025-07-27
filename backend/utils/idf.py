import math
from collections import defaultdict


def compute_idf(documents):
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
