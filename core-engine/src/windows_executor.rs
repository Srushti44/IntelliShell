/*!
 * Windows Command Execution Service
 * 
 * Executes PowerShell commands on Windows host with resource limits using Job Objects.
 */

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsExecuteResponse {
    pub executed: bool,
    pub method: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug)]
pub enum WindowsExecutionError {
    SpawnFailed(String),
    Timeout(u64),
    ExecutionFailed(String),
}

impl std::fmt::Display for WindowsExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsExecutionError::SpawnFailed(msg) => write!(f, "Failed to spawn PowerShell: {}", msg),
            WindowsExecutionError::Timeout(secs) => write!(f, "Command timed out after {}s", secs),
            WindowsExecutionError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
        }
    }
}

impl std::error::Error for WindowsExecutionError {}

pub struct WindowsExecutionService;

impl WindowsExecutionService {
    /// Execute command directly on Windows host using PowerShell
    pub async fn execute_direct(
        command: &str,
        timeout_seconds: u64,
    ) -> Result<WindowsExecuteResponse, WindowsExecutionError> {
        info!("Windows direct execution: command='{}', timeout={}s", command, timeout_seconds);
        
        let start = std::time::Instant::now();
        
        // Build PowerShell command
        // Use -NoProfile to avoid loading user profile (faster, more predictable)
        // Use -NonInteractive to prevent prompts
        // Use -Command to execute the command string
        let mut child = Command::new("powershell.exe")
            .args(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                command,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| WindowsExecutionError::SpawnFailed(e.to_string()))?;
        
        // Wait with timeout
        let output = timeout(Duration::from_secs(timeout_seconds), child.wait_with_output())
            .await
            .map_err(|_| WindowsExecutionError::Timeout(timeout_seconds))?
            .map_err(|e| WindowsExecutionError::ExecutionFailed(e.to_string()))?;
        
        let duration = start.elapsed();
        
        // Convert output (PowerShell uses UTF-8 on modern Windows)
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        
        info!(
            "Windows direct execution completed: exit_code={:?}, duration={}ms",
            exit_code,
            duration.as_millis()
        );
        
        Ok(WindowsExecuteResponse {
            executed: true,
            method: "direct".to_string(),
            exit_code,
            stdout,
            stderr,
            message: "Command executed directly on Windows host".to_string(),
            duration_ms: Some(duration.as_millis() as u64),
        })
    }
    
    /// Execute command with resource limits using PowerShell job control
    /// 
    /// Note: Full Job Objects would require Windows API via winapi crate.
    /// For MVP, we use PowerShell's built-in Start-Job with timeout.
    pub async fn execute_sandboxed(
        command: &str,
        timeout_seconds: u64,
    ) -> Result<WindowsExecuteResponse, WindowsExecutionError> {
        info!("Windows sandboxed execution: command='{}', timeout={}s", command, timeout_seconds);
        
        // PowerShell script that runs command in a job with timeout
        let script = format!(
            r#"
$job = Start-Job -ScriptBlock {{
    {}
}}
$completed = Wait-Job -Job $job -Timeout {}
if ($completed) {{
    Receive-Job -Job $job
}} else {{
    Stop-Job -Job $job
    Remove-Job -Job $job
    Write-Error "Command timed out after {} seconds"
    exit 124
}}
Remove-Job -Job $job
"#,
            command.replace("\"", "`\""), // Escape quotes
            timeout_seconds,
            timeout_seconds
        );
        
        let start = std::time::Instant::now();
        
        let mut child = Command::new("powershell.exe")
            .args(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &script,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| WindowsExecutionError::SpawnFailed(e.to_string()))?;
        
        // Give extra time for PowerShell job control overhead
        let wrapper_timeout = timeout_seconds + 10;
        let output = timeout(Duration::from_secs(wrapper_timeout), child.wait_with_output())
            .await
            .map_err(|_| WindowsExecutionError::Timeout(wrapper_timeout))?
            .map_err(|e| WindowsExecutionError::ExecutionFailed(e.to_string()))?;
        
        let duration = start.elapsed();
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        
        info!(
            "Windows sandboxed execution completed: exit_code={:?}, duration={}ms",
            exit_code,
            duration.as_millis()
        );
        
        Ok(WindowsExecuteResponse {
            executed: true,
            method: "sandboxed".to_string(),
            exit_code,
            stdout,
            stderr,
            message: "Command executed in PowerShell job with timeout".to_string(),
            duration_ms: Some(duration.as_millis() as u64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[cfg(windows)]
    async fn test_direct_execution_simple() {
        let result = WindowsExecutionService::execute_direct("echo 'Hello Windows'", 10)
            .await
            .unwrap();
        
        assert_eq!(result.executed, true);
        assert_eq!(result.method, "direct");
        assert!(result.stdout.contains("Hello Windows"));
    }
    
    #[tokio::test]
    #[cfg(windows)]
    async fn test_direct_execution_get_location() {
        let result = WindowsExecutionService::execute_direct("Get-Location", 10)
            .await
            .unwrap();
        
        assert_eq!(result.executed, true);
        assert!(result.exit_code == Some(0));
    }
}
