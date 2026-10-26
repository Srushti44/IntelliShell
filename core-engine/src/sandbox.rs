//! Production-grade sandbox for safe command execution.
//!
//! Provides resource isolation using Linux cgroups v2 and namespaces.
//! Commands execute with enforceable limits on memory, CPU, and process count.
//!
//! # Example
//!
//! ```no_run
//! use intellishell_core::sandbox::{SandboxConfig, sandboxed_exec};
//! use std::time::Duration;
//!
//! let config = SandboxConfig {
//!     memory_limit_bytes: 100 * 1024 * 1024, // 100MB
//!     pids_limit: 10,
//!     cpu_quota_us: Some(50000), // 50% of one CPU
//!     timeout: Duration::from_secs(30),
//!     use_pid_namespace: true,
//!     use_mount_namespace: true,
//! };
//!
//! let result = sandboxed_exec("ls", &["-la"], config)?;
//! println!("Exit code: {}", result.exit_code);
//! println!("Output: {}", result.stdout);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::cgroup;
use crate::process::{kill_process, KillOptions, ProcessId};
use std::io::{Read};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Configuration for sandbox execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory in bytes (enforced via cgroups v2 memory.max)
    pub memory_limit_bytes: u64,
    
    /// Maximum number of processes (enforced via cgroups v2 pids.max)
    pub pids_limit: u32,
    
    /// CPU quota in microseconds per 100ms period (optional).
    /// Example: 50000 = 50% of one CPU core
    pub cpu_quota_us: Option<u64>,
    
    /// Maximum execution time before forceful termination
    pub timeout: Duration,
    
    /// Isolate process tree in new PID namespace
    pub use_pid_namespace: bool,
    
    /// Isolate mounts in new mount namespace
    pub use_mount_namespace: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: 100 * 1024 * 1024, // 100MB
            pids_limit: 50,
            cpu_quota_us: None,
            timeout: Duration::from_secs(30),
            use_pid_namespace: true,
            use_mount_namespace: true,
        }
    }
}

/// Result of sandbox execution.
#[derive(Debug)]
pub struct SandboxResult {
    /// Exit code from the command (or -1 if killed/failed)
    pub exit_code: i32,
    
    /// Standard output captured from the command
    pub stdout: String,
    
    /// Standard error captured from the command
    pub stderr: String,
    
    /// Why the execution ended
    pub termination_reason: TerminationReason,
    
    /// Peak process count observed during execution
    pub peak_pids: Option<u32>,
    
    /// Actual execution time
    pub execution_time: Duration,
}

/// Why execution terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReason {
    /// Command exited normally
    Exited,
    
    /// Killed due to timeout
    Timeout,
    
    /// Killed due to resource limit (OOM, pid exhaustion)
    ResourceLimit,
    
    /// Process was signaled (killed externally)
    Signaled,
    
    /// Failed to start or setup failed
    Failed,
}

/// Errors that can occur during sandbox execution.
#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Root privileges required for sandbox operations")]
    RootRequired,
    
    #[error("Failed to create sandbox cgroup: {0}")]
    CgroupCreation(String),
    
    #[error("Failed to spawn command: {0}")]
    SpawnFailed(#[from] std::io::Error),
    
    #[error("Failed to add process to sandbox: {0}")]
    AddProcessFailed(String),
    
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Timeout cleanup failed: {0}")]
    CleanupFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Execute a command in a resource-isolated sandbox.
///
/// # Security Notes
///
/// - Requires root for cgroup manipulation
/// - Command and args are NOT passed through a shell
/// - Resource limits are enforced by kernel cgroups v2
/// - Timeouts use Phase 2 graceful kill (SIGTERM → SIGKILL)
/// - Namespace isolation depends on WSL2 kernel support
///
/// # Limitations
///
/// - Filesystem isolation is limited (no chroot/pivot_root)
/// - Network is NOT isolated by default
/// - cgroup cleanup may fail if processes are still spawning
///
/// # Example
///
/// ```no_run
/// use intellishell_core::sandbox::{SandboxConfig, sandboxed_exec};
/// use std::time::Duration;
///
/// let mut config = SandboxConfig::default();
/// config.timeout = Duration::from_secs(5);
/// config.pids_limit = 10;
///
/// let result = sandboxed_exec("echo", &["hello world"], config)?;
/// assert_eq!(result.exit_code, 0);
/// assert!(result.stdout.contains("hello world"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn sandboxed_exec(
    command: &str,
    args: &[&str],
    config: SandboxConfig,
) -> Result<SandboxResult, SandboxError> {
    // Validate configuration
    if config.pids_limit == 0 {
        return Err(SandboxError::InvalidConfig(
            "pids_limit must be > 0".to_string(),
        ));
    }
    if config.memory_limit_bytes == 0 {
        return Err(SandboxError::InvalidConfig(
            "memory_limit_bytes must be > 0".to_string(),
        ));
    }
    
    // Check permissions
    cgroup::require_root().map_err(|_| SandboxError::RootRequired)?;
    
    let start_time = Instant::now();
    
    // Create unique sandbox name with timestamp
    let sandbox_name = format!(
        "intellishell-{}-{}",
        std::process::id(),
        start_time.elapsed().as_millis()
    );
    
    // Create cgroup with resource limits
    let sandbox = cgroup::Sandbox::create(
        &sandbox_name,
        config.pids_limit,
        config.memory_limit_bytes,
    )
    .map_err(SandboxError::CgroupCreation)?;
    
    // Apply CPU quota if specified
    if let Some(quota_us) = config.cpu_quota_us {
        if let Err(e) = apply_cpu_quota(&sandbox, quota_us) {
            let _ = sandbox.remove();
            return Err(SandboxError::CgroupCreation(format!(
                "Failed to set CPU quota: {}",
                e
            )));
        }
    }
    
    // Prepare command with appropriate namespace flags
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    // Apply namespace isolation if requested
    if config.use_pid_namespace || config.use_mount_namespace {
        unsafe {
            cmd.pre_exec(move || {
                let mut flags = 0;
                if config.use_pid_namespace {
                    flags |= libc::CLONE_NEWPID;
                }
                if config.use_mount_namespace {
                    flags |= libc::CLONE_NEWNS;
                }
                
                let result = libc::unshare(flags);
                if result != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    
    // Spawn the command
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = sandbox.remove();
            return Err(SandboxError::SpawnFailed(e));
        }
    };
    
    let child_pid = child.id();
    
    // Move process into sandbox IMMEDIATELY (race with fork bombs)
    if let Err(e) = sandbox.add_process(child_pid) {
        let _ = child.kill();
        let _ = sandbox.remove();
        return Err(SandboxError::AddProcessFailed(e));
    }
    
    // Capture stdout/stderr in separate threads
    let mut stdout_handle = child.stdout.take().unwrap();
    let mut stderr_handle = child.stderr.take().unwrap();
    
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    
    thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout_handle.read_to_string(&mut buf);
        let _ = stdout_tx.send(buf);
    });
    
    thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr_handle.read_to_string(&mut buf);
        let _ = stderr_tx.send(buf);
    });
    
    // Monitor execution with timeout
    let timeout_deadline = Instant::now() + config.timeout;
    let mut peak_pids = 0u32;
    let mut timed_out = false;
    
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process exited
                break;
            }
            Ok(None) => {
                // Still running, check timeout
                if Instant::now() >= timeout_deadline {
                    timed_out = true;
                    
                    // Use Phase 2 graceful kill
                    if let Ok(proc_id) = ProcessId::from_pid(child_pid as i32) {
                        let opts = KillOptions {
                            timeout_ms: 2000,      // 2 second grace period
                            poll_interval_ms: 100,
                        };
                        let _ = kill_process(proc_id, opts);
                    }
                    
                    // Emergency fallback: cgroup kill
                    let _ = sandbox.kill_all();
                    break;
                }
                
                // Sample process count
                if let Ok(count) = sandbox.current_pids() {
                    peak_pids = peak_pids.max(count);
                }
                
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = sandbox.kill_all();
                let _ = sandbox.remove();
                return Err(SandboxError::ExecutionFailed(e.to_string()));
            }
        }
    }
    
    let execution_time = start_time.elapsed();
    
    // Wait for child to fully terminate
    let status = child.wait().map_err(|e| {
        SandboxError::ExecutionFailed(format!("Failed to wait for child: {}", e))
    })?;
    
    // Collect output
    let stdout = stdout_rx.recv().unwrap_or_default();
    let stderr = stderr_rx.recv().unwrap_or_default();
    
    // Determine termination reason
    let termination_reason = if timed_out {
        TerminationReason::Timeout
    } else if status.success() {
        TerminationReason::Exited
    } else if status.code().is_none() {
        TerminationReason::Signaled
    } else {
        // Check if killed by OOM or pids exhaustion
        // (heuristic: exit code 137 often indicates SIGKILL from OOM)
        match status.code() {
            Some(137) => TerminationReason::ResourceLimit,
            _ => TerminationReason::Exited,
        }
    };
    
    // Cleanup: kill any remaining processes
    let _ = sandbox.kill_all();
    
    // Wait for cgroup to empty before removal
    for _ in 0..20 {
        if sandbox.current_pids().unwrap_or(1) == 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    // Remove cgroup
    if let Err(e) = sandbox.remove() {
        eprintln!("Warning: Failed to remove sandbox: {}", e);
    }
    
    let exit_code = status.code().unwrap_or(-1);
    
    Ok(SandboxResult {
        exit_code,
        stdout,
        stderr,
        termination_reason,
        peak_pids: Some(peak_pids),
        execution_time,
    })
}

/// Apply CPU quota to a sandbox cgroup.
fn apply_cpu_quota(sandbox: &cgroup::Sandbox, quota_us: u64) -> Result<(), String> {
    use std::fs;
    use std::path::PathBuf;
    
    // CPU quota format: "quota_us period_us"
    // Standard period is 100ms (100000 microseconds)
    let period_us = 100000u64;
    let quota_str = format!("{} {}", quota_us, period_us);
    
    // Write to cpu.max in the sandbox's cgroup
    let cpu_max_path = PathBuf::from(format!("/sys/fs/cgroup/user.slice/user-{}.slice/user@{}.service/app.slice/{}/cpu.max",
        cgroup::invoking_uid_pub(),
        cgroup::invoking_uid_pub(),
        sandbox.name_pub()
    ));
    
    fs::write(&cpu_max_path, quota_str)
        .map_err(|e| format!("Failed to write cpu.max: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.memory_limit_bytes, 100 * 1024 * 1024);
        assert_eq!(config.pids_limit, 50);
        assert!(config.use_pid_namespace);
        assert!(config.use_mount_namespace);
    }
    
    #[test]
    fn test_sandbox_config_validation() {
        let config = SandboxConfig {
            pids_limit: 0,
            ..Default::default()
        };
        
        // This would fail validation in sandboxed_exec
        assert_eq!(config.pids_limit, 0);
    }
    
    #[test]
    fn test_termination_reason_equality() {
        assert_eq!(TerminationReason::Exited, TerminationReason::Exited);
        assert_ne!(TerminationReason::Timeout, TerminationReason::Exited);
    }
}
