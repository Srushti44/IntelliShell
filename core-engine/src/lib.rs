//! IntelliShell Core Library
//! 
//! Production-grade process control and system monitoring for safe command execution.

#![deny(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]
#![allow(unsafe_code)] // Allow for sandbox module which requires syscalls

pub mod process;
pub mod sandbox;
pub mod cgroup;

// Re-export main types
pub use process::{
    ProcessId, ProcessError, KillOptions, KillOutcome,
    kill_process, pause_process, resume_process, renice_process,
};

pub use sandbox::{
    SandboxConfig, SandboxResult, SandboxError, TerminationReason,
    sandboxed_exec,
};
