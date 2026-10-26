//! Process control operations with PID-reuse safety.
//!
//! This module provides production-grade process management primitives:
//! - Graceful termination with timeout and fallback to force-kill
//! - Pause/resume with state verification
//! - Priority adjustment with readback verification
//! - PID-reuse mitigation via start-time validation
//!
//! # PID Reuse Safety
//!
//! Linux reuses PIDs. To mitigate targeting the wrong process, we validate
//! process identity using (PID, start_time) tuples read from /proc/<pid>/stat.
//! This reduces but does not eliminate TOCTOU races:
//!
//! - **Residual race**: Between our final start_time check and signal delivery,
//!   the target can exit and a new process can spawn with the same PID and
//!   (by chance) the same start_time value.
//! - **Probability**: Extremely low (requires PID wraparound + clock tick alignment).
//! - **Future improvement**: Use `pidfd_open(2)` + `pidfd_send_signal(2)` on kernels >= 5.1.

use nix::errno::Errno;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::fs;
use std::thread;
use std::time::Duration;
use thiserror::Error;

/// Process identity: PID + start time (clock ticks since boot).
///
/// Start time is read from /proc/<pid>/stat field 22 (starttime).
/// This mitigates PID reuse by ensuring we target the same process instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId {
    pub pid: i32,
    pub start_time: u64,
}

impl ProcessId {
    /// Create a ProcessId and validate it exists with the given start time.
    ///
    /// # Errors
    ///
    /// Returns `ProcessError::ProcessNotFound` if the process doesn't exist or
    /// start time doesn't match.
    pub fn new(pid: i32, start_time: u64) -> Result<Self, ProcessError> {
        let proc_id = Self { pid, start_time };
        proc_id.verify()?;
        Ok(proc_id)
    }

    /// Create a ProcessId from just a PID, reading the current start time.
    ///
    /// # Errors
    ///
    /// Returns `ProcessError::ProcessNotFound` if the process doesn't exist.
    pub fn from_pid(pid: i32) -> Result<Self, ProcessError> {
        let start_time = read_start_time(pid)?;
        Ok(Self { pid, start_time })
    }

    /// Verify this process still exists with the same start time.
    ///
    /// # Errors
    ///
    /// Returns `ProcessError::ProcessNotFound` if verification fails.
    pub fn verify(&self) -> Result<(), ProcessError> {
        let current_start = read_start_time(self.pid)?;
        if current_start != self.start_time {
            return Err(ProcessError::ProcessNotFound(self.pid));
        }
        Ok(())
    }

    /// Convert to nix::unistd::Pid for signal operations.
    fn as_nix_pid(&self) -> Pid {
        Pid::from_raw(self.pid)
    }
}

/// Errors that can occur during process control operations.
#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Process {0} not found or PID reused")]
    ProcessNotFound(i32),

    #[error("Permission denied for process {0}")]
    PermissionDenied(i32),

    #[error("Failed to parse /proc/{0}/stat: {1}")]
    ProcParseFailed(i32, String),

    #[error("Process {0} state verification failed: expected {1}, got {2}")]
    StateVerificationFailed(i32, char, char),

    #[error("Process {0} did not respond to SIGTERM within timeout")]
    TerminationTimeout(i32),

    #[error("Signal operation failed: {0}")]
    SignalFailed(#[from] Errno),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid priority value: {0}")]
    InvalidPriority(i32),
}

/// Options for kill_process operation.
#[derive(Debug, Clone)]
pub struct KillOptions {
    /// How long to wait for SIGTERM before sending SIGKILL (in milliseconds).
    pub timeout_ms: u64,
    /// How often to poll process state during timeout (in milliseconds).
    pub poll_interval_ms: u64,
}

impl Default for KillOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,       // 5 seconds
            poll_interval_ms: 100,  // 100ms
        }
    }
}

/// Outcome of a kill_process operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillOutcome {
    /// Process terminated gracefully after SIGTERM.
    TerminatedGracefully,
    /// Process was force-killed with SIGKILL after timeout.
    ForcedKill,
    /// Process was already dead when we checked.
    AlreadyDead,
}

/// Gracefully terminate a process with SIGTERM, escalating to SIGKILL if needed.
///
/// # Process
///
/// 1. Verify process identity (PID + start time)
/// 2. Send SIGTERM
/// 3. Poll process state until it exits or timeout expires
/// 4. If timeout expires, send SIGKILL
/// 5. Return outcome
///
/// # Errors
///
/// Returns error if process identity verification fails or signal delivery fails.
/// Timeout is not an error; it results in `KillOutcome::ForcedKill`.
///
/// # Example
///
/// ```no_run
/// use intellishell_core::process::{ProcessId, KillOptions, kill_process};
///
/// let proc_id = ProcessId::from_pid(1234)?;
/// let opts = KillOptions::default();
/// let outcome = kill_process(proc_id, opts)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn kill_process(
    proc_id: ProcessId,
    options: KillOptions,
) -> Result<KillOutcome, ProcessError> {
    // Verify process exists before starting
    proc_id.verify()?;

    // Send SIGTERM
    let pid = proc_id.as_nix_pid();
    match signal::kill(pid, Signal::SIGTERM) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return Ok(KillOutcome::AlreadyDead),
        Err(Errno::EPERM) => return Err(ProcessError::PermissionDenied(proc_id.pid)),
        Err(e) => return Err(ProcessError::SignalFailed(e)),
    }

    // Poll until process exits or timeout
    let poll_interval = Duration::from_millis(options.poll_interval_ms);
    let max_polls = options.timeout_ms / options.poll_interval_ms;

    for _ in 0..max_polls {
        thread::sleep(poll_interval);

        // Check if process still exists
        match proc_id.verify() {
            Ok(()) => continue, // Still alive
            Err(ProcessError::ProcessNotFound(_)) => {
                return Ok(KillOutcome::TerminatedGracefully)
            }
            Err(e) => return Err(e),
        }
    }

    // Timeout expired, escalate to SIGKILL
    match signal::kill(pid, Signal::SIGKILL) {
        Ok(()) => Ok(KillOutcome::ForcedKill),
        Err(Errno::ESRCH) => Ok(KillOutcome::AlreadyDead),
        Err(Errno::EPERM) => Err(ProcessError::PermissionDenied(proc_id.pid)),
        Err(e) => Err(ProcessError::SignalFailed(e)),
    }
}

/// Pause (suspend) a process using SIGSTOP and verify it entered stopped state.
///
/// # Process
///
/// 1. Verify process identity
/// 2. Send SIGSTOP
/// 3. Read /proc/<pid>/stat and verify state is 'T' (stopped)
///
/// # Errors
///
/// Returns error if verification fails, signal delivery fails, or state
/// verification shows the process is not stopped.
///
/// # Example
///
/// ```no_run
/// use intellishell_core::process::{ProcessId, pause_process};
///
/// let proc_id = ProcessId::from_pid(1234)?;
/// pause_process(proc_id)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn pause_process(proc_id: ProcessId) -> Result<(), ProcessError> {
    // Verify process exists
    proc_id.verify()?;

    // Send SIGSTOP
    let pid = proc_id.as_nix_pid();
    match signal::kill(pid, Signal::SIGSTOP) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return Err(ProcessError::ProcessNotFound(proc_id.pid)),
        Err(Errno::EPERM) => return Err(ProcessError::PermissionDenied(proc_id.pid)),
        Err(e) => return Err(ProcessError::SignalFailed(e)),
    }

    // Verify process is stopped (state 'T')
    let state = read_process_state(proc_id.pid)?;
    if state != 'T' {
        return Err(ProcessError::StateVerificationFailed(
            proc_id.pid,
            'T',
            state,
        ));
    }

    Ok(())
}

/// Resume a paused process using SIGCONT and verify it's no longer stopped.
///
/// # Process
///
/// 1. Verify process identity
/// 2. Send SIGCONT
/// 3. Read /proc/<pid>/stat and verify state is not 'T'
///
/// # Errors
///
/// Returns error if verification fails, signal delivery fails, or state
/// verification shows the process is still stopped.
///
/// # Example
///
/// ```no_run
/// use intellishell_core::process::{ProcessId, resume_process};
///
/// let proc_id = ProcessId::from_pid(1234)?;
/// resume_process(proc_id)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn resume_process(proc_id: ProcessId) -> Result<(), ProcessError> {
    // Verify process exists
    proc_id.verify()?;

    // Send SIGCONT
    let pid = proc_id.as_nix_pid();
    match signal::kill(pid, Signal::SIGCONT) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return Err(ProcessError::ProcessNotFound(proc_id.pid)),
        Err(Errno::EPERM) => return Err(ProcessError::PermissionDenied(proc_id.pid)),
        Err(e) => return Err(ProcessError::SignalFailed(e)),
    }

    // Verify process is no longer stopped
    let state = read_process_state(proc_id.pid)?;
    if state == 'T' {
        return Err(ProcessError::StateVerificationFailed(
            proc_id.pid,
            'R', // Expected running or sleeping
            state,
        ));
    }

    Ok(())
}

/// Change process priority (nice value) and verify the change.
///
/// # Process
///
/// 1. Verify process identity
/// 2. Read current priority from /proc/<pid>/stat field 19
/// 3. Write new priority to /proc/<pid>/autogroup (if available) or use setpriority
/// 4. Read back and verify new priority matches
///
/// # Priority Range
///
/// Nice values range from -20 (highest priority) to 19 (lowest priority).
/// Regular users can only increase nice values (lower priority).
/// Root can set any value.
///
/// # Errors
///
/// Returns error if priority is out of range, verification fails, or operation
/// requires elevated privileges.
///
/// # Note
///
/// This implementation uses the `renice` command via std::process::Command
/// because nix crate doesn't expose setpriority. A future version could use
/// libc::setpriority directly.
///
/// # Example
///
/// ```no_run
/// use intellishell_core::process::{ProcessId, renice_process};
///
/// let proc_id = ProcessId::from_pid(1234)?;
/// renice_process(proc_id, 10)?; // Lower priority
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn renice_process(proc_id: ProcessId, priority: i32) -> Result<(), ProcessError> {
    // Validate priority range
    if !(-20..=19).contains(&priority) {
        return Err(ProcessError::InvalidPriority(priority));
    }

    // Verify process exists
    proc_id.verify()?;

    // Use renice command (nix crate doesn't expose setpriority)
    let output = std::process::Command::new("renice")
        .arg("-n")
        .arg(priority.to_string())
        .arg("-p")
        .arg(proc_id.pid.to_string())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Permission denied") {
            return Err(ProcessError::PermissionDenied(proc_id.pid));
        }
        return Err(ProcessError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("renice failed: {}", stderr),
        )));
    }

    // Verify new priority
    let current_priority = read_process_priority(proc_id.pid)?;
    if current_priority != priority {
        return Err(ProcessError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Priority verification failed: set {}, got {}",
                priority, current_priority
            ),
        )));
    }

    Ok(())
}

/// Read process start time from /proc/<pid>/stat field 22.
///
/// Start time is in clock ticks since boot.
fn read_start_time(pid: i32) -> Result<u64, ProcessError> {
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::ProcessNotFound(pid)
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            ProcessError::PermissionDenied(pid)
        } else {
            ProcessError::IoError(e)
        }
    })?;

    parse_stat_field(&stat, 22, pid)
}

/// Read process state from /proc/<pid>/stat field 3.
///
/// State is a single character: R (running), S (sleeping), D (disk sleep),
/// Z (zombie), T (stopped), etc.
fn read_process_state(pid: i32) -> Result<char, ProcessError> {
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::ProcessNotFound(pid)
        } else {
            ProcessError::IoError(e)
        }
    })?;

    // Field 3 is the state character, immediately after the comm field
    // Format: "pid (comm) state ..."
    let state_start = stat
        .rfind(')')
        .ok_or_else(|| {
            ProcessError::ProcParseFailed(pid, "missing ')' in stat".to_string())
        })?
        + 2; // Skip ") "

    stat.chars()
        .nth(state_start)
        .ok_or_else(|| ProcessError::ProcParseFailed(pid, "state field missing".to_string()))
}

/// Read process priority (nice value) from /proc/<pid>/stat field 19.
fn read_process_priority(pid: i32) -> Result<i32, ProcessError> {
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ProcessError::ProcessNotFound(pid)
        } else {
            ProcessError::IoError(e)
        }
    })?;

    let value: u64 = parse_stat_field(&stat, 19, pid)?;
    // Nice value is stored as unsigned but represents signed range -20..19
    // Convert from kernel representation
    Ok(value as i32 - 20)
}

/// Parse a field from /proc/<pid>/stat by field number.
///
/// /proc/pid/stat format is: "pid (comm) field3 field4 ... fieldN"
/// where comm can contain spaces and parentheses.
///
/// Fields are 1-indexed. Field 1 is pid, field 2 is comm, field 3 is state, etc.
fn parse_stat_field(stat: &str, field_num: usize, pid: i32) -> Result<u64, ProcessError> {
    // Find the last ')' to skip past comm field
    let after_comm = stat
        .rfind(')')
        .ok_or_else(|| {
            ProcessError::ProcParseFailed(pid, "missing ')' in stat".to_string())
        })?
        + 2; // Skip ") "

    // Field 3 starts here, so adjust field number
    let adjusted_field = field_num - 2;

    let fields: Vec<&str> = stat[after_comm..].split_whitespace().collect();

    if adjusted_field == 0 || adjusted_field > fields.len() {
        return Err(ProcessError::ProcParseFailed(
            pid,
            format!("field {} out of range", field_num),
        ));
    }

    fields[adjusted_field - 1]
        .parse::<u64>()
        .map_err(|e| ProcessError::ProcParseFailed(pid, format!("parse error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_id_from_pid() {
        // Use PID 1 (init/systemd) which should always exist
        let result = ProcessId::from_pid(1);
        assert!(result.is_ok());
        let proc_id = result.unwrap();
        assert_eq!(proc_id.pid, 1);
        assert!(proc_id.start_time > 0);
    }

    #[test]
    fn test_process_id_verify() {
        let proc_id = ProcessId::from_pid(1).unwrap();
        assert!(proc_id.verify().is_ok());
    }

    #[test]
    fn test_process_id_nonexistent() {
        let result = ProcessId::from_pid(999999);
        assert!(matches!(result, Err(ProcessError::ProcessNotFound(_))));
    }

    #[test]
    fn test_process_id_reuse_detection() {
        // Create a process ID with wrong start time
        let proc_id = ProcessId {
            pid: 1,
            start_time: 0, // Wrong start time
        };
        assert!(matches!(
            proc_id.verify(),
            Err(ProcessError::ProcessNotFound(_))
        ));
    }

    #[test]
    fn test_read_start_time() {
        let result = read_start_time(1);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_read_process_state() {
        let result = read_process_state(1);
        assert!(result.is_ok());
        let state = result.unwrap();
        // PID 1 should be running or sleeping
        assert!(matches!(state, 'R' | 'S'));
    }

    #[test]
    fn test_read_process_priority() {
        let result = read_process_priority(1);
        assert!(result.is_ok());
        let priority = result.unwrap();
        // Priority should be in valid range
        assert!((-20..=19).contains(&priority));
    }

    #[test]
    fn test_kill_options_default() {
        let opts = KillOptions::default();
        assert_eq!(opts.timeout_ms, 5000);
        assert_eq!(opts.poll_interval_ms, 100);
    }

    #[test]
    fn test_parse_stat_field() {
        // Realistic /proc/pid/stat line
        // Fields after comm: 3=state, 4=ppid, ..., 22=starttime
        let stat = "1 (systemd) S 0 1 1 0 -1 4194560 12345 67890 1 2 3 4 5 6 7 8 9 10 100";
        
        // Field 22: starttime (last field in this test)
        assert_eq!(parse_stat_field(stat, 22, 1).unwrap(), 100);
    }

    #[test]
    fn test_parse_stat_field_with_spaces_in_comm() {
        // Comm can contain spaces and parentheses
        let stat = "1234 (my (process) name) S 1 1234 1234 0 -1 4194304 1 2 3 4 5 6 7 8 9 10 11 12 999";
        // Field 22 is the 20th field after comm (field 3 starts after comm)
        assert_eq!(parse_stat_field(stat, 22, 1234).unwrap(), 999);
    }

    #[test]
    fn test_renice_invalid_priority() {
        let proc_id = ProcessId::from_pid(1).unwrap();
        assert!(matches!(
            renice_process(proc_id, -21),
            Err(ProcessError::InvalidPriority(-21))
        ));
        assert!(matches!(
            renice_process(proc_id, 20),
            Err(ProcessError::InvalidPriority(20))
        ));
    }
}
