#!/usr/bin/env bash
# LLM smoke test: Verifies Ollama is running, sends a test prompt with JSON schema,
# validates the response, and reports tokens/sec and device (CPU/GPU).

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Determine script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Source configuration
if [[ -f "${PROJECT_ROOT}/config/default.env" ]]; then
    # shellcheck source=/dev/null
    source "${PROJECT_ROOT}/config/default.env"
fi

# Override with local config if present
if [[ -f "${PROJECT_ROOT}/config/local.env" ]]; then
    # shellcheck source=/dev/null
    source "${PROJECT_ROOT}/config/local.env"
fi

# Default to phi3:mini if not set
OLLAMA_MODEL="${OLLAMA_MODEL:-phi3:mini}"
OLLAMA_PORT="${OLLAMA_PORT:-11434}"
OLLAMA_URL="http://127.0.0.1:${OLLAMA_PORT}"

echo "IntelliShell LLM Smoke Test"
echo "============================"
echo ""

# Check if Ollama is running
echo -n "Checking Ollama service... "
if ! curl -s -f "${OLLAMA_URL}/api/tags" >/dev/null 2>&1; then
    echo -e "${RED}FAIL${NC}"
    echo ""
    echo "Error: Ollama is not responding at ${OLLAMA_URL}"
    echo ""
    echo "Please ensure Ollama is running:"
    echo "  systemctl --user status ollama"
    echo "  systemctl --user start ollama"
    echo ""
    exit 1
fi
echo -e "${GREEN}OK${NC}"

# Check if model is available
echo -n "Checking model ${OLLAMA_MODEL}... "
if ! curl -s "${OLLAMA_URL}/api/tags" | jq -e ".models[] | select(.name == \"${OLLAMA_MODEL}\")" >/dev/null 2>&1; then
    echo -e "${YELLOW}WARN${NC}"
    echo ""
    echo "Warning: Model ${OLLAMA_MODEL} not found locally."
    echo "Pulling model... (this may take several minutes)"
    if ! curl -s -X POST "${OLLAMA_URL}/api/pull" -d "{\"name\": \"${OLLAMA_MODEL}\"}" | jq -r '.status' | grep -q "success"; then
        echo -e "${RED}FAIL${NC}"
        echo "Error: Failed to pull model ${OLLAMA_MODEL}"
        exit 1
    fi
fi
echo -e "${GREEN}OK${NC}"

# Define test prompt
PROMPT="Analyze this command for safety: ls -la. Respond with a JSON object containing a 'safe' boolean and a 'reason' string."

# Define JSON schema for structured output
SCHEMA='{
  "type": "object",
  "properties": {
    "safe": {
      "type": "boolean",
      "description": "Whether the command is safe to execute"
    },
    "reason": {
      "type": "string",
      "description": "Explanation for the safety assessment"
    }
  },
  "required": ["safe", "reason"]
}'

# Create request payload
REQUEST_PAYLOAD=$(jq -n \
    --arg model "${OLLAMA_MODEL}" \
    --arg prompt "${PROMPT}" \
    --argjson schema "${SCHEMA}" \
    '{
        model: $model,
        prompt: $prompt,
        format: $schema,
        stream: false,
        options: {
            temperature: 0.1
        }
    }')

echo ""
echo "Sending test prompt to ${OLLAMA_MODEL}..."
echo "Prompt: ${PROMPT}"
echo ""

# Send request to Ollama
RESPONSE_FILE=$(mktemp)
HTTP_CODE=$(curl -s -w "%{http_code}" -o "${RESPONSE_FILE}" \
    -X POST "${OLLAMA_URL}/api/generate" \
    -H "Content-Type: application/json" \
    -d "${REQUEST_PAYLOAD}")

if [[ "${HTTP_CODE}" != "200" ]]; then
    echo -e "${RED}FAIL${NC}"
    echo "Error: HTTP ${HTTP_CODE} from Ollama API"
    cat "${RESPONSE_FILE}"
    rm -f "${RESPONSE_FILE}"
    exit 1
fi

# Parse response
if ! jq empty "${RESPONSE_FILE}" 2>/dev/null; then
    echo -e "${RED}FAIL${NC}"
    echo "Error: Invalid JSON response from Ollama"
    cat "${RESPONSE_FILE}"
    rm -f "${RESPONSE_FILE}"
    exit 1
fi

# Extract response text
RESPONSE_TEXT=$(jq -r '.response // empty' "${RESPONSE_FILE}")
if [[ -z "${RESPONSE_TEXT}" ]]; then
    echo -e "${RED}FAIL${NC}"
    echo "Error: Empty response from model"
    jq '.' "${RESPONSE_FILE}"
    rm -f "${RESPONSE_FILE}"
    exit 1
fi

echo "Model response:"
echo "${RESPONSE_TEXT}"
echo ""

# Validate response against schema
echo -n "Validating JSON schema... "
if ! echo "${RESPONSE_TEXT}" | jq -e '.safe != null and .reason != null and (.safe | type) == "boolean" and (.reason | type) == "string"' >/dev/null 2>&1; then
    echo -e "${RED}FAIL${NC}"
    echo "Error: Response does not match expected schema"
    echo "Expected: {safe: boolean, reason: string}"
    echo "Got: ${RESPONSE_TEXT}"
    rm -f "${RESPONSE_FILE}"
    exit 1
fi
echo -e "${GREEN}PASS${NC}"

# Extract performance metrics
EVAL_COUNT=$(jq -r '.eval_count // 0' "${RESPONSE_FILE}")
EVAL_DURATION=$(jq -r '.eval_duration // 0' "${RESPONSE_FILE}")

# Calculate tokens per second
if [[ "${EVAL_DURATION}" -gt 0 ]]; then
    TOKENS_PER_SEC=$(echo "scale=2; ${EVAL_COUNT} / (${EVAL_DURATION} / 1000000000)" | bc)
else
    TOKENS_PER_SEC="N/A"
fi

# Determine device (CPU or GPU)
DEVICE="CPU"
# Check load_duration metadata or other indicators
# Ollama doesn't always expose this directly, so we check for CUDA-related strings in the response
if jq -e '.load_duration' "${RESPONSE_FILE}" >/dev/null 2>&1; then
    # If we can run nvidia-smi and see processes, it's likely GPU
    if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi >/dev/null 2>&1; then
        # Check if Ollama is using GPU by looking for ollama in nvidia-smi output
        if nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader 2>/dev/null | grep -iq ollama; then
            DEVICE="GPU"
        fi
    fi
fi

# Alternative: check for GPU indicators in the system
if [[ "${DEVICE}" == "CPU" ]] && command -v nvidia-smi >/dev/null 2>&1; then
    if nvidia-smi -L 2>/dev/null | grep -q "GPU"; then
        # GPU is available, so if eval is fast enough, assume GPU
        if [[ "${TOKENS_PER_SEC}" != "N/A" ]] && (( $(echo "${TOKENS_PER_SEC} > 30" | bc -l) )); then
            DEVICE="GPU (inferred)"
        fi
    fi
fi

# Clean up
rm -f "${RESPONSE_FILE}"

# Print summary
echo ""
echo "Summary"
echo "-------"
echo "Model:        ${OLLAMA_MODEL}"
echo "Tokens:       ${EVAL_COUNT}"
echo "Duration:     $(echo "scale=3; ${EVAL_DURATION} / 1000000000" | bc)s"
echo "Tokens/sec:   ${TOKENS_PER_SEC}"
echo "Device:       ${DEVICE}"
echo ""
echo -e "${GREEN}✓ Smoke test passed${NC}"

exit 0
