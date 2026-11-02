# IntelliShell Dual-OS Architecture Design Document

**Task**: Dual Operating System Support (Linux + Windows)  
**Date**: 2025-01-30  
**Status**: Architecture Design Complete  

---

## Executive Summary

This document provides a comprehensive architecture audit of IntelliShell and designs the dual-OS architecture to support both Linux (WSL2) and Windows (native host) execution environments. The implementation will enable users to select their target operating system via the UI, with all components (AI command generation, risk assessment, execution, and telemetry) adapting accordingly.

### Key Findings

1. **Current System**: Linux-only, WSL2-aware command generation
2. **Reusable Components**: 70% of codebase can be shared (AI providers, risk patterns, database, HTTP routing)
3. **New Components Required**: Windows execution backend, Windows telemetry service, Windows-specific risk rules
4. **Primary Challenge**: Maintaining separate execution contexts while preventing cross-OS command leakage

---

## 1. Current System Architecture

### 1.1 System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    React Dashboard (Port 5173)                   │
│  • AssistantPage.jsx - Command generation UI                    │
│  • MonitoringPage.jsx - System telemetry visualization          │
│  • ProcessesPage.jsx - Process management                       │
└──────────────┬──────────────────────────┬────────────────────────┘
               │                          │
               ▼                          ▼
┌──────────────────────────┐  ┌─────────────────────────────────┐
│  Python AI Service       │  │   Rust Core Engine              │
│  (Port 8000 - FastAPI)   │  │   (Port 3000 - Axum)            │
│                          │  │                                 │
│  • main.py               │  │  • server.rs                    │
│  • providers.py          │  │  • sandbox.rs (cgroups)         │
│  • risk.py               │  │  • process.rs                   │
│  • impact.py             │  │  • Database (SQLite)            │
└──────────────────────────┘  └─────────────────────────────────┘
```

### 1.2 Current Command Generation Flow

**File**: `ai-service/src/intellishell_ai_service/main.py`

```
User Query (AssistantPage.jsx)
        ↓
POST /generate-command
        ↓
SYSTEM_INSTRUCTIONS (Linux-focused, WSL2-aware)
        ↓
Provider Manager (Groq primary, Ollama fallback)
        ↓
AI generates {"command": "...", "explanation": "..."}
        ↓
classify_command(command) → (RiskLevel, reason)
        ↓
suggest_safer_alternative(command) → alternative
        ↓
analyze_impact(command) → filesystem/process impact
        ↓
Generate confirmation_token
        ↓
Return CommandResponse
```

**Key Observations**:
- Lines 41-142: SYSTEM_INSTRUCTIONS hardcoded for Linux/WSL2
- Lines 171-177: WSL-specific path rules (/mnt/c/, /mnt/d/)
- No OS parameter in request model
- Risk classification OS-agnostic (treats everything as Linux)

### 1.3 Current Execution Backend

**File**: `core-engine/src/server.rs`

```
POST /execute { command, risk_level, timeout_seconds }
        ↓
Risk-based routing:
    • HIGH/CRITICAL → Block (lines 373-393)
    • MEDIUM → Sandboxed execution (lines 394-397)
    • LOW/SAFE → Direct execution (lines 398-401)
        ↓
execute_direct() or execute_sandboxed()
        ↓
Command → bash -c "<command>"
        ↓
Linux-only execution via tokio::process::Command
        ↓
Update SQLite database with result
```

**Key Observations**:
- Lines 407-436: `execute_direct()` uses `bash -c`
- Lines 439-518: `execute_sandboxed()` uses Linux cgroups v2, requires root
- Lines 522-559: Telemetry reads `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`
- No OS parameter in ExecuteRequest model
- Execution target is always Linux shell

### 1.4 Current Risk Classification

**File**: `ai-service/src/intellishell_ai_service/risk.py`

```python
def classify_command(command: str) -> Tuple[RiskLevel, reason]:
    # Pattern-based classification
    # Lines 35-51: CRITICAL patterns (rm -rf /, dd, fork bombs, etc.)
    # Lines 54-62: HIGH patterns (recursive delete, /dev write, etc.)
    # Lines 65-77: MEDIUM patterns (kill -9, chmod 777, sudo, systemctl)
    # Lines 80-88: LOW patterns (rm, kill, mv)
    # Default: SAFE
```

**Key Observations**:
- Pure regex pattern matching
- Linux command-centric (rm, chmod, systemctl, apt-get)
- No Windows patterns (Remove-Item, Stop-Service, Set-ExecutionPolicy)
- Reusable structure, but patterns need OS-specific versions

### 1.5 Current Telemetry Collection

**File**: `core-engine/src/server.rs` (lines 561-648)

```rust
async fn get_snapshot(State(state): State<AppState>) -> Result<Json<SystemSnapshot>, ApiError> {
    // Lines 565-571: Parse /proc/meminfo
    // Lines 574-579: get_disk_usage() via libc::statvfs
    // Lines 582: get_network_stats() from /proc/net/dev
    // Lines 585: get_cpu_usage() from /proc/stat
    // Lines 588: get_top_processes() via /proc/<pid>/status
    
    return SystemSnapshot {
        cpu_percent, memory_total_bytes, memory_used_bytes, ...
    }
}
```

**Key Observations**:
- All telemetry Linux-specific (`/proc` filesystem)
- No Windows equivalent (WMI, CIM, Performance Counters)
- Reusable: HTTP endpoint structure, JSON serialization
- Not reusable: Data source (must abstract OS layer)

### 1.6 Frontend Command Generation Interface

**File**: `dashboard/src/pages/AssistantPage.jsx`

```jsx
// Lines 8-12: State management (query, generated, executionResult)
// Lines 14-18: generateMutation → pythonApi.generateCommand(query)
// Lines 20-24: confirmExecuteMutation → pythonApi.confirmExecute(token)

// UI Structure:
// Lines 40-78: Textarea input + "Generate Command" button
// Lines 86-229: Generated result display (command, explanation, risk badge, safer alternative)
// Lines 192-207: "Execute Command" button
```

**Key Observations**:
- **NO OS selection UI** (requirement states "two buttons: Linux and Windows")
- No OS state in component
- API calls don't pass OS parameter
- Risk badge display is OS-agnostic

**UI Changes Required**:
1. Add OS selection buttons (Linux/Windows) above textarea
2. Store selected OS in component state
3. Pass OS to `generateCommand(query, os)`
4. Display selected OS in results section
5. Validate OS on execution

---

## 2. Component Reusability Analysis

### 2.1 Fully Reusable Components (No Changes)

| Component | Location | Reason |
|-----------|----------|--------|
| HTTP routing | `core-engine/src/server.rs` (lines 886-906) | Axum router is OS-agnostic |
| Database layer | `core-engine/src/server.rs` (lines 18-151) | SQLite works on both OSes |
| AI Provider abstraction | `ai-service/.../providers.py` | Groq/Ollama are OS-agnostic |
| Command history UI | `dashboard/src/pages/HistoryPage.jsx` | Displays data, doesn't execute |
| Token confirmation flow | `ai-service/main.py` (lines 252-295) | Business logic only |
| Impact analysis structure | `ai-service/.../impact.py` (functions) | Refactor for OS param |
| Risk classification enum | `ai-service/.../risk.py` (RiskLevel) | Enum values OS-agnostic |

### 2.2 Components Requiring OS Parameter (Refactor)

| Component | Current State | Change Required |
|-----------|---------------|-----------------|
| `/generate-command` endpoint | No OS parameter | Add `os: str` to `CommandRequest` |
| SYSTEM_INSTRUCTIONS | Hardcoded Linux | Split into `LINUX_INSTRUCTIONS` and `WINDOWS_INSTRUCTIONS` |
| `classify_command()` | Linux patterns only | Add `classify_command(command, os)`, dual pattern sets |
| `suggest_safer_alternative()` | Linux-focused | Add OS-aware suggestions |
| `/execute` endpoint | No OS parameter | Add `os: str` to `ExecuteRequest` |
| Execution router | Single branch | Add OS switch before risk routing |

### 2.3 Components Requiring Duplication (New Implementations)

| Component | Linux Version | Windows Version |
|-----------|---------------|-----------------|
| Execution backend | `execute_direct()` + `execute_sandboxed()` | `execute_windows_direct()` + `execute_windows_sandboxed()` |
| Telemetry service | `get_snapshot()` (reads `/proc`) | `get_windows_snapshot()` (reads WMI/CIM) |
| Process listing | `get_top_processes()` (`/proc`) | `get_windows_processes()` (CIM Win32_Process) |
| System shell | `bash -c` | `powershell.exe -Command` or `pwsh -Command` |

---

## 3. Dual-OS Architecture Design

### 3.1 OS Parameter Flow (End-to-End)

```
┌─────────────────────────────────────────────────────────────────┐
│                  Frontend (AssistantPage.jsx)                    │
│                                                                  │
│  [Linux] [Windows] ← OS selection buttons (new)                 │
│  selectedOS: "linux" | "windows"                                │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
          POST /generate-command { query: "...", os: "linux" }
                         │
┌────────────────────────┴─────────────────────────────────────────┐
│              Python AI Service (main.py)                          │
│                                                                   │
│  1. Select SYSTEM_INSTRUCTIONS based on request.os               │
│     • os == "linux" → LINUX_SYSTEM_INSTRUCTIONS                  │
│     • os == "windows" → WINDOWS_SYSTEM_INSTRUCTIONS              │
│                                                                   │
│  2. Generate command via provider                                │
│                                                                   │
│  3. Classify risk: classify_command(command, os)                 │
│     • Uses OS-specific pattern sets                              │
│                                                                   │
│  4. Suggest alternative: suggest_safer_alternative(command, os)  │
│                                                                   │
│  5. Analyze impact: analyze_impact(command, os, cwd)             │
│                                                                   │
│  6. Return CommandResponse { ..., os: request.os }               │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
         POST /confirm-execute { token: "..." }
                         │
┌────────────────────────┴─────────────────────────────────────────┐
│              Python AI Service (confirm handler)                  │
│                                                                   │
│  1. Retrieve command metadata from PENDING_COMMANDS[token]       │
│     • Extract: command, risk_level, query, os                    │
│                                                                   │
│  2. Forward to Rust: POST /execute {                             │
│       command, risk_level, timeout_seconds, os                   │
│     }                                                             │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌────────────────────────┴─────────────────────────────────────────┐
│                Rust Core Engine (server.rs)                       │
│                                                                   │
│  async fn execute_command(..., Json(req): Json<ExecuteRequest>)  │
│  {                                                                │
│      // NEW: OS routing BEFORE risk routing                      │
│      match req.os.as_str() {                                     │
│          "linux" => execute_linux_pipeline(req),                 │
│          "windows" => execute_windows_pipeline(req),             │
│          _ => return Err(InvalidOS)                              │
│      }                                                            │
│  }                                                                │
│                                                                   │
│  fn execute_linux_pipeline(req: ExecuteRequest) {                │
│      match req.risk_level {                                      │
│          HIGH/CRITICAL → block,                                  │
│          MEDIUM → execute_sandboxed(...) // existing WSL2/cgroupsv2
│          LOW/SAFE → execute_direct(...) // bash -c              │
│      }                                                            │
│  }                                                                │
│                                                                   │
│  fn execute_windows_pipeline(req: ExecuteRequest) {              │
│      match req.risk_level {                                      │
│          HIGH/CRITICAL → block,                                  │
│          MEDIUM → execute_windows_sandboxed(...) // Job Objects  │
│          LOW/SAFE → execute_windows_direct(...) // powershell    │
│      }                                                            │
│  }                                                                │
└───────────────────────────────────────────────────────────────────┘
```

### 3.2 Windows Execution Service Architecture

**File**: `core-engine/src/windows_execution.rs` (new)

```rust
pub struct WindowsExecutionService;

impl WindowsExecutionService {
    /// Execute command directly on Windows host using PowerShell
    pub async fn execute_direct(
        command: &str,
        timeout_seconds: u64
    ) -> Result<ExecuteResponse, WindowsExecutionError> {
        // Strategy:
        // 1. Spawn powershell.exe -NoProfile -Command "<command>"
        // 2. Use tokio::process::Command (Windows-aware)
        // 3. Capture stdout/stderr (UTF-16 → UTF-8 conversion)
        // 4. Handle timeout with tokio::time::timeout()
        // 5. Map Windows exit codes to ExecuteResponse
    }

    /// Execute command with resource limits using Windows Job Objects
    pub async fn execute_sandboxed(
        command: &str,
        timeout_seconds: u64,
        memory_limit_bytes: u64,
        process_limit: u32
    ) -> Result<ExecuteResponse, WindowsExecutionError> {
        // Strategy:
        // 1. Create Job Object via Windows API (winapi crate)
        // 2. Set JOBOBJECT_EXTENDED_LIMIT_INFORMATION:
        //    • ProcessMemoryLimit
        //    • ActiveProcessLimit
        //    • JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        // 3. Spawn PowerShell as child process
        // 4. Assign child process to Job Object (AssignProcessToJobObject)
        // 5. Monitor via Job Object handles
        // 6. Cleanup: TerminateJobObject on timeout/error
        
        // Limitations:
        // • CPU quota requires Windows 10 or Job Object CPU rate control
        // • No namespace isolation (Windows doesn't have Linux-style namespaces)
        // • Process tree kill via Job Object (more reliable than Linux pkill)
    }
}

#[derive(Debug, Error)]
pub enum WindowsExecutionError {
    #[error("Failed to spawn PowerShell: {0}")]
    SpawnFailed(String),
    
    #[error("Job Object creation failed: {0}")]
    JobObjectFailed(String),
    
    #[error("Command timed out after {0}s")]
    Timeout(u64),
    
    #[error("UTF-16 to UTF-8 conversion failed")]
    EncodingError,
}
```

**Dependencies** (add to `core-engine/Cargo.toml`):
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["jobapi2", "processthreadsapi", "winnt", "handleapi"] }
```

**Key Design Decisions**:
1. **PowerShell as default shell**: More powerful than `cmd.exe`, native JSON support, WSL interop
2. **Job Objects for sandboxing**: Windows equivalent to cgroups, enforces memory/process limits
3. **No namespace isolation**: Windows lacks Linux-style namespaces; rely on Job Object containment
4. **UTF-16 handling**: Windows uses UTF-16, must convert to UTF-8 for JSON responses

### 3.3 Windows Telemetry Service Architecture

**File**: `core-engine/src/windows_telemetry.rs` (new)

```rust
use windows::Win32::System::Performance::*;
use windows::Win32::System::SystemInformation::*;

pub struct WindowsTelemetryService;

impl WindowsTelemetryService {
    /// Get system snapshot using Windows CIM/Performance Counters
    pub fn get_snapshot() -> Result<SystemSnapshot, TelemetryError> {
        // CPU Usage:
        // Strategy 1: Read Performance Counters (\\Processor(_Total)\\% Processor Time)
        // Strategy 2: GetSystemTimes() delta calculation (similar to Linux /proc/stat)
        let cpu_percent = Self::get_cpu_usage()?;
        
        // Memory Usage:
        // Call GlobalMemoryStatusEx() → MEMORYSTATUSEX
        // Fields: ullTotalPhys, ullAvailPhys
        let mem_status = Self::get_memory_info()?;
        
        // Disk Usage:
        // Call GetDiskFreeSpaceExW() for C:\ (or user-specified drive)
        let disk_info = Self::get_disk_info("C:\\")?;
        
        // Network Usage:
        // Call GetIfTable2() → MIB_IF_TABLE2 (from iphlpapi.dll)
        // Sum InOctets/OutOctets across non-loopback interfaces
        let network = Self::get_network_stats()?;
        
        // Process List:
        // Via WMI Win32_Process query (requires wmi crate)
        // OR EnumProcesses() + OpenProcess() + GetProcessMemoryInfo()
        let processes = Self::get_windows_processes()?;
        
        Ok(SystemSnapshot {
            cpu_percent,
            memory_total_bytes: mem_status.ullTotalPhys,
            memory_used_bytes: mem_status.ullTotalPhys - mem_status.ullAvailPhys,
            memory_available_bytes: mem_status.ullAvailPhys,
            memory_used_percent: /* calculate */,
            disk_total_bytes: disk_info.total,
            disk_used_bytes: disk_info.used,
            disk_used_percent: /* calculate */,
            network_rx_bytes: network.rx,
            network_tx_bytes: network.tx,
            processes,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
    
    fn get_cpu_usage() -> Result<f64, TelemetryError> {
        // Implementation using GetSystemTimes() delta
        // Store previous idle/kernel/user times in AppState
        // Calculate: 100.0 * (1.0 - idle_delta / total_delta)
    }
    
    fn get_windows_processes() -> Result<Vec<ProcessInfo>, TelemetryError> {
        // Use wmi crate to query: SELECT * FROM Win32_Process
        // Map fields:
        // • ProcessId → pid
        // • Name → name
        // • WorkingSetSize → memory_kb
        // • PercentProcessorTime → cpu_percent (requires second sample)
    }
}
```

**Dependencies** (add to `core-engine/Cargo.toml`):
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.51", features = [
    "Win32_System_SystemInformation",
    "Win32_System_Performance",
    "Win32_NetworkManagement_IpHelper",
] }
wmi = "0.13" # For Win32_Process queries
```

**Key Design Decisions**:
1. **CIM/WMI for processes**: Windows-native, rich metadata, no parsing required
2. **Performance Counters for CPU**: Most accurate, used by Task Manager
3. **GlobalMemoryStatusEx for RAM**: Fast, kernel call, matches Task Manager
4. **Fallback to polling**: Windows lacks `/proc`-like instant snapshots, may need cached deltas

### 3.4 Windows Risk Analyzer Architecture

**File**: `ai-service/src/intellishell_ai_service/windows_risk.py` (new)

```python
"""
Windows-specific risk classification patterns.
Mirrors risk.py structure but with PowerShell/Windows commands.
"""

WINDOWS_CRITICAL_PATTERNS = [
    (r"Remove-Item\s+.*-Recurse.*\s+C:\\$", "Attempts to delete C drive root"),
    (r"Remove-Item\s+.*-Recurse.*-Force.*\s+C:\\", "Recursive force delete on system drive"),
    (r"Format-Volume\s+", "Disk format operation"),
    (r"Stop-Computer|Restart-Computer", "System shutdown/restart"),
    (r"(Invoke-WebRequest|iwr|curl).*\|\s*(Invoke-Expression|iex)", "Piping remote script to execution"),
    (r"Set-ExecutionPolicy\s+Unrestricted\s+-Scope\s+LocalMachine", "Disabling PowerShell execution policy globally"),
    (r"takeown\s+/f\s+C:\\Windows", "Taking ownership of system directory"),
    (r"bcdedit\s+", "Boot configuration modification"),
    (r"reg\s+delete\s+HKLM\\", "Registry deletion of system keys"),
]

WINDOWS_HIGH_PATTERNS = [
    (r"Remove-Item\s+.*-Recurse.*-Force", "Recursive force delete"),
    (r"Stop-Service\s+.*\s+-Force", "Force stopping Windows service"),
    (r"Set-ItemProperty\s+.*HKLM:", "Modifying system registry"),
    (r"Disable-WindowsOptionalFeature", "Disabling Windows features"),
    (r"Uninstall-WindowsFeature", "Removing Windows features"),
]

WINDOWS_MEDIUM_PATTERNS = [
    (r"Stop-Process\s+.*-Force", "Force killing process"),
    (r"Set-ExecutionPolicy", "Changing PowerShell execution policy"),
    (r"Stop-Service", "Stopping Windows service"),
    (r"Restart-Service", "Restarting Windows service"),
    (r"New-NetFirewallRule", "Modifying firewall rules"),
    (r"Uninstall-Package", "Removing software package"),
]

WINDOWS_LOW_PATTERNS = [
    (r"Remove-Item\s+[^-]", "Delete operation without force"),
    (r"Stop-Process\s+\d+", "Process termination"),
    (r"Move-Item", "Move/rename operation"),
]

def classify_windows_command(command: str) -> Tuple[RiskLevel, str]:
    """Classify Windows/PowerShell command risk."""
    # Same structure as classify_command() but with WINDOWS_*_PATTERNS
    # Falls through to SAFE for Get-*, Test-*, Read-* cmdlets
```

**Integration** (`ai-service/.../risk.py`):
```python
def classify_command(command: str, os: str = "linux") -> Tuple[RiskLevel, str]:
    """OS-aware risk classification."""
    if os == "windows":
        from intellishell_ai_service.windows_risk import classify_windows_command
        return classify_windows_command(command)
    else:
        # Existing Linux classification logic
        return classify_linux_command(command)
```

### 3.5 Execution Routing Logic

**Updated**: `core-engine/src/server.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    command: String,
    risk_level: String,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default = "default_os")] // NEW
    os: String, // "linux" or "windows"
}

fn default_os() -> String {
    "linux".to_string() // Backward compatibility
}

async fn execute_command(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    info!("Execute request: command='{}', risk_level='{}', os='{}'", 
          req.command, req.risk_level, req.os);
    
    // Validate OS
    if req.os != "linux" && req.os != "windows" {
        return Err(ApiError::InvalidRequest(format!("Invalid OS: {}", req.os)));
    }
    
    // CRITICAL: OS routing BEFORE risk routing
    let execution_result = match req.os.as_str() {
        "linux" => execute_linux_pipeline(&req, &state).await?,
        "windows" => execute_windows_pipeline(&req, &state).await?,
        _ => unreachable!(), // Already validated above
    };
    
    Ok(Json(execution_result))
}

async fn execute_linux_pipeline(
    req: &ExecuteRequest,
    state: &AppState,
) -> Result<ExecuteResponse, ApiError> {
    // Record history
    let history_id = record_command_attempt(req, state).await?;
    
    // Risk routing (existing logic)
    let result = match req.risk_level.to_lowercase().as_str() {
        "high" | "critical" => block_command(&req, state, history_id).await?,
        "medium" | "warn" => {
            // Check if running in WSL2 (can use Linux cgroups)
            if is_wsl2() && is_root() {
                execute_sandboxed(&req.command, req.timeout_seconds, &state.db, history_id).await?
            } else {
                warn!("Sandbox unavailable in WSL2 non-root, degrading to direct");
                execute_direct(&req.command, req.timeout_seconds, &state.db, history_id).await?
            }
        },
        "low" | "safe" => execute_direct(&req.command, req.timeout_seconds, &state.db, history_id).await?,
        _ => return Err(ApiError::InvalidRequest(format!("Invalid risk level: {}", req.risk_level))),
    };
    
    Ok(result)
}

async fn execute_windows_pipeline(
    req: &ExecuteRequest,
    state: &AppState,
) -> Result<ExecuteResponse, ApiError> {
    // Record history
    let history_id = record_command_attempt(req, state).await?;
    
    // Risk routing
    let result = match req.risk_level.to_lowercase().as_str() {
        "high" | "critical" => block_command(&req, state, history_id).await?,
        "medium" | "warn" => {
            #[cfg(windows)]
            {
                use crate::windows_execution::WindowsExecutionService;
                WindowsExecutionService::execute_sandboxed(
                    &req.command,
                    req.timeout_seconds,
                    256 * 1024 * 1024, // 256MB
                    100, // max 100 processes
                    &state.db,
                    history_id,
                ).await?
            }
            #[cfg(not(windows))]
            {
                return Err(ApiError::Execution("Windows execution not available on this platform".to_string()));
            }
        },
        "low" | "safe" => {
            #[cfg(windows)]
            {
                use crate::windows_execution::WindowsExecutionService;
                WindowsExecutionService::execute_direct(
                    &req.command,
                    req.timeout_seconds,
                    &state.db,
                    history_id,
                ).await?
            }
            #[cfg(not(windows))]
            {
                return Err(ApiError::Execution("Windows execution not available on this platform".to_string()));
            }
        },
        _ => return Err(ApiError::InvalidRequest(format!("Invalid risk level: {}", req.risk_level))),
    };
    
    Ok(result)
}

fn is_wsl2() -> bool {
    // Check /proc/version for "microsoft" or "WSL2"
    std::fs::read_to_string("/proc/version")
        .map(|s| s.to_lowercase().contains("microsoft") || s.to_lowercase().contains("wsl"))
        .unwrap_or(false)
}
```

---

## 4. API Contract Changes

### 4.1 Python AI Service API Changes

#### `POST /generate-command`

**Before**:
```json
{
  "query": "list all files"
}
```

**After**:
```json
{
  "query": "list all files",
  "os": "linux"
}
```

**Response** (add `os` field):
```json
{
  "query": "list all files",
  "command": "ls -la",
  "explanation": "...",
  "risk_level": "safe",
  "risk_reason": "...",
  "safer_alternative": "",
  "impact_context": "...",
  "confirmation_token": "...",
  "provider_used": "groq",
  "generation_latency_ms": 1234,
  "os": "linux"
}
```

#### `POST /confirm-execute`

**No change to request**, but internally must:
1. Retrieve `os` from `PENDING_COMMANDS[token]`
2. Forward `os` to Rust backend in `/execute` call

### 4.2 Rust Core Engine API Changes

#### `POST /execute`

**Before**:
```json
{
  "command": "ls -la",
  "risk_level": "safe",
  "timeout_seconds": 30
}
```

**After**:
```json
{
  "command": "ls -la",
  "risk_level": "safe",
  "timeout_seconds": 30,
  "os": "linux"
}
```

**Response** (no change):
```json
{
  "executed": true,
  "method": "direct",
  "exit_code": 0,
  "stdout": "...",
  "stderr": "",
  "message": "Command executed directly",
  "duration_ms": 123
}
```

#### `GET /snapshot`

**Option 1: Query parameter** (recommended)
```
GET /snapshot?os=linux
GET /snapshot?os=windows
```

**Option 2: Separate endpoints**
```
GET /snapshot/linux
GET /snapshot/windows
```

**Response** (no change to structure):
```json
{
  "cpu_percent": 25.3,
  "memory_total_bytes": 16777216000,
  "memory_used_bytes": 8388608000,
  ...
  "processes": [...],
  "timestamp": 1738272000
}
```

**Implementation**:
```rust
async fn get_snapshot(
    State(state): State<AppState>,
    Query(params): Query<SnapshotQuery>,
) -> Result<Json<SystemSnapshot>, ApiError> {
    let os = params.os.unwrap_or_else(|| "linux".to_string());
    
    match os.as_str() {
        "linux" => get_linux_snapshot(&state),
        "windows" => get_windows_snapshot(&state),
        _ => Err(ApiError::InvalidRequest("Invalid OS parameter".to_string())),
    }
}

#[derive(Deserialize)]
struct SnapshotQuery {
    os: Option<String>,
}
```

### 4.3 Database Schema Changes

**Add `os` column to `command_history` table**:

```sql
ALTER TABLE command_history ADD COLUMN os TEXT DEFAULT 'linux' NOT NULL;
```

**Migration strategy**:
1. Add column with default value (backward compatible)
2. Update insert statements to include `os` value
3. Update queries to filter/display OS

---

## 5. Implementation Strategy by Component

### 5.1 Phase 1: Frontend OS Selection UI

**Files**: `dashboard/src/pages/AssistantPage.jsx`

**Changes**:
1. Add state: `const [selectedOS, setSelectedOS] = useState('linux')`
2. Add UI component before textarea:
```jsx
<div className="os-selector mb-4">
  <button
    className={selectedOS === 'linux' ? 'selected' : ''}
    onClick={() => setSelectedOS('linux')}
  >
    🐧 Linux (WSL2)
  </button>
  <button
    className={selectedOS === 'windows' ? 'selected' : ''}
    onClick={() => setSelectedOS('windows')}
  >
    🪟 Windows
  </button>
</div>
```
3. Update `generateMutation`: `pythonApi.generateCommand(query, selectedOS)`
4. Display OS badge in results: `<Badge>{generated.os}</Badge>`
5. Add confirmation modal: "You selected {OS}. Execute on {OS} system?"

**Verification**: Manual UI test, screenshot both states

### 5.2 Phase 2: Python AI Service - OS-Aware Generation

**Files**:
- `ai-service/src/intellishell_ai_service/main.py`
- `ai-service/src/intellishell_ai_service/windows_risk.py` (new)

**Changes**:

1. **Update request model** (line 196):
```python
class CommandRequest(BaseModel):
    query: str
    os: str = "linux"  # Default for backward compat
```

2. **Split SYSTEM_INSTRUCTIONS** (lines 41-142):
```python
LINUX_SYSTEM_INSTRUCTIONS = """...(existing)..."""

WINDOWS_SYSTEM_INSTRUCTIONS = """You are an expert Windows PowerShell command generator...

CRITICAL RULES:
1. Respond with ONLY a JSON object - no markdown, no code blocks
2. {"command": "<single PowerShell command>", "explanation": "..."}
3. Use ONLY real PowerShell cmdlets (Get-*, Set-*, New-*, Remove-*, etc.)
4. Generate commands safe by default - avoid destructive operations
5. Keep commands simple and focused

COMMAND GENERATION GUIDELINES:
- File operations: Get-ChildItem, Get-Content, Copy-Item, Move-Item
- Process management: Get-Process, Stop-Process, Start-Process
- System info: Get-ComputerInfo, Get-PSDrive, Get-NetAdapter
- Text processing: Select-String, Where-Object, Sort-Object
- Network: Invoke-WebRequest, Test-Connection, Get-NetIPConfiguration
- Permissions: Set-Acl, Get-Acl, icacls
- Package management: Get-Package, winget search

SAFETY PRINCIPLES:
- Never generate commands that:
  * Delete C:\Windows, C:\Program Files, system directories
  * Modify registry HKLM: keys without explicit request
  * Run remote scripts (Invoke-WebRequest | Invoke-Expression)
  * Shutdown/reboot without explicit request
  * Disable Windows Defender or firewall
  * Set-ExecutionPolicy Unrestricted globally

WINDOWS PATH RULES:
- Use backslashes: C:\Users\, D:\Projects\
- Quote paths with spaces: "C:\Program Files\App"
- Support environment variables: $env:USERPROFILE, $env:TEMP
- UNC paths: \\server\share\

EXAMPLES:
Request: "list all files in current directory"
{"command": "Get-ChildItem -Force", "explanation": "Lists all files including hidden ones"}

Request: "find large files over 100MB"
{"command": "Get-ChildItem -Recurse | Where-Object {$_.Length -gt 100MB} | Sort-Object Length -Descending", "explanation": "Recursively finds files larger than 100MB sorted by size"}

Request: "check memory usage"
{"command": "Get-CimInstance Win32_OperatingSystem | Select-Object TotalVisibleMemorySize, FreePhysicalMemory", "explanation": "Displays total and free physical memory"}
"""
```

3. **Update generate_command handler** (lines 245-310):
```python
@app.post("/generate-command", response_model=CommandResponse)
def generate_command(request: CommandRequest):
    # Validate OS
    if request.os not in ["linux", "windows"]:
        raise HTTPException(status_code=400, detail=f"Invalid OS: {request.os}")
    
    # Select instructions
    if request.os == "windows":
        system_instructions = WINDOWS_SYSTEM_INSTRUCTIONS
    else:
        system_instructions = LINUX_SYSTEM_INSTRUCTIONS
    
    full_prompt = f"{system_instructions}\n\nRequest: {request.query}"
    
    # Generate (existing provider logic)
    response = provider_manager.generate(full_prompt, timeout_seconds=120)
    ...
    
    # OS-aware risk classification
    risk_level_enum, risk_reason = classify_command(command, request.os)
    
    # OS-aware safer alternative
    safer_alternative = suggest_safer_alternative(command, request.os)
    
    # OS-aware impact analysis
    impact_context = analyze_impact(command, request.os)
    
    # Store OS in token metadata
    token = str(uuid.uuid4())
    PENDING_COMMANDS[token] = {
        "command": command,
        "risk_level": risk_level,
        "query": request.query,
        "os": request.os,  # NEW
        "created_at": time.time(),
    }
    
    return CommandResponse(
        ...,
        os=request.os,  # NEW field
    )
```

4. **Update confirm_execute handler** (lines 313-356):
```python
@app.post("/confirm-execute", response_model=ConfirmResponse)
def confirm_execute(request: ConfirmRequest):
    entry = PENDING_COMMANDS.pop(request.token, None)
    ...
    
    # Forward to core-engine with OS
    response = requests.post(
        f"{CORE_ENGINE_URL}/execute",
        json={
            "command": entry["command"],
            "risk_level": entry["risk_level"],
            "timeout_seconds": 30,
            "os": entry["os"],  # NEW
        },
        timeout=35,
    )
    ...
```

5. **Create windows_risk.py** (new file):
```python
"""Windows-specific risk classification."""
# (implementation from section 3.4)
```

6. **Update risk.py**:
```python
def classify_command(command: str, os: str = "linux") -> Tuple[RiskLevel, str]:
    if os == "windows":
        from intellishell_ai_service.windows_risk import classify_windows_command
        return classify_windows_command(command)
    else:
        # Existing Linux logic (rename to classify_linux_command for clarity)
        return classify_linux_command(command)

def suggest_safer_alternative(command: str, os: str = "linux") -> str:
    if os == "windows":
        # Windows-specific suggestions
        if re.search(r"Remove-Item.*-Recurse.*-Force", command):
            return "Use -Confirm to prompt for each item | Move to Recycle Bin instead"
        # ... more Windows patterns
    else:
        # Existing Linux suggestions
        ...
```

**Verification**: Unit tests for both OS paths, integration test with mock Rust backend

### 5.3 Phase 3: Rust Core Engine - Windows Execution Backend

**Files**:
- `core-engine/src/windows_execution.rs` (new)
- `core-engine/src/windows_telemetry.rs` (new)
- `core-engine/src/server.rs` (update)
- `core-engine/Cargo.toml` (update)

**Changes**:

1. **Update Cargo.toml**:
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["jobapi2", "processthreadsapi", "winnt", "handleapi", "synchapi"] }
windows = { version = "0.51", features = [
    "Win32_System_SystemInformation",
    "Win32_System_Performance",
    "Win32_NetworkManagement_IpHelper",
] }
wmi = "0.13"
```

2. **Implement windows_execution.rs** (section 3.2 design)

3. **Implement windows_telemetry.rs** (section 3.3 design)

4. **Update server.rs**:
   - Add `os` field to `ExecuteRequest` (with default)
   - Add `os` field to `CommandHistoryRecord`
   - Implement `execute_linux_pipeline()` (extract from existing)
   - Implement `execute_windows_pipeline()` (call windows_execution)
   - Update `execute_command()` to route by OS
   - Add `os` query param to `get_snapshot()`
   - Implement `get_linux_snapshot()` (existing logic)
   - Implement `get_windows_snapshot()` (call windows_telemetry)

5. **Database migration**:
```sql
ALTER TABLE command_history ADD COLUMN os TEXT DEFAULT 'linux' NOT NULL;
```

**Verification**:
- Unit tests for Windows execution (mock Job Objects)
- Integration test: generate Windows command, execute on Windows host
- Telemetry test: fetch snapshot?os=windows, validate WMI data

### 5.4 Phase 4: Frontend OS-Aware Telemetry

**Files**: 
- `dashboard/src/pages/MonitoringPage.jsx`
- `dashboard/src/pages/OverviewPage.jsx`
- `dashboard/src/hooks/useSystemSnapshot.js`

**Changes**:

1. **Add OS selector to MonitoringPage**:
```jsx
const [selectedOS, setSelectedOS] = useState('linux');
const { snapshot, isLoading, error } = useSystemSnapshot(selectedOS);
```

2. **Update useSystemSnapshot hook**:
```javascript
export function useSystemSnapshot(os = 'linux') {
  return useQuery({
    queryKey: ['snapshot', os],
    queryFn: () => rustApi.getSnapshot(os),
    refetchInterval: 5000,
  });
}
```

3. **Update rustApi.getSnapshot()**:
```javascript
async getSnapshot(os = 'linux') {
  const response = await rustClient.get('/snapshot', { params: { os } });
  return response.data;
}
```

4. **Add OS badge to telemetry display**:
```jsx
<div className="os-indicator">
  Currently monitoring: {selectedOS === 'linux' ? '🐧 Linux (WSL2)' : '🪟 Windows'}
</div>
```

**Verification**: Switch OS in UI, verify telemetry updates, check network requests include `?os=`

### 5.5 Phase 5: Integration & Cross-OS Safety

**Files**: All components

**Tasks**:

1. **Prevent cross-OS command leakage**:
   - Frontend: Disable execute button if OS changed after generation
   - Backend: Validate `token.os == execution_request.os`
   - UI warning: "Command generated for {OS1}, cannot execute on {OS2}"

2. **Add OS indicators everywhere**:
   - Command history: OS column/badge
   - Execution results: OS tag
   - Risk assessment: "Risk level (Linux)" vs "Risk level (Windows)"

3. **Update error messages**:
   - "Windows execution not available on Linux-only installations"
   - "Linux execution requires WSL2 on Windows"

4. **Add health check**:
```python
@app.get("/health")
def health_check():
    return {
        "status": "ok",
        "ai_available": provider_manager is not None,
        "core_engine_reachable": check_core_engine(),
        "supported_os": ["linux", "windows"],  # NEW
        "current_platform": sys.platform,  # NEW
    }
```

**Verification**: End-to-end test suite covering all OS paths

---

## 6. Testing Strategy

### 6.1 Unit Tests

**Python AI Service**:
```python
# tests/test_dual_os_generation.py
def test_generate_linux_command():
    response = client.post("/generate-command", json={"query": "list files", "os": "linux"})
    assert response.json()["command"].startswith("ls")
    assert response.json()["os"] == "linux"

def test_generate_windows_command():
    response = client.post("/generate-command", json={"query": "list files", "os": "windows"})
    assert "Get-ChildItem" in response.json()["command"]
    assert response.json()["os"] == "windows"

def test_risk_classification_linux():
    risk, reason = classify_command("rm -rf /", "linux")
    assert risk == RiskLevel.CRITICAL

def test_risk_classification_windows():
    risk, reason = classify_command("Remove-Item C:\\ -Recurse -Force", "windows")
    assert risk == RiskLevel.CRITICAL

def test_os_mismatch_rejected():
    response = client.post("/generate-command", json={"query": "...", "os": "invalid"})
    assert response.status_code == 400
```

**Rust Core Engine**:
```rust
#[tokio::test]
async fn test_linux_execution_direct() {
    let req = ExecuteRequest {
        command: "echo test".to_string(),
        risk_level: "safe".to_string(),
        timeout_seconds: 10,
        os: "linux".to_string(),
    };
    let result = execute_command(State(test_state()), Json(req)).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0.method, "direct");
}

#[cfg(windows)]
#[tokio::test]
async fn test_windows_execution_direct() {
    let req = ExecuteRequest {
        command: "Write-Output 'test'".to_string(),
        risk_level: "safe".to_string(),
        timeout_seconds: 10,
        os: "windows".to_string(),
    };
    let result = execute_windows_pipeline(&req, &test_state()).await;
    assert!(result.is_ok());
    assert!(result.unwrap().stdout.contains("test"));
}

#[tokio::test]
async fn test_os_routing() {
    // Test that OS parameter correctly routes to respective pipelines
    let state = test_state();
    
    let linux_req = ExecuteRequest { os: "linux".to_string(), ... };
    let result = execute_command(State(state.clone()), Json(linux_req)).await;
    // Verify Linux execution path used
    
    let windows_req = ExecuteRequest { os: "windows".to_string(), ... };
    let result = execute_command(State(state.clone()), Json(windows_req)).await;
    // Verify Windows execution path used (or error if not on Windows)
}
```

### 6.2 Integration Tests

**End-to-End Flow**:
```python
def test_linux_e2e():
    # 1. Generate
    gen_resp = client.post("/generate-command", json={"query": "list files", "os": "linux"})
    token = gen_resp.json()["confirmation_token"]
    
    # 2. Execute
    exec_resp = client.post("/confirm-execute", json={"token": token})
    assert exec_resp.json()["executed"] == True
    
    # 3. Verify history
    hist_resp = client.get("/history")
    assert hist_resp.json()[0]["os"] == "linux"

def test_windows_e2e():
    gen_resp = client.post("/generate-command", json={"query": "list files", "os": "windows"})
    token = gen_resp.json()["confirmation_token"]
    exec_resp = client.post("/confirm-execute", json={"token": token})
    assert exec_resp.json()["executed"] == True

def test_cross_os_prevention():
    # Generate for Linux
    gen_resp = client.post("/generate-command", json={"query": "list files", "os": "linux"})
    token = gen_resp.json()["confirmation_token"]
    
    # Modify token metadata to Windows (simulates attack)
    PENDING_COMMANDS[token]["os"] = "windows"
    
    # Execute should detect mismatch
    exec_resp = client.post("/confirm-execute", json={"token": token})
    assert "mismatch" in exec_resp.json()["message"].lower()
```

**Telemetry Tests**:
```rust
#[tokio::test]
async fn test_linux_telemetry() {
    let snapshot = get_linux_snapshot(&test_state()).await.unwrap();
    assert!(snapshot.cpu_percent >= 0.0 && snapshot.cpu_percent <= 100.0);
    assert!(snapshot.memory_total_bytes > 0);
    assert!(snapshot.processes.len() > 0);
}

#[cfg(windows)]
#[tokio::test]
async fn test_windows_telemetry() {
    let snapshot = get_windows_snapshot(&test_state()).await.unwrap();
    assert!(snapshot.cpu_percent >= 0.0 && snapshot.cpu_percent <= 100.0);
    assert!(snapshot.memory_total_bytes > 0);
}
```

### 6.3 Manual Testing Checklist

- [ ] UI: OS selection buttons visible and functional
- [ ] UI: Selected OS persists during generation
- [ ] UI: OS badge displayed in results
- [ ] UI: Switching OS clears previous results
- [ ] Generation: Linux query generates `ls`, `grep`, etc.
- [ ] Generation: Windows query generates `Get-ChildItem`, `Select-String`, etc.
- [ ] Risk: `rm -rf /` classified as CRITICAL (Linux)
- [ ] Risk: `Remove-Item C:\ -Recurse -Force` classified as CRITICAL (Windows)
- [ ] Execution: Linux command executes in WSL2
- [ ] Execution: Windows command executes on host PowerShell
- [ ] Execution: HIGH risk commands blocked on both OSes
- [ ] Execution: Cross-OS execution prevented (error message)
- [ ] Telemetry: Linux snapshot shows `/proc` data
- [ ] Telemetry: Windows snapshot shows WMI/CIM data
- [ ] History: OS column populated correctly
- [ ] History: Can filter by OS

---

## 7. Risk Assessment

### 7.1 Risks to Existing Functionality

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Breaking backward compatibility** | Existing API clients fail | Add `os` field with default value `"linux"` |
| **Linux execution regresses** | WSL2 commands break | Extensive regression testing, feature flags |
| **Database migration fails** | Command history lost | Backup database before migration, test on copy |
| **Windows execution unavailable on Linux** | Feature unusable in Linux-only installs | Graceful degradation, clear error messages |
| **Cross-OS command leakage** | Linux command executed on Windows | Token validation, UI prevention, backend validation |
| **Performance degradation** | Telemetry polling slows system | Optimize Windows WMI queries, cache results |

### 7.2 Security Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **PowerShell injection** | HIGH | Never concatenate user input into commands; use parameterized execution |
| **Privilege escalation** | HIGH | Validate commands don't request admin rights without explicit user consent |
| **Windows Defender bypass** | MEDIUM | Block `Set-ExecutionPolicy Unrestricted -Scope LocalMachine` |
| **Registry tampering** | HIGH | Classify `reg delete HKLM:\` as CRITICAL |
| **Job Object escape** | LOW | Windows kernel enforces Job Object limits; no known escapes |

### 7.3 Operational Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| **WSL2 vs Windows confusion** | User executes on wrong OS | Prominent OS indicator, confirmation dialog |
| **Windows-only deployment** | Linux features unavailable | Document platform requirements, conditional compilation |
| **Mixed-OS history confusion** | Users can't find commands | Add OS filter to history UI |
| **AI generates wrong OS commands** | User frustration | Extensive prompt engineering, validation layer |

---

## 8. Open Questions & Decisions Needed

### 8.1 Resolved Decisions

1. **Q: Should we support cmd.exe in addition to PowerShell?**  
   **A: No. PowerShell is more powerful, modern, and provides better output parsing. Cmd.exe support can be added later if needed.**

2. **Q: How to handle WSL2 interop (wsl.exe command)?**  
   **A: Not in initial implementation. Users can manually select Linux and run commands in WSL. Future enhancement: auto-detect and suggest.**

3. **Q: Should telemetry show both OSes simultaneously?**  
   **A: No. Single OS view at a time. Avoids UI clutter and resource overhead. Users can switch via toggle.**

4. **Q: What if user switches OS mid-generation?**  
   **A: Clear generated results when OS changes. Prevent execution of stale commands.**

5. **Q: Default OS on first load?**  
   **A: Detect platform via `sys.platform` (backend) or `navigator.platform` (frontend). WSL → Linux, Windows → Windows.**

### 8.2 Implementation Details to Finalize

1. **Windows Job Object configuration**:
   - Memory limit enforcement strategy (commit vs working set)
   - CPU quota granularity (requires Windows 10 RS5+)
   - I/O limit support (requires Windows 10+)

2. **Windows telemetry polling frequency**:
   - WMI queries can be slow; cache duration vs accuracy tradeoff
   - Performance counter vs GetSystemTimes for CPU

3. **Error message localization**:
   - Windows returns localized error messages; parse in neutral locale

4. **Frontend OS selector placement**:
   - Mockup needed for final design approval
   - Persist selected OS in localStorage?

---

## 9. Dependencies & Prerequisites

### 9.1 System Requirements

**Development Environment**:
- Windows 10/11 with WSL2 (Ubuntu recommended)
- Rust 1.88+ (with `x86_64-pc-windows-gnu` target installed)
- Python 3.10+ (in both WSL and Windows)
- Node.js 18+ (in WSL for frontend dev)
- PowerShell 7+ (for Windows execution testing)

**Production Environment**:
- Same as development for full dual-OS support
- Linux-only deployment: Windows features gracefully disabled

### 9.2 Library Dependencies

**Rust (Cargo.toml additions)**:
```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["jobapi2", "processthreadsapi", "winnt", "handleapi", "synchapi", "winerror"] }
windows = { version = "0.51", features = [
    "Win32_System_SystemInformation",
    "Win32_System_Threading",
    "Win32_System_Performance",
    "Win32_NetworkManagement_IpHelper",
    "Win32_Security",
] }
wmi = "0.13"
```

**Python (pyproject.toml additions)**:
```toml
# No new dependencies required (existing FastAPI, requests, pydantic sufficient)
```

**Frontend (package.json additions)**:
```json
// No new dependencies required
```

### 9.3 External Services

- **Groq API** (existing): No changes
- **Ollama** (existing): No changes
- **Windows WMI Service**: Must be running (default on Windows)

---

## 10. Success Criteria

### 10.1 Functional Requirements Met

- [ ] User can select Linux or Windows in UI
- [ ] AI generates OS-appropriate commands (bash vs PowerShell)
- [ ] Risk classifier recognizes OS-specific dangerous patterns
- [ ] Linux commands execute in WSL2
- [ ] Windows commands execute on Windows host
- [ ] Cross-OS execution is prevented
- [ ] Telemetry displays OS-specific system information
- [ ] Command history records OS for each command

### 10.2 Performance Requirements

- [ ] Command generation latency < 5s (existing baseline maintained)
- [ ] Windows execution latency < 2s for simple commands
- [ ] Windows telemetry refresh < 1s (vs 200ms for Linux /proc)
- [ ] UI OS switch < 100ms
- [ ] No memory leaks in Windows Job Object handling

### 10.3 Quality Requirements

- [ ] 90%+ test coverage on new code
- [ ] Zero regressions in existing Linux functionality
- [ ] Error messages clear and actionable
- [ ] Documentation updated (README, API docs)
- [ ] Code review approved by 2+ maintainers

---

## 11. Conclusion

This architecture document provides a comprehensive design for dual-OS support in IntelliShell. The implementation strategy minimizes changes to existing Linux functionality while adding robust Windows support through:

1. **OS parameter threading** through all layers (UI → AI → Execution)
2. **Separate execution backends** (WSL2 bash vs Windows PowerShell)
3. **Dual risk classification** (Linux patterns vs Windows patterns)
4. **OS-specific telemetry** (procfs vs WMI/CIM)
5. **Cross-OS safety validation** at every stage

**Key Architectural Decisions**:
- OS routing occurs BEFORE risk routing (prevents cross-OS leakage)
- PowerShell as Windows shell (modern, consistent)
- Job Objects for Windows sandboxing (closest equivalent to cgroups)
- WMI/CIM for Windows telemetry (native, reliable)
- Graceful degradation when platform unavailable

**Estimated Implementation Effort**:
- Phase 1 (Frontend UI): 2-3 days
- Phase 2 (Python AI Service): 3-4 days
- Phase 3 (Rust Windows Backend): 5-7 days
- Phase 4 (Frontend Telemetry): 2-3 days
- Phase 5 (Integration & Testing): 3-5 days
- **Total: 15-22 days** (3-4 weeks)

**Next Steps**:
1. Review and approve this architecture
2. Create detailed implementation plan (FEAT decomposition)
3. Begin Phase 1 (Frontend OS selection UI)
4. Iterative implementation with continuous integration testing

---

## Appendix A: File Change Summary

| File | Change Type | LOC Impact |
|------|-------------|-----------|
| `dashboard/src/pages/AssistantPage.jsx` | Modify | +50 |
| `dashboard/src/pages/MonitoringPage.jsx` | Modify | +30 |
| `dashboard/src/hooks/useSystemSnapshot.js` | Modify | +10 |
| `dashboard/src/lib/api.js` | Modify | +20 |
| `ai-service/.../main.py` | Modify | +150 |
| `ai-service/.../risk.py` | Modify | +50 |
| `ai-service/.../windows_risk.py` | Create | +200 |
| `ai-service/.../impact.py` | Modify | +30 |
| `core-engine/src/server.rs` | Modify | +200 |
| `core-engine/src/windows_execution.rs` | Create | +300 |
| `core-engine/src/windows_telemetry.rs` | Create | +250 |
| `core-engine/src/lib.rs` | Modify | +10 |
| `core-engine/Cargo.toml` | Modify | +15 |
| **Total** | | **~1,315 LOC** |

---

## Appendix B: API Endpoint Reference

### Python AI Service (Port 8000)

| Endpoint | Method | Request | Response | Change |
|----------|--------|---------|----------|--------|
| `/health` | GET | - | `{status, ai_available, supported_os[]}` | Modified |
| `/generate-command` | POST | `{query, os}` | `{command, explanation, risk_level, os, ...}` | Modified |
| `/confirm-execute` | POST | `{token}` | `{executed, method, stdout, ...}` | Internal change |
| `/activity` | GET | - | `{events[]}` | No change |
| `/history` | GET | - | `{history[]}` | No change (proxies to Rust) |

### Rust Core Engine (Port 3000)

| Endpoint | Method | Request | Response | Change |
|----------|--------|---------|----------|--------|
| `/health` | GET | - | `{status, version, uptime_seconds}` | No change |
| `/snapshot` | GET | `?os=linux\|windows` | `{cpu_percent, memory_*, processes[], ...}` | Modified |
| `/execute` | POST | `{command, risk_level, timeout_seconds, os}` | `{executed, method, exit_code, stdout, ...}` | Modified |
| `/history` | GET | - | `{history[]}` (with `os` column) | Modified |
| `/processes` | GET | - | `{processes[]}` | No change |
| `/processes/:pid` | POST | `{signal?, priority?}` | `{success, message}` | No change |

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-30  
**Author**: Kiro AI Planning Agent  
**Status**: Ready for Implementation
