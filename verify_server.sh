#!/bin/bash
set -e

# Build server
echo "Building server..."
cargo build -p polarisdb-server

# Cleanup previous data
rm -rf ./data/test_col

# Start server
echo "Starting server..."
./target/debug/polarisdb-server > server.log 2>&1 &
SERVER_PID=$!

# Ensure cleanup on exit
cleanup() {
    echo "Stopping server..."
    kill $SERVER_PID || true
    cat server.log || true
}
trap cleanup EXIT

# Wait for server to start
sleep 2

# Create Collection
echo "Creating collection..."
curl -v -X POST http://localhost:8080/collections/test_col \
  -H "Content-Type: application/json" \
  -d '{"dimension": 3, "metric": "cosine"}'

# Insert Vector
echo -e "\nInserting vector..."
curl -v -X POST http://localhost:8080/collections/test_col/insert \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "vector": [1.0, 0.0, 0.0], "payload": {"foo": "bar"}}'

# Search Vector
echo -e "\nSearching vector..."
RESPONSE=$(curl -s -X POST http://localhost:8080/collections/test_col/search \
  -H "Content-Type: application/json" \
  -d '{"vector": [1.0, 0.0, 0.0], "k": 1}')

echo -e "\nSearch response: $RESPONSE"

if [[ $RESPONSE == *'"id":1'* ]]; then
    echo "✅ Verification Successful: Found vector 1"
else
    echo "❌ Verification Failed: Vector 1 not found"
    exit 1
fi
