// /proc/<pid>/stat parser for per-process metrics
// Format documented in: man 5 proc (search for /proc/[pid]/stat)

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_u64, read_proc_file};
use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: usize = 4096;

/// Per-process statistics from /proc/<pid>/stat
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcStat {
    pub pid: i32,
    pub comm: String, // Command name in parentheses
    pub state: char,
    pub ppid: i32,
    pub pgrp: i32,
    pub session: i32,
    pub utime: u64,   // User mode time (CLK_TCK units)
    pub stime: u64,   // Kernel mode time (CLK_TCK units)
    pub cutime: i64,  // Children user time
    pub cstime: i64,  // Children kernel time
    pub priority: i64,
    pub nice: i64,
    pub num_threads: i64,
    pub starttime: u64, // Process start time since boot (CLK_TCK units)
    pub vsize: u64,   // Virtual memory size (bytes)
    pub rss: i64,     // Resident set size (pages)
}

/// Parse /proc/<pid>/stat
pub fn parse_proc_pid_stat(pid: i32) -> Result<ProcStat, TelemetryError> {
    let path = PathBuf::from(format!("/proc/{}/stat", pid));
    let content = read_proc_file(&path, MAX_FILE_SIZE)?;
    parse_proc_stat_content(&content, &path)
}

fn parse_proc_stat_content(content: &str, path: &Path) -> Result<ProcStat, TelemetryError> {
    // The format is tricky because comm can contain spaces and parentheses
    // Format: pid (comm) state ppid pgrp session ...
    //
    // Strategy: find the LAST ')' to reliably split comm from other fields
    
    let rparen_pos = content.rfind(')').ok_or_else(|| {
        TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "No closing parenthesis found for comm".to_string(),
        })
    })?;

    let lparen_pos = content.find('(').ok_or_else(|| {
        TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "No opening parenthesis found for comm".to_string(),
        })
    })?;

    // Extract pid (before lparen)
    let pid_str = content[..lparen_pos].trim();
    let pid = pid_str.parse::<i32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: "pid".to_string(),
            value: pid_str.to_string(),
            source: e,
        })
    })?;

    // Extract comm (between parentheses, INCLUDING the parentheses)
    let comm = content[lparen_pos..=rparen_pos].to_string();

    // Extract remaining fields (after rparen)
    let remaining = &content[rparen_pos + 1..];
    let fields: Vec<&str> = remaining.split_whitespace().collect();

    if fields.len() < 50 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Expected at least 50 fields, found {}", fields.len()),
        }));
    }

    // Parse fields (0-indexed after comm)
    let state = fields[0].chars().next().ok_or_else(|| {
        TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "Empty state field".to_string(),
        })
    })?;

    let ppid = parse_i32(fields[1], "ppid", path)?;
    let pgrp = parse_i32(fields[2], "pgrp", path)?;
    let session = parse_i32(fields[3], "session", path)?;
    
    // Fields 11-12: utime, stime (indices 11, 12 in original, 9, 10 after comm)
    let utime = parse_u64(fields[11], "utime", path)?;
    let stime = parse_u64(fields[12], "stime", path)?;
    let cutime = parse_i64(fields[13], "cutime", path)?;
    let cstime = parse_i64(fields[14], "cstime", path)?;
    let priority = parse_i64(fields[15], "priority", path)?;
    let nice = parse_i64(fields[16], "nice", path)?;
    let num_threads = parse_i64(fields[17], "num_threads", path)?;
    
    // Field 21: starttime (index 21 in original, 19 after comm)
    let starttime = parse_u64(fields[19], "starttime", path)?;
    
    // Field 22-23: vsize, rss
    let vsize = parse_u64(fields[20], "vsize", path)?;
    let rss = parse_i64(fields[21], "rss", path)?;

    Ok(ProcStat {
        pid,
        comm,
        state,
        ppid,
        pgrp,
        session,
        utime,
        stime,
        cutime,
        cstime,
        priority,
        nice,
        num_threads,
        starttime,
        vsize,
        rss,
    })
}

fn parse_i32(s: &str, field: &str, path: &Path) -> Result<i32, TelemetryError> {
    s.trim().parse::<i32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: field.to_string(),
            value: s.to_string(),
            source: e,
        })
    })
}

fn parse_i64(s: &str, field: &str, path: &Path) -> Result<i64, TelemetryError> {
    s.trim().parse::<i64>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: field.to_string(),
            value: s.to_string(),
            source: e,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_stat_content() {
        let path = PathBuf::from("/proc/1234/stat");
        // Real example with spaces in comm
        let content = "1234 (some process) S 1 1234 1234 0 -1 4194304 500 0 10 0 100 50 0 0 20 0 2 0 12345 123456789 1000 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 1 0 0 0 0 0 0 0 0 0 0 0 0 0";
        
        let stat = parse_proc_stat_content(content, &path).unwrap();
        
        assert_eq!(stat.pid, 1234);
        assert_eq!(stat.comm, "(some process)");
        assert_eq!(stat.state, 'S');
        assert_eq!(stat.ppid, 1);
        assert_eq!(stat.utime, 100);
        assert_eq!(stat.stime, 50);
        assert_eq!(stat.num_threads, 2);
        assert_eq!(stat.starttime, 12345);
        assert_eq!(stat.vsize, 123456789);
        assert_eq!(stat.rss, 1000);
    }

    #[test]
    fn test_parse_with_parentheses_in_comm() {
        let path = PathBuf::from("/proc/1234/stat");
        let content = "1234 ((weird) name) R 1 1234 1234 0 -1 4194304 500 0 10 0 100 50 0 0 20 0 1 0 12345 123456789 1000 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 1 0 0 0 0 0 0 0 0 0 0 0 0 0";
        
        let stat = parse_proc_stat_content(content, &path).unwrap();
        
        assert_eq!(stat.pid, 1234);
        assert_eq!(stat.comm, "((weird) name)");
        assert_eq!(stat.state, 'R');
    }

    #[test]
    fn test_invalid_format() {
        let path = PathBuf::from("/proc/1234/stat");
        let content = "1234 no_parens S 1";
        let result = parse_proc_stat_content(content, &path);
        assert!(result.is_err());
    }
}
