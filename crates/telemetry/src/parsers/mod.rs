// Parser modules for /proc and /sys filesystem parsing
// All parsers implement zero-allocation, defensive parsing with detailed error context

pub mod stat;
pub mod meminfo;
pub mod loadavg;
pub mod pressure;
pub mod diskstats;
pub mod netdev;
pub mod procstat;

use crate::error::{ParseError, TelemetryError};
use std::fs;
use std::path::Path;

/// Read file with size limit and UTF-8 validation
pub(crate) fn read_proc_file(path: &Path, max_size: usize) -> Result<String, TelemetryError> {
    let bytes = fs::read(path)?;
    
    if bytes.len() > max_size {
        return Err(TelemetryError::Parse(ParseError::FileTooLarge {
            path: path.to_path_buf(),
            size: bytes.len(),
            limit: max_size,
        }));
    }
    
    String::from_utf8(bytes).map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidUtf8 {
            path: path.to_path_buf(),
            source: e,
        })
    })
}

/// Parse u64 from string slice with context
pub(crate) fn parse_u64(s: &str, field: &str, path: &Path) -> Result<u64, TelemetryError> {
    s.trim().parse::<u64>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: field.to_string(),
            value: s.to_string(),
            source: e,
        })
    })
}

/// Parse f64 from string slice with context
pub(crate) fn parse_f64(s: &str, field: &str, path: &Path) -> Result<f64, TelemetryError> {
    s.trim().parse::<f64>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidFloat {
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
    use std::path::PathBuf;

    #[test]
    fn test_parse_u64_valid() {
        let path = PathBuf::from("/test");
        assert_eq!(parse_u64("123", "test", &path).unwrap(), 123);
        assert_eq!(parse_u64("  456  ", "test", &path).unwrap(), 456);
    }

    #[test]
    fn test_parse_u64_invalid() {
        let path = PathBuf::from("/test");
        assert!(parse_u64("abc", "test", &path).is_err());
        assert!(parse_u64("-1", "test", &path).is_err());
    }

    #[test]
    fn test_parse_f64_valid() {
        let path = PathBuf::from("/test");
        assert!((parse_f64("1.5", "test", &path).unwrap() - 1.5).abs() < f64::EPSILON);
        assert!((parse_f64("  2.75  ", "test", &path).unwrap() - 2.75).abs() < f64::EPSILON);
    }
}
