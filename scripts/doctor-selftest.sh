#!/usr/bin/env bash
# Self-test for doctor.sh - force a failure and verify detection
set -euo pipefail

echo "=== Doctor Self-Test ==="
echo

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCTOR_SCRIPT="$SCRIPT_DIR/doctor.sh"

if [ ! -f "$DOCTOR_SCRIPT" ]; then
    echo "✗ FAIL: doctor.sh not found at $DOCTOR_SCRIPT"
    exit 1
fi

# Test 1: Force disk space check to fail
echo "Test 1: Forcing MIN_DISK_GB to unreachable value..."
export MIN_DISK_GB=999999

# Run doctor and capture output and exit code
OUTPUT=$("$DOCTOR_SCRIPT" 2>&1) || DOCTOR_EXIT=$?

# Verify it failed
if [ "${DOCTOR_EXIT:-0}" -eq 0 ]; then
    echo "✗ FAIL: Doctor script should have failed but returned 0"
    echo "Output:"
    echo "$OUTPUT"
    exit 1
fi

# Verify output contains FAIL and mentions disk space
if echo "$OUTPUT" | grep -q "FAIL" && echo "$OUTPUT" | grep -qi "disk"; then
    echo "✓ PASS: Doctor correctly detected forced disk failure"
else
    echo "✗ FAIL: Doctor output did not contain expected FAIL message for disk check"
    echo "Output:"
    echo "$OUTPUT"
    exit 1
fi

# Test 2: Verify JSON output works
echo
echo "Test 2: Verifying JSON output format..."
JSON_OUTPUT=$("$DOCTOR_SCRIPT" --json 2>&1) || true

# Use python3 to validate JSON (jq may not be installed yet)
if echo "$JSON_OUTPUT" | python3 -m json.tool >/dev/null 2>&1; then
    echo "✓ PASS: JSON output is valid"

    # Verify it contains expected structure
    if echo "$JSON_OUTPUT" | python3 -c "import sys, json; data = json.load(sys.stdin); sys.exit(0 if data and 'check' in data[0] and 'status' in data[0] and 'message' in data[0] else 1)" 2>&1; then
        echo "✓ PASS: JSON structure is correct"
    else
        echo "✗ FAIL: JSON structure missing expected fields"
        exit 1
    fi
else
    echo "✗ FAIL: JSON output is not valid JSON"
    echo "Output:"
    echo "$JSON_OUTPUT"
    exit 1
fi

echo
echo "=== Self-Test Complete ==="
echo "✓ All tests passed"
exit 0
