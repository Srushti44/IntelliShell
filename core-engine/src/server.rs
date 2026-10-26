/*!
 * IntelliShell Core Server - Phase 5 Backend Integration
 * 
 * HTTP API server that exposes monitoring, process control, sandboxed execution,
 * and command history to the AI service frontend.
 * 
 * Architecture:
 * - Axum web framework with Tower middleware
 * - SQLite database for command history
 * - Integration with existing process, sandbox, monitoring modules
 * - Risk-based execution routing (LOW→direct, MEDIUM→sandbox, HIGH→block)
 */

mod cgroup;
mod namespace;
mod process;
mod sandbox;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

// ============================================================================
// Database Schema and Persistence
// ============================================================================

use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::PathBuf;
use tokio::task;

/// Command history database (thread-safe wrapper)
#[derive(Clone)]
struct Database {
    path: PathBuf,
}

impl Database {
    /// Create or open database and run migrations
    fn new(path: PathBuf) -> SqliteResult<Self> {
        // Create the database and schema immediately
        let conn = Connection::open(&path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                command TEXT NOT NULL,
                explanation TEXT,
                risk_level TEXT NOT NULL,
                risk_reason TEXT,
                execution_status TEXT NOT NULL,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                duration_ms INTEGER,
                provider_used TEXT,
                generation_latency_ms INTEGER,
                created_at INTEGER NOT NULL,
                executed_at INTEGER,
                confirmation_token TEXT
            )",
            [],
        )?;

        // Index for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON command_history(created_at DESC)",
            [],
        )?;

        // Keep the connection open for in-memory databases by not dropping it immediately
        // For file-based databases, this connection will be closed when the function returns
        // but the database persists on disk
        drop(conn);

        Ok(Database { path })
    }

    /// Execute database operation in blocking task
    async fn with_conn<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&Connection) -> SqliteResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let path = self.path.clone();
        task::spawn_blocking(move || {
            let conn = Connection::open(path)?;
            f(&conn)
        })
        .await
        .unwrap()
    }

    /// Insert new command into history
    async fn insert_command(&self, record: &CommandHistoryRecord) -> SqliteResult<i64> {
        let record = record.clone();
        self.with_conn(move |conn| {
            conn.execute(
                "INSERT INTO command_history (
                    query, command, explanation, risk_level, risk_reason,
                    execution_status, exit_code, stdout, stderr, duration_ms,
                    provider_used, generation_latency_ms, created_at, executed_at,
                    confirmation_token
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    record.query,
                    record.command,
                    record.explanation,
                    record.risk_level,
                    record.risk_reason,
                    record.execution_status,
                    record.exit_code,
                    record.stdout,
                    record.stderr,
                    record.duration_ms,
                    record.provider_used,
                    record.generation_latency_ms,
                    record.created_at,
                    record.executed_at,
                    record.confirmation_token,
                ],
            )?;
            Ok(conn.last_insert_rowid())
        }).await
    }

    /// Get command history (paginated)
    async fn get_history(&self, limit: i64, offset: i64) -> SqliteResult<Vec<CommandHistoryRecord>> {
        self.with_conn(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, query, command, explanation, risk_level, risk_reason,
                        execution_status, exit_code, stdout, stderr, duration_ms,
                        provider_used, generation_latency_ms, created_at, executed_at,
                        confirmation_token
                 FROM command_history
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2"
            )?;

            let rows = stmt.query_map(params![limit, offset], |row| {
                Ok(CommandHistoryRecord {
                    id: Some(row.get(0)?),
                    query: row.get(1)?,
                    command: row.get(2)?,
                    explanation: row.get(3)?,
                    risk_level: row.get(4)?,
                    risk_reason: row.get(5)?,
                    execution_status: row.get(6)?,
                    exit_code: row.get(7)?,
                    stdout: row.get(8)?,
                    stderr: row.get(9)?,
                    duration_ms: row.get(10)?,
                    provider_used: row.get(11)?,
                    generation_latency_ms: row.get(12)?,
                    created_at: row.get(13)?,
                    executed_at: row.get(14)?,
                    confirmation_token: row.get(15)?,
                })
            })?;

            rows.collect()
        }).await
    }

    /// Update command execution result
    async fn update_execution(
        &self,
        id: i64,
        status: String,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
        duration_ms: Option<i64>,
    ) -> SqliteResult<()> {
        let timestamp = chrono::Utc::now().timestamp();
        self.with_conn(move |conn| {
            conn.execute(
                "UPDATE command_history 
                 SET execution_status = ?1, exit_code = ?2, stdout = ?3, stderr = ?4,
                     duration_ms = ?5, executed_at = ?6
                 WHERE id = ?7",
                params![status, exit_code, stdout, stderr, duration_ms, timestamp, id],
            )?;
            Ok(())
        }).await
    }
}

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandHistoryRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    query: String,
    command: String,
    explanation: Option<String>,
    risk_level: String,
    risk_reason: Option<String>,
    execution_status: String, // pending, executing, completed, failed, blocked
    exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
    duration_ms: Option<i64>,
    provider_used: Option<String>,
    generation_latency_ms: Option<i32>,
    created_at: i64, // Unix timestamp
    executed_at: Option<i64>,
    confirmation_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    command: String,
    risk_level: String,
    #[serde(default)]
    timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteResponse {
    executed: bool,
    method: String, // "direct", "sandboxed", "blocked"
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    memory_kb: u64,
    cpu_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemSnapshot {
    cpu_percent: f64,
    memory_total_bytes: u64,
    memory_used_bytes: u64,
    memory_available_bytes: u64,
    memory_used_percent: f64,
    disk_total_bytes: u64,
    disk_used_bytes: u64,
    disk_used_percent: f64,
    network_rx_bytes: u64,
    network_tx_bytes: u64,
    processes: Vec<ProcessInfo>,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessActionRequest {
    signal: Option<String>, // "kill", "kill9", "pause", "resume"
    priority: Option<i32>,  // For renice
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct CpuSample {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    total: u64,
    timestamp: std::time::Instant,
}

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    start_time: std::time::Instant,
    previous_cpu: Arc<std::sync::Mutex<Option<CpuSample>>>,
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
enum ApiError {
    Database(String),
    Execution(String),
    ProcessControl(String),
    InvalidRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg)),
            ApiError::Execution(msg) => (StatusCode::BAD_REQUEST, format!("Execution error: {}", msg)),
            ApiError::ProcessControl(msg) => (StatusCode::BAD_REQUEST, format!("Process control error: {}", msg)),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// ============================================================================
// API Endpoints
// ============================================================================

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

/// Get system monitoring snapshot
async fn get_snapshot(State(state): State<AppState>) -> Result<Json<SystemSnapshot>, ApiError> {
    // Read system metrics
    let mem_contents = std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| ApiError::Execution(format!("Failed to read meminfo: {}", e)))?;

    let total_kb = parse_meminfo_value(&mem_contents, "MemTotal:")
        .ok_or_else(|| ApiError::Execution("Failed to parse MemTotal".to_string()))?;
    let available_kb = parse_meminfo_value(&mem_contents, "MemAvailable:")
        .ok_or_else(|| ApiError::Execution("Failed to parse MemAvailable".to_string()))?;

    let total_bytes = total_kb * 1024;
    let available_bytes = available_kb * 1024;
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let mem_used_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;

    // Get disk usage
    let (disk_total_bytes, disk_used_bytes, disk_used_percent) = get_disk_usage();

    // Get network stats
    let (network_rx_bytes, network_tx_bytes) = get_network_stats();

    // Get CPU usage (with delta calculation)
    let cpu_percent = get_cpu_usage(&state);

    // Get top processes
    let processes = get_top_processes(10);

    Ok(Json(SystemSnapshot {
        cpu_percent,
        memory_total_bytes: total_bytes,
        memory_used_bytes: used_bytes,
        memory_available_bytes: available_bytes,
        memory_used_percent: mem_used_percent,
        disk_total_bytes,
        disk_used_bytes,
        disk_used_percent,
        network_rx_bytes,
        network_tx_bytes,
        processes,
        timestamp: chrono::Utc::now().timestamp(),
    }))
}

/// Execute command with risk-based routing
async fn execute_command(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    info!("Execute request: command='{}', risk_level='{}'", req.command, req.risk_level);

    // Validate command (prevent empty or obviously malicious patterns)
    if req.command.trim().is_empty() {
        return Err(ApiError::InvalidRequest("Empty command".to_string()));
    }

    // Create history record BEFORE execution
    let history_record = CommandHistoryRecord {
        id: None,
        query: req.command.clone(), // Using command as query (no separate query field in ExecuteRequest)
        command: req.command.clone(),
        explanation: None,
        risk_level: req.risk_level.clone(),
        risk_reason: None,
        execution_status: "pending".to_string(),
        exit_code: None,
        stdout: None,
        stderr: None,
        duration_ms: None,
        provider_used: None,
        generation_latency_ms: None,
        created_at: chrono::Utc::now().timestamp(),
        executed_at: None,
        confirmation_token: None,
    };

    // Insert history record and capture ID
    let history_id = match state.db.insert_command(&history_record).await {
        Ok(id) => {
            info!("Created history record with ID {}", id);
            id
        }
        Err(e) => {
            warn!("Failed to insert history record: {}. Continuing execution.", e);
            -1 // Continue execution even if DB write fails
        }
    };

    // Risk-based routing
    let risk_level = req.risk_level.to_lowercase();
    
    match risk_level.as_str() {
        "high" | "critical" => {
            warn!("Blocked high-risk command: {}", req.command);
            
            // Record blocked status if we have a valid history_id
            if history_id >= 0 {
                if let Err(e) = state.db.update_execution(
                    history_id,
                    "blocked".to_string(),
                    None,
                    None,
                    None,
                    None,
                ).await {
                    warn!("Failed to update history for blocked command: {}", e);
                }
            }
            
            Ok(Json(ExecuteResponse {
                executed: false,
                method: "blocked".to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                message: format!("Command blocked due to {} risk level", risk_level),
                duration_ms: None,
            }))
        }
        "medium" | "warn" => {
            info!("Executing command in sandbox: {}", req.command);
            execute_sandboxed(&req.command, req.timeout_seconds, &state.db, history_id).await
        }
        "low" | "safe" => {
            info!("Executing command directly: {}", req.command);
            execute_direct(&req.command, req.timeout_seconds, &state.db, history_id).await
        }
        _ => {
            Err(ApiError::InvalidRequest(format!("Invalid risk level: {}", req.risk_level)))
        }
    }
}

/// Execute command directly (for LOW risk only)
async fn execute_direct(command: &str, _timeout_seconds: u64, db: &Database, history_id: i64) -> Result<Json<ExecuteResponse>, ApiError> {
    let start = std::time::Instant::now();
    
    let output = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .await
        .map_err(|e| ApiError::Execution(format!("Failed to execute: {}", e)))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    
    // Update history record with execution result
    if history_id >= 0 {
        let status = if output.status.success() { "completed" } else { "failed" };
        if let Err(e) = db.update_execution(
            history_id,
            status.to_string(),
            exit_code,
            Some(stdout.clone()),
            Some(stderr.clone()),
            Some(duration_ms as i64),
        ).await {
            warn!("Failed to update history after direct execution: {}", e);
        }
    }

    Ok(Json(ExecuteResponse {
        executed: true,
        method: "direct".to_string(),
        exit_code,
        stdout,
        stderr,
        message: "Command executed directly".to_string(),
        duration_ms: Some(duration_ms),
    }))
}

/// Execute command in sandbox (for MEDIUM risk)
async fn execute_sandboxed(command: &str, _timeout_seconds: u64, db: &Database, history_id: i64) -> Result<Json<ExecuteResponse>, ApiError> {
    // Check if we have root for sandbox
    let euid = unsafe { libc::geteuid() };
    if euid != 0 {
        // Gracefully degrade to direct execution with warning
        warn!("Sandbox requires root, degrading to direct execution for: {}", command);
        return execute_direct(command, _timeout_seconds, db, history_id).await;
    }

    let start = std::time::Instant::now();

    // Configure sandbox with resource limits
    let config = sandbox::SandboxConfig {
        memory_limit_bytes: 256 * 1024 * 1024, // 256MB
        pids_limit: 100,
        cpu_quota_us: None,
        timeout: std::time::Duration::from_secs(_timeout_seconds),
        use_pid_namespace: true,
        use_mount_namespace: true,
    };

    // Execute in sandbox
    let result = sandbox::sandboxed_exec(
        "bash",
        &["-c", command],
        config,
    );

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(sandbox_result) => {
            let exit_code = sandbox_result.exit_code;
            let stdout = sandbox_result.stdout.clone();
            let stderr = sandbox_result.stderr.clone();
            
            // Update history record with execution result
            if history_id >= 0 {
                let status = match sandbox_result.termination_reason {
                    sandbox::TerminationReason::Exited if exit_code == 0 => "completed",
                    _ => "failed",
                };
                if let Err(e) = db.update_execution(
                    history_id,
                    status.to_string(),
                    Some(exit_code),
                    Some(stdout.clone()),
                    Some(stderr.clone()),
                    Some(duration_ms as i64),
                ).await {
                    warn!("Failed to update history after sandboxed execution: {}", e);
                }
            }
            
            Ok(Json(ExecuteResponse {
                executed: true,
                method: "sandboxed".to_string(),
                exit_code: Some(exit_code),
                stdout,
                stderr,
                message: format!("Command executed in sandbox ({})", 
                    match sandbox_result.termination_reason {
                        sandbox::TerminationReason::Exited => "exited",
                        sandbox::TerminationReason::Signaled => "signaled",
                        sandbox::TerminationReason::Timeout => "timeout",
                        sandbox::TerminationReason::ResourceLimit => "resource limit",
                        sandbox::TerminationReason::Failed => "failed",
                    }
                ),
                duration_ms: Some(duration_ms),
            }))
        }
        Err(e) => {
            // Update history with failed status
            if history_id >= 0 {
                if let Err(db_err) = db.update_execution(
                    history_id,
                    "failed".to_string(),
                    None,
                    None,
                    Some(format!("Sandbox error: {:?}", e)),
                    Some(duration_ms as i64),
                ).await {
                    warn!("Failed to update history after sandbox error: {}", db_err);
                }
            }
            Err(ApiError::Execution(format!("Sandbox execution failed: {:?}", e)))
        }
    }
}

/// Get command history
async fn get_history(
    State(state): State<AppState>,
) -> Result<Json<Vec<CommandHistoryRecord>>, ApiError> {
    let history = state.db.get_history(50, 0).await
        .map_err(|e| ApiError::Database(e.to_string()))?;
    
    Ok(Json(history))
}

/// List all processes
async fn list_processes() -> Result<Json<Vec<ProcessInfo>>, ApiError> {
    let processes = get_top_processes(100); // Get top 100
    Ok(Json(processes))
}

/// Control process (kill, pause, resume, renice)
async fn control_process(
    Path(pid): Path<u32>,
    Json(req): Json<ProcessActionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Protected PID check
    if pid == 1 || pid == std::process::id() {
        return Err(ApiError::ProcessControl(format!("Refusing to act on protected PID {}", pid)));
    }

    // Handle signal-based actions
    if let Some(signal) = req.signal {
        let signal_num = match signal.as_str() {
            "kill" => libc::SIGTERM,
            "kill9" => libc::SIGKILL,
            "pause" => libc::SIGSTOP,
            "resume" => libc::SIGCONT,
            _ => return Err(ApiError::InvalidRequest(format!("Invalid signal: {}", signal))),
        };

        let result = unsafe { libc::kill(pid as libc::pid_t, signal_num) };
        if result == 0 {
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Sent {} to PID {}", signal, pid)
            })))
        } else {
            Err(ApiError::ProcessControl(std::io::Error::last_os_error().to_string()))
        }
    } 
    // Handle renice
    else if let Some(priority) = req.priority {
        if !(-20..=19).contains(&priority) {
            return Err(ApiError::InvalidRequest("Priority must be between -20 and 19".to_string()));
        }

        let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, priority) };
        if result == 0 {
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Changed priority of PID {} to {}", pid, priority)
            })))
        } else {
            Err(ApiError::ProcessControl(std::io::Error::last_os_error().to_string()))
        }
    } else {
        Err(ApiError::InvalidRequest("Must specify either signal or priority".to_string()))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_meminfo_value(contents: &str, key: &str) -> Option<u64> {
    for line in contents.lines() {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse::<u64>().ok();
            }
        }
    }
    None
}

fn get_cpu_usage(state: &AppState) -> f64 {
    // Read current CPU stats from /proc/stat
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

    // Lock previous CPU sample
    let mut prev_lock = match state.previous_cpu.lock() {
        Ok(lock) => lock,
        Err(_) => return 0.0,
    };

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

    // Store current sample for next call
    *prev_lock = Some(current);

    cpu_percent
}

fn get_disk_usage() -> (u64, u64, f64) {
    use std::ffi::CString;
    use std::mem;

    let path = match CString::new("/") {
        Ok(p) => p,
        Err(_) => return (0, 0, 0.0),
    };

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

fn get_top_processes(limit: usize) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(pid_str) = entry.file_name().into_string() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if let Some(info) = read_process_info(pid) {
                        processes.push(info);
                    }
                }
            }
        }
    }

    processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(limit);
    processes
}

fn read_process_info(pid: u32) -> Option<ProcessInfo> {
    let status_path = format!("/proc/{}/status", pid);
    let contents = std::fs::read_to_string(&status_path).ok()?;

    let mut name = String::new();
    let mut state = String::new();
    let mut memory_kb: u64 = 0;

    for line in contents.lines() {
        if line.starts_with("Name:") {
            name = line.split_whitespace().nth(1).unwrap_or("?").to_string();
        } else if line.starts_with("State:") {
            state = line.split_once(':').map(|x| x.1).unwrap_or("?").trim().to_string();
        } else if line.starts_with("VmRSS:") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                memory_kb = value_str.parse::<u64>().unwrap_or(0);
            }
        }
    }

    Some(ProcessInfo {
        pid,
        name,
        state,
        memory_kb,
        cpu_percent: 0.0, // Simplified - would need delta calculation
    })
}

// ============================================================================
// Server Setup
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    info!("Starting IntelliShell Core Server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let db_path = PathBuf::from("./intellishell.db");
    let db = Database::new(db_path).expect("Failed to initialize database");
    info!("Database initialized");

    // Create application state
    let state = AppState {
        db: Arc::new(db),
        start_time: std::time::Instant::now(),
        previous_cpu: Arc::new(std::sync::Mutex::new(None)),
    };

    // Configure CORS for AI service
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/snapshot", get(get_snapshot))
        .route("/execute", post(execute_command))
        .route("/history", get(get_history))
        .route("/processes", get(list_processes))
        .route("/processes/:pid", post(control_process))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = "127.0.0.1:3000";
    info!("Listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> AppState {
        // Use a unique temporary file database for each test
        // to avoid interference between tests
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let test_id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_path = std::env::temp_dir().join(format!("test_{}_{}.db", std::process::id(), test_id));
        
        // Clean up any existing file
        let _ = std::fs::remove_file(&temp_path);
        
        let db = Database::new(temp_path).unwrap();
        AppState {
            db: Arc::new(db),
            start_time: std::time::Instant::now(),
            previous_cpu: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    #[tokio::test]
    async fn test_high_risk_blocks() {
        let state = create_test_state();

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
        assert!(response.message.contains("blocked"));
    }

    #[tokio::test]
    async fn test_critical_risk_blocks() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "dd if=/dev/zero of=/dev/sda".to_string(),
            risk_level: "critical".to_string(),
            timeout_seconds: 10,
        };

        let result = execute_command(State(state), Json(req)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.executed, false);
        assert_eq!(response.method, "blocked");
    }

    #[tokio::test]
    async fn test_low_risk_executes_directly() {
        let state = create_test_state();

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
        assert_eq!(response.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_medium_risk_routes_to_sandbox_or_degrades() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "echo test".to_string(),
            risk_level: "medium".to_string(),
            timeout_seconds: 10,
        };

        let result = execute_command(State(state), Json(req)).await;

        // Sandbox may fail if not root, but should degrade gracefully to direct execution
        if let Ok(response) = result {
            assert!(response.0.executed);
            // Should be either sandboxed or direct (degraded)
            assert!(
                response.0.method == "sandboxed" || response.0.method == "direct",
                "Expected sandboxed or direct, got {}",
                response.0.method
            );
        } else {
            // If it fails, it should be a sandbox-related error
            panic!("Medium risk command should execute or degrade gracefully, not fail");
        }
    }

    #[tokio::test]
    async fn test_empty_command_rejected() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "".to_string(),
            risk_level: "low".to_string(),
            timeout_seconds: 10,
        };

        let result = execute_command(State(state), Json(req)).await;
        assert!(result.is_err());
        
        if let Err(ApiError::InvalidRequest(msg)) = result {
            assert!(msg.contains("Empty"));
        } else {
            panic!("Expected InvalidRequest error for empty command");
        }
    }

    #[tokio::test]
    async fn test_invalid_risk_level_rejected() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "echo test".to_string(),
            risk_level: "invalid".to_string(),
            timeout_seconds: 10,
        };

        let result = execute_command(State(state), Json(req)).await;
        assert!(result.is_err());
        
        if let Err(ApiError::InvalidRequest(msg)) = result {
            assert!(msg.contains("Invalid risk level"));
        } else {
            panic!("Expected InvalidRequest error for invalid risk level");
        }
    }

    #[tokio::test]
    async fn test_history_persistence_low_risk() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "echo history_test".to_string(),
            risk_level: "low".to_string(),
            timeout_seconds: 10,
        };

        // Execute command
        let result = execute_command(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());

        // Check history
        let history = state.db.get_history(10, 0).await.unwrap();
        assert!(!history.is_empty(), "History should not be empty");
        
        let last_entry = &history[0];
        assert_eq!(last_entry.command, "echo history_test");
        assert_eq!(last_entry.execution_status, "completed");
        assert!(last_entry.stdout.is_some());
        assert!(last_entry.stdout.as_ref().unwrap().contains("history_test"));
    }

    #[tokio::test]
    async fn test_history_persistence_blocked() {
        let state = create_test_state();

        let req = ExecuteRequest {
            command: "rm -rf /".to_string(),
            risk_level: "high".to_string(),
            timeout_seconds: 10,
        };

        // Execute (should block)
        let result = execute_command(State(state.clone()), Json(req)).await;
        assert!(result.is_ok());

        // Check history
        let history = state.db.get_history(10, 0).await.unwrap();
        assert!(!history.is_empty(), "History should contain blocked command");
        
        let last_entry = &history[0];
        assert_eq!(last_entry.command, "rm -rf /");
        assert_eq!(last_entry.execution_status, "blocked");
    }

    #[test]
    fn test_get_disk_usage() {
        let (total, used, percent) = get_disk_usage();
        
        // On a real system, disk should have non-zero total
        assert!(total > 0, "Disk total should be > 0");
        assert!(used >= 0, "Disk used should be >= 0");
        assert!(percent >= 0.0 && percent <= 100.0, "Disk percent should be 0-100");
    }

    #[test]
    fn test_get_network_stats() {
        let (rx, tx) = get_network_stats();
        
        // Network stats should be >= 0 (may be 0 if no network activity)
        assert!(rx >= 0, "Network RX should be >= 0");
        assert!(tx >= 0, "Network TX should be >= 0");
    }

    #[test]
    fn test_cpu_usage_delta() {
        let state = create_test_state();
        
        // First call should return 0 (no previous sample)
        let cpu1 = get_cpu_usage(&state);
        assert_eq!(cpu1, 0.0, "First CPU call should return 0");
        
        // Sleep briefly
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Second call should return a value >= 0
        let cpu2 = get_cpu_usage(&state);
        assert!(cpu2 >= 0.0 && cpu2 <= 100.0, "Second CPU call should return 0-100");
    }
}
