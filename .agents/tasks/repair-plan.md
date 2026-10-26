# IntelliShell Critical Defects Repair Plan

**Created**: 2024-01-23  
**Status**: READY FOR IMPLEMENTATION  
**Objective**: Fix 5 critical defects blocking IntelliShell release, add comprehensive tests, verify end-to-end functionality

---

## Context Summary

**Technology Stack**:
- **Core Engine**: Rust (Axum web server, SQLite database, /proc filesystem monitoring)
- **AI Service**: Python 3.13 + FastAPI + Ollama/Groq providers
- **Frontend**: React 19 + Vite + TanStack Query
- **Database**: SQLite (embedded in Rust server)
- **Build System**: Make + Cargo + uv + npm
- **Test Frameworks**: Rust cargo test, Python pytest, WSL2/Ubuntu environment required

**Key Files**:
- `core-engine/src/server.rs` — Main HTTP server, monitoring, execution, database
- `ai-service/src/intellishell_ai_service/main.py` — AI generation, risk assessment, token management
- `ai-service/src/intellishell_ai_service/risk.py` — Deterministic risk classifier
- `dashboard/src/pages/AssistantPage.jsx` — Command generation UI
- `dashboard/src/lib/api.js` — API client with pythonApi.confirmExecute method

**Build Commands** (discovered from Makefile):
- Rust: `cd core-engine && cargo test` (must run in WSL, requires Linux /proc)
- Python: `cd ai-service && uv run pytest`
- Frontend: `cd dashboard && npm run build`
- Integration: Run all services (Rust:3000, Python:8000, React:5173)

**Verification Strategy** (from .agents/tasks/DEFECT_REGISTER.md and PHASE8_RELEASE_VERIFICATION.md):
- Each defect repair includes regression test
- Tests must use actual commands/APIs, not grep checks
- Run under WSL (not PowerShell) for /proc filesystem access
- Restart services between tests to verify persistence

---

## Implementation Plan

### FEAT-001: Fix Command History Persistence (DEF-001) — CRITICAL PATH

**Status**: 🔴 NOT STARTED  
**Type**: fix  
**Priority**: 1 (foundation for Feature 7)

**Problem**: execute_command() in server.rs never writes to database, leaving history empty despite schema existing.

**Root Cause**: Database insert/update calls never added when execution logic was implemented.

**Steps**:

- [ ] 1. Read database API implementation in `core-engine/src/server.rs` lines 99-186 to confirm insert_command and update_execution signatures.
      Files: `core-engine/src/server.rs`
      Verify: Understand return types (SqliteResult<i64> for insert, SqliteResult<()> for update)

- [ ] 2. Modify execute_command function (lines 311-356) to insert pending history record BEFORE execution.
      Implementation:
      - After request validation, before risk routing block
      - Create CommandHistoryRecord with: query="" (not available in ExecuteRequest, use command as fallback), command=req.command, risk_level=req.risk_level, execution_status="pending", created_at=chrono::Utc::now().timestamp(), all other fields None
      - Call state.db.insert_command(&record).await
      - Capture returned ID: `let history_id = state.db.insert_command(&record).await.map_err(|e| { warn!("Failed to insert history: {}", e); e })?;`
      - Handle error gracefully: log warning but continue execution (do not fail execution if DB write fails)
      Files: `core-engine/src/server.rs`
      Verify: Record appears in history with status="pending"

- [ ] 3. Update execute_direct function (lines 358-382) to write execution result to database.
      Implementation:
      - Accept history_id as parameter: `async fn execute_direct(command: &str, timeout_seconds: u64, db: &Database, history_id: i64)`
      - After successful execution, before returning response
      - Call `db.update_execution(history_id, "completed".to_string(), output.status.code(), Some(stdout), Some(stderr), Some(duration_ms as i64)).await`
      - Handle DB error: log warning, continue
      - On execution error: call update_execution with status="failed"
      Files: `core-engine/src/server.rs`
      Verify: History record updated with stdout, stderr, exit_code

- [ ] 4. Update execute_sandboxed function (lines 384-433) to write execution result.
      Implementation:
      - Accept history_id as parameter
      - After sandbox execution completes, call update_execution with sandbox_result fields
      - Map sandbox termination reason to status: Exited → "completed", others → "failed"
      Files: `core-engine/src/server.rs`
      Verify: Sandboxed execution writes to history

- [ ] 5. For HIGH/CRITICAL blocked commands, insert history record with status="blocked".
      Implementation:
      - In execute_command risk routing, "high" | "critical" branch
      - Create record with execution_status="blocked", no exit_code/outputs
      - Call insert_command (no update needed)
      Files: `core-engine/src/server.rs`
      Verify: Blocked commands appear in history

- [ ] 6. Update execute_command call sites to pass db and history_id to execute_direct/execute_sandboxed.
      Files: `core-engine/src/server.rs`
      Verify: Code compiles

**Acceptance Criteria**:
- Execute a LOW risk command → history record appears with status="completed", stdout, exit_code
- Execute a MEDIUM risk command → history record appears (sandboxed or degraded to direct)
- Execute a HIGH risk command → history record appears with status="blocked"
- Restart Rust server → history persists (SQLite)
- GET /history returns populated array

**Verification Commands**:
```bash
# In WSL
cd /mnt/d/3rd\ year/os\ project/IntelliShell/core-engine
cargo build
./target/debug/intellishell-server &
sleep 2

# Execute a command
curl -X POST http://localhost:3000/execute \
  -H "Content-Type: application/json" \
  -d '{"command":"echo test","risk_level":"low","timeout_seconds":10}'

# Check history
curl http://localhost:3000/history | jq '.[] | {command, execution_status, exit_code, stdout}'

# Expected: [{command: "echo test", execution_status: "completed", exit_code: 0, stdout: "test\n"}]
```

**Regression Test**: Add integration test `tests/history_persistence_test.sh` that executes command, queries history, verifies entry exists, restarts server, queries again, verifies persistence.

---

### FEAT-002: Implement Disk and Network Monitoring (DEF-002, DEF-003) — CRITICAL PATH

**Status**: 🔴 NOT STARTED  
**Type**: feat  
**Priority**: 2 (completes Feature 5)

**Problem**: Frontend expects disk_used_percent, network_rx_bytes, network_tx_bytes in SystemSnapshot but backend does not collect or return these fields.

**Root Cause**: Monitoring implementation incomplete — only CPU/memory implemented, disk/network never added.

**Steps**:

- [ ] 1. Add disk and network fields to SystemSnapshot struct in `core-engine/src/server.rs` lines 260-271.
      Implementation:
      ```rust
      #[derive(Debug, Serialize, Deserialize)]
      struct SystemSnapshot {
          cpu_percent: f64,
          memory_total_bytes: u64,
          memory_used_bytes: u64,
          memory_available_bytes: u64,
          memory_used_percent: f64,
          disk_total_bytes: u64,       // NEW
          disk_used_bytes: u64,        // NEW
          disk_used_percent: f64,      // NEW
          network_rx_bytes: u64,       // NEW
          network_tx_bytes: u64,       // NEW
          processes: Vec<ProcessInfo>,
          timestamp: i64,
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: Code compiles

- [ ] 2. Implement get_disk_usage() function using statvfs syscall.
      Implementation:
      ```rust
      fn get_disk_usage() -> (u64, u64, f64) {
          use std::ffi::CString;
          use std::mem;
          
          let path = CString::new("/").unwrap();
          let mut stat: libc::statvfs = unsafe { mem::zeroed() };
          
          let result = unsafe { libc::statvfs(path.as_ptr(), &mut stat) };
          
          if result == 0 {
              let total_bytes = stat.f_blocks * stat.f_frsize;
              let available_bytes = stat.f_bavail * stat.f_frsize;
              let used_bytes = total_bytes - available_bytes;
              let used_percent = if total_bytes > 0 {
                  (used_bytes as f64 / total_bytes as f64) * 100.0
              } else {
                  0.0
              };
              (total_bytes, used_bytes, used_percent)
          } else {
              (0, 0, 0.0)
          }
      }
      ```
      Add after get_cpu_usage function (around line 514)
      Files: `core-engine/src/server.rs`
      Verify: Function compiles, returns non-zero values

- [ ] 3. Implement get_network_stats() function reading /proc/net/dev.
      Implementation:
      ```rust
      fn get_network_stats() -> (u64, u64) {
          let contents = match std::fs::read_to_string("/proc/net/dev") {
              Ok(c) => c,
              Err(_) => return (0, 0),
          };
          
          let mut total_rx = 0u64;
          let mut total_tx = 0u64;
          
          for line in contents.lines().skip(2) {
              let parts: Vec<&str> = line.split_whitespace().collect();
              if parts.len() < 10 {
                  continue;
              }
              
              let interface = parts[0].trim_end_matches(':');
              if interface == "lo" {
                  continue; // Skip loopback
              }
              
              if let Ok(rx) = parts[1].parse::<u64>() {
                  total_rx += rx;
              }
              if let Ok(tx) = parts[9].parse::<u64>() {
                  total_tx += tx;
              }
          }
          
          (total_rx, total_tx)
      }
      ```
      Add after get_disk_usage function
      Files: `core-engine/src/server.rs`
      Verify: Function compiles, returns non-zero values on active network

- [ ] 4. Call get_disk_usage and get_network_stats in get_snapshot function (lines 273-309).
      Implementation:
      - After mem_used_percent calculation, before get_cpu_usage call
      - `let (disk_total_bytes, disk_used_bytes, disk_used_percent) = get_disk_usage();`
      - `let (network_rx_bytes, network_tx_bytes) = get_network_stats();`
      - Add fields to SystemSnapshot initialization
      Files: `core-engine/src/server.rs`
      Verify: Code compiles

- [ ] 5. Update SystemSnapshot return value in get_snapshot to include new fields.
      Files: `core-engine/src/server.rs`
      Verify: Code compiles, response JSON includes disk_used_percent, network_rx_bytes, network_tx_bytes

**Acceptance Criteria**:
- GET /snapshot returns disk_total_bytes, disk_used_bytes, disk_used_percent (all > 0)
- GET /snapshot returns network_rx_bytes, network_tx_bytes (>= 0)
- Disk percentage matches `df -h /` within ±5%
- Network bytes are cumulative since boot (consistent with /proc/net/dev)
- Frontend MonitoringPage displays disk and network metrics without errors

**Verification Commands**:
```bash
# In WSL
curl http://localhost:3000/snapshot | jq '{disk_used_percent, network_rx_bytes, network_tx_bytes}'

# Compare with system tools
df -h / | awk 'NR==2 {print "Disk: " $5}'
cat /proc/net/dev | grep -v lo | awk '{rx+=$2; tx+=$10} END {print "RX:", rx, "TX:", tx}'

# Expected: Values match within reasonable range
```

**Regression Test**: Add unit test in server.rs that calls get_disk_usage and get_network_stats, asserts disk_total > 0 and either network_rx > 0 or network_tx > 0 (allow 0 if no network activity).

---

### FEAT-003: Fix CPU Calculation Accuracy (DEF-004) — HIGH PRIORITY

**Status**: 🔴 NOT STARTED  
**Type**: fix  
**Priority**: 3

**Problem**: get_cpu_usage does single-snapshot calculation which is meaningless for CPU percentage. Needs delta between two samples.

**Root Cause**: CPU usage requires comparing two /proc/stat reads separated by time, but current implementation only reads once.

**Steps**:

- [ ] 1. Add previous CPU sample storage to AppState in `core-engine/src/server.rs` lines 273-277.
      Implementation:
      ```rust
      use std::sync::Mutex;
      
      #[derive(Clone)]
      struct AppState {
          db: Arc<Database>,
          start_time: std::time::Instant,
          previous_cpu: Arc<Mutex<Option<CpuSample>>>,  // NEW
      }
      
      #[derive(Clone)]
      struct CpuSample {
          user: u64,
          nice: u64,
          system: u64,
          idle: u64,
          total: u64,
          timestamp: std::time::Instant,
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: Code compiles

- [ ] 2. Update main() to initialize previous_cpu in AppState.
      Implementation:
      ```rust
      let state = AppState {
          db: Arc::new(db),
          start_time: std::time::Instant::now(),
          previous_cpu: Arc::new(Mutex::new(None)),
      };
      ```
      Files: `core-engine/src/server.rs` line ~589
      Verify: Code compiles

- [ ] 3. Rewrite get_cpu_usage to accept AppState and calculate delta.
      Implementation:
      ```rust
      fn get_cpu_usage(state: &AppState) -> f64 {
          let contents = match std::fs::read_to_string("/proc/stat") {
              Ok(c) => c,
              Err(_) => return 0.0,
          };
          
          let first_line = match contents.lines().next() {
              Some(line) => line,
              None => return 0.0,
          };
          
          let fields: Vec<u64> = first_line
              .split_whitespace()
              .skip(1)
              .filter_map(|f| f.parse::<u64>().ok())
              .collect();
          
          if fields.len() < 4 {
              return 0.0;
          }
          
          let user = fields[0];
          let nice = fields[1];
          let system = fields[2];
          let idle = fields[3];
          let total: u64 = fields.iter().sum();
          
          let current = CpuSample {
              user,
              nice,
              system,
              idle,
              total,
              timestamp: std::time::Instant::now(),
          };
          
          let mut prev_lock = state.previous_cpu.lock().unwrap();
          
          let cpu_percent = if let Some(ref prev) = *prev_lock {
              let total_delta = current.total.saturating_sub(prev.total);
              let idle_delta = current.idle.saturating_sub(prev.idle);
              
              if total_delta > 0 {
                  100.0 * (1.0 - (idle_delta as f64 / total_delta as f64))
              } else {
                  0.0
              }
          } else {
              0.0 // First call, no previous sample
          };
          
          *prev_lock = Some(current);
          
          cpu_percent
      }
      ```
      Files: `core-engine/src/server.rs` replace lines 493-514
      Verify: Code compiles

- [ ] 4. Update get_snapshot to pass state to get_cpu_usage.
      Implementation:
      - Change signature: `async fn get_snapshot(State(state): State<AppState>)`
      - Change call: `let cpu_percent = get_cpu_usage(&state);`
      Files: `core-engine/src/server.rs` line 273
      Verify: Code compiles

**Acceptance Criteria**:
- First query to /snapshot returns cpu_percent: 0.0 (no previous sample)
- Second query 1 second later returns reasonable cpu_percent (0-100)
- CPU percent changes between queries when load changes
- CPU percent matches `top` command within ±10% under load

**Verification Commands**:
```bash
# In WSL
curl http://localhost:3000/snapshot | jq '.cpu_percent'
# Expected: 0.0 (first call)

sleep 1
curl http://localhost:3000/snapshot | jq '.cpu_percent'
# Expected: non-zero value

# Generate load
yes > /dev/null &
YES_PID=$!
sleep 2
curl http://localhost:3000/snapshot | jq '.cpu_percent'
kill $YES_PID
# Expected: higher CPU percentage
```

**Regression Test**: Add integration test that queries /snapshot twice with 1s delay, verifies second call returns non-zero CPU percent.

---

### FEAT-004: Add Execute Button to Frontend (DEF-005) — CRITICAL PATH

**Status**: 🔴 NOT STARTED  
**Type**: feat  
**Priority**: 4 (completes end-to-end workflow)

**Problem**: AssistantPage has command generation UI but no Execute button to call /confirm-execute endpoint, blocking end-to-end workflow.

**Root Cause**: UI not connected to existing confirmExecute API.

**Steps**:

- [ ] 1. Add execution state to AssistantPage component in `dashboard/src/pages/AssistantPage.jsx` line 14.
      Implementation:
      ```javascript
      const [executionResult, setExecutionResult] = useState(null);
      ```
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Code compiles

- [ ] 2. Create confirmExecute mutation after generateMutation (line 25).
      Implementation:
      ```javascript
      const confirmExecuteMutation = useMutation({
        mutationFn: (token) => pythonApi.confirmExecute(token),
        onSuccess: (data) => {
          setExecutionResult(data);
        },
      });
      ```
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Code compiles

- [ ] 3. Add handleExecute function after handleCopy (line 34).
      Implementation:
      ```javascript
      const handleExecute = () => {
        if (!generated?.confirmation_token) return;
        setExecutionResult(null);
        confirmExecuteMutation.mutate(generated.confirmation_token);
      };
      ```
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Code compiles

- [ ] 4. Clear execution result when generating new command.
      Implementation:
      - In generateMutation onSuccess, add: `setExecutionResult(null);`
      Files: `dashboard/src/pages/AssistantPage.jsx` line 21
      Verify: Previous execution result clears on new generation

- [ ] 5. Add Execute button after the command display section (after line 122).
      Implementation:
      ```jsx
      {/* Execute Button */}
      {generated.confirmation_token && !executionResult && (
        <div className="mt-4">
          <button
            onClick={handleExecute}
            disabled={confirmExecuteMutation.isPending}
            className="flex items-center gap-2 px-6 py-2.5 rounded-md font-medium transition-colors disabled:opacity-50"
            style={{
              backgroundColor: 'var(--brown-deep)',
              color: 'var(--cream-warm)',
            }}
          >
            {confirmExecuteMutation.isPending ? (
              <>
                <LoadingSpinner size="sm" />
                <span>Executing...</span>
              </>
            ) : (
              <>
                <CheckCircle className="w-4 h-4" />
                <span>Execute Command</span>
              </>
            )}
          </button>
        </div>
      )}
      ```
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Button appears after command is generated

- [ ] 6. Add execution result display after Execute button.
      Implementation:
      ```jsx
      {/* Execution Result */}
      {executionResult && (
        <div className="mt-4 border rounded-lg p-4" style={{ borderColor: 'var(--color-border)' }}>
          <div className="flex items-center justify-between mb-3">
            <h5 className="font-heading font-semibold" style={{ color: 'var(--ink-dark)' }}>
              Execution Result
            </h5>
            <StatusBadge type={executionResult.executed ? 'success' : 'error'}>
              {executionResult.executed ? executionResult.method : 'blocked'}
            </StatusBadge>
          </div>
          
          {executionResult.message && (
            <p className="text-sm mb-3" style={{ color: 'var(--color-text-muted)' }}>
              {executionResult.message}
            </p>
          )}
          
          {executionResult.exit_code !== null && (
            <p className="text-sm mb-2">
              <strong>Exit Code:</strong> {executionResult.exit_code}
            </p>
          )}
          
          {executionResult.stdout && (
            <div className="mb-3">
              <p className="text-sm font-medium mb-1">Output:</p>
              <pre className="bg-subtle p-3 rounded text-sm overflow-x-auto" style={{ backgroundColor: 'var(--color-bg-subtle)' }}>
                {executionResult.stdout}
              </pre>
            </div>
          )}
          
          {executionResult.stderr && (
            <div>
              <p className="text-sm font-medium mb-1" style={{ color: 'var(--color-error)' }}>
                Error Output:
              </p>
              <pre className="bg-subtle p-3 rounded text-sm overflow-x-auto" style={{ backgroundColor: 'var(--color-error-bg)', color: 'var(--color-error)' }}>
                {executionResult.stderr}
              </pre>
            </div>
          )}
          
          {executionResult.duration_ms && (
            <p className="text-xs mt-3" style={{ color: 'var(--color-text-muted)' }}>
              Execution time: {executionResult.duration_ms}ms
            </p>
          )}
        </div>
      )}
      ```
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Result displays after execution

- [ ] 7. Add error display for execution failures.
      Implementation:
      ```jsx
      {/* Execution Error */}
      {confirmExecuteMutation.isError && (
        <ErrorAlert
          title="Execution Failed"
          message={confirmExecuteMutation.error?.message || 'Could not execute command'}
          onRetry={() => confirmExecuteMutation.mutate(generated.confirmation_token)}
        />
      )}
      ```
      Add after execution result section
      Files: `dashboard/src/pages/AssistantPage.jsx`
      Verify: Error shows when execution fails

**Acceptance Criteria**:
- Generate command → Execute button appears
- Click Execute → loading spinner shows, result appears
- LOW risk command → shows stdout, exit_code, method: "direct"
- HIGH risk command → shows method: "blocked", message explains why
- Token expiry → error message "Invalid or expired token"
- Execute again on same command → error (token already used)

**Verification Commands**:
```bash
# Manual UI test (run all three services):
# 1. Start Rust server: cd core-engine && cargo run
# 2. Start Python service: cd ai-service && uv run uvicorn main:app --host 127.0.0.1 --port 8000
# 3. Start frontend: cd dashboard && npm run dev
# 4. Open http://localhost:5173/assistant
# 5. Enter "echo hello world"
# 6. Click Generate
# 7. Click Execute
# 8. Verify result shows stdout: "hello world\n", exit_code: 0
```

**Regression Test**: Add Playwright or manual test checklist for end-to-end workflow.

---

### FEAT-005: Add Rust Unit Tests for Execution Routing (DEF-007) — HIGH PRIORITY

**Status**: 🔴 NOT STARTED  
**Type**: test  
**Priority**: 5

**Problem**: Zero automated tests for execute_command risk-based routing logic, despite being security-critical.

**Root Cause**: Tests not written.

**Steps**:

- [ ] 1. Create `#[cfg(test)] mod tests` module at end of `core-engine/src/server.rs`.
      Implementation:
      ```rust
      #[cfg(test)]
      mod tests {
          use super::*;
          
          // Tests go here
      }
      ```
      Files: `core-engine/src/server.rs` (append at end)
      Verify: `cargo test` runs (0 tests)

- [ ] 2. Add test for HIGH risk blocking.
      Implementation:
      ```rust
      #[tokio::test]
      async fn test_high_risk_blocks() {
          let db = Database::new(PathBuf::from(":memory:")).unwrap();
          let state = AppState {
              db: Arc::new(db),
              start_time: std::time::Instant::now(),
              previous_cpu: Arc::new(Mutex::new(None)),
          };
          
          let req = ExecuteRequest {
              command: "rm -rf /".to_string(),
              risk_level: "high".to_string(),
              timeout_seconds: 10,
          };
          
          let result = execute_command(State(state), Json(req)).await;
          assert!(result.is_ok());
          
          let response = result.unwrap().0;
          assert_eq!(response.executed, false);
          assert_eq!(response.method, "blocked");
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: `cargo test test_high_risk_blocks` passes

- [ ] 3. Add test for LOW risk direct execution.
      Implementation:
      ```rust
      #[tokio::test]
      async fn test_low_risk_executes_directly() {
          let db = Database::new(PathBuf::from(":memory:")).unwrap();
          let state = AppState {
              db: Arc::new(db),
              start_time: std::time::Instant::now(),
              previous_cpu: Arc::new(Mutex::new(None)),
          };
          
          let req = ExecuteRequest {
              command: "echo test".to_string(),
              risk_level: "low".to_string(),
              timeout_seconds: 10,
          };
          
          let result = execute_command(State(state), Json(req)).await;
          assert!(result.is_ok());
          
          let response = result.unwrap().0;
          assert_eq!(response.executed, true);
          assert_eq!(response.method, "direct");
          assert!(response.stdout.contains("test"));
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: `cargo test test_low_risk_executes_directly` passes

- [ ] 4. Add test for MEDIUM risk sandboxed execution (or graceful degradation).
      Implementation:
      ```rust
      #[tokio::test]
      async fn test_medium_risk_routes_to_sandbox() {
          let db = Database::new(PathBuf::from(":memory:")).unwrap();
          let state = AppState {
              db: Arc::new(db),
              start_time: std::time::Instant::now(),
              previous_cpu: Arc::new(Mutex::new(None)),
          };
          
          let req = ExecuteRequest {
              command: "echo test".to_string(),
              risk_level: "medium".to_string(),
              timeout_seconds: 10,
          };
          
          let result = execute_command(State(state), Json(req)).await;
          
          // Sandbox may fail if not root, check that it attempted or degraded gracefully
          if let Ok(response) = result {
              assert!(response.0.method == "sandboxed" || response.0.method == "direct");
          } else {
              // Expected failure if no root
              let err_msg = format!("{:?}", result);
              assert!(err_msg.contains("sandbox") || err_msg.contains("root"));
          }
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: `cargo test test_medium_risk_routes_to_sandbox` passes or degrades gracefully

- [ ] 5. Add test for empty command rejection.
      Implementation:
      ```rust
      #[tokio::test]
      async fn test_empty_command_rejected() {
          let db = Database::new(PathBuf::from(":memory:")).unwrap();
          let state = AppState {
              db: Arc::new(db),
              start_time: std::time::Instant::now(),
              previous_cpu: Arc::new(Mutex::new(None)),
          };
          
          let req = ExecuteRequest {
              command: "".to_string(),
              risk_level: "low".to_string(),
              timeout_seconds: 10,
          };
          
          let result = execute_command(State(state), Json(req)).await;
          assert!(result.is_err());
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: `cargo test test_empty_command_rejected` passes

- [ ] 6. Add test for invalid risk level.
      Implementation:
      ```rust
      #[tokio::test]
      async fn test_invalid_risk_level_rejected() {
          let db = Database::new(PathBuf::from(":memory:")).unwrap();
          let state = AppState {
              db: Arc::new(db),
              start_time: std::time::Instant::now(),
              previous_cpu: Arc::new(Mutex::new(None)),
          };
          
          let req = ExecuteRequest {
              command: "echo test".to_string(),
              risk_level: "invalid".to_string(),
              timeout_seconds: 10,
          };
          
          let result = execute_command(State(state), Json(req)).await;
          assert!(result.is_err());
      }
      ```
      Files: `core-engine/src/server.rs`
      Verify: `cargo test test_invalid_risk_level_rejected` passes

**Acceptance Criteria**:
- `cargo test` runs all 6 new tests
- All tests pass in WSL environment
- Tests cover: HIGH block, LOW direct, MEDIUM sandbox/degrade, empty reject, invalid reject
- Tests use in-memory SQLite database (no file pollution)

**Verification Commands**:
```bash
# In WSL
cd /mnt/d/3rd\ year/os\ project/IntelliShell/core-engine
cargo test -- --test-threads=1

# Expected: 6+ tests pass (existing integration tests + new unit tests)
```

**Regression Test**: The tests themselves are the regression test.

---

### FEAT-006: Add Python Tests for Risk Classifier (DEF-008) — HIGH PRIORITY

**Status**: 🔴 NOT STARTED  
**Type**: test  
**Priority**: 6

**Problem**: Existing test_classify.py uses wrong API (classify_command returns Tuple[RiskLevel, str] not dict), tests fail.

**Root Cause**: Tests written against outdated API signature.

**Steps**:

- [ ] 1. Fix test_classify.py imports and API usage in `ai-service/tests/test_classify.py`.
      Implementation:
      - Import: `from intellishell_ai_service.risk import classify_command, suggest_safer_alternative, RiskLevel`
      - Update all classify_command calls: `risk_level, reason = classify_command("command")`
      - Update assertions: `assert risk_level == RiskLevel.CRITICAL` (not `result["risk_level"] == "critical"`)
      - Update reason checks: `assert "root directory" in reason.lower()`
      Files: `ai-service/tests/test_classify.py`
      Verify: Code is syntactically correct

- [ ] 2. Rewrite all 15 existing tests to use correct API.
      Implementation: Replace all test functions with correct assertions following the pattern:
      ```python
      def test_classify_rm_rf_root():
          risk_level, reason = classify_command("rm -rf /")
          assert risk_level == RiskLevel.CRITICAL
          assert "root directory" in reason.lower()
      ```
      Files: `ai-service/tests/test_classify.py`
      Verify: Tests follow correct pattern

- [ ] 3. Fix suggest_safer_alternative tests (returns str not dict).
      Implementation:
      ```python
      def test_suggest_safer_alternative_rm():
          suggestion = suggest_safer_alternative("rm -rf /tmp/test")
          assert len(suggestion) > 0
          assert "interactive" in suggestion.lower() or "trash" in suggestion.lower()
      ```
      Files: `ai-service/tests/test_classify.py`
      Verify: Tests use correct API

- [ ] 4. Add missing CRITICAL risk pattern tests.
      Implementation:
      ```python
      def test_classify_mkfs():
          risk_level, reason = classify_command("mkfs.ext4 /dev/sdb1")
          assert risk_level == RiskLevel.CRITICAL
          assert "filesystem format" in reason.lower() or "format" in reason.lower()
      
      def test_classify_fdisk():
          risk_level, reason = classify_command("fdisk /dev/sda")
          assert risk_level == RiskLevel.CRITICAL
          assert "partition" in reason.lower()
      
      def test_classify_wget_pipe_sh():
          risk_level, reason = classify_command("wget -O - http://example.com/install.sh | sh")
          assert risk_level == RiskLevel.CRITICAL
          assert "piping" in reason.lower() or "remote" in reason.lower()
      ```
      Files: `ai-service/tests/test_classify.py`
      Verify: Tests added

- [ ] 5. Add edge case tests (escaped patterns, quoted commands).
      Implementation:
      ```python
      def test_classify_quoted_rm():
          risk_level, reason = classify_command("rm -rf '/tmp/test'")
          assert risk_level in [RiskLevel.HIGH, RiskLevel.MEDIUM]
      
      def test_classify_safe_commands():
          safe_commands = ["ls -la", "cat file.txt", "grep pattern file", "ps aux", "df -h"]
          for cmd in safe_commands:
              risk_level, reason = classify_command(cmd)
              assert risk_level == RiskLevel.SAFE
      ```
      Files: `ai-service/tests/test_classify.py`
      Verify: Tests added

**Acceptance Criteria**:
- `uv run pytest ai-service/tests/test_classify.py` passes all tests
- 20+ risk classification tests pass
- Tests cover: CRITICAL (10 patterns), HIGH (3), MEDIUM (5), LOW (2), SAFE (multiple)
- Tests cover edge cases: quoted paths, multiple safe commands

**Verification Commands**:
```bash
# In WSL
cd /mnt/d/3rd\ year/os\ project/IntelliShell/ai-service
uv run pytest tests/test_classify.py -v

# Expected: 20+ tests pass
```

**Regression Test**: The tests themselves are the regression test.

---

### FEAT-007: End-to-End Integration Verification — VALIDATION

**Status**: 🔴 NOT STARTED  
**Type**: test  
**Priority**: 7 (final validation)

**Problem**: Need comprehensive end-to-end test confirming all 7 features work together.

**Steps**:

- [ ] 1. Create integration test script `tests/end_to_end_verification.sh`.
      Implementation:
      ```bash
      #!/bin/bash
      set -e
      
      echo "=== IntelliShell End-to-End Integration Test ==="
      
      RUST_URL="http://localhost:3000"
      PYTHON_URL="http://localhost:8000"
      
      # Feature 1 & 3: Generate + Confirm Execute
      echo "Testing Feature 1 & 3: Natural language generation + Execution"
      TOKEN=$(curl -s -X POST $PYTHON_URL/generate-command \
        -H "Content-Type: application/json" \
        -d '{"query":"list files in current directory"}' | jq -r '.confirmation_token')
      
      if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
        echo "FAIL: No confirmation token generated"
        exit 1
      fi
      echo "✓ Token generated: $TOKEN"
      
      EXEC_RESULT=$(curl -s -X POST $PYTHON_URL/confirm-execute \
        -H "Content-Type: application/json" \
        -d "{\"token\":\"$TOKEN\"}")
      
      EXECUTED=$(echo $EXEC_RESULT | jq -r '.executed')
      if [ "$EXECUTED" != "true" ]; then
        echo "FAIL: Command not executed"
        echo $EXEC_RESULT
        exit 1
      fi
      echo "✓ Command executed successfully"
      
      # Feature 7: History persistence
      echo "Testing Feature 7: Command history"
      sleep 1
      HISTORY=$(curl -s $RUST_URL/history)
      HISTORY_COUNT=$(echo $HISTORY | jq 'length')
      if [ "$HISTORY_COUNT" -eq 0 ]; then
        echo "FAIL: History is empty"
        exit 1
      fi
      echo "✓ History contains $HISTORY_COUNT entries"
      
      # Feature 5: System monitoring
      echo "Testing Feature 5: System monitoring"
      SNAPSHOT=$(curl -s $RUST_URL/snapshot)
      
      CPU=$(echo $SNAPSHOT | jq -r '.cpu_percent')
      DISK=$(echo $SNAPSHOT | jq -r '.disk_used_percent')
      NET_RX=$(echo $SNAPSHOT | jq -r '.network_rx_bytes')
      
      if [ "$DISK" == "null" ]; then
        echo "FAIL: Disk monitoring missing"
        exit 1
      fi
      echo "✓ Disk monitoring: ${DISK}%"
      
      if [ "$NET_RX" == "null" ]; then
        echo "FAIL: Network monitoring missing"
        exit 1
      fi
      echo "✓ Network monitoring: RX ${NET_RX} bytes"
      
      # Feature 4: Process management
      echo "Testing Feature 4: Process management"
      PROCESSES=$(curl -s $RUST_URL/processes)
      PROC_COUNT=$(echo $PROCESSES | jq 'length')
      if [ "$PROC_COUNT" -eq 0 ]; then
        echo "FAIL: No processes returned"
        exit 1
      fi
      echo "✓ Process list contains $PROC_COUNT processes"
      
      # Feature 6: Troubleshooting
      echo "Testing Feature 6: AI troubleshooting"
      TROUBLE=$(curl -s -X POST $PYTHON_URL/troubleshoot \
        -H "Content-Type: application/json" \
        -d '{"issue":"System is slow"}')
      
      ANALYSIS=$(echo $TROUBLE | jq -r '.analysis')
      if [ "$ANALYSIS" == "null" ] || [ -z "$ANALYSIS" ]; then
        echo "FAIL: No troubleshooting analysis"
        exit 1
      fi
      echo "✓ Troubleshooting returned analysis"
      
      echo ""
      echo "=== All Integration Tests PASSED ==="
      ```
      Files: `tests/end_to_end_verification.sh`
      Verify: Script is executable

- [ ] 2. Make script executable and add to test suite.
      Implementation:
      ```bash
      chmod +x tests/end_to_end_verification.sh
      ```
      Files: `tests/end_to_end_verification.sh`
      Verify: Script can run

- [ ] 3. Run integration test with all services running.
      Prerequisites:
      - Terminal 1: `cd core-engine && cargo run`
      - Terminal 2: `cd ai-service && uv run uvicorn main:app --host 127.0.0.1 --port 8000`
      - Terminal 3: `cd tests && ./end_to_end_verification.sh`
      Files: N/A
      Verify: All checks pass

**Acceptance Criteria**:
- Script completes without errors
- All 7 features verified: generation (1), explanation (2), confirmation (3), process mgmt (4), monitoring (5), troubleshooting (6), history (7)
- History persists after execution
- Monitoring returns disk + network metrics
- Troubleshooting returns analysis

**Verification Commands**:
```bash
# In WSL, with services running
cd /mnt/d/3rd\ year/os\ project/IntelliShell
./tests/end_to_end_verification.sh

# Expected: "All Integration Tests PASSED"
```

**Regression Test**: This IS the regression test for end-to-end functionality.

---

## Success Criteria

**All defects repaired**:
- ✅ DEF-001: Commands appear in history with correct status/outputs
- ✅ DEF-002: Disk metrics present in /snapshot
- ✅ DEF-003: Network metrics present in /snapshot
- ✅ DEF-004: CPU calculation uses delta (accurate)
- ✅ DEF-005: Execute button functional in UI

**All tests pass**:
- ✅ Rust: `cargo test` (6+ new unit tests + 2 existing integration tests)
- ✅ Python: `uv run pytest` (20+ risk classifier tests)
- ✅ Integration: `./tests/end_to_end_verification.sh` (7 feature checks)

**End-to-end workflow functional**:
- ✅ User enters natural language query
- ✅ AI generates command + explanation + risk + token
- ✅ User clicks Execute
- ✅ Backend routes by risk (block/sandbox/direct)
- ✅ Execution result displays (stdout, stderr, exit_code)
- ✅ History contains executed command
- ✅ Monitoring shows CPU/RAM/Disk/Network
- ✅ Process management works
- ✅ Troubleshooting analyzes telemetry

---

## Testing Strategy

**Environment Requirements**:
- Must run in WSL2/Ubuntu (not Windows PowerShell)
- Rust tests require /proc filesystem access
- Python tests can run anywhere (no /proc dependency)
- Integration tests require all three services running
- No admin/root required for basic tests (sandbox tests degrade gracefully)

**Test Execution Order**:
1. Python tests first (fastest, no dependencies)
2. Rust unit tests (medium speed, database in-memory)
3. Start services for integration tests
4. Run end-to-end verification
5. Stop services

**Continuous Integration**:
- Add test command to Makefile: `make test-all`
- Run in WSL environment
- Report coverage

**Manual Verification**:
- Start all services
- Open http://localhost:5173/assistant
- Complete full user workflow
- Check monitoring page
- Check history page

---

## Rollback Plan

If any defect repair causes regressions:
1. Revert the specific file/function to git HEAD
2. Mark defect as blocked in DEFECT_REGISTER.md
3. Continue with remaining defects
4. Re-attempt blocked defect with different approach

---

## Completion Checklist

- [ ] All 5 critical defects marked 🟢 REPAIRED in DEFECT_REGISTER.md
- [ ] All Rust tests pass: `cargo test`
- [ ] All Python tests pass: `uv run pytest`
- [ ] Integration test passes: `./tests/end_to_end_verification.sh`
- [ ] Frontend builds without errors: `cd dashboard && npm run build`
- [ ] Manual UI workflow tested end-to-end
- [ ] History persists across server restarts (verified)
- [ ] Monitoring displays all 4 metrics (CPU, RAM, disk, network)
- [ ] No new security vulnerabilities introduced
- [ ] Documentation updated with test instructions

---

**Plan Status**: READY FOR IMPLEMENTATION  
**Estimated Total Effort**: 15-20 hours focused work  
**Critical Path**: FEAT-001 → FEAT-002 → FEAT-004 → FEAT-007  
**Can parallelize**: FEAT-003, FEAT-005, FEAT-006 (no dependencies on each other)

