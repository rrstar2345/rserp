#!/bin/bash

# Example search script for RSERP

BASE_URL="http://127.0.0.1:7000"

echo "=== RSERP Examples ==="
echo

# Example 1: DuckDuckGo search
echo "1. DuckDuckGo search:"
curl -s "${BASE_URL}/duckduckgo/search?text=rust%20programming&limit=3" | jq '.results[] | {title: .title, url: .url, snippet: .snippet}'
echo

# Example 2: Bing search
echo "2. Bing search:"
curl -s "${BASE_URL}/bing/search?text=rust%20programming&limit=3" | jq '.results[] | {title: .title, url: .url, snippet: .snippet}'
echo

# Example 3: Multi-engine search
echo "3. Multi-engine search (DuckDuckGo + Bing):"
curl -s "${BASE_URL}/search?text=rust%20programming&engines=duckduckgo,bing&limit=5" | jq '{meta: .meta, results_count: (.results | length), results: .results[] | {title, url, snippet}}'
echo

# Example 4: Health check
echo "4. Health check:"
curl -s "${BASE_URL}/health" | jq '.'