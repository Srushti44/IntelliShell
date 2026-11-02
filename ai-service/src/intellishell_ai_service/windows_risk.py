"""
Windows-specific risk classification patterns.
Mirrors risk.py structure but with PowerShell/Windows commands.
"""

from enum import Enum
from typing import Tuple
import re


class RiskLevel(Enum):
    """Risk levels for Windows commands."""
    SAFE = "safe"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


# CRITICAL PATTERNS - Catastrophic system damage
WINDOWS_CRITICAL_PATTERNS = [
    (r"format-volume", "Disk format operation - will erase all data"),
    (r"remove-item.*-recurse.*c:\\windows", "Attempting to delete Windows system directory"),
    (r"remove-item.*-recurse.*c:\\program files", "Attempting to delete Program Files"),
    (r"remove-item.*-recurse.*-force.*c:\\", "Recursive force delete on C drive root"),
    (r"stop-computer|restart-computer", "System shutdown/restart command"),
    (r"clear-disk", "Disk wipe operation"),
    (r"(invoke-webrequest|iwr|curl).*(invoke-expression|iex)", "Downloading and executing remote script"),
    (r"set-executionpolicy\s+unrestricted.*-scope\s+localmachine", "Disabling PowerShell execution policy globally"),
    (r"takeown.*c:\\windows", "Taking ownership of system directory"),
    (r"bcdedit", "Boot configuration modification"),
    (r"reg\s+delete.*hklm", "Registry deletion of system keys"),
    (r"reg\s+add.*hklm.*\\run", "Adding to system startup"),
]

# HIGH PATTERNS - Severe system impact
WINDOWS_HIGH_PATTERNS = [
    (r"remove-item.*-recurse.*-force", "Recursive force delete operation"),
    (r"stop-service.*-force", "Force stopping Windows service"),
    (r"disable-windowsoptionalfeature", "Disabling Windows features"),
    (r"uninstall-windowsfeature", "Removing Windows features"),
    (r"set-itemproperty.*hklm:", "Modifying system registry"),
    (r"new-itemproperty.*hklm:", "Adding to system registry"),
    (r"remove-itemproperty.*hklm:", "Removing from system registry"),
    (r"set-acl", "Changing file/directory permissions"),
    (r"disable-computerrestore", "Disabling system restore"),
    (r"new-netfirewallrule.*allow", "Adding firewall rules"),
]

# MEDIUM PATTERNS - Moderate system impact
WINDOWS_MEDIUM_PATTERNS = [
    (r"stop-process.*-force", "Force killing process"),
    (r"remove-item", "Delete operation"),
    (r"stop-service", "Stopping Windows service"),
    (r"restart-service", "Restarting Windows service"),
    (r"set-executionpolicy", "Changing PowerShell execution policy"),
    (r"rename-item", "Rename operation"),
    (r"move-item", "Move operation"),
    (r"uninstall-package", "Removing software package"),
    (r"remove-appxpackage", "Removing Windows app"),
    (r"disable-netadapter", "Disabling network adapter"),
    (r"set-netipaddress", "Modifying network configuration"),
]

# LOW PATTERNS - Minor impact
WINDOWS_LOW_PATTERNS = [
    (r"stop-process", "Process termination"),
    (r"new-item", "Creating new file or directory"),
    (r"copy-item", "Copy operation"),
    (r"clear-content", "Clearing file content"),
    (r"set-content", "Writing to file"),
    (r"add-content", "Appending to file"),
    (r"out-file", "Writing output to file"),
]


def classify_windows_command(command: str) -> Tuple[RiskLevel, str]:
    """
    Classify Windows/PowerShell command risk level.
    
    Args:
        command: The PowerShell command to classify
    
    Returns:
        Tuple of (RiskLevel, reason string)
    """
    cmd_lower = command.lower().strip()
    
    # CRITICAL patterns
    for pattern, reason in WINDOWS_CRITICAL_PATTERNS:
        if re.search(pattern, cmd_lower, re.IGNORECASE):
            return (RiskLevel.CRITICAL, f"CRITICAL: {reason}")
    
    # HIGH patterns
    for pattern, reason in WINDOWS_HIGH_PATTERNS:
        if re.search(pattern, cmd_lower, re.IGNORECASE):
            return (RiskLevel.HIGH, f"HIGH RISK: {reason}")
    
    # MEDIUM patterns
    for pattern, reason in WINDOWS_MEDIUM_PATTERNS:
        if re.search(pattern, cmd_lower, re.IGNORECASE):
            return (RiskLevel.MEDIUM, f"MEDIUM RISK: {reason}")
    
    # LOW patterns
    for pattern, reason in WINDOWS_LOW_PATTERNS:
        if re.search(pattern, cmd_lower, re.IGNORECASE):
            return (RiskLevel.LOW, f"LOW RISK: {reason}")
    
    # Default to SAFE for read-only cmdlets
    if cmd_lower.startswith(("get-", "test-", "show-", "find-", "measure-", "compare-", "select-")):
        return (RiskLevel.SAFE, "Command appears safe (read-only cmdlet)")
    
    return (RiskLevel.SAFE, "Command appears safe (no dangerous patterns detected)")


def suggest_safer_alternative_windows(command: str) -> str:
    """
    Suggest safer alternatives for risky Windows commands.
    
    Args:
        command: The Windows command to analyze
    
    Returns:
        Suggested safer alternative, or empty string if none
    """
    cmd_lower = command.lower().strip()
    
    # Critical/High risk suggestions
    if "format-volume" in cmd_lower:
        return "⚠️ NO SAFE ALTERNATIVE - This operation is inherently destructive"
    
    if "remove-item" in cmd_lower and "-recurse" in cmd_lower and "-force" in cmd_lower:
        return "Consider: Review items first with Get-ChildItem, then remove selectively without -Force"
    
    if "remove-item" in cmd_lower and "-recurse" in cmd_lower:
        return "Add -WhatIf parameter to preview what would be deleted: Remove-Item <path> -Recurse -WhatIf"
    
    if "stop-service" in cmd_lower and "-force" in cmd_lower:
        return "Consider: Stop-Service <name> (without -Force for graceful shutdown)"
    
    if "set-executionpolicy" in cmd_lower and "localmachine" in cmd_lower:
        return "Consider: Set-ExecutionPolicy RemoteSigned -Scope CurrentUser (safer scope)"
    
    # Medium risk suggestions
    if "remove-item" in cmd_lower:
        return "Add -Confirm parameter to require confirmation: Remove-Item <path> -Confirm"
    
    if "stop-process" in cmd_lower and "-force" in cmd_lower:
        return "Consider: Stop-Process <id> (without -Force to allow graceful shutdown)"
    
    return ""
