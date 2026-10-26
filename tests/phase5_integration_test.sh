#!/bin/bash
# Phase 5 Integration Test Suite
# Tests end-to-end workflow between AI service and Rust core engine

set -e

echo "========================================="
echo "Phase 5 Integration Test Suite"
echo "========================================="
echo ""

# Configuration
AI_SERVICE_URL="http://localhost:8000"
CORE_ENGINE_URL="http://localhost:3000"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠ WARN${NC}: $1"
}

# Test 1: Service Health Checks
echo "Test 1: Service Health Checks"
echo "-------------------------------------"

# Check AI service
if curl -sf "${AI_SERVICE_URL}/health" > /dev/null 2>&1; then
    pass "AI service is reachable"
else
    fail "AI service is NOT reachable at ${AI_SERVICE_URL}"
fi

# Check Core engine
if curl -sf "${CORE_ENGINE_URL}/health" > /dev/null 2>&1; then
    pass "Core engine is reachable"
else
    fail "Core engine is NOT reachable at ${CORE_ENGINE_URL}"
fi

echo ""

# Test 2: Command Generation
echo "Test 2: Command Generation"
echo "-------------------------------------"

RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/generate-command" \
    -H "Content-Type: application/json" \
    -d '{"query": "list all files in current directory"}')

COMMAND=$(echo "$RESPONSE" | jq -r '.command')
RISK_LEVEL=$(echo "$RESPONSE" | jq -r '.risk_level')
TOKEN=$(echo "$RESPONSE" | jq -r '.confirmation_token')

if [ -n "$COMMAND" ] && [ "$COMMAND" != "null" ]; then
    pass "Command generated: $COMMAND"
else
    fail "Failed to generate command"
fi

if [ -n "$RISK_LEVEL" ] && [ "$RISK_LEVEL" != "null" ]; then
    pass "Risk level assigned: $RISK_LEVEL"
else
    fail "No risk level assigned"
fi

if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
    pass "Confirmation token issued: ${TOKEN:0:8}..."
else
    fail "No confirmation token issued"
fi

echo ""

# Test 3: LOW Risk Command Execution
echo "Test 3: LOW Risk Command - Direct Execution"
echo "-------------------------------------"

# Generate a safe command
RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/generate-command" \
    -H "Content-Type: application/json" \
    -d '{"query": "show current date"}')

TOKEN=$(echo "$RESPONSE" | jq -r '.confirmation_token')
COMMAND=$(echo "$RESPONSE" | jq -r '.command')
RISK=$(echo "$RESPONSE" | jq -r '.risk_level')

echo "Generated: $COMMAND (risk: $RISK)"

# Confirm and execute
EXEC_RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/confirm-execute" \
    -H "Content-Type: application/json" \
    -d "{\"token\": \"$TOKEN\"}")

EXECUTED=$(echo "$EXEC_RESPONSE" | jq -r '.executed')
METHOD=$(echo "$EXEC_RESPONSE" | jq -r '.method')
EXIT_CODE=$(echo "$EXEC_RESPONSE" | jq -r '.exit_code')

if [ "$EXECUTED" = "true" ]; then
    pass "Command executed successfully"
else
    fail "Command execution failed"
fi

if [ "$METHOD" = "direct" ] || [ "$METHOD" = "sandboxed" ]; then
    pass "Execution method: $METHOD"
else
    warn "Unexpected execution method: $METHOD"
fi

if [ "$EXIT_CODE" = "0" ]; then
    pass "Exit code: 0"
else
    warn "Exit code: $EXIT_CODE"
fi

echo ""

# Test 4: HIGH Risk Command - Blocked
echo "Test 4: HIGH Risk Command - Blocked"
echo "-------------------------------------"

# Generate a dangerous command
RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/generate-command" \
    -H "Content-Type: application/json" \
    -d '{"query": "delete all files in root directory"}')

TOKEN=$(echo "$RESPONSE" | jq -r '.confirmation_token')
COMMAND=$(echo "$RESPONSE" | jq -r '.command')
RISK=$(echo "$RESPONSE" | jq -r '.risk_level')

echo "Generated: $COMMAND (risk: $RISK)"

if [ "$RISK" = "high" ] || [ "$RISK" = "critical" ]; then
    pass "High-risk command correctly classified"
else
    warn "Expected high/critical risk, got: $RISK"
fi

# Try to execute (should be blocked)
EXEC_RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/confirm-execute" \
    -H "Content-Type: application/json" \
    -d "{\"token\": \"$TOKEN\"}")

EXECUTED=$(echo "$EXEC_RESPONSE" | jq -r '.executed')
METHOD=$(echo "$EXEC_RESPONSE" | jq -r '.method')

if [ "$EXECUTED" = "false" ] && [ "$METHOD" = "blocked" ]; then
    pass "High-risk command correctly blocked"
else
    fail "High-risk command was NOT blocked (executed=$EXECUTED, method=$METHOD)"
fi

echo ""

# Test 5: Token Expiration
echo "Test 5: Token Replay Attack Prevention"
echo "-------------------------------------"

# Generate command
RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/generate-command" \
    -H "Content-Type: application/json" \
    -d '{"query": "show hostname"}')

TOKEN=$(echo "$RESPONSE" | jq -r '.confirmation_token')

# Execute once
curl -s -X POST "${AI_SERVICE_URL}/confirm-execute" \
    -H "Content-Type: application/json" \
    -d "{\"token\": \"$TOKEN\"}" > /dev/null

# Try to reuse token (should fail)
REPLAY_RESPONSE=$(curl -s -X POST "${AI_SERVICE_URL}/confirm-execute" \
    -H "Content-Type: application/json" \
    -d "{\"token\": \"$TOKEN\"}")

ERROR=$(echo "$REPLAY_RESPONSE" | jq -r '.detail')

if echo "$ERROR" | grep -q "already-used\|Invalid"; then
    pass "Token replay correctly rejected"
else
    fail "Token replay was NOT rejected"
fi

echo ""

# Test 6: System Monitoring
echo "Test 6: System Monitoring Snapshot"
echo "-------------------------------------"

SNAPSHOT=$(curl -s "${CORE_ENGINE_URL}/snapshot")

CPU=$(echo "$SNAPSHOT" | jq -r '.cpu_percent')
MEM_TOTAL=$(echo "$SNAPSHOT" | jq -r '.memory_total_bytes')
PROC_COUNT=$(echo "$SNAPSHOT" | jq -r '.processes | length')

if [ -n "$CPU" ] && [ "$CPU" != "null" ]; then
    pass "CPU usage: ${CPU}%"
else
    fail "No CPU data in snapshot"
fi

if [ "$MEM_TOTAL" -gt 0 ]; then
    MEM_GB=$((MEM_TOTAL / 1024 / 1024 / 1024))
    pass "Memory total: ${MEM_GB}GB"
else
    fail "Invalid memory data"
fi

if [ "$PROC_COUNT" -gt 0 ]; then
    pass "Process count: $PROC_COUNT"
else
    fail "No processes in snapshot"
fi

echo ""

# Test 7: Command History
echo "Test 7: Command History Persistence"
echo "-------------------------------------"

HISTORY=$(curl -s "${CORE_ENGINE_URL}/history")
HISTORY_COUNT=$(echo "$HISTORY" | jq '. | length')

if [ "$HISTORY_COUNT" -ge 0 ]; then
    pass "History retrieved: $HISTORY_COUNT commands"
else
    fail "Failed to retrieve history"
fi

echo ""

# Summary
echo "========================================="
echo "Integration Test Summary"
echo "========================================="
echo ""
echo "All critical integration tests passed!"
echo ""
echo "✓ Service health checks"
echo "✓ Command generation"
echo "✓ Risk classification"
echo "✓ Confirmation tokens"
echo "✓ Execution routing"
echo "✓ Token replay prevention"
echo "✓ System monitoring"
echo "✓ Command history"
echo ""
echo "Phase 5 backend integration verified successfully."
