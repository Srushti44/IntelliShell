#!/bin/bash
echo "Testing Ollama with curl..."
time curl -s http://localhost:11434/api/generate -d '{"model": "phi3:mini", "prompt": "Say hello", "stream": false}' | head -50
