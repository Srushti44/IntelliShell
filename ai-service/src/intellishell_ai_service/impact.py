"""
Impact Analysis Module

Analyzes potential impact of commands on the filesystem and system.
Uses real filesystem inspection, not AI inference.
"""

import os
import re
from pathlib import Path
from typing import List, Dict, Any, Optional


def analyze_impact(command: str, cwd: Optional[str] = None) -> str:
    """
    Analyze the potential impact of a command on files and directories.
    
    Args:
        command: Shell command to analyze
        cwd: Current working directory (defaults to os.getcwd())
    
    Returns:
        Human-readable impact summary string
    """
    if cwd is None:
        cwd = os.getcwd()
    
    impacts = []
    
    # Extract operation type
    operations = extract_operations(command)
    if operations:
        impacts.append(f"Operations: {', '.join(operations)}")
    
    # Extract and validate target paths
    paths = extract_paths(command, cwd)
    if paths:
        existing = [p for p in paths if os.path.exists(p)]
        non_existing = [p for p in paths if not os.path.exists(p)]
        
        if existing:
            impacts.append(f"Affected existing paths: {len(existing)}")
            # Show first few paths
            for path in existing[:3]:
                if os.path.isdir(path):
                    try:
                        items = len(list(os.scandir(path)))
                        impacts.append(f"  • {path} (directory with {items} items)")
                    except PermissionError:
                        impacts.append(f"  • {path} (directory, permission denied)")
                else:
                    try:
                        size = os.path.getsize(path)
                        impacts.append(f"  • {path} ({format_size(size)})")
                    except (PermissionError, OSError):
                        impacts.append(f"  • {path} (file, cannot read)")
            
            if len(existing) > 3:
                impacts.append(f"  ... and {len(existing) - 3} more")
        
        if non_existing:
            impacts.append(f"Targets non-existing paths: {', '.join(non_existing[:3])}")
    
    # Check if operating on system directories
    system_dirs = ["/bin", "/sbin", "/usr", "/lib", "/lib64", "/etc", "/boot", "/sys", "/proc"]
    for sys_dir in system_dirs:
        if any(p.startswith(sys_dir) for p in paths):
            impacts.append(f"⚠️  Operating on system directory: {sys_dir}")
            break
    
    # Check if operating on root or home
    if any(p == "/" for p in paths):
        impacts.append("⚠️  CRITICAL: Operating on root directory /")
    if any(p == os.path.expanduser("~") for p in paths):
        impacts.append("⚠️  Operating on home directory")
    
    if not impacts:
        return "No significant filesystem impact detected"
    
    return " | ".join(impacts)


def extract_operations(command: str) -> List[str]:
    """Extract operation types from command."""
    operations = []
    
    operation_patterns = [
        (r"\brm\b", "delete"),
        (r"\bmv\b", "move/rename"),
        (r"\bcp\b", "copy"),
        (r"\bchmod\b", "permission change"),
        (r"\bchown\b", "ownership change"),
        (r"\bmkdir\b", "create directory"),
        (r"\btouch\b", "create/update file"),
        (r"\bln\b", "create link"),
        (r"\btar\b.*-[xc]", "archive operation"),
        (r"\bunzip\b|\buntar\b|\bgunzip\b", "extract archive"),
        (r"\b>", "redirect output (overwrite)"),
        (r"\b>>", "redirect output (append)"),
    ]
    
    for pattern, operation in operation_patterns:
        if re.search(pattern, command):
            operations.append(operation)
    
    return operations


def extract_paths(command: str, cwd: str) -> List[str]:
    """
    Extract file/directory paths from command.
    
    This is a heuristic parser - not perfect but catches common patterns.
    """
    paths = []
    
    # Remove quoted strings first (they might contain paths we shouldn't resolve)
    # Then split by whitespace to get potential paths
    parts = command.split()
    
    for part in parts:
        # Skip flags
        if part.startswith("-"):
            continue
        
        # Skip command names (common ones)
        if part in ["rm", "mv", "cp", "chmod", "chown", "mkdir", "touch", "ln", 
                   "cat", "ls", "find", "grep", "tar", "zip", "unzip"]:
            continue
        
        # Check if it looks like a path
        if "/" in part or part.startswith("~") or part == ".." or part == ".":
            # Clean up the path
            clean_path = part.strip("'\"")
            
            # Expand home directory
            if clean_path.startswith("~"):
                clean_path = os.path.expanduser(clean_path)
            
            # Make absolute if relative
            if not clean_path.startswith("/"):
                clean_path = os.path.join(cwd, clean_path)
            
            # Normalize
            clean_path = os.path.normpath(clean_path)
            
            paths.append(clean_path)
    
    # Deduplicate while preserving order
    seen = set()
    unique_paths = []
    for p in paths:
        if p not in seen:
            seen.add(p)
            unique_paths.append(p)
    
    return unique_paths


def format_size(bytes: int) -> str:
    """Format byte size as human-readable string."""
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if bytes < 1024.0:
            return f"{bytes:.1f} {unit}"
        bytes /= 1024.0
    return f"{bytes:.1f} PB"


def estimate_recursive_impact(path: str, max_depth: int = 3) -> Dict[str, Any]:
    """
    Estimate impact of recursive operations on a directory.
    
    Args:
        path: Directory path
        max_depth: Maximum depth to scan
    
    Returns:
        Dict with file_count, dir_count, total_size
    """
    if not os.path.isdir(path):
        return {"error": "Not a directory"}
    
    file_count = 0
    dir_count = 0
    total_size = 0
    
    try:
        for root, dirs, files in os.walk(path):
            # Limit depth
            depth = root[len(path):].count(os.sep)
            if depth >= max_depth:
                del dirs[:]  # Don't recurse further
                continue
            
            dir_count += len(dirs)
            file_count += len(files)
            
            for file in files:
                try:
                    file_path = os.path.join(root, file)
                    total_size += os.path.getsize(file_path)
                except (PermissionError, OSError):
                    pass  # Skip files we can't read
    
    except PermissionError:
        return {"error": "Permission denied"}
    
    return {
        "file_count": file_count,
        "dir_count": dir_count,
        "total_size": total_size,
        "total_size_human": format_size(total_size),
    }
