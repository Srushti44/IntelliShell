// /proc/loadavg parser for load average metrics
// Format documented in: man 5 proc (search for /proc/loadavg)

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_f64, read_proc_file};
use std::path::Path;

const PROC_LOADAVG_PATH: &str = "/proc/loadavg";
const MAX_FILE_SIZE: usize = 256; // Tiny file

/// Load average metrics
#[derive(Debug, Clone, PartialEq)]
pub struct LoadAvg {
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
    pub runnable_entities: u32,
    pub total_entities: u32,
}

/// Parse /proc/loadavg
pub fn parse_proc_loadavg() -> Result<LoadAvg, TelemetryError> {
    let path = Path::new(PROC_LOADAVG_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_loadavg_content(content.trim(), path)
}

fn parse_loadavg_content(content: &str, path: &Path) -> Result<LoadAvg, TelemetryError> {
    // Format: "0.52 0.58 0.59 1/602 12345"
    //         load1 load5 load15 runnable/total last_pid
    let fields: Vec<&str> = content.split_whitespace().collect();

    if fields.len() < 5 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Expected 5 fields, found {}", fields.len()),
        }));
    }

    let load1 = parse_f64(fields[0], "load1", path)?;
    let load5 = parse_f64(fields[1], "load5", path)?;
    let load15 = parse_f64(fields[2], "load15", path)?;

    // Parse "runnable/total" field
    let entities = fields[3].split('/').collect::<Vec<_>>();
    if entities.len() != 2 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Invalid entities format: {}", fields[3]),
        }));
    }

    let runnable_entities = entities[0].parse::<u32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: "runnable_entities".to_string(),
            value: entities[0].to_string(),
            source: e,
        })
    })?;

    let total_entities = entities[1].parse::<u32>().map_err(|e| {
        TelemetryError::Parse(ParseError::InvalidInteger {
            path: path.to_path_buf(),
            field: "total_entities".to_string(),
            value: entities[1].to_string(),
            source: e,
        })
    })?;

    // Validate values
    if load1 < 0.0 || load5 < 0.0 || load15 < 0.0 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: "Load average values cannot be negative".to_string(),
        }));
    }

    if runnable_entities > total_entities {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!(
                "Runnable entities ({}) exceeds total entities ({})",
                runnable_entities, total_entities
            ),
        }));
    }

    Ok(LoadAvg {
        load1,
        load5,
        load15,
        runnable_entities,
        total_entities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_loadavg_content() {
        let path = PathBuf::from("/proc/loadavg");
        let content = "0.52 0.58 0.59 1/602 12345";
        let load = parse_loadavg_content(content, &path).unwrap();

        assert!((load.load1 - 0.52).abs() < f64::EPSILON);
        assert!((load.load5 - 0.58).abs() < f64::EPSILON);
        assert!((load.load15 - 0.59).abs() < f64::EPSILON);
        assert_eq!(load.runnable_entities, 1);
        assert_eq!(load.total_entities, 602);
    }

    #[test]
    fn test_high_load() {
        let path = PathBuf::from("/proc/loadavg");
        let content = "12.45 8.32 5.67 15/1024 99999";
        let load = parse_loadavg_content(content, &path).unwrap();

        assert!((load.load1 - 12.45).abs() < 0.01);
        assert_eq!(load.runnable_entities, 15);
        assert_eq!(load.total_entities, 1024);
    }

    #[test]
    fn test_invalid_format() {
        let path = PathBuf::from("/proc/loadavg");
        
        // Too few fields
        let result = parse_loadavg_content("0.52 0.58", &path);
        assert!(result.is_err());

        // Invalid entities format
        let result = parse_loadavg_content("0.52 0.58 0.59 invalid 12345", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_negative_load() {
        let path = PathBuf::from("/proc/loadavg");
        let result = parse_loadavg_content("-0.5 0.58 0.59 1/602 12345", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_runnable_exceeds_total() {
        let path = PathBuf::from("/proc/loadavg");
        let result = parse_loadavg_content("0.52 0.58 0.59 100/50 12345", &path);
        assert!(result.is_err());
    }
}
