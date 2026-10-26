#!/usr/bin/env python3
"""List available Groq models for the current API key."""

import json
import os
import requests
from dotenv import load_dotenv

load_dotenv()

API_KEY = os.getenv("GROQ_API_KEY")
if not API_KEY:
    print("ERROR: GROQ_API_KEY not found in environment")
    exit(1)

url = "https://api.groq.com/openai/v1/models"
headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

print("Fetching available models from Groq API...\n")

try:
    response = requests.get(url, headers=headers, timeout=30)
    response.raise_for_status()
    
    result = response.json()
    models = result.get("data", [])
    
    print(f"✅ Found {len(models)} available models:\n")
    print(f"{'Model ID':<50} {'Owned By':<20}")
    print("=" * 70)
    
    for model in models:
        model_id = model.get("id", "unknown")
        owned_by = model.get("owned_by", "unknown")
        print(f"{model_id:<50} {owned_by:<20}")
    
    # Check for specific models
    print("\n" + "=" * 70)
    print("\nModel availability check:")
    test_models = [
        "llama-3.3-70b-versatile",
        "llama-3.1-8b-instant",
        "openai/gpt-oss-120b",
        "openai/gpt-oss-20b",
        "qwen/qwen3.8-27b",
    ]
    
    available_ids = [m["id"] for m in models]
    for test_model in test_models:
        status = "✅ Available" if test_model in available_ids else "❌ Not available"
        print(f"{test_model:<40} {status}")
    
except Exception as e:
    print(f"❌ Error: {type(e).__name__}: {e}")
