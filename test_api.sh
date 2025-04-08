#!/bin/bash

# Create a shortened URL
echo "Creating a shortened URL..."
curl -X POST -H "Content-Type: application/json" -d '{"url": "https://example.com"}' http://localhost:8081/api/shorten -s | jq .

# Wait a moment
sleep 1

# Create another URL with a custom ID
echo "Creating a shortened URL with custom ID..."
curl -X POST -H "Content-Type: application/json" -d '{"url": "https://rust-lang.org", "custom_id": "rust"}' http://localhost:8081/api/shorten -s | jq .

# Wait a moment
sleep 1

# Get statistics for the custom URL
echo "Getting statistics for custom URL..."
curl http://localhost:8081/api/stats/rust -s | jq .

# List all URLs
echo "Listing all URLs..."
curl http://localhost:8081/api/urls -s | jq .

echo "Now open a browser and navigate to http://localhost:8081/rust to test redirection" 