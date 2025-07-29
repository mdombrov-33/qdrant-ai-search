def test_search_empty_query(client):
    response = client.post("/api/search", json={"query": ""})
    assert response.status_code == 400
