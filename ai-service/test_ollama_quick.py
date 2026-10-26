#!/usr/bin/env python3
"""Quick Ollama test to verify it's working."""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "app"))

from providers import OllamaProvider
import time

print("Testing Ollama with simple prompt...")

ollama = OllamaProvider(model_name="phi3:mini")

if not ollama.health_check():
    print("❌ Ollama health check failed")
    exit(1)

print("✅ Health check passed")
print("Sending simple request...")

start = time.time()
try:
    response = ollama.generate(
        prompt='Generate JSON: {"command": "ls", "explanation": "list files"}',
        timeout_seconds=60  # Increased timeout
    )
    elapsed = time.time() - start
    
    print(f"✅ Got response in {elapsed:.2f}s")
    print(f"Latency: {response.latency_ms}ms")
    print(f"Parsed: {response.parsed_json}")
except Exception as e:
    print(f"❌ Failed: {type(e).__name__}: {e}")
