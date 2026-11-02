/*!
 * Windows Command Risk Analysis
 * 
 * Analyzes PowerShell commands for potential risks and dangerous operations.
 */

#[derive(Debug, Clone, PartialEq)]
pub enum WindowsRiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl WindowsRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowsRiskLevel::Safe => "safe",
            WindowsRiskLevel::Low => "low",
            WindowsRiskLevel::Medium => "medium",
            WindowsRiskLevel::High => "high",
            WindowsRiskLevel::Critical => "critical",
        }
    }
}

/// Windows-specific risk patterns
pub struct WindowsRiskAnalyzer;

impl WindowsRiskAnalyzer {
    /// Classify Windows PowerShell command risk level
    pub fn classify_command(command: &str) -> (WindowsRiskLevel, String) {
        let cmd_lower = command.to_lowercase();
        
        // CRITICAL PATTERNS - Catastrophic system damage
        if Self::matches_critical_pattern(&cmd_lower) {
            return (WindowsRiskLevel::Critical, Self::get_critical_reason(&cmd_lower));
        }
        
        // HIGH PATTERNS - Severe system impact
        if Self::matches_high_pattern(&cmd_lower) {
            return (WindowsRiskLevel::High, Self::get_high_reason(&cmd_lower));
        }
        
        // MEDIUM PATTERNS - Moderate system impact
        if Self::matches_medium_pattern(&cmd_lower) {
            return (WindowsRiskLevel::Medium, Self::get_medium_reason(&cmd_lower));
        }
        
        // LOW PATTERNS - Minor impact, still worth noting
        if Self::matches_low_pattern(&cmd_lower) {
            return (WindowsRiskLevel::Low, Self::get_low_reason(&cmd_lower));
        }
        
        // SAFE - Read-only operations, information gathering
        (WindowsRiskLevel::Safe, "Command appears safe (read-only or informational)".to_string())
    }
    
    fn matches_critical_pattern(cmd: &str) -> bool {
        // System-destroying operations
        cmd.contains("format-volume") ||
        cmd.contains("remove-item") && cmd.contains("c:\\") && cmd.contains("-recurse") ||
        cmd.contains("remove-item") && cmd.contains("c:\\windows") ||
        cmd.contains("remove-item") && cmd.contains("c:\\program files") ||
        (cmd.contains("stop-computer") || cmd.contains("restart-computer")) && !cmd.contains("-whatif") ||
        cmd.contains("clear-disk") ||
        cmd.contains("invoke-webrequest") && cmd.contains("invoke-expression") ||
        cmd.contains("iwr") && cmd.contains("iex") ||
        cmd.contains("curl") && cmd.contains("iex") ||
        cmd.contains("set-executionpolicy") && cmd.contains("unrestricted") && cmd.contains("localmachine") ||
        cmd.contains("takeown") && cmd.contains("c:\\windows") ||
        cmd.contains("bcdedit") ||
        cmd.contains("reg delete") && cmd.contains("hklm") ||
        cmd.contains("reg add") && cmd.contains("hklm\\software\\microsoft\\windows\\currentversion\\run")
    }
    
    fn get_critical_reason(cmd: &str) -> String {
        if cmd.contains("format-volume") {
            "CRITICAL: Disk format operation - will erase all data".to_string()
        } else if cmd.contains("remove-item") && cmd.contains("c:\\windows") {
            "CRITICAL: Attempting to delete Windows system directory".to_string()
        } else if cmd.contains("stop-computer") || cmd.contains("restart-computer") {
            "CRITICAL: System shutdown/restart command".to_string()
        } else if cmd.contains("invoke-expression") && (cmd.contains("invoke-webrequest") || cmd.contains("iwr") || cmd.contains("curl")) {
            "CRITICAL: Downloading and executing remote script".to_string()
        } else if cmd.contains("set-executionpolicy") && cmd.contains("unrestricted") {
            "CRITICAL: Disabling PowerShell execution policy globally".to_string()
        } else if cmd.contains("bcdedit") {
            "CRITICAL: Modifying boot configuration".to_string()
        } else if cmd.contains("reg delete") && cmd.contains("hklm") {
            "CRITICAL: Deleting system registry keys".to_string()
        } else {
            "CRITICAL: Command has potential for catastrophic system damage".to_string()
        }
    }
    
    fn matches_high_pattern(cmd: &str) -> bool {
        cmd.contains("remove-item") && cmd.contains("-recurse") && cmd.contains("-force") ||
        cmd.contains("stop-service") && cmd.contains("-force") ||
        cmd.contains("disable-windowsoptionalfeature") ||
        cmd.contains("uninstall-windowsfeature") ||
        cmd.contains("set-itemproperty") && cmd.contains("hklm:") ||
        cmd.contains("new-itemproperty") && cmd.contains("hklm:") ||
        cmd.contains("remove-itemproperty") && cmd.contains("hklm:") ||
        cmd.contains("grant-permission") ||
        cmd.contains("set-acl") && !cmd.contains("-whatif") ||
        cmd.contains("disable-computerrestore") ||
        cmd.contains("new-netfirewallrule") && cmd.contains("allow")
    }
    
    fn get_high_reason(cmd: &str) -> String {
        if cmd.contains("remove-item") && cmd.contains("-recurse") {
            "HIGH RISK: Recursive force delete operation".to_string()
        } else if cmd.contains("stop-service") && cmd.contains("-force") {
            "HIGH RISK: Force stopping Windows service".to_string()
        } else if cmd.contains("disable-windowsoptionalfeature") || cmd.contains("uninstall-windowsfeature") {
            "HIGH RISK: Disabling/removing Windows features".to_string()
        } else if cmd.contains("set-itemproperty") && cmd.contains("hklm") {
            "HIGH RISK: Modifying system registry".to_string()
        } else if cmd.contains("set-acl") {
            "HIGH RISK: Changing file/directory permissions".to_string()
        } else if cmd.contains("new-netfirewallrule") {
            "HIGH RISK: Modifying Windows Firewall rules".to_string()
        } else {
            "HIGH RISK: Command may cause significant system changes".to_string()
        }
    }
    
    fn matches_medium_pattern(cmd: &str) -> bool {
        cmd.contains("stop-process") && cmd.contains("-force") ||
        cmd.contains("remove-item") && !cmd.contains("-recurse") ||
        cmd.contains("stop-service") && !cmd.contains("-force") ||
        cmd.contains("restart-service") ||
        cmd.contains("set-executionpolicy") ||
        cmd.contains("new-item") && cmd.contains("-itemtype") && cmd.contains("directory") ||
        cmd.contains("rename-item") ||
        cmd.contains("move-item") ||
        cmd.contains("uninstall-package") ||
        cmd.contains("remove-appxpackage") ||
        cmd.contains("disable-netadapter") ||
        cmd.contains("set-netipaddress")
    }
    
    fn get_medium_reason(cmd: &str) -> String {
        if cmd.contains("stop-process") && cmd.contains("-force") {
            "MEDIUM RISK: Force killing process".to_string()
        } else if cmd.contains("remove-item") {
            "MEDIUM RISK: Delete operation".to_string()
        } else if cmd.contains("stop-service") || cmd.contains("restart-service") {
            "MEDIUM RISK: Modifying Windows service state".to_string()
        } else if cmd.contains("set-executionpolicy") {
            "MEDIUM RISK: Changing PowerShell execution policy".to_string()
        } else if cmd.contains("uninstall-package") || cmd.contains("remove-appxpackage") {
            "MEDIUM RISK: Removing installed software".to_string()
        } else if cmd.contains("disable-netadapter") || cmd.contains("set-netipaddress") {
            "MEDIUM RISK: Modifying network configuration".to_string()
        } else {
            "MEDIUM RISK: Command may modify system state".to_string()
        }
    }
    
    fn matches_low_pattern(cmd: &str) -> bool {
        cmd.contains("stop-process") && !cmd.contains("-force") ||
        cmd.contains("new-item") && !cmd.contains("c:\\windows") ||
        cmd.contains("copy-item") ||
        cmd.contains("clear-content") ||
        cmd.contains("set-content") ||
        cmd.contains("add-content") ||
        cmd.contains("out-file")
    }
    
    fn get_low_reason(cmd: &str) -> String {
        if cmd.contains("stop-process") {
            "LOW RISK: Stopping process (graceful)".to_string()
        } else if cmd.contains("new-item") {
            "LOW RISK: Creating new file or directory".to_string()
        } else if cmd.contains("copy-item") || cmd.contains("move-item") {
            "LOW RISK: File operation".to_string()
        } else if cmd.contains("set-content") || cmd.contains("out-file") {
            "LOW RISK: Writing to file".to_string()
        } else {
            "LOW RISK: Minor file or process operation".to_string()
        }
    }
    
    /// Suggest safer alternative for risky Windows commands
    pub fn suggest_safer_alternative(command: &str, risk_level: &WindowsRiskLevel) -> Option<String> {
        let cmd_lower = command.to_lowercase();
        
        match risk_level {
            WindowsRiskLevel::Critical | WindowsRiskLevel::High => {
                if cmd_lower.contains("remove-item") && cmd_lower.contains("-recurse") {
                    Some("Consider: Review items first with Get-ChildItem, then remove selectively".to_string())
                } else if cmd_lower.contains("stop-service") && cmd_lower.contains("-force") {
                    Some("Consider: Stop-Service <name> (without -Force for graceful shutdown)".to_string())
                } else if cmd_lower.contains("format-volume") {
                    Some("⚠️ NO SAFE ALTERNATIVE - This operation is inherently destructive".to_string())
                } else {
                    Some("Add -WhatIf parameter to preview changes before applying".to_string())
                }
            }
            WindowsRiskLevel::Medium => {
                if cmd_lower.contains("remove-item") {
                    Some("Add -Confirm parameter to require confirmation for each deletion".to_string())
                } else if cmd_lower.contains("set-executionpolicy") {
                    Some("Consider: Set-ExecutionPolicy RemoteSigned -Scope CurrentUser (safer scope)".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_commands() {
        let (level, _) = WindowsRiskAnalyzer::classify_command("Get-Process");
        assert_eq!(level, WindowsRiskLevel::Safe);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Get-Location");
        assert_eq!(level, WindowsRiskLevel::Safe);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Get-ChildItem");
        assert_eq!(level, WindowsRiskLevel::Safe);
    }
    
    #[test]
    fn test_low_risk_commands() {
        let (level, _) = WindowsRiskAnalyzer::classify_command("Stop-Process -Id 1234");
        assert_eq!(level, WindowsRiskLevel::Low);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("New-Item test.txt");
        assert_eq!(level, WindowsRiskLevel::Low);
    }
    
    #[test]
    fn test_medium_risk_commands() {
        let (level, _) = WindowsRiskAnalyzer::classify_command("Remove-Item test.txt");
        assert_eq!(level, WindowsRiskLevel::Medium);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Stop-Service wuauserv");
        assert_eq!(level, WindowsRiskLevel::Medium);
    }
    
    #[test]
    fn test_high_risk_commands() {
        let (level, _) = WindowsRiskAnalyzer::classify_command("Remove-Item test -Recurse -Force");
        assert_eq!(level, WindowsRiskLevel::High);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Stop-Service wuauserv -Force");
        assert_eq!(level, WindowsRiskLevel::High);
    }
    
    #[test]
    fn test_critical_commands() {
        let (level, _) = WindowsRiskAnalyzer::classify_command("Format-Volume C:");
        assert_eq!(level, WindowsRiskLevel::Critical);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Remove-Item C:\\Windows -Recurse");
        assert_eq!(level, WindowsRiskLevel::Critical);
        
        let (level, _) = WindowsRiskAnalyzer::classify_command("Stop-Computer");
        assert_eq!(level, WindowsRiskLevel::Critical);
    }
}
