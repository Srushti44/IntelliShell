// /proc/stat parser for system-wide CPU metrics and boot time
// Format documented in: man 5 proc (search for /proc/stat)

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_u64, read_proc_file};
use std::path::Path;

const PROC_STAT_PATH: &str = "/proc/stat";
const MAX_FILE_SIZE: usize = 65536; // 64KB limit

/// Raw CPU time counters from /proc/stat (in CLK_TCK units)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuStat {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

impl CpuStat {
    /// Total CPU time including guest time
    /// Guest time is ALREADY INCLUDED in user+nice, so don't double-count
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + self.iowait 
            + self.irq + self.softirq + self.steal
    }

    /// Non-idle time (busy time)
    pub fn busy(&self) -> u64 {
        self.total() - self.idle - self.iowait
    }
}

/// Parse /proc/stat and extract system CPU line and boot time
pub fn parse_proc_stat(clk_tck: u64) -> Result<(CpuStat, u64), TelemetryError> {
    let path = Path::new(PROC_STAT_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_stat_content(&content, clk_tck, path)
}

/// Read boot time from /proc/stat
pub fn read_boot_time() -> Result<u64, TelemetryError> {
    let path = Path::new(PROC_STAT_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("btime ") {
            return parse_btime_line(line, path);
        }
    }
    
    Err(TelemetryError::Parse(ParseError::MissingField {
        path: path.to_path_buf(),
        field: "btime".to_string(),
    }))
}

fn parse_stat_content(content: &str, clk_tck: u64, path: &Path) -> Result<(CpuStat, u64), TelemetryError> {
    let mut cpu_stat: Option<CpuStat> = None;
    let mut boot_time: Option<u64> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("cpu ") {
            cpu_stat = Some(parse_cpu_line(line, path)?);
        } else if line.starts_with("btime ") {
            boot_time = Some(parse_btime_line(line, path)?);
        }

        // Early exit if both found
        if cpu_stat.is_some() && boot_time.is_some() {
            break;
        }
    }

    let cpu_stat = cpu_stat.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "cpu".to_string(),
        })
    })?;

    let boot_time = boot_time.ok_or_else(|| {
        TelemetryError::Parse(ParseError::MissingField {
            path: path.to_path_buf(),
            field: "btime".to_string(),
        })
    })?;

    // Validate CLK_TCK value (common values: 100, 250, 1000)
    if clk_tck == 0 || clk_tck > 10000 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("Invalid CLK_TCK value: {}", clk_tck),
        }));
    }

    Ok((cpu_stat, boot_time))
}

fn parse_cpu_line(line: &str, path: &Path) -> Result<CpuStat, TelemetryError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    
    // cpu user nice system idle iowait irq softirq steal guest guest_nice
    // Minimum 5 fields (kernel 2.5.41+), up to 11 fields (kernel 2.6.33+)
    if fields.len() < 5 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("cpu line has only {} fields, expected at least 5", fields.len()),
        }));
    }

    let user = parse_u64(fields.get(1).copied().unwrap_or("0"), "user", path)?;
    let nice = parse_u64(fields.get(2).copied().unwrap_or("0"), "nice", path)?;
    let system = parse_u64(fields.get(3).copied().unwrap_or("0"), "system", path)?;
    let idle = parse_u64(fields.get(4).copied().unwrap_or("0"), "idle", path)?;
    let iowait = parse_u64(fields.get(5).copied().unwrap_or("0"), "iowait", path)?;
    let irq = parse_u64(fields.get(6).copied().unwrap_or("0"), "irq", path)?;
    let softirq = parse_u64(fields.get(7).copied().unwrap_or("0"), "softirq", path)?;
    let steal = parse_u64(fields.get(8).copied().unwrap_or("0"), "steal", path)?;
    let guest = parse_u64(fields.get(9).copied().unwrap_or("0"), "guest", path)?;
    let guest_nice = parse_u64(fields.get(10).copied().unwrap_or("0"), "guest_nice", path)?;

    Ok(CpuStat {
        user,
        nice,
        system,
        idle,
        iowait,
        irq,
        softirq,
        steal,
        guest,
        guest_nice,
    })
}

fn parse_btime_line(line: &str, path: &Path) -> Result<u64, TelemetryError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    
    if fields.len() != 2 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("btime line has {} fields, expected 2", fields.len()),
        }));
    }

    parse_u64(fields[1], "btime", path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_STAT: &str = r#"cpu  74608 2520 24433 1117073 2122 0 253 0 0 0
cpu0 18652 630 6108 279268 531 0 63 0 0 0
cpu1 18656 630 6108 279268 531 0 63 0 0 0
cpu2 18650 630 6108 279268 530 0 63 0 0 0
cpu3 18649 630 6108 279268 530 0 63 0 0 0
intr 1234567 890
ctxt 2345678
btime 1609459200
processes 12345
procs_running 2
procs_blocked 0
"#;

    #[test]
    fn test_parse_cpu_line() {
        let path = PathBuf::from("/proc/stat");
        let line = "cpu  74608 2520 24433 1117073 2122 0 253 0 0 0";
        let cpu = parse_cpu_line(line, &path).unwrap();
        
        assert_eq!(cpu.user, 74608);
        assert_eq!(cpu.nice, 2520);
        assert_eq!(cpu.system, 24433);
        assert_eq!(cpu.idle, 1117073);
        assert_eq!(cpu.iowait, 2122);
        assert_eq!(cpu.irq, 0);
        assert_eq!(cpu.softirq, 253);
        assert_eq!(cpu.steal, 0);
        assert_eq!(cpu.guest, 0);
        assert_eq!(cpu.guest_nice, 0);
    }

    #[test]
    fn test_cpu_stat_total() {
        let cpu = CpuStat {
            user: 100,
            nice: 10,
            system: 50,
            idle: 800,
            iowait: 20,
            irq: 5,
            softirq: 10,
            steal: 5,
            guest: 0, // Already in user
            guest_nice: 0, // Already in nice
        };
        
        // Total should NOT double-count guest time
        assert_eq!(cpu.total(), 1000);
        assert_eq!(cpu.busy(), 180);
    }

    #[test]
    fn test_parse_stat_content() {
        let path = PathBuf::from("/proc/stat");
        let (cpu, btime) = parse_stat_content(TEST_STAT, 100, &path).unwrap();
        
        assert_eq!(cpu.user, 74608);
        assert_eq!(btime, 1609459200);
    }

    #[test]
    fn test_parse_stat_missing_cpu() {
        let path = PathBuf::from("/proc/stat");
        let content = "btime 1609459200\n";
        let result = parse_stat_content(content, 100, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_stat_missing_btime() {
        let path = PathBuf::from("/proc/stat");
        let content = "cpu  74608 2520 24433 1117073 2122 0 253 0 0 0\n";
        let result = parse_stat_content(content, 100, &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_clk_tck() {
        let path = PathBuf::from("/proc/stat");
        let result = parse_stat_content(TEST_STAT, 0, &path);
        assert!(result.is_err());
        
        let result = parse_stat_content(TEST_STAT, 99999, &path);
        assert!(result.is_err());
    }
}
