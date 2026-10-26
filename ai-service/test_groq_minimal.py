#!/usr/bin/env python3
"""Minimal Groq API test to diagnose 404 error."""

import json
import os
import requests
from dotenv import load_dotenv

load_dotenv()

API_KEY = os.getenv("GROQ_API_KEY")
if not API_KEY:
    print("ERROR: GROQ_API_KEY not found in environment")
    exit(1)

print(f"API Key (first 10 chars): {API_KEY[:10]}...")
print(f"API Key length: {len(API_KEY)}")

# Test 1: Minimal request with only required fields
url = "https://api.groq.com/openai/v1/chat/completions"
headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json",
}

payload = {
    "model": "openai/gpt-oss-20b",
    "messages": [
        {"role": "system", "content": "You are a Linux command generator."},
        {"role": "user", "content": "List all files in current directory"}
    ],
    "temperature": 0.1,
    "max_tokens": 500
}

print(f"\n=== Test 1: Minimal Request ===")
print(f"URL: {url}")
print(f"Headers: {headers}")
print(f"Payload: {json.dumps(payload, indent=2)}")

try:
    response = requests.post(url, headers=headers, json=payload, timeout=30)
    print(f"\nStatus Code: {response.status_code}")
    print(f"Response Headers: {dict(response.headers)}")
    print(f"Response Body: {response.text[:500]}")
    
    if response.status_code == 200:
        result = response.json()
        print(f"\n✅ SUCCESS!")
        print(f"Content: {result['choices'][0]['message']['content']}")
    else:
        print(f"\n❌ FAILED with status {response.status_code}")
        
except Exception as e:
    print(f"\n❌ Exception: {type(e).__name__}: {e}")

# Test 2: Check if API key format is correct
print(f"\n=== Test 2: API Key Validation ===")
if API_KEY.startswith("gsk_"):
    print("✅ API key has correct prefix (gsk_)")
else:
    print(f"⚠️  API key doesn't start with 'gsk_', starts with: {API_KEY[:4]}")

# Test 3: Try alternate endpoint without /openai prefix
print(f"\n=== Test 3: Try Alternate Endpoint ===")
alt_url = "https://api.groq.com/v1/chat/completions"
print(f"Trying: {alt_url}")

try:
    response = requests.post(alt_url, headers=headers, json=payload, timeout=30)
    print(f"Status Code: {response.status_code}")
    if response.status_code == 200:
        print("✅ This endpoint works!")
    else:
        print(f"❌ Also failed: {response.status_code}")
except Exception as e:
    print(f"❌ Exception: {type(e).__name__}: {e}")
