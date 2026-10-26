// /proc/diskstats parser for block device I/O metrics
// Format documented in: Documentation/admin-guide/iostats.rst in kernel source

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_u64, read_proc_file};
use std::collections::HashMap;
use std::path::Path;

const PROC_DISKSTATS_PATH: &str = "/proc/diskstats";
const MAX_FILE_SIZE: usize = 262144; // 256KB limit

/// Disk I/O statistics for a single device
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskStats {
    pub major: u32,
    pub minor: u32,
    pub device_name: String,
    pub reads_completed: u64,
    pub reads_merged: u64,
    pub sectors_read: u64,
    pub time_reading_ms: u64,
    pub writes_completed: u64,
    pub writes_merged: u64,
    pub sectors_written: u64,
    pub time_writing_ms: u64,
    pub io_in_progress: u64,
    pub time_io_ms: u64,
    pub weighted_time_io_ms: u64,
}

impl DiskStats {
    /// Total I/O operations
    pub fn total_io(&self) -> u64 {
        self.reads_completed + self.writes_completed
    }

    /// Total sectors transferred
    pub fn total_sectors(&self) -> u64 {
        self.sectors_read + self.sectors_written
    }
}

/// Parse /proc/diskstats and return map of device_name -> stats
pub fn parse_proc_diskstats() -> Result<HashMap<String, DiskStats>, TelemetryError> {
    let path = Path::new(PROC_DISKSTATS_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_diskstats_content(&content, path)
}

fn parse_diskstats_content(
    content: &str,
    path: &Path,
) -> Result<HashMap<String, DiskStats>, TelemetryError> {
    let mut devices = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let stats = parse_diskstats_line(line, path)?;
        devices.insert(stats.device_name.clone(), stats);
    }

    Ok(devices)
}

fn parse_diskstats_line(line: &str, path: &Path) -> Result<DiskStats, TelemetryError> {
    // Format: "major minor device_name field1 field2 ... field11"
    let fields: Vec<&str> = line.split_whitespace().collect();

    if fields.len() < 14 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Expected at least 14 fields, found {}", fields.len()),
        }));
    }

    let major = fields[0].parse::<u32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: "major".to_string(),
            value: fields[0].to_string(),
            source: e,
        })
    })?;

    let minor = fields[1].parse::<u32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: "minor".to_string(),
            value: fields[1].to_string(),
            source: e,
        })
    })?;

    let device_name = fields[2].to_string();

    // Parse I/O statistics (fields 3-13)
    let reads_completed = parse_u64(fields[3], "reads_completed", path)?;
    let reads_merged = parse_u64(fields[4], "reads_merged", path)?;
    let sectors_read = parse_u64(fields[5], "sectors_read", path)?;
    let time_reading_ms = parse_u64(fields[6], "time_reading_ms", path)?;
    let writes_completed = parse_u64(fields[7], "writes_completed", path)?;
    let writes_merged = parse_u64(fields[8], "writes_merged", path)?;
    let sectors_written = parse_u64(fields[9], "sectors_written", path)?;
    let time_writing_ms = parse_u64(fields[10], "time_writing_ms", path)?;
    let io_in_progress = parse_u64(fields[11], "io_in_progress", path)?;
    let time_io_ms = parse_u64(fields[12], "time_io_ms", path)?;
    let weighted_time_io_ms = parse_u64(fields[13], "weighted_time_io_ms", path)?;

    Ok(DiskStats {
        major,
        minor,
        device_name,
        reads_completed,
        reads_merged,
        sectors_read,
        time_reading_ms,
        writes_completed,
        writes_merged,
        sectors_written,
        time_writing_ms,
        io_in_progress,
        time_io_ms,
        weighted_time_io_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_DISKSTATS: &str = r#"   8       0 sda 12345 100 98765 5000 54321 200 87654 10000 0 15000 15000
   8       1 sda1 2000 50 16000 1000 10000 100 80000 8000 0 9000 9000
   8      16 sdb 500 10 4000 250 600 15 4800 300 0 550 550
"#;

    #[test]
    fn test_parse_diskstats_line() {
        let path = PathBuf::from("/proc/diskstats");
        let line = "   8       0 sda 12345 100 98765 5000 54321 200 87654 10000 0 15000 15000";
        let stats = parse_diskstats_line(line, &path).unwrap();

        assert_eq!(stats.major, 8);
        assert_eq!(stats.minor, 0);
        assert_eq!(stats.device_name, "sda");
        assert_eq!(stats.reads_completed, 12345);
        assert_eq!(stats.sectors_read, 98765);
        assert_eq!(stats.writes_completed, 54321);
        assert_eq!(stats.sectors_written, 87654);
        assert_eq!(stats.total_io(), 66666);
    }

    #[test]
    fn test_parse_diskstats_content() {
        let path = PathBuf::from("/proc/diskstats");
        let devices = parse_diskstats_content(TEST_DISKSTATS, &path).unwrap();

        assert_eq!(devices.len(), 3);
        assert!(devices.contains_key("sda"));
        assert!(devices.contains_key("sda1"));
        assert!(devices.contains_key("sdb"));

        let sda = &devices["sda"];
        assert_eq!(sda.reads_completed, 12345);
        assert_eq!(sda.writes_completed, 54321);
    }

    #[test]
    fn test_invalid_line() {
        let path = PathBuf::from("/proc/diskstats");
        let line = "8 0 sda";
        let result = parse_diskstats_line(line, &path);
        assert!(result.is_err());
    }
}
