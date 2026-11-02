//! IntelliShell Core Library
//! 
//! Production-grade process control and system monitoring for safe command execution.
//! Supports dual operating systems: Linux (WSL2) and Windows (native host).

#![deny(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]
#![allow(unsafe_code)] // Allow for sandbox module which requires syscalls

pub mod process;
pub mod sandbox;
pub mod cgroup;

// Windows-specific modules
pub mod windows_executor;
pub mod windows_telemetry;
pub mod windows_risk;

// Re-export main types
pub use process::{
    ProcessId, ProcessError, KillOptions, KillOutcome,
    kill_process, pause_process, resume_process, renice_process,
};

pub use sandbox::{
    SandboxConfig, SandboxResult, SandboxError, TerminationReason,
    sandboxed_exec,
};

// Re-export Windows types
pub use windows_executor::{WindowsExecutionService, WindowsExecuteResponse, WindowsExecutionError};
pub use windows_telemetry::{WindowsTelemetryService, WindowsProcessInfo, TelemetryError};
pub use windows_risk::{WindowsRiskAnalyzer, WindowsRiskLevel};
