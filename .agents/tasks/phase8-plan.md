# Phase 8 Implementation Plan: Final Audit, Verification & Release Readiness

**Date**: 2025-01-XX  
**Phase**: 8 - End-to-End Verification, Security Hardening & Release Readiness  
**Status**: PLANNING  
**Repository**: d:\3rd year\os project\IntelliShell

---

## Executive Summary

This plan covers a comprehensive audit of IntelliShell across all three services (Rust core-engine, Python ai-service, React dashboard), verification of the 7 required features, security hardening, comprehensive testing, and release documentation.

**Assessment Approach**: Feature-by-feature source code inspection → gap identification → security audit → test execution → documentation → final verification.

---

## 1. Feature Traceability Matrix (Current State)

Based on actual source code inspection:

### Feature 1: Natural-Language Shell Command Generation
**Status**: ✅ **IMPLEMENTED**

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **AI Service** | `ai-service/src/intellishell_ai_service/main.py:108-181` | `POST /generate-command` endpoint |
| **AI Provider** | `ai-service/src/intellishell_ai_service/providers.py` | OllamaProvider (primary), GroqProvider (fallback) |
| **LLM Prompt** | `ai-service/src/intellishell_ai_service/main.py:56-60` | System instructions for command generation |
| **Frontend** | `dashboard/src/pages/AssistantPage.jsx:19-26` | Natural language input textarea + generation button |
| **API Client** | `dashboard/src/lib/api.js:159-167` | `pythonApi.generateCommand(query)` |
| **Tests** | None found | **GAP** |

**Verification**:
- ✅ Endpoint exists: `POST http://localhost:8000/generate-command`
- ✅ Returns structured JSON: `{command, explanation, risk_level, confirmation_token}`
- ✅ Integrates with Ollama (phi3:mini) + Groq fallback
- ❌ No automated tests for generation accuracy
- ❌ No tests for fallback behavior

---

### Feature 2: Command Explanations
**Status**: ✅ **IMPLEMENTED**

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **AI Service** | `ai-service/src/intellishell_ai_service/main.py:156` | Returns `explanation` field from LLM |
| **Prompt** | `ai-service/src/intellishell_ai_service/main.py:57` | Requires one-sentence explanation in JSON |
| **Frontend** | `dashboard/src/pages/AssistantPage.jsx:122-129` | Displays explanation under command |
| **Tests** | None found | **GAP** |

**Verification**:
- ✅ LLM instructed to provide explanation
- ✅ Response model includes `explanation: str`
- ✅ UI renders explanation
- ❌ No validation that explanation is meaningful

---

### Feature 3: Safe Execution with Explicit User Confirmation
**Status**: ✅ **IMPLEMENTED** (Token-based confirmation)

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **Token Generation** | `ai-service/src/intellishell_ai_service/main.py:168-173` | UUID token with 5-minute TTL |
| **Token Storage** | `ai-service/src/intellishell_ai_service/main.py:33-34` | In-memory dict `PENDING_COMMANDS` |
| **Confirmation** | `ai-service/src/intellishell_ai_service/main.py:184-226` | `POST /confirm-execute` with token |
| **Token Expiry** | `ai-service/src/intellishell_ai_service/main.py:200-205` | 300-second TTL check |
| **Replay Prevention** | `ai-service/src/intellishell_ai_service/main.py:193` | Token popped (one-time use) |
| **Risk Routing** | `core-engine/src/server.rs:311-332` | HIGH→blocked, MEDIUM→sandbox, LOW→direct |
| **Frontend** | `dashboard/src/pages/AssistantPage.jsx` | Token managed but execution not connected |
| **Tests** | `tests/phase5_integration_test.sh:134-155` | Token replay test exists |

**Verification**:
- ✅ Token-based confirmation implemented
- ✅ One-time use enforced (pop from dict)
- ✅ TTL enforced (300s)
- ✅ Risk-based routing in core-engine
- ✅ Integration test for replay prevention
- ❌ Frontend does NOT call `/confirm-execute` - **PARTIAL GAP**
- ❌ No test for token expiry
- ❌ No test for command/context binding

**Security Issues**:
- ⚠️ In-memory token storage lost on service restart
- ⚠️ No binding between token and command hash (replay different command with same token)
- ⚠️ No rate limiting on token generation

---

### Feature 4: Process Viewing, Termination, Suspension, and Resumption
**Status**: ✅ **IMPLEMENTED**

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **Process List** | `core-engine/src/server.rs:365-368` | `GET /processes` endpoint |
| **Process Reading** | `core-engine/src/server.rs:517-542` | Reads `/proc/<pid>/status` |
| **Process Control** | `core-engine/src/server.rs:370-422` | `POST /processes/:pid` with signals |
| **Signals** | `core-engine/src/server.rs:384-391` | SIGTERM, SIGKILL, SIGSTOP, SIGCONT |
| **PID Protection** | `core-engine/src/server.rs:377-380` | Refuses PID 1 and self |
| **Frontend** | `dashboard/src/pages/ProcessesPage.jsx` + `components/ProcessManager.jsx` | Process table + control buttons |
| **API Client** | `dashboard/src/lib/api.js:117-127` | `rustApi.controlProcess(pid, action)` |
| **Tests** | None found | **GAP** |

**Verification**:
- ✅ `/processes` endpoint returns process list from `/proc`
- ✅ Kill (SIGTERM), Kill9 (SIGKILL), Pause (SIGSTOP), Resume (SIGCONT) implemented
- ✅ Protected PID check (PID 1, self)
- ✅ Frontend UI exists
- ❌ No tests for process control
- ❌ No permission checks beyond PID 1 (can kill any process user owns)

**Security Issues**:
- ⚠️ No additional permission checks (user can kill their own processes)
- ⚠️ No audit logging for process control actions

---

### Feature 5: Live CPU, RAM, Disk, and Network Monitoring
**Status**: ⚠️ **PARTIAL** (CPU, RAM implemented; Disk, Network MISSING from backend)

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **Snapshot Endpoint** | `core-engine/src/server.rs:273-309` | `GET /snapshot` |
| **CPU Monitoring** | `core-engine/src/server.rs:493-514` | Reads `/proc/stat` (simplified single snapshot) |
| **Memory Monitoring** | `core-engine/src/server.rs:278-291` | Reads `/proc/meminfo` (MemTotal, MemAvailable) |
| **Process List** | `core-engine/src/server.rs:294` | Top N processes by CPU |
| **Disk Monitoring** | ❌ NOT IMPLEMENTED | **MISSING** |
| **Network Monitoring** | ❌ NOT IMPLEMENTED | **MISSING** |
| **Frontend Charts** | `dashboard/src/components/SystemCharts.jsx` | Recharts visualization |
| **Polling** | `dashboard/src/hooks/useSystemSnapshot.js` | 5-second interval |
| **Tests** | `tests/phase5_integration_test.sh:157-186` | Snapshot format test |

**Verification**:
- ✅ CPU usage from `/proc/stat`
- ✅ Memory usage from `/proc/meminfo`
- ✅ Process list with PID, name, state, memory
- ❌ Disk usage NOT in backend (frontend expects `disk_used_percent`)
- ❌ Network RX/TX NOT in backend (frontend expects `network_rx_bytes`, `network_tx_bytes`)
- ❌ CPU calculation is single-snapshot (inaccurate, needs delta)

**Frontend Expectations**:
```javascript
// dashboard/src/pages/MonitoringPage.jsx:42-54
snapshot?.system?.disk_used_percent  // EXPECTED but NOT PROVIDED
snapshot?.system?.network_rx_bytes   // EXPECTED but NOT PROVIDED
snapshot?.system?.network_tx_bytes   // EXPECTED but NOT PROVIDED
```

**CRITICAL GAP**: Backend does not provide disk or network metrics, but frontend expects them. This is a **FAIL** for Feature 5.

---

### Feature 6: AI Troubleshooting Grounded in Live System Telemetry
**Status**: ✅ **IMPLEMENTED**

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **Troubleshooting** | `ai-service/src/intellishell_ai_service/main.py:229-298` | `POST /troubleshoot` |
| **Telemetry Fetch** | `ai-service/src/intellishell_ai_service/main.py:250-256` | Fetches `/snapshot` from core-engine |
| **AI Analysis** | `ai-service/src/intellishell_ai_service/main.py:258-283` | Sends telemetry + issue to LLM |
| **Evidence** | `ai-service/src/intellishell_ai_service/main.py:293` | Returns actual snapshot as evidence |
| **Frontend** | ❌ NOT CONNECTED | No UI for troubleshooting |
| **Tests** | None found | **GAP** |

**Verification**:
- ✅ Endpoint exists and fetches real telemetry
- ✅ Sends telemetry to AI for analysis
- ✅ Returns diagnosis + recommendations
- ❌ No frontend UI to trigger troubleshooting
- ❌ No tests for troubleshooting flow

**GAP**: Feature is implemented in backend but NOT exposed in frontend.

---

### Feature 7: Persistent Command History and Safe Reuse
**Status**: ⚠️ **PARTIAL** (Database schema exists, but NO writes on command execution)

| Component | File Path | Implementation |
|-----------|-----------|----------------|
| **Database** | `core-engine/src/server.rs:49-176` | SQLite with `command_history` table |
| **Schema** | `core-engine/src/server.rs:60-77` | 16 columns including query, command, risk, status, outputs |
| **Insert Method** | `core-engine/src/server.rs:99-125` | `async fn insert_command()` |
| **Get History** | `core-engine/src/server.rs:127-164` | `async fn get_history(limit, offset)` |
| **Update Execution** | `core-engine/src/server.rs:166-186` | Updates exit_code, stdout, stderr |
| **History Endpoint** | `core-engine/src/server.rs:358-363` | `GET /history` |
| **Execution Flow** | `core-engine/src/server.rs:311-356` | ❌ NO database writes in execute_command |
| **Frontend** | `dashboard/src/pages/HistoryPage.jsx` + `components/CommandHistory.jsx` | UI exists |
| **Tests** | `tests/phase5_integration_test.sh:188-201` | History retrieval test |

**Verification**:
- ✅ Database schema created on server start
- ✅ Insert/get methods implemented
- ✅ `/history` endpoint exists
- ❌ `execute_command` does NOT call `insert_command` - **CRITICAL GAP**
- ❌ No history persistence on execution
- ❌ No reuse flow (frontend can copy command but no re-validation)

**CRITICAL GAP**: Commands are NOT persisted to database on execution. History is always empty.

**Database File**: `d:\3rd year\os project\IntelliShell\intellishell.db` (does NOT exist, created on first run)

---

## 2. Gap Analysis Summary

### Critical Gaps (Must Fix for Release)

1. **Feature 5: Disk and Network Monitoring MISSING**
   - Backend does NOT provide `disk_used_percent`, `network_rx_bytes`, `network_tx_bytes`
   - Frontend expects these fields and will show 0% or NaN
   - **Files to modify**: `core-engine/src/server.rs` (add disk/network reading)
   - **Impact**: Feature 5 is INCOMPLETE

2. **Feature 7: Command History Persistence NOT WIRED**
   - `execute_command` does not call database insert
   - History is always empty
   - **Files to modify**: `core-engine/src/server.rs:311-356` (add DB insert calls)
   - **Impact**: Feature 7 is BROKEN

3. **Feature 3: Frontend Does Not Execute Commands**
   - AssistantPage generates commands but has no "Execute" button
   - Token is generated but never sent to `/confirm-execute`
   - **Files to modify**: `dashboard/src/pages/AssistantPage.jsx` (add confirm/execute flow)
   - **Impact**: End-to-end workflow is INCOMPLETE

### High-Priority Gaps (Should Fix)

4. **Feature 6: Troubleshooting UI Missing**
   - Backend endpoint exists but no frontend to use it
   - **Files to create**: Troubleshooting UI component or add to AssistantPage
   - **Impact**: Feature 6 not user-accessible

5. **CPU Usage Calculation Inaccurate**
   - Single-snapshot CPU reading is meaningless
   - Needs two snapshots with delta calculation
   - **Files to modify**: `core-engine/src/server.rs:493-514`
   - **Impact**: CPU % is incorrect

6. **No Automated Tests for Core Features**
   - Only one integration test script exists (`phase5_integration_test.sh`)
   - No Rust unit tests for execution routing
   - No Python tests for providers/risk classification
   - **Files to create**: Multiple test files
   - **Impact**: Regression risk high

### Medium-Priority Gaps (Nice to Have)

7. **Token Security Weaknesses**
   - Tokens lost on service restart (in-memory)
   - No command hash binding
   - No rate limiting
   - **Files to modify**: `ai-service/src/intellishell_ai_service/main.py`
   - **Impact**: Security hardening

8. **Process Control Audit Logging**
   - No logging of who killed which process
   - **Files to modify**: `core-engine/src/server.rs:370-422`
   - **Impact**: Audit trail missing

---

## 3. Security Audit Checklist

Based on actual source code review:

### High-Risk Command Blocking
**Location**: `ai-service/src/intellishell_ai_service/risk.py:34-48`

✅ **IMPLEMENTED**:
```python
# Critical patterns (lines 34-48)
- rm -rf / detection
- dd to /dev/sd* detection
- Fork bomb detection
- chmod 777 / detection
- kill -9 1 detection
- shutdown/reboot detection
- Piped remote scripts (curl | bash)
- mkfs/fdisk detection
```

✅ **Enforcement**: `core-engine/src/server.rs:318-326` blocks HIGH/CRITICAL risk commands

❌ **GAP**: No tests for risk classifier edge cases

### Confirmation Token TTL
**Location**: `ai-service/src/intellishell_ai_service/main.py:33-34, 200-205`

✅ **IMPLEMENTED**:
```python
TOKEN_TTL_SECONDS = 300  # 5 minutes
if time.time() - entry["created_at"] > TOKEN_TTL_SECONDS:
    raise HTTPException(status_code=410, detail="Token expired")
```

❌ **GAP**: No test for expiry behavior

### Token Replay Prevention
**Location**: `ai-service/src/intellishell_ai_service/main.py:193`

✅ **IMPLEMENTED**:
```python
entry = PENDING_COMMANDS.pop(request.token, None)  # One-time use
```

✅ **TEST EXISTS**: `tests/phase5_integration_test.sh:134-155`

⚠️ **WEAKNESS**: No binding between token and command content (can swap command after generation)

### Protected PID Guard
**Location**: `core-engine/src/server.rs:377-380`

✅ **IMPLEMENTED**:
```rust
if pid == 1 || pid == std::process::id() {
    return Err(ApiError::ProcessControl(format!("Refusing to act on protected PID {}", pid)));
}
```

❌ **GAP**: No test for protected PID rejection

### Input Validation
**Location**: `core-engine/src/server.rs:313-316`

⚠️ **PARTIAL**:
```rust
if req.command.trim().is_empty() {
    return Err(ApiError::InvalidRequest("Empty command".to_string()));
}
```

❌ **GAPS**:
- No shell metacharacter sanitization
- No path traversal prevention
- No command length limit
- Relies entirely on risk classifier

### SQL Injection Prevention
**Location**: `core-engine/src/server.rs:99-125`

✅ **SAFE**: Uses parameterized queries via `rusqlite::params![]`
```rust
conn.execute("INSERT INTO command_history (...) VALUES (?1, ?2, ...)", params![...])
```

### Secret Handling
**Status**: ⚠️ **PARTIAL**

✅ **Good**:
- API keys read from environment variables
- No hardcoded secrets in source code

❌ **GAPS**:
- No `.env` in `.gitignore` verification
- No secret scanning in CI
- No documentation on secret management

---

## 4. Build and Test Status Assessment

### Rust Core Engine

**Build Dependencies** (`core-engine/Cargo.toml`):
- axum 0.7 (HTTP server)
- tokio 1.x (async runtime)
- rusqlite 0.31 (SQLite)
- serde/serde_json (serialization)
- nix 0.29 (signal/process control)
- libc 0.2 (low-level syscalls)

**Likely Build Status**: ✅ PASS (standard dependencies)

**Test Status**:
- ❌ No `#[test]` modules found in server.rs
- ✅ Sandbox module has unit tests (lines 301-330 in sandbox.rs)
- ❌ No integration tests in `core-engine/tests/`

**To Run**:
```bash
cd core-engine
cargo fmt --check    # Format check
cargo clippy         # Lint
cargo test           # Unit tests
cargo build --release  # Production build
```

### Python AI Service

**Dependencies** (`ai-service/pyproject.toml`):
- fastapi 0.115+
- uvicorn 0.32+
- requests 2.32+
- pydantic 2.10+

**Test Dependencies**:
- pytest 8.3+
- ruff 0.8+ (linting)

**Likely Build Status**: ✅ PASS (standard dependencies)

**Test Status**:
- ❌ No tests found in `ai-service/tests/` directory
- ✅ Standalone test scripts exist (test_providers_integration.py, test_phase4_final.py)
- ❌ Not integrated into pytest suite

**To Run**:
```bash
cd ai-service
uv run ruff format --check  # Format check
uv run ruff check           # Lint
uv run pytest               # Tests (currently empty)
uv run uvicorn main:app     # Start service
```

### React Dashboard

**Dependencies** (`dashboard/package.json`):
- react 19.2.8
- react-dom 19.2.8
- react-router-dom 7.18.4
- recharts 3.10.1
- @tanstack/react-query 5.104.5
- axios 1.20.0
- vite 8.2.0

**Build Status**: ✅ **VERIFIED PASSING** (from Phase 7 report)
- Build time: 572ms
- Bundle size: 782KB JS, 12.77KB CSS
- No ESLint errors

**Test Status**:
- ❌ No test suite configured
- ❌ No component tests
- ❌ No E2E tests

**To Run**:
```bash
cd dashboard
npm run lint      # ESLint
npm run build     # Production build
npm run dev       # Dev server
```

### Cross-Service Tests

**Integration Test**: `tests/phase5_integration_test.sh`
- ✅ Tests service health checks
- ✅ Tests command generation flow
- ✅ Tests risk-based execution
- ✅ Tests token replay prevention
- ✅ Tests system monitoring snapshot
- ✅ Tests history retrieval

**Gaps**:
- ❌ No test for command history persistence (would fail - history empty)
- ❌ No test for disk/network metrics (would fail - not provided)
- ❌ No test for troubleshooting endpoint
- ❌ No E2E test with actual browser

---

## 5. Implementation Plan: Ordered by Dependency

---

### Critical Path: Must Fix for Release

- [ ] **1. Add Disk and Network Monitoring to Backend**
      
      **What**: Implement disk usage and network traffic monitoring in Rust core-engine
      
      **Files**:
      - `core-engine/src/server.rs` (add functions: `get_disk_usage()`, `get_network_stats()`)
      - `core-engine/src/server.rs` (update `SystemSnapshot` struct with new fields)
      - `core-engine/src/server.rs` (update `get_snapshot()` endpoint to include disk/network)
      
      **Implementation**:
      - Read `/proc/mounts` + `statvfs()` for disk usage (used/total per mount point, aggregate)
      - Read `/proc/net/dev` for RX/TX byte counters (aggregate across interfaces excluding loopback)
      - Add `disk_total_bytes`, `disk_used_bytes`, `disk_used_percent`, `network_rx_bytes`, `network_tx_bytes` to `SystemSnapshot`
      
      **Verify**: 
      - Run `cargo build` (success)
      - Start server, call `curl http://localhost:3000/snapshot`
      - Verify JSON includes `disk_used_percent`, `network_rx_bytes`, `network_tx_bytes` with non-zero values
      - Frontend charts should display disk and network data

---

- [ ] **2. Wire Command History Persistence**
      
      **What**: Connect execution flow to database persistence
      
      **Files**:
      - `core-engine/src/server.rs` (modify `execute_command`, `execute_direct`, `execute_sandboxed`)
      
      **Implementation**:
      - In `execute_command` (after request validation): create `CommandHistoryRecord` with status="pending", insert into DB, capture record ID
      - After execution (direct or sandboxed): call `db.update_execution()` with actual exit_code, stdout, stderr, duration
      - Handle errors gracefully (log DB errors but don't fail execution)
      
      **Verify**:
      - Run `cargo test` (add unit test for DB insert/update)
      - Start server, execute command via `POST /execute`
      - Call `GET /history`, verify command appears with correct outputs
      - Restart server, verify history persists (SQLite file)

---

- [ ] **3. Fix CPU Usage Calculation (Delta-Based)**
      
      **What**: Implement two-snapshot CPU calculation for accuracy
      
      **Files**:
      - `core-engine/src/server.rs` (modify `get_cpu_usage()`)
      
      **Implementation**:
      - Store previous `/proc/stat` snapshot in `AppState` (with `Mutex<Option<CpuSnapshot>>`)
      - On each call: read current snapshot, calculate delta with previous, store current for next call
      - Formula: `cpu_percent = 100.0 * (1.0 - (idle_delta / total_delta))`
      - First call returns 0.0 (no previous snapshot)
      
      **Verify**:
      - Run server, call `/snapshot` twice with 1s delay
      - Second call should return reasonable CPU % (not 0.0 or 100.0 unless accurate)
      - Compare with `top` command output

---

- [ ] **4. Add Command Execution Flow to Frontend**
      
      **What**: Connect AssistantPage to confirm/execute endpoint
      
      **Files**:
      - `dashboard/src/pages/AssistantPage.jsx` (add confirm/execute button and mutation)
      - `dashboard/src/lib/api.js` (verify `pythonApi.confirmExecute()` exists - already implemented)
      
      **Implementation**:
      - After command generation, display "Execute" button (only if `confirmation_token` exists)
      - On click: call `pythonApi.confirmExecute(token)`, display loading state
      - On success: show execution result (stdout, stderr, exit_code, method)
      - On error: show error message (expired token, blocked command, etc.)
      - Add "executed" state to track whether command was run
      
      **Verify**:
      - Generate command via UI
      - Click "Execute", verify API call to `/confirm-execute`
      - Verify result displayed (stdout/stderr)
      - Check backend logs for execution
      - Verify history updates (after step 2)

---

- [ ] **5. Add Troubleshooting UI**
      
      **What**: Expose troubleshooting endpoint in frontend
      
      **Files**:
      - `dashboard/src/pages/AssistantPage.jsx` OR create new `TroubleshootSection` component
      - `dashboard/src/lib/api.js` (verify `pythonApi.troubleshoot()` exists - already implemented)
      
      **Implementation**:
      - Add textarea for issue description
      - Add "Analyze System" button
      - On submit: call `pythonApi.troubleshoot(issueDescription)`
      - Display: diagnosis, confidence, evidence (CPU/memory/processes), recommendations
      - Show actual telemetry values as evidence
      
      **Verify**:
      - Enter issue description (e.g., "system is slow")
      - Click "Analyze", verify API call to `/troubleshoot`
      - Verify AI analysis displayed with real telemetry
      - Check that evidence includes actual snapshot data

---

### Security Hardening

- [ ] **6. Add Input Validation and Rate Limiting**
      
      **What**: Harden command generation and execution endpoints
      
      **Files**:
      - `ai-service/src/intellishell_ai_service/main.py` (add rate limiting, query length limits)
      - `core-engine/src/server.rs` (add command length limit, basic shell metacharacter check)
      
      **Implementation**:
      - Python: Add max query length (1000 chars), rate limit (10 req/min per IP using in-memory counter)
      - Python: Add command hash to token entry, verify hash on confirm-execute
      - Rust: Add max command length (500 chars), reject commands with suspicious patterns (e.g., `&&`, `;`, `|` in unexpected contexts)
      
      **Verify**:
      - Try to generate command with 2000-char query (should reject)
      - Generate 11 commands in 1 minute (11th should rate-limit)
      - Generate command, modify token entry's command, try to execute (should reject)

---

- [ ] **7. Add Process Control Audit Logging**
      
      **What**: Log all process control actions to database
      
      **Files**:
      - `core-engine/src/server.rs` (add `process_actions` table in database schema)
      - `core-engine/src/server.rs` (log every control_process call)
      
      **Implementation**:
      - Create table: `process_actions(id, timestamp, pid, action, success, error)`
      - In `control_process` endpoint: insert log entry before and after action
      - Add `GET /audit/process-actions` endpoint to retrieve logs
      
      **Verify**:
      - Kill a process via API
      - Query `/audit/process-actions`, verify entry exists
      - Restart server, verify logs persist

---

### Testing

- [ ] **8. Add Rust Unit Tests for Execution Routing**
      
      **What**: Test risk-based execution routing logic
      
      **Files**:
      - `core-engine/src/server.rs` (add `#[cfg(test)] mod tests` module)
      
      **Implementation**:
      - Test HIGH risk → blocked
      - Test MEDIUM risk → sandboxed (mock or skip if no root)
      - Test LOW risk → direct execution
      - Test invalid risk level → error
      - Test empty command → error
      
      **Verify**:
      - Run `cargo test`, verify all tests pass

---

- [ ] **9. Add Python Tests for Risk Classifier**
      
      **What**: Test risk classification patterns
      
      **Files**:
      - `ai-service/tests/test_risk.py` (new file)
      
      **Implementation**:
      - Test critical patterns: `rm -rf /`, fork bomb, `dd of=/dev/sda`, etc.
      - Test high patterns: `rm -rf`, `chmod 000`, etc.
      - Test medium patterns: `kill -9`, `chmod 777`, `sudo`
      - Test low patterns: `rm file`, `kill PID`
      - Test safe patterns: `ls`, `cat`, `grep`
      
      **Verify**:
      - Run `uv run pytest`, verify all tests pass

---

- [ ] **10. Add Python Tests for Provider Fallback**
      
      **What**: Test Ollama → Groq fallback behavior
      
      **Files**:
      - `ai-service/tests/test_providers.py` (new file)
      
      **Implementation**:
      - Mock Ollama unavailable → verify Groq called
      - Mock Ollama timeout → verify Groq called
      - Mock both unavailable → verify exception
      - Mock malformed output → verify retry logic
      
      **Verify**:
      - Run `uv run pytest`, verify all tests pass

---

- [ ] **11. Add Integration Test for Disk/Network Monitoring**
      
      **What**: Verify snapshot includes disk and network
      
      **Files**:
      - `tests/test_monitoring.sh` (new file)
      
      **Implementation**:
      - Call `GET /snapshot`
      - Parse JSON, assert `disk_used_percent > 0`
      - Assert `network_rx_bytes >= 0` and `network_tx_bytes >= 0`
      
      **Verify**:
      - Run `bash tests/test_monitoring.sh`, verify PASS

---

- [ ] **12. Add Integration Test for History Persistence**
      
      **What**: Verify commands are persisted to history
      
      **Files**:
      - `tests/test_history_persistence.sh` (new file)
      
      **Implementation**:
      - Execute command via `POST /execute`
      - Call `GET /history`
      - Verify command appears with correct query, command, exit_code, stdout
      - Restart core-engine service
      - Call `GET /history` again, verify history still present
      
      **Verify**:
      - Run `bash tests/test_history_persistence.sh`, verify PASS

---

- [ ] **13. Add E2E Test for Full Command Flow**
      
      **What**: Test generate → confirm → execute → history
      
      **Files**:
      - `tests/test_e2e_command_flow.sh` (new file)
      
      **Implementation**:
      - Generate command: `POST /generate-command`
      - Extract token
      - Confirm and execute: `POST /confirm-execute` with token
      - Verify execution result
      - Check history: `GET /history`, verify command appears
      
      **Verify**:
      - Run `bash tests/test_e2e_command_flow.sh`, verify PASS

---

### Documentation

- [ ] **14. Write Verified Setup Instructions**
      
      **What**: Document exact setup and run instructions for Windows + WSL2
      
      **Files**:
      - `docs/SETUP.md` (new or update README.md)
      
      **Content**:
      - Prerequisites: WSL2, Ubuntu 22.04+, systemd, cgroup v2
      - Toolchain: Rust 1.87, Python 3.13, Node 24, Ollama
      - Clone repository (must be in `/home/`, not `/mnt/`)
      - Run `./scripts/bootstrap.sh`
      - Install Ollama and pull model
      - Create `.env` file with example values
      - Start services: `make run-core`, `make run-api`, `make run-dashboard`
      - Health check: visit `http://localhost:5173`
      
      **Verify**:
      - Follow instructions on fresh WSL2 Ubuntu VM
      - Verify all services start successfully

---

- [ ] **15. Document Environment Variables**
      
      **What**: Create `.env.example` with all required/optional variables
      
      **Files**:
      - `.env.example` (new file)
      - `docs/CONFIGURATION.md` (new file)
      
      **Content**:
      ```bash
      # Core Engine
      RUST_LOG=info
      DATABASE_PATH=./intellishell.db
      
      # AI Service
      OLLAMA_URL=http://localhost:11434
      OLLAMA_MODEL=phi3:mini
      GROQ_API_KEY=                    # Optional, for fallback
      CORE_ENGINE_URL=http://localhost:3000
      
      # Dashboard
      VITE_RUST_API_URL=http://localhost:3000
      VITE_PYTHON_API_URL=http://localhost:8000
      ```
      
      **Verify**:
      - Copy `.env.example` to `.env`, services start with default values

---

- [ ] **16. Document API Endpoints**
      
      **What**: Create OpenAPI-style documentation for all endpoints
      
      **Files**:
      - `docs/API.md` (new file)
      
      **Content**:
      - Core Engine endpoints (9 endpoints)
        - `GET /health`, `GET /snapshot`, `POST /execute`, `GET /history`, `GET /processes`, `POST /processes/:pid`
      - AI Service endpoints (7 endpoints)
        - `GET /health`, `POST /generate-command`, `POST /confirm-execute`, `POST /troubleshoot`, `GET /activity`, `GET /history`, `POST /classify`, `POST /suggest`
      - Request/response schemas
      - Example curl commands
      
      **Verify**:
      - Test each example curl command, verify it works

---

- [ ] **17. Document Known Limitations and Platform Restrictions**
      
      **What**: Honest documentation of what works and what doesn't
      
      **Files**:
      - `docs/LIMITATIONS.md` (new file)
      
      **Content**:
      - **Tested Platform**: WSL2 Ubuntu 22.04+ on Windows 10/11
      - **Sandbox Requirements**: Root required for cgroup manipulation (sandboxed execution)
      - **Filesystem**: Must be cloned to `/home/`, not `/mnt/*` (drvfs limitations)
      - **Network Isolation**: NOT implemented (sandboxed commands have network access)
      - **Filesystem Isolation**: NOT implemented (no chroot/pivot_root)
      - **Token Storage**: In-memory only (lost on service restart)
      - **AI Fallback**: Requires Groq API key (optional)
      - **Windows Native**: NOT supported (WSL2 required for Linux APIs)
      
      **Verify**:
      - Review with user expectations

---

- [ ] **18. Create Troubleshooting Guide**
      
      **What**: Common issues and solutions
      
      **Files**:
      - `docs/TROUBLESHOOTING.md` (new file or update existing)
      
      **Content**:
      - Service not starting (port already in use, missing dependencies)
      - Ollama not responding (systemd service not enabled)
      - Sandboxed execution fails (not running as root, cgroup v2 not enabled)
      - Frontend shows 0% CPU/memory (backend not running)
      - History empty (database not persisted - check after step 2 fix)
      - GPU not detected (NVIDIA driver passthrough in WSL2)
      
      **Verify**:
      - Intentionally break each scenario, verify fix works

---

- [ ] **19. Write Release Notes**
      
      **What**: Final release report with PASS/FAIL/BLOCKED for all features
      
      **Files**:
      - `RELEASE_NOTES.md` (new file)
      - `docs/PHASE8_COMPLETION_REPORT.md` (new file)
      
      **Content**:
      - **Feature Status**: PASS/FAIL/BLOCKED for all 7 features
      - **Build Status**: Rust/Python/Dashboard build results
      - **Test Results**: Unit, integration, E2E test summary
      - **Security Audit**: Checklist with findings
      - **Known Issues**: Documented limitations
      - **Run Instructions**: Quick start commands
      - **Release Decision**: READY / PARTIALLY READY / NOT READY
      
      **Verify**:
      - Review against requirements from original Phase 8 prompt

---

### Final Verification

- [ ] **20. Run Full Test Suite**
      
      **What**: Execute all tests and verify passing
      
      **Commands**:
      ```bash
      # Rust tests
      cd core-engine && cargo test
      
      # Python tests
      cd ai-service && uv run pytest
      
      # Integration tests
      bash tests/phase5_integration_test.sh
      bash tests/test_monitoring.sh
      bash tests/test_history_persistence.sh
      bash tests/test_e2e_command_flow.sh
      
      # Frontend build
      cd dashboard && npm run lint && npm run build
      ```
      
      **Expected**:
      - All Rust tests pass
      - All Python tests pass
      - All integration tests pass
      - Frontend builds with no errors
      
      **Verify**:
      - Document actual results in Phase 8 completion report

---

- [ ] **21. Manual End-to-End Workflow Verification**
      
      **What**: Execute all 5 end-to-end workflows from Phase 8 requirements
      
      **Workflow A: Natural language to execution**
      1. Open `http://localhost:5173/assistant`
      2. Enter query: "list all files modified in the last 24 hours"
      3. Click "Generate Command"
      4. Verify command appears with explanation and risk level
      5. Click "Execute" (after step 4 implementation)
      6. Verify result displayed (stdout, exit code)
      7. Navigate to History page, verify command appears
      
      **Workflow B: System monitoring**
      1. Open `http://localhost:5173/monitoring`
      2. Verify CPU, memory, disk, network charts display data
      3. Wait 10 seconds, verify charts update with new data
      4. Verify metrics are reasonable (CPU 0-100%, memory > 0, disk > 0)
      
      **Workflow C: Process management**
      1. Open `http://localhost:5173/processes`
      2. Verify process list displays (PID, name, state, memory)
      3. Start a test process: `sleep 300 &`
      4. Find process in list
      5. Click "Pause" → verify process state changes
      6. Click "Resume" → verify process resumes
      7. Click "Terminate" → verify process exits
      
      **Workflow D: Persistent history**
      1. Execute a command (via Workflow A or direct executor)
      2. Navigate to History page, verify command appears
      3. Stop core-engine service (`pkill intellishell-server`)
      4. Restart core-engine
      5. Refresh History page, verify command still appears
      6. Click "Reuse Command" → verify copied to executor
      
      **Workflow E: AI troubleshooting**
      1. Open troubleshooting UI (after step 5 implementation)
      2. Enter issue: "system is running slow"
      3. Click "Analyze System"
      4. Verify AI analysis displays with:
         - Diagnosis (based on actual telemetry)
         - Evidence (real CPU/memory/process data)
         - Recommendations (actionable steps)
      
      **Verify**:
      - Document results for each workflow in completion report

---

- [ ] **22. Security Test Scenarios**
      
      **What**: Verify security controls with deliberate attacks
      
      **Tests**:
      1. **High-risk command blocking**:
         - Generate command: "delete all files in root directory"
         - Verify risk_level = "high" or "critical"
         - Try to execute → verify blocked (not executed)
      
      2. **Token replay attack**:
         - Generate command, capture token
         - Execute with token (success)
         - Try to execute same token again → verify rejected (404 or 410)
      
      3. **Token expiration**:
         - Generate command, capture token
         - Wait 6 minutes (TTL = 5 minutes)
         - Try to execute → verify rejected (410 Gone)
      
      4. **Protected PID**:
         - Try to kill PID 1 via API → verify rejected
         - Try to kill core-engine's own PID → verify rejected
      
      5. **SQL injection attempt**:
         - Generate command with SQL injection in query: `'; DROP TABLE command_history; --`
         - Verify command executes safely (no SQL injection)
      
      6. **Rate limiting** (after step 6):
         - Generate 11 commands in 1 minute → verify 11th rejected
      
      **Verify**:
      - Document results (PASS/FAIL) for each security test

---

- [ ] **23. Build Production Artifacts**
      
      **What**: Create release-ready builds
      
      **Commands**:
      ```bash
      # Rust release build
      cd core-engine && cargo build --release
      # Binary: target/release/intellishell-server
      
      # Dashboard production build
      cd dashboard && npm run build
      # Output: dist/ directory
      ```
      
      **Verify**:
      - Rust binary exists and runs: `./target/release/intellishell-server`
      - Dashboard dist/ can be served: `npx serve -s dist -p 5173`

---

- [ ] **24. Final Release Decision**
      
      **What**: Assess readiness and write final report
      
      **Criteria**:
      - ✅ All 7 required features: PASS or DOCUMENTED LIMITATION
      - ✅ Critical end-to-end workflows: PASS
      - ✅ Frontend builds: PASS
      - ✅ Core services start: PASS
      - ✅ Security controls verified: PASS
      - ✅ Known issues documented: COMPLETE
      
      **Output**: `docs/PHASE8_COMPLETION_REPORT.md` with:
      - PASS/FAIL/BLOCKED status for every required feature
      - Actual build and test outputs (copy/paste from terminal)
      - End-to-end workflow results
      - Remaining defects and limitations
      - Exact run instructions
      - **Final conclusion: READY / PARTIALLY READY / NOT READY**
      
      **Verify**:
      - Review with original Phase 8 requirements

---

## 6. Priority Summary

### Must Complete (for "PARTIALLY READY" status)
1. Add disk and network monitoring (Feature 5 completion)
2. Wire command history persistence (Feature 7 completion)
3. Add command execution to frontend (Feature 3 completion)
4. Add troubleshooting UI (Feature 6 accessibility)
5. Write verified setup instructions
6. Document known limitations
7. Manual workflow verification (all 5 workflows)
8. Final release report

### Should Complete (for "READY" status)
9. Fix CPU usage calculation (accuracy)
10. Add security hardening (rate limiting, command hash)
11. Add Rust unit tests (execution routing)
12. Add Python tests (risk classifier, providers)
13. Add integration tests (monitoring, history, E2E)
14. Process control audit logging
15. API endpoint documentation
16. Troubleshooting guide
17. Security test scenarios
18. Build production artifacts

### Nice to Have (for "POLISHED" release)
19. Environment variable documentation
20. Configuration guide
21. Expanded test coverage
22. Performance benchmarks
23. Accessibility audit

---

## 7. Estimated Effort

| Item | Lines of Code | Complexity | Time Estimate |
|------|---------------|------------|---------------|
| Disk/network monitoring | ~150 | Medium | 2-3 hours |
| History persistence | ~50 | Low | 1 hour |
| CPU delta calculation | ~80 | Medium | 1-2 hours |
| Frontend execution flow | ~100 | Low | 1-2 hours |
| Troubleshooting UI | ~150 | Medium | 2-3 hours |
| Security hardening | ~200 | Medium | 3-4 hours |
| Unit tests (Rust) | ~300 | Medium | 3-4 hours |
| Unit tests (Python) | ~400 | Medium | 4-5 hours |
| Integration tests | ~300 | Medium | 3-4 hours |
| Documentation | ~2000 words | Low | 4-5 hours |
| Manual verification | N/A | High | 4-5 hours |
| **Total** | ~1730 LOC + docs | | **28-38 hours** |

---

## 8. Risk Assessment

### High Risk
- **Disk/network monitoring**: May encounter WSL2 `/proc` limitations (mitigate: test early)
- **Sandboxed execution**: Requires root, may fail in restricted environments (mitigate: document requirement)
- **History persistence**: Database file location/permissions (mitigate: use project directory, not system)

### Medium Risk
- **CPU delta calculation**: Needs careful state management in async context (mitigate: use `Mutex`)
- **Token security**: Race conditions in concurrent access (mitigate: existing `PENDING_COMMANDS` dict should be thread-safe in Python GIL)
- **Test coverage**: Time-consuming, may find unexpected bugs (mitigate: prioritize critical paths)

### Low Risk
- **Documentation**: Straightforward, no code changes
- **Frontend execution flow**: Well-defined APIs already exist
- **Troubleshooting UI**: Backend already implemented, just needs frontend

---

## 9. Acceptance Criteria

This phase is complete when:

1. ✅ All 7 required features have been individually tested
2. ✅ Feature status matrix shows PASS, FAIL (with documented fixes), or BLOCKED (with justification)
3. ✅ All 5 end-to-end workflows execute successfully and results are documented
4. ✅ Frontend builds without errors (`npm run build`)
5. ✅ Core services start and respond to health checks
6. ✅ Security controls verified through deliberate attack tests
7. ✅ Known limitations and unresolved issues are clearly documented
8. ✅ Release report written with PASS/FAIL/BLOCKED for every feature
9. ✅ Exact run instructions provided and verified on clean environment
10. ✅ Final conclusion states: READY, PARTIALLY READY, or NOT READY

**Definition of "READY"**:
- All 7 features: PASS
- All 5 workflows: PASS
- Critical security controls: VERIFIED
- No unresolved HIGH or CRITICAL defects
- Documentation complete

**Definition of "PARTIALLY READY"**:
- 5+ features: PASS
- 3+ workflows: PASS
- Core security controls: VERIFIED
- Remaining issues documented with workarounds

**Definition of "NOT READY"**:
- <5 features: PASS
- Critical security controls: FAIL
- Services do not start
- Unresolved CRITICAL defects

---

## 10. Out of Scope

The following are explicitly **NOT** part of Phase 8:

- ❌ New feature development (only completing existing incomplete features)
- ❌ UI/UX redesign (Phase 7 design is final)
- ❌ Performance optimization (beyond fixing CPU calculation)
- ❌ Scalability enhancements (single-user application)
- ❌ Deployment automation (CI/CD, Docker, cloud deployment)
- ❌ Windows native support (WSL2 only)
- ❌ Multi-user authentication (single user, local only)
- ❌ Production deployment to cloud (local development only)
- ❌ Mobile app development
- ❌ API versioning or breaking changes
- ❌ Third-party integrations beyond Ollama/Groq

---

## 11. Success Metrics

| Metric | Target | Actual (TBD) |
|--------|--------|--------------|
| Features PASS | 7/7 | - |
| Workflows PASS | 5/5 | - |
| Unit tests passing | 100% | - |
| Integration tests passing | 100% | - |
| Security tests PASS | 6/6 | - |
| Build success | 3/3 (Rust, Python, Dashboard) | - |
| Documentation coverage | 100% of user-facing features | - |
| Known limitations documented | Yes | - |
| Release decision | READY | - |

---

## 12. Verification Commands Reference

Quick reference for verifying each step:

```bash
# Step 1: Verify disk/network monitoring
curl -s http://localhost:3000/snapshot | jq '.disk_used_percent, .network_rx_bytes, .network_tx_bytes'

# Step 2: Verify history persistence
curl -s -X POST http://localhost:3000/execute -H 'Content-Type: application/json' -d '{"command":"echo test","risk_level":"low","timeout_seconds":5}'
curl -s http://localhost:3000/history | jq '. | length'

# Step 3: Verify CPU delta
curl -s http://localhost:3000/snapshot | jq '.cpu_percent'  # Call twice, 5s apart

# Step 4-5: Manual UI verification in browser

# Step 8: Run Rust tests
cd core-engine && cargo test

# Step 9: Run Python tests
cd ai-service && uv run pytest

# Step 11: Run monitoring test
bash tests/test_monitoring.sh

# Step 12: Run history test
bash tests/test_history_persistence.sh

# Step 13: Run E2E test
bash tests/test_e2e_command_flow.sh

# Step 20: Full test suite
make test

# Step 23: Production build
cd core-engine && cargo build --release
cd dashboard && npm run build
```

---

## 13. Conclusion

This plan addresses all Phase 8 requirements:

✅ **Feature Traceability Matrix**: Documented with actual file paths and line numbers  
✅ **Gap Analysis**: Critical, high-priority, and medium-priority gaps identified  
✅ **Security Audit**: Checklist completed with source code verification  
✅ **Build/Test Status**: Assessed for all three services  
✅ **Implementation Plan**: 24 steps ordered by dependency  
✅ **Verification Procedures**: Specific commands and expected outcomes  
✅ **Release Documentation**: Comprehensive docs planned  
✅ **Honest Assessment**: No fabricated implementations, all findings based on actual source code

**Current Assessment** (before implementation):
- Feature 1 (Command Generation): ✅ IMPLEMENTED
- Feature 2 (Explanations): ✅ IMPLEMENTED
- Feature 3 (Safe Execution): ⚠️ PARTIAL (token flow complete, frontend execution missing)
- Feature 4 (Process Control): ✅ IMPLEMENTED
- Feature 5 (Monitoring): ❌ PARTIAL (CPU/RAM only, disk/network MISSING)
- Feature 6 (Troubleshooting): ⚠️ PARTIAL (backend only, no UI)
- Feature 7 (History): ❌ BROKEN (schema exists, no writes)

**Estimated Post-Implementation Status** (if all steps completed):
- All 7 features: ✅ PASS
- All 5 workflows: ✅ PASS
- Security: ✅ HARDENED
- Tests: ✅ COMPREHENSIVE
- Documentation: ✅ COMPLETE
- **Release Status**: ✅ **READY**

---

**Plan Status**: ✅ COMPLETE  
**Next Step**: Begin implementation starting with Step 1 (Disk/Network Monitoring)  
**Estimated Completion**: 28-38 hours of implementation + verification work
