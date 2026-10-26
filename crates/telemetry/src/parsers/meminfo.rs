// /proc/meminfo parser for memory metrics
// Format documented in: man 5 proc (search for /proc/meminfo)

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_u64, read_proc_file};
use std::collections::HashMap;
use std::path::Path;

const PROC_MEMINFO_PATH: &str = "/proc/meminfo";
const MAX_FILE_SIZE: usize = 65536; // 64KB limit

/// Required memory fields that must be present
const REQUIRED_FIELDS: &[&str] = &[
    "MemTotal",
    "MemFree",
    "MemAvailable",
    "Buffers",
    "Cached",
    "SwapTotal",
    "SwapFree",
];

/// Parse /proc/meminfo and return field map (values in bytes)
pub fn parse_proc_meminfo() -> Result<HashMap<String, u64>, TelemetryError> {
    let path = Path::new(PROC_MEMINFO_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_meminfo_content(&content, path)
}

fn parse_meminfo_content(content: &str, path: &Path) -> Result<HashMap<String, u64>, TelemetryError> {
    let mut fields = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (key, value_kb) = parse_meminfo_line(line, path)?;
        
        // Convert kB to bytes (kernel uses 1024-based units)
        let value_bytes = value_kb.checked_mul(1024).ok_or_else(|| {
            TelemetryError::Parse(ParseError::InvalidFormat {
                path: path.to_path_buf(),
                reason: format!("Overflow converting {} kB to bytes", value_kb),
            })
        })?;
        
        fields.insert(key.to_string(), value_bytes);
    }

    // Verify all required fields are present
    for &required in REQUIRED_FIELDS {
        if !fields.contains_key(required) {
            return Err(TelemetryError::Parse(ParseError::MissingField {
                path: path.to_path_buf(),
                field: required.to_string(),
            }));
        }
    }

    Ok(fields)
}

fn parse_meminfo_line<'a>(line: &'a str, path: &Path) -> Result<(&'a str, u64), TelemetryError> {
    // Format: "FieldName:       12345 kB" or "FieldName:       12345"
    let colon_pos = line.find(':').ok_or_else(|| {
        TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("No colon found in line: {}", line),
        })
    })?;

    let key = &line[..colon_pos];
    let value_part = line[colon_pos + 1..].trim();

    // Remove " kB" suffix if present
    let value_str = value_part
        .strip_suffix(" kB")
        .or_else(|| value_part.strip_suffix("kB"))
        .unwrap_or(value_part)
        .trim();

    let value = parse_u64(value_str, key, path)?;

    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_MEMINFO: &str = r#"MemTotal:       16384000 kB
MemFree:         8192000 kB
MemAvailable:   12288000 kB
Buffers:          512000 kB
Cached:          4096000 kB
SwapCached:            0 kB
Active:          6144000 kB
Inactive:        2048000 kB
SwapTotal:       4096000 kB
SwapFree:        4096000 kB
Dirty:             10240 kB
Writeback:             0 kB
"#;

    #[test]
    fn test_parse_meminfo_line() {
        let path = PathBuf::from("/proc/meminfo");
        
        let (key, val) = parse_meminfo_line("MemTotal:       16384000 kB", &path).unwrap();
        assert_eq!(key, "MemTotal");
        assert_eq!(val, 16384000);

        let (key, val) = parse_meminfo_line("MemFree:         8192000 kB", &path).unwrap();
        assert_eq!(key, "MemFree");
        assert_eq!(val, 8192000);
    }

    #[test]
    fn test_parse_meminfo_content() {
        let path = PathBuf::from("/proc/meminfo");
        let fields = parse_meminfo_content(TEST_MEMINFO, &path).unwrap();

        // Values should be in bytes (kB * 1024)
        assert_eq!(fields.get("MemTotal"), Some(&(16384000 * 1024)));
        assert_eq!(fields.get("MemFree"), Some(&(8192000 * 1024)));
        assert_eq!(fields.get("MemAvailable"), Some(&(12288000 * 1024)));
        assert_eq!(fields.get("Buffers"), Some(&(512000 * 1024)));
        assert_eq!(fields.get("Cached"), Some(&(4096000 * 1024)));
    }

    #[test]
    fn test_missing_required_field() {
        let path = PathBuf::from("/proc/meminfo");
        let content = "MemTotal:       16384000 kB\n";
        let result = parse_meminfo_content(content, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_line_format() {
        let path = PathBuf::from("/proc/meminfo");
        let result = parse_meminfo_line("InvalidLine", &path);
        assert!(result.is_err());
    }
}
