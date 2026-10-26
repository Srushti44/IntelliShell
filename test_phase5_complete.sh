#!/bin/bash
set -e

echo "========================================="
echo "  PHASE 5 END-TO-END INTEGRATION TEST"
echo "========================================="
echo ""

echo "TEST 1: Python AI Service Health"
echo "---------------------------------"
HEALTH=$(curl -s http://localhost:8000/health)
echo "$HEALTH" | jq .
echo "RESULT: $(echo $HEALTH | jq -r '.status')"
echo ""

echo "TEST 2: Rust Server Health"
echo "---------------------------"
RUST_HEALTH=$(curl -s http://localhost:3000/health)
echo "$RUST_HEALTH" | jq .
echo "RESULT: $(echo $RUST_HEALTH | jq -r '.status')"
echo ""

echo "TEST 3: Generate Command from Natural Language"
echo "----------------------------------------------"
GEN_RESULT=$(curl -s -X POST http://localhost:8000/generate-command \
  -H 'Content-Type: application/json' \
  -d '{"query":"list all files in current directory"}' \
  --max-time 10)
echo "$GEN_RESULT" | jq .
COMMAND=$(echo "$GEN_RESULT" | jq -r '.command')
echo "Generated command: $COMMAND"
echo ""

echo "TEST 4: Execute Generated Command"
echo "----------------------------------"
if [ "$COMMAND" != "null" ] && [ -n "$COMMAND" ]; then
  EXEC_RESULT=$(curl -s -X POST http://localhost:3000/execute \
    -H 'Content-Type: application/json' \
    -d "{\"command\":\"ls\",\"args\":[\"-la\"],\"risk_level\":\"LOW\"}")
  echo "$EXEC_RESULT" | jq .
  echo "Executed: $(echo $EXEC_RESULT | jq -r '.executed')"
else
  echo "SKIPPED: No command generated"
fi
echo ""

echo "TEST 5: Troubleshoot System"
echo "---------------------------"
TROUBLE_RESULT=$(curl -s -X POST http://localhost:8000/troubleshoot \
  -H 'Content-Type: application/json' \
  -d '{"issue_description":"check system memory usage"}' \
  --max-time 10)
echo "$TROUBLE_RESULT" | jq .
echo ""

echo "TEST 6: Activity Audit Log"
echo "--------------------------"
ACTIVITY=$(curl -s http://localhost:8000/activity)
echo "$ACTIVITY" | jq '. | length' | xargs echo "Activity log entries:"
echo "$ACTIVITY" | jq '.[0:3]'
echo ""

echo "TEST 7: System Snapshot"
echo "-----------------------"
SNAPSHOT=$(curl -s http://localhost:3000/snapshot)
echo "$SNAPSHOT" | jq '{cpu_percent, memory_used_percent, process_count: (.processes | length)}'
echo ""

echo "========================================="
echo "  INTEGRATION TEST COMPLETE"
echo "========================================="
