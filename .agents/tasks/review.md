# Critical Defects Repair Review

This review examines the repairs to five critical defects in IntelliShell: command history persistence (DEF-001), disk monitoring (DEF-002), network monitoring (DEF-003), CPU calculation (DEF-004), and the frontend execute button (DEF-005), plus the addition of comprehensive Rust and Python tests (DEF-007, DEF-008).

The repairs target command execution that previously failed to record history, system monitoring that returned incomplete telemetry, CPU metrics that used a fundamentally incorrect calculation method, and a frontend that could generate commands but not execute them.

**Watch for:** Database connection architecture that won't scale; hardcoded root filesystem path for disk monitoring; missing validation for PID 1 protection in process control; execution button UI behavior that may confuse users when tokens expire.

**Verdict**: APPROVED

---

## High-level view

Command history now persists to SQLite before execution begins (status="pending") and updates with outcomes afterward, with DB failures logged but not blocking execution. Disk metrics use statvfs on the root filesystem, returning total/used/percent as u64 fields in the snapshot. Network metrics aggregate RX/TX bytes from `/proc/net/dev` across all interfaces except loopback. CPU calculation stores a previous sample in application state and computes usage from the delta between idle and total jiffies, returning 0.0 on the first call. The frontend execute button appears after command generation, calls the confirm-execute endpoint with the token, and displays stdout/stderr/exit-code/method or errors. Eleven Rust unit tests cover execution routing (HIGH/CRITICAL block, LOW direct, MEDIUM sandbox-or-degrade, empty rejection, invalid risk rejection) and metric collection (disk, network, CPU delta, history persistence). Twenty-eight Python tests verify risk classification across all levels (CRITICAL, HIGH, MEDIUM, LOW, SAFE) with edge cases.

---

<details>
<summary>Issues (4)</summary>

1. **Database connection per operation** — Every `with_conn` call opens a new SQLite connection, which breaks :memory: databases (each connection sees an empty schema) and wastes file handles for file-based databases. Refactor to a connection pool or keep a single connection alive in AppState via `Arc<Mutex<Connection>>`.

2. **Hardcoded root filesystem for disk monitoring** — `get_disk_usage()` always calls `statvfs("/")`, which reports the root partition. If the user cares about a different mount (e.g., `/home` on a separate partition), this metric is irrelevant. Consider reading `/proc/mounts` to find the main data partition or making the path configurable.

3. **Protected PID validation missing for PID 1** — `control_process` checks `pid == 1` before sending signals, but the test `test_classify_kill_init` in Python verifies that `kill -9 1` is classified as CRITICAL. The Rust server correctly blocks PID 1, but there is no Rust unit test confirming this protection works. Add a test that attempts `control_process` on PID 1 and verifies rejection.

4. **Execute button behavior when token expires** — The UI shows the Execute button whenever `generated.confirmation_token` exists and `!executionResult`. If the token expires on the backend, clicking Execute triggers a mutation error. The user sees "Execution Failed" with a retry button, but retrying sends the same expired token. The UI should either hide the Execute button after some time (requires tracking token creation timestamp) or handle 401/403 errors by regenerating the command instead of offering a useless retry.

</details>

---

<details>
<summary>Details</summary>

## History persistence wired into execution flow

The `execute_command` handler creates a `CommandHistoryRecord` with `status="pending"` before routing to direct/sandboxed/blocked paths, inserts it via `db.insert_command()`, and captures the returned `history_id`. The `execute_direct` and `execute_sandboxed` functions accept `history_id` and `db` parameters and call `db.update_execution()` after running the command. Blocked commands (HIGH/CRITICAL risk) call `update_execution` with `status="blocked"` immediately. Database errors are logged with `warn!` but do not fail execution.

The database connection architecture opens a new SQLite connection for every operation inside `with_conn`, which spawns a blocking task that opens the file, runs the closure, and closes the connection. For `:memory:` databases, each connection sees an empty schema (the in-memory database is per-connection), which is why the tests switched to temporary files. For file-based databases, this wastes file descriptors and prevents connection reuse. A connection pool or a single `Arc<Mutex<Connection>>` in AppState would be more appropriate.

## Disk monitoring via statvfs

The `SystemSnapshot` struct gains three fields: `disk_total_bytes: u64`, `disk_used_bytes: u64`, `disk_used_percent: f64`. The `get_disk_usage()` function calls `libc::statvfs` on `CString::new("/")`, extracts `f_blocks`, `f_frsize`, and `f_bavail`, computes totals and percentage, and returns a tuple. The hardcoded "/" path means the function always reports the root partition, which may not be the partition the user cares about (e.g., if /home is a separate mount).

## Network monitoring via /proc/net/dev

The `SystemSnapshot` struct gains two fields: `network_rx_bytes: u64`, `network_tx_bytes: u64`. The `get_network_stats()` function reads `/proc/net/dev`, parses column 1 (RX bytes) and column 9 (TX bytes) from each line, skips loopback ("lo"), and accumulates the totals.

## CPU usage delta calculation

A new `CpuSample` struct holds `user`, `nice`, `system`, `idle`, `total` (all u64) and `timestamp` (Instant). AppState gains `previous_cpu: Arc<Mutex<Option<CpuSample>>>`. The `get_cpu_usage` function reads `/proc/stat`, constructs a `CpuSample` with the current values, locks `previous_cpu`, and if a previous sample exists, computes `total_delta = current.total - prev.total` and `idle_delta = current.idle - prev.idle`, then returns `100.0 * (1.0 - idle_delta / total_delta)`. If no previous sample exists (first call), it returns 0.0. The current sample is stored for the next call.

## Frontend execute button and result display

AssistantPage.jsx adds `executionResult` state. A new `confirmExecuteMutation` uses `pythonApi.confirmExecute(token)` and on success sets `executionResult` to the response data. The `handleExecute` function calls `confirmExecuteMutation.mutate(generated.confirmation_token)`.

The UI renders an "Execute Command" button when `generated.confirmation_token` exists and `!executionResult`. The button is disabled during `confirmExecuteMutation.isPending`. If `executionResult` exists, a bordered section displays the result: a status badge, the message, exit_code, stdout (in a `<pre>`), stderr (in a red `<pre>`), and duration_ms.

The retry button in ErrorAlert calls `confirmExecuteMutation.mutate(generated.confirmation_token)` again, which sends the same token. If the token expired on the backend, the retry will fail with the same error. A better UX would handle token expiry by either hiding the Execute button after a timeout or catching 401/403 errors and prompting the user to regenerate the command.

## Execution routing tests

Eleven unit tests in `core-engine/src/server.rs` use a helper `create_test_state()` that generates a unique temporary file path (using an atomic counter and the process ID) and creates a Database from it. Each test constructs an ExecuteRequest, calls `execute_command(State(state), Json(req))`, and asserts properties of the response.

`test_high_risk_blocks` and `test_critical_risk_blocks` verify that HIGH and CRITICAL risk levels return `executed: false` and `method: "blocked"`. `test_low_risk_executes_directly` verifies LOW risk executes and stdout contains "test". `test_medium_risk_routes_to_sandbox_or_degrades` verifies MEDIUM risk either sandboxes or degrades to direct (no panic). `test_empty_command_rejected` verifies empty command returns `Err(ApiError::InvalidRequest)`. `test_invalid_risk_level_rejected` verifies an invalid risk level returns the same error. `test_history_persistence_low_risk` executes "echo history_test" at LOW risk and queries the database afterward, asserting the entry exists with `execution_status="completed"` and stdout contains "history_test". `test_history_persistence_blocked` does the same for a blocked HIGH risk command. `test_get_disk_usage`, `test_get_network_stats`, and `test_cpu_usage_delta` are non-async tests that call the metric functions directly.

**confirmed**: TEST_EXECUTION_REPORT.md shows 25 tests passing in server.rs, including all 11 new tests. The tests cover the required paths. The use of temporary file-based databases works around the `:memory:` connection issue.

## Risk classifier tests

Twenty-eight tests in `ai-service/tests/test_risk.py` call `classify_command(cmd)` and assert the returned `(RiskLevel, reason)`. Tests cover CRITICAL patterns (rm -rf /, rm -rf ~, dd to /dev/sda, fork bomb, chmod -R 777 /, kill -9 1, shutdown, curl|bash, mkfs), HIGH patterns (rm -r, chown /, chmod 000, mv to /dev/null, disk writes), MEDIUM patterns (kill -9, chmod 777, sudo, systemctl, package removal), LOW patterns (rm file, kill, mv), and SAFE patterns (ls, cat, grep, empty).

**confirmed**: TEST_EXECUTION_REPORT.md shows all 28 tests passing. The tests verify both the RiskLevel enum and that the reason string contains expected keywords. A bug was fixed during testing: `rm -r /tmp/test` was incorrectly classified as CRITICAL; the pattern was made more specific to only match `rm -rf /` (root directory).

The test `test_classify_kill_init` verifies that `kill -9 1` is classified as CRITICAL, but there is no corresponding Rust unit test verifying that `control_process` rejects PID 1. The code has a check `if pid == 1 || pid == std::process::id()`, but no test exercises it.

</details>

---

<details>
<summary>File map</summary>

- **core-engine/src/server.rs**: Added history insertion/update calls in execute_command, execute_direct, execute_sandboxed; added SystemSnapshot fields for disk/network; implemented get_disk_usage (statvfs on "/"), get_network_stats (/proc/net/dev), get_cpu_usage (delta calculation); added CpuSample struct and previous_cpu to AppState; added 11 unit tests (execution routing, history persistence, metrics)
- **dashboard/src/pages/AssistantPage.jsx**: Added executionResult state, confirmExecuteMutation, handleExecute function, Execute button (conditional render), execution result display (status badge, exit_code, stdout, stderr, duration), error handling with retry
- **ai-service/tests/test_risk.py**: Added 28 tests covering all risk levels (CRITICAL, HIGH, MEDIUM, LOW, SAFE) with edge cases
- **ai-service/src/intellishell_ai_service/risk.py**: Fixed pattern matching bug (rm -r vs rm -rf /)
- **rust-toolchain.toml**: Updated to 1.88.0 (was 1.87.0)
- **Cargo.toml**: Updated rust-version to 1.88 (was 1.75)
- **ai-service/pyproject.toml**: Adjusted requires-python to >=3.10 (was >=3.13)
- **.agents/tasks/DEFECT_REGISTER.md**: Updated status to 🟢 REPAIRED for DEF-001 through DEF-008
- **.agents/tasks/REPAIR_VERIFICATION.md**: Implementation and verification summary
- **.agents/tasks/TEST_EXECUTION_REPORT.md**: Test execution evidence showing 41 Rust tests passing, 28 Python tests passing

Full diff: 192 files changed, 35469 insertions, 2985 deletions (includes documentation, configuration, and unrelated phase work)

</details>
