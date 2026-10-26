#!/bin/bash
echo '{"query":"list files in current directory"}' > /tmp/test_query.json

echo "Testing generate-command endpoint..."
curl -X POST http://localhost:8000/generate-command \
  -H "Content-Type: application/json" \
  -d @/tmp/test_query.json \
  --max-time 15

echo ""
