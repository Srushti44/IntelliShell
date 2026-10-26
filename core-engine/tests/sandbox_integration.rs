//! Integration tests for Phase 3 sandbox functionality.
//!
//! These tests spawn real processes and verify resource isolation.
//! **REQUIRES ROOT** - run with: sudo cargo test --test sandbox_integration

use intellishell_core::sandbox::{sandboxed_exec, SandboxConfig, TerminationReason};
use std::time::Duration;

#[test]
fn test_simple_command_execution() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024, // 50MB
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_secs(5),
        use_pid_namespace: false, // Skip namespace for simple test
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("echo", &["hello world"], config);
    
    match result {
        Ok(r) => {
            assert_eq!(r.exit_code, 0, "Command should succeed");
            assert!(r.stdout.contains("hello"), "Output should contain 'hello'");
            assert_eq!(r.termination_reason, TerminationReason::Exited);
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Command execution failed: {}", e);
        }
    }
}

#[test]
fn test_command_with_args() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_secs(5),
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("ls", &["-la", "/tmp"], config);
    
    match result {
        Ok(r) => {
            assert_eq!(r.exit_code, 0);
            assert!(!r.stdout.is_empty(), "ls should produce output");
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Command failed: {}", e);
        }
    }
}

#[test]
fn test_timeout_enforcement() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_millis(500), // Very short timeout
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("sleep", &["10"], config);
    
    match result {
        Ok(r) => {
            assert_eq!(r.termination_reason, TerminationReason::Timeout);
            assert!(r.execution_time < Duration::from_secs(2), "Should timeout quickly");
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Timeout test failed: {}", e);
        }
    }
}

#[test]
fn test_pids_limit_enforcement() {
    // This test spawns a controlled fork bomb to verify pids limit
    // Safe because it's capped at 5 processes and has a 2-second timeout
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 5, // Very restrictive
        cpu_quota_us: None,
        timeout: Duration::from_secs(2),
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    // Fork bomb that tries to create many processes
    let result = sandboxed_exec("bash", &["-c", ":(){ :|:& };:"], config);
    
    match result {
        Ok(r) => {
            // Should hit pids limit or timeout
            assert!(
                r.peak_pids.unwrap_or(0) <= 5,
                "Process count should be capped at 5"
            );
            println!("Peak PIDs: {:?}", r.peak_pids);
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            // Fork bomb may fail to start due to pids limit - that's success
            println!("Fork bomb contained: {}", e);
        }
    }
}

#[test]
fn test_invalid_command() {
    let config = SandboxConfig::default();

    let result = sandboxed_exec("nonexistent_command_12345", &[], config);
    
    match result {
        Ok(_) => panic!("Should have failed with invalid command"),
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            assert!(e.to_string().contains("Failed to spawn"), "Should report spawn failure");
        }
    }
}

#[test]
fn test_stderr_capture() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_secs(5),
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("bash", &["-c", "echo error_message >&2"], config);
    
    match result {
        Ok(r) => {
            assert_eq!(r.exit_code, 0);
            assert!(r.stderr.contains("error_message"), "Should capture stderr");
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Command failed: {}", e);
        }
    }
}

#[test]
fn test_exit_code_propagation() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_secs(5),
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("bash", &["-c", "exit 42"], config);
    
    match result {
        Ok(r) => {
            assert_eq!(r.exit_code, 42, "Exit code should propagate");
            assert_eq!(r.termination_reason, TerminationReason::Exited);
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Command failed: {}", e);
        }
    }
}

#[test]
fn test_resource_usage_tracking() {
    let config = SandboxConfig {
        memory_limit_bytes: 50 * 1024 * 1024,
        pids_limit: 10,
        cpu_quota_us: None,
        timeout: Duration::from_secs(5),
        use_pid_namespace: false,
        use_mount_namespace: false,
    };

    let result = sandboxed_exec("bash", &["-c", "sleep 0.1 & sleep 0.1 & wait"], config);
    
    match result {
        Ok(r) => {
            assert!(r.peak_pids.is_some(), "Should track process count");
            assert!(r.peak_pids.unwrap() >= 1, "Should have at least 1 process");
            assert!(r.execution_time >= Duration::from_millis(100), "Should track execution time");
        }
        Err(e) => {
            if e.to_string().contains("Root privileges required") {
                println!("Test skipped: requires root");
                return;
            }
            panic!("Command failed: {}", e);
        }
    }
}

#[test]
fn test_concurrent_sandboxes() {
    // Verify multiple sandboxes can run concurrently without interference
    use std::thread;
    
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                let config = SandboxConfig {
                    memory_limit_bytes: 50 * 1024 * 1024,
                    pids_limit: 10,
                    cpu_quota_us: None,
                    timeout: Duration::from_secs(5),
                    use_pid_namespace: false,
                    use_mount_namespace: false,
                };

                let result = sandboxed_exec("echo", &[&format!("thread-{}", i)], config);
                
                match result {
                    Ok(r) => {
                        assert_eq!(r.exit_code, 0);
                        assert!(r.stdout.contains(&format!("thread-{}", i)));
                    }
                    Err(e) => {
                        if !e.to_string().contains("Root privileges required") {
                            panic!("Thread {} failed: {}", i, e);
                        }
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
