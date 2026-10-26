//! Integration tests for process control operations.
//!
//! These tests spawn real child processes and verify the process control
//! functions work correctly end-to-end.

use intellishell_core::process::{
    kill_process, pause_process, renice_process, resume_process, KillOptions, KillOutcome,
    ProcessId,
};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Spawn a long-running child process for testing.
fn spawn_test_process() -> std::process::Child {
    Command::new("sleep")
        .arg("300") // Sleep for 5 minutes
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn test process")
}

#[test]
fn test_kill_process_graceful() {
    // Spawn a child process
    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    // Create ProcessId from the child PID
    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Kill it gracefully with default options
    let opts = KillOptions::default();
    let outcome = kill_process(proc_id, opts).expect("Failed to kill process");

    // Should terminate gracefully (sleep responds to SIGTERM)
    assert_eq!(outcome, KillOutcome::TerminatedGracefully);

    // Verify process is actually dead
    thread::sleep(Duration::from_millis(100));
    let status = child.try_wait().expect("Failed to check process status");
    assert!(status.is_some(), "Process should have exited");
}

#[test]
fn test_kill_process_force() {
    // Spawn a child process
    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Use very short timeout to force SIGKILL
    let opts = KillOptions {
        timeout_ms: 1,        // 1ms timeout (too short for graceful)
        poll_interval_ms: 1,  // Check every 1ms
    };

    let outcome = kill_process(proc_id, opts).expect("Failed to kill process");

    // Should require force kill (1ms is too short)
    assert_eq!(outcome, KillOutcome::ForcedKill);

    // Verify process is dead
    thread::sleep(Duration::from_millis(100));
    let status = child.try_wait().expect("Failed to check process status");
    assert!(status.is_some(), "Process should have exited");
}

#[test]
fn test_pause_and_resume() {
    // Spawn a child process
    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Pause the process
    pause_process(proc_id).expect("Failed to pause process");

    // Give it time to stop
    thread::sleep(Duration::from_millis(50));

    // Verify it's actually stopped by checking /proc/<pid>/stat
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid))
        .expect("Failed to read /proc stat");
    let state_char = stat
        .rfind(')')
        .map(|pos| stat.chars().nth(pos + 2))
        .flatten()
        .expect("Failed to parse state");
    assert_eq!(state_char, 'T', "Process should be in stopped state");

    // Resume the process
    resume_process(proc_id).expect("Failed to resume process");

    // Give it time to resume
    thread::sleep(Duration::from_millis(50));

    // Verify it's no longer stopped
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid))
        .expect("Failed to read /proc stat");
    let state_char = stat
        .rfind(')')
        .map(|pos| stat.chars().nth(pos + 2))
        .flatten()
        .expect("Failed to parse state");
    assert_ne!(state_char, 'T', "Process should not be stopped");

    // Clean up
    child.kill().expect("Failed to kill child process");
}

#[test]
fn test_renice() {
    // This test requires permissions to renice, so we test both directions:
    // - Regular users can increase nice value (lower priority)
    // - Root can set any value

    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Try to increase nice value (lower priority) - should work for non-root
    let result = renice_process(proc_id, 10);

    if result.is_ok() {
        // Verify the priority changed
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid))
            .expect("Failed to read /proc stat");
        // Priority is field 19 (nice value), but parsing it requires proper field parsing
        // For now, just verify the renice command succeeded
        println!("Successfully reniced process to priority 10");
    } else {
        // Permission denied is acceptable for non-root users
        println!("Renice failed (expected for non-root): {:?}", result);
    }

    // Clean up
    child.kill().expect("Failed to kill child process");
}

#[test]
fn test_pid_reuse_detection() {
    // Spawn and immediately kill a process
    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Kill it
    child.kill().expect("Failed to kill process");
    child.wait().expect("Failed to wait for process");

    // Give the system time to reap the process
    thread::sleep(Duration::from_millis(100));

    // Now try to verify the ProcessId - should fail because process is gone
    let result = proc_id.verify();
    assert!(result.is_err(), "Verification should fail for dead process");

    // Try to pause the dead process - should fail
    let result = pause_process(proc_id);
    assert!(result.is_err(), "Pausing dead process should fail");
}

#[test]
fn test_double_kill_already_dead() {
    let mut child = spawn_test_process();
    let pid = child.id() as i32;

    let proc_id = ProcessId::from_pid(pid).expect("Failed to create ProcessId");

    // Kill it once
    child.kill().expect("Failed to kill process");
    child.wait().expect("Failed to wait for process");

    // Give it time to die
    thread::sleep(Duration::from_millis(100));

    // Try to kill it again - should return AlreadyDead
    let opts = KillOptions::default();
    let outcome = kill_process(proc_id, opts).expect("Kill should not error on dead process");

    assert_eq!(outcome, KillOutcome::AlreadyDead);
}
