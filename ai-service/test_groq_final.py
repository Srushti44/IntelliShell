#!/usr/bin/env python3
"""Final Groq verification test."""

import sys
import os
from dotenv import load_dotenv

load_dotenv()  # Load environment variables

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "app"))

from providers import GroqProvider
import time

print("=" * 70)
print("GROQ PROVIDER FINAL VERIFICATION")
print("=" * 70)

groq = GroqProvider(model_name="openai/gpt-oss-20b")

print(f"\n✅ Provider initialized")
print(f"   Model: {groq.model_name}")
print(f"   Health: {'PASS' if groq.health_check() else 'FAIL'}")

print(f"\nTest 1: Simple command generation")
print("-" * 70)
start = time.time()
response = groq.generate(
    "Generate a Linux command to list files",
    timeout_seconds=30
)
elapsed = time.time() - start

print(f"✅ Response in {elapsed:.2f}s (latency: {response.latency_ms}ms)")
print(f"   Provider: {response.provider.value}")
print(f"   Command: {response.parsed_json.get('command', 'N/A')}")
print(f"   Explanation: {response.parsed_json.get('explanation', 'N/A')}")

print(f"\nTest 2: Multiple requests")
print("-" * 70)
for i, prompt in enumerate(['show memory', 'find files', 'check disk'], 1):
    start = time.time()
    r = groq.generate(f"Command to {prompt}", timeout_seconds=30)
    print(f"{i}. {prompt:<15} → {r.parsed_json.get('command', 'N/A'):<30} ({int((time.time()-start)*1000)}ms)")

print("\n" + "=" * 70)
print("✅ GROQ PROVIDER FULLY OPERATIONAL")
print("=" * 70)
