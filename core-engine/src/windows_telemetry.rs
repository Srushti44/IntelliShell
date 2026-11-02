/*!
 * Windows System Telemetry Service
 * 
 * Collects system metrics from Windows host using PowerShell and WMI/CIM.
 */

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_kb: u64,
    pub cpu_percent: f64,
}

#[derive(Debug)]
pub enum TelemetryError {
    CommandFailed(String),
    ParseError(String),
}

impl std::fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            TelemetryError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for TelemetryError {}

pub struct WindowsTelemetryService;

impl WindowsTelemetryService {
    /// Get CPU usage percentage using PowerShell
    pub async fn get_cpu_usage() -> Result<f64, TelemetryError> {
        // Use Get-CimInstance to get CPU load percentage
        let script = r#"
(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
"#;
        
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TelemetryError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Ok(0.0); // Default on error
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let cpu = stdout.trim().parse::<f64>().unwrap_or(0.0);
        
        Ok(cpu)
    }
    
    /// Get memory information using PowerShell
    pub async fn get_memory_info() -> Result<(u64, u64, u64), TelemetryError> {
        // Get memory stats from Win32_OperatingSystem
        let script = r#"
$os = Get-CimInstance Win32_OperatingSystem
$total = $os.TotalVisibleMemorySize * 1024
$free = $os.FreePhysicalMemory * 1024
$used = $total - $free
Write-Output "$total,$free,$used"
"#;
        
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TelemetryError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Ok((0, 0, 0));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split(',').collect();
        
        if parts.len() != 3 {
            return Ok((0, 0, 0));
        }
        
        let total = parts[0].parse::<u64>().unwrap_or(0);
        let free = parts[1].parse::<u64>().unwrap_or(0);
        let used = parts[2].parse::<u64>().unwrap_or(0);
        
        Ok((total, free, used))
    }
    
    /// Get disk usage for C: drive
    pub async fn get_disk_info() -> Result<(u64, u64, f64), TelemetryError> {
        let script = r#"
$drive = Get-PSDrive C
$total = $drive.Used + $drive.Free
$used = $drive.Used
$percent = ($used / $total) * 100
Write-Output "$total,$used,$percent"
"#;
        
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TelemetryError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Ok((0, 0, 0.0));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split(',').collect();
        
        if parts.len() != 3 {
            return Ok((0, 0, 0.0));
        }
        
        let total = parts[0].parse::<u64>().unwrap_or(0);
        let used = parts[1].parse::<u64>().unwrap_or(0);
        let percent = parts[2].parse::<f64>().unwrap_or(0.0);
        
        Ok((total, used, percent))
    }
    
    /// Get network statistics (total received and transmitted bytes)
    pub async fn get_network_stats() -> Result<(u64, u64), TelemetryError> {
        let script = r#"
$stats = Get-NetAdapterStatistics | Where-Object { $_.Name -notlike '*Loopback*' }
$rx = ($stats | Measure-Object -Property ReceivedBytes -Sum).Sum
$tx = ($stats | Measure-Object -Property SentBytes -Sum).Sum
Write-Output "$rx,$tx"
"#;
        
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TelemetryError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Ok((0, 0));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.trim().split(',').collect();
        
        if parts.len() != 2 {
            return Ok((0, 0));
        }
        
        let rx = parts[0].parse::<u64>().unwrap_or(0);
        let tx = parts[1].parse::<u64>().unwrap_or(0);
        
        Ok((rx, tx))
    }
    
    /// Get top processes by memory usage
    pub async fn get_top_processes(limit: usize) -> Result<Vec<WindowsProcessInfo>, TelemetryError> {
        let script = format!(
            r#"
Get-Process | Sort-Object -Property WorkingSet64 -Descending | Select-Object -First {} | ForEach-Object {{
    $cpu = if ($_.CPU) {{ [math]::Round($_.CPU, 2) }} else {{ 0 }}
    Write-Output "$($_.Id),$($_.ProcessName),$($_.WorkingSet64),$cpu"
}}
"#,
            limit
        );
        
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", &script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TelemetryError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let pid = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let memory_bytes = parts[2].parse::<u64>().unwrap_or(0);
            let cpu = parts[3].parse::<f64>().unwrap_or(0.0);
            
            processes.push(WindowsProcessInfo {
                pid,
                name,
                memory_kb: memory_bytes / 1024,
                cpu_percent: cpu,
            });
        }
        
        Ok(processes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[cfg(windows)]
    async fn test_get_cpu_usage() {
        let cpu = WindowsTelemetryService::get_cpu_usage().await.unwrap();
        assert!(cpu >= 0.0 && cpu <= 100.0);
    }
    
    #[tokio::test]
    #[cfg(windows)]
    async fn test_get_memory_info() {
        let (total, _free, _used) = WindowsTelemetryService::get_memory_info().await.unwrap();
        assert!(total > 0);
    }
}
