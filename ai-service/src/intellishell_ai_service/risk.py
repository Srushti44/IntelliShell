"""
Risk Classification Module

Deterministic, rule-based risk assessment for shell commands.
This is NOT AI-generated - it's a verifiable pattern matcher that
classifies commands based on known dangerous patterns.
"""

import re
from enum import Enum
from typing import Tuple


class RiskLevel(Enum):
    """Risk levels for command classification."""
    SAFE = "safe"
    LOW = "low"
    MEDIUM = "medium"
    WARN = "warn"
    HIGH = "high"
    CRITICAL = "critical"


def classify_command(command: str) -> Tuple[RiskLevel, str]:
    """
    Classify command risk level using pattern matching.
    
    Returns:
        (RiskLevel, reason: str)
    """
    command = command.strip()
    
    if not command:
        return (RiskLevel.SAFE, "Empty command")
    
    # Critical patterns (check first, most dangerous)
    critical_patterns = [
        (r"rm\s+.*(-[rfRF]+|--recursive|--force).*\s+/\s*$", "Attempts to delete root directory"),
        (r"rm\s+.*(-[rfRF]+|--recursive|--force).*\s+~\s*$", "Attempts to delete home directory"),
        (r"rm\s+-[rfRF]+\s+/$", "Recursive/force delete on root"),
        (r"rm\s+-[rfRF]+\s+/\s+", "Recursive/force delete on root"),
        (r"dd\s+.*of=/dev/sd[a-z]", "Direct disk write operation"),
        (r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:", "Fork bomb detected"),
        (r"chmod\s+(-R|--recursive)\s+777\s+/", "Recursive 777 permissions on root"),
        (r"kill\s+-9\s+1\b", "Attempting to kill init process"),
        (r"(shutdown|reboot|halt|poweroff)\b", "System shutdown/reboot command"),
        (r"(curl|wget)\s+.*\|\s*(bash|sh|python)", "Piping remote script to shell"),
        (r"mkfs\.", "Filesystem format command"),
        (r"fdisk|parted|gparted", "Disk partitioning tool"),
    ]
    
    for pattern, reason in critical_patterns:
        if re.search(pattern, command, re.IGNORECASE):
            return (RiskLevel.CRITICAL, reason)
    
    # High risk patterns
    high_patterns = [
        (r"rm\s+-[rfRF]+", "Recursive/force delete operation"),
        (r"mv\s+.*\s+/dev/null", "Moving files to /dev/null (permanent deletion)"),
        (r">\s*/dev/sd[a-z]", "Direct write to disk device"),
        (r"chown\s+.*\s+/", "Changing ownership of root filesystem"),
        (r"chmod\s+000", "Removing all permissions"),
    ]
    
    for pattern, reason in high_patterns:
        if re.search(pattern, command, re.IGNORECASE):
            return (RiskLevel.HIGH, reason)
    
    # Medium/Warning patterns
    warn_patterns = [
        (r"kill\s+-9\s+\d+", "Force killing process with SIGKILL"),
        (r"chmod\s+777", "Setting overly permissive file permissions"),
        (r"chmod\s+.*-R", "Recursive permission change"),
        (r"chown\s+.*-R", "Recursive ownership change"),
        (r"sudo\s+", "Elevated privileges command"),
        (r"systemctl\s+(stop|restart|disable)", "System service modification"),
        (r"apt-get\s+(remove|purge)", "Package removal"),
        (r"yum\s+(remove|erase)", "Package removal"),
        (r"pacman\s+-R", "Package removal"),
    ]
    
    for pattern, reason in warn_patterns:
        if re.search(pattern, command, re.IGNORECASE):
            return (RiskLevel.MEDIUM, reason)
    
    # Low risk patterns (potentially dangerous but usually safe)
    low_patterns = [
        (r"rm\s+[^-]", "Delete operation without force flags"),
        (r"kill\s+\d+", "Process termination"),
        (r"mv\s+", "Move/rename operation"),
        (r"cp\s+.*-[rfR]", "Recursive copy operation"),
    ]
    
    for pattern, reason in low_patterns:
        if re.search(pattern, command, re.IGNORECASE):
            return (RiskLevel.LOW, reason)
    
    # Default: Safe (read-only operations)
    return (RiskLevel.SAFE, "Command appears to be read-only or safe")


def suggest_safer_alternative(command: str) -> str:
    """
    Suggest safer alternatives for dangerous commands.
    
    Returns:
        String with safer alternative, or empty string if command is already safe.
    """
    command = command.strip()
    
    suggestions = []
    
    # rm -rf suggestions
    if re.search(r"rm\s+-[rfRF]+", command):
        suggestions.append("Use 'rm -i' for interactive confirmation")
        suggestions.append("Move files to trash instead: mv <files> ~/.local/share/Trash/")
        suggestions.append("Preview what will be deleted: ls -la <path>")
    
    # chmod 777 suggestions
    if re.search(r"chmod\s+777", command):
        suggestions.append("Use more restrictive permissions like 755 (rwxr-xr-x) or 644 (rw-r--r--)")
        suggestions.append("Apply principle of least privilege")
        suggestions.append("Consider: chmod 755 for executables, 644 for regular files")
    
    # kill -9 suggestions
    if re.search(r"kill\s+-9", command):
        suggestions.append("Try 'kill <pid>' (SIGTERM) first to allow graceful shutdown")
        suggestions.append("Use 'kill -15 <pid>' explicitly for SIGTERM")
        suggestions.append("Only use kill -9 if process is unresponsive to SIGTERM")
    
    # Piping to shell suggestions
    if re.search(r"(curl|wget)\s+.*\|\s*(bash|sh|python)", command):
        suggestions.append("Download script first: curl <url> -o script.sh")
        suggestions.append("Inspect the script before running: cat script.sh | less")
        suggestions.append("Run after review: bash script.sh")
        suggestions.append("Prefer package managers over piped scripts")
    
    # Recursive operations
    if re.search(r"(chmod|chown|rm).*-R", command):
        suggestions.append("Use -v (verbose) flag to see what's being modified")
        suggestions.append("Test on a small subset first")
        suggestions.append("Consider using find for more control")
    
    # Root filesystem operations
    if re.search(r"\s+/\s*$", command):
        suggestions.append("NEVER operate on root directory directly")
        suggestions.append("Double-check your path - did you mean a subdirectory?")
        suggestions.append("Use absolute paths to avoid accidents")
    
    if not suggestions:
        return ""
    
    return " | ".join(suggestions)


def is_read_only_command(command: str) -> bool:
    """
    Check if command is read-only (safe to execute without side effects).
    """
    read_only_commands = [
        r"^ls\b", r"^cat\b", r"^grep\b", r"^find\b", r"^locate\b",
        r"^which\b", r"^whereis\b", r"^file\b", r"^stat\b", r"^head\b",
        r"^tail\b", r"^less\b", r"^more\b", r"^wc\b", r"^du\b", r"^df\b",
        r"^ps\b", r"^top\b", r"^htop\b", r"^free\b", r"^uptime\b",
        r"^uname\b", r"^hostname\b", r"^whoami\b", r"^id\b", r"^groups\b",
        r"^env\b", r"^printenv\b", r"^pwd\b", r"^date\b", r"^cal\b",
    ]
    
    for pattern in read_only_commands:
        if re.search(pattern, command.strip(), re.IGNORECASE):
            return True
    
    return False
