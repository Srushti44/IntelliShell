// /proc/pressure/* parser for PSI (Pressure Stall Information)
// Format documented in: Documentation/accounting/psi.rst in kernel source

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_f64, parse_u64, read_proc_file};
use std::path::Path;

const MAX_FILE_SIZE: usize = 1024;

/// PSI metrics for a single resource
#[derive(Debug, Clone, PartialEq)]
pub struct PressureMetrics {
    pub some_avg10: f64,  // 10-second average (%)
    pub some_avg60: f64,  // 60-second average (%)
    pub some_avg300: f64, // 300-second average (%)
    pub some_total: u64,  // Total stall time (microseconds)
    pub full_avg10: Option<f64>,
    pub full_avg60: Option<f64>,
    pub full_avg300: Option<f64>,
    pub full_total: Option<u64>,
}

/// Parse a pressure file (cpu, memory, or io)
pub fn parse_pressure_file(path: &Path) -> Result<PressureMetrics, TelemetryError> {
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_pressure_content(&content, path)
}

fn parse_pressure_content(content: &str, path: &Path) -> Result<PressureMetrics, TelemetryError> {
    // Format:
    // some avg10=0.00 avg60=0.00 avg300=0.00 total=123456
    // full avg10=0.00 avg60=0.00 avg300=0.00 total=78910
    //
    // Note: /proc/pressure/cpu only has "some" line (no "full")
    
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "Empty pressure file".to_string(),
        }));
    }

    let some_line = lines[0];
    if !some_line.starts_with("some ") {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Expected 'some' line, got: {}", some_line),
        }));
    }

    let (some_avg10, some_avg60, some_avg300, some_total) = 
        parse_pressure_line(&some_line[5..], path)?;

    let (full_avg10, full_avg60, full_avg300, full_total) = if lines.len() > 1 {
        let full_line = lines[1];
        if !full_line.starts_with("full ") {
            return Err(TelemetryError::Parse(ParseError::InvalidFormat {
                path: path.to_path_buf(),
                reason: format!("Expected 'full' line, got: {}", full_line),
            }));
        }
        let (avg10, avg60, avg300, total) = parse_pressure_line(&full_line[5..], path)?;
        (Some(avg10), Some(avg60), Some(avg300), Some(total))
    } else {
        (None, None, None, None)
    };

    Ok(PressureMetrics {
        some_avg10,
        some_avg60,
        some_avg300,
        some_total,
        full_avg10,
        full_avg60,
        full_avg300,
        full_total,
    })
}

fn parse_pressure_line(line: &str, path: &Path) -> Result<(f64, f64, f64, u64), TelemetryError> {
    // Format: "avg10=0.00 avg60=0.00 avg300=0.00 total=123456"
    
    let mut avg10: Option<f64> = None;
    let mut avg60: Option<f64> = None;
    let mut avg300: Option<f64> = None;
    let mut total: Option<u64> = None;

    for field in line.split_whitespace() {
        if let Some(value) = field.strip_prefix("avg10=") {
            avg10 = Some(parse_f64(value, "avg10", path)?);
        } else if let Some(value) = field.strip_prefix("avg60=") {
            avg60 = Some(parse_f64(value, "avg60", path)?);
        } else if let Some(value) = field.strip_prefix("avg300=") {
            avg300 = Some(parse_f64(value, "avg300", path)?);
        } else if let Some(value) = field.strip_prefix("total=") {
            total = Some(parse_u64(value, "total", path)?);
        }
    }

    let avg10 = avg10.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "avg10".to_string(),
        })
    })?;

    let avg60 = avg60.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "avg60".to_string(),
        })
    })?;

    let avg300 = avg300.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "avg300".to_string(),
        })
    })?;

    let total = total.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "total".to_string(),
        })
    })?;

    // Validate percentages
    if avg10 < 0.0 || avg10 > 100.0 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("avg10 out of range: {}", avg10),
        }));
    }

    Ok((avg10, avg60, avg300, total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_pressure_both_lines() {
        let path = PathBuf::from("/proc/pressure/memory");
        let content = "some avg10=0.50 avg60=1.25 avg300=2.75 total=123456789\n\
                       full avg10=0.25 avg60=0.50 avg300=1.00 total=56789012\n";
        let metrics = parse_pressure_content(content, &path).unwrap();

        assert!((metrics.some_avg10 - 0.50).abs() < f64::EPSILON);
        assert!((metrics.some_avg60 - 1.25).abs() < 0.01);
        assert_eq!(metrics.some_total, 123456789);
        assert_eq!(metrics.full_avg10, Some(0.25));
        assert_eq!(metrics.full_total, Some(56789012));
    }

    #[test]
    fn test_parse_pressure_some_only() {
        let path = PathBuf::from("/proc/pressure/cpu");
        let content = "some avg10=5.25 avg60=3.50 avg300=2.00 total=987654321\n";
        let metrics = parse_pressure_content(content, &path).unwrap();

        assert!((metrics.some_avg10 - 5.25).abs() < 0.01);
        assert_eq!(metrics.some_total, 987654321);
        assert_eq!(metrics.full_avg10, None);
        assert_eq!(metrics.full_total, None);
    }

    #[test]
    fn test_parse_pressure_invalid_percentage() {
        let path = PathBuf::from("/proc/pressure/memory");
        let content = "some avg10=150.0 avg60=1.0 avg300=1.0 total=123\n";
        let result = parse_pressure_content(content, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pressure_missing_field() {
        let path = PathBuf::from("/proc/pressure/memory");
        let content = "some avg10=0.5 avg60=1.0 total=123\n";
        let result = parse_pressure_content(content, &path);
        assert!(result.is_err());
    }
}
