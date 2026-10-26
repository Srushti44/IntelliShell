// /proc/net/dev parser for network interface statistics
// Format documented in: net/core/net-procfs.c in kernel source

use crate::error::{ParseError, TelemetryError};
use crate::parsers::{parse_u64, read_proc_file};
use std::collections::HashMap;
use std::path::Path;

const PROC_NET_DEV_PATH: &str = "/proc/net/dev";
const MAX_FILE_SIZE: usize = 65536; // 64KB limit

/// Network interface statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetDevStats {
    pub interface: String,
    // Receive stats
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errors: u64,
    pub rx_dropped: u64,
    pub rx_fifo: u64,
    pub rx_frame: u64,
    pub rx_compressed: u64,
    pub rx_multicast: u64,
    // Transmit stats
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errors: u64,
    pub tx_dropped: u64,
    pub tx_fifo: u64,
    pub tx_collisions: u64,
    pub tx_carrier: u64,
    pub tx_compressed: u64,
}

/// Parse /proc/net/dev and return map of interface -> stats
pub fn parse_proc_net_dev() -> Result<HashMap<String, NetDevStats>, TelemetryError> {
    let path = Path::new(PROC_NET_DEV_PATH);
    let content = read_proc_file(path, MAX_FILE_SIZE)?;
    parse_net_dev_content(&content, path)
}

fn parse_net_dev_content(
    content: &str,
    path: &Path,
) -> Result<HashMap<String, NetDevStats>, TelemetryError> {
    let mut interfaces = HashMap::new();
    let mut lines = content.lines();

    // Skip header lines (first 2 lines)
    lines.next();
    lines.next();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let stats = parse_net_dev_line(line, path)?;
        interfaces.insert(stats.interface.clone(), stats);
    }

    Ok(interfaces)
}

fn parse_net_dev_line(line: &str, path: &Path) -> Result<NetDevStats, TelemetryError> {
    // Format: "  eth0: 1234 100 0 0 0 0 0 0 5678 200 0 0 0 0 0 0"
    //         interface: rx_bytes rx_packets ... tx_bytes tx_packets ...
    
    let colon_pos = line.find(':').ok_or_else(|| {
        TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!("No colon found in line: {}", line),
        })
    })?;

    let interface = line[..colon_pos].trim().to_string();
    let stats_part = &line[colon_pos + 1..];
    
    let fields: Vec<&str> = stats_part.split_whitespace().collect();
    
    if fields.len() < 16 {
        return Err(TelemetryError::Parse(ParseError::InvalidFormat {
            path: path.to_path_buf(),
            reason: format!(
                "Interface {} has {} fields, expected 16",
                interface,
                fields.len()
            ),
        }));
    }

    Ok(NetDevStats {
        interface,
        rx_bytes: parse_u64(fields[0], "rx_bytes", path)?,
        rx_packets: parse_u64(fields[1], "rx_packets", path)?,
        rx_errors: parse_u64(fields[2], "rx_errors", path)?,
        rx_dropped: parse_u64(fields[3], "rx_dropped", path)?,
        rx_fifo: parse_u64(fields[4], "rx_fifo", path)?,
        rx_frame: parse_u64(fields[5], "rx_frame", path)?,
        rx_compressed: parse_u64(fields[6], "rx_compressed", path)?,
        rx_multicast: parse_u64(fields[7], "rx_multicast", path)?,
        tx_bytes: parse_u64(fields[8], "tx_bytes", path)?,
        tx_packets: parse_u64(fields[9], "tx_packets", path)?,
        tx_errors: parse_u64(fields[10], "tx_errors", path)?,
        tx_dropped: parse_u64(fields[11], "tx_dropped", path)?,
        tx_fifo: parse_u64(fields[12], "tx_fifo", path)?,
        tx_collisions: parse_u64(fields[13], "tx_collisions", path)?,
        tx_carrier: parse_u64(fields[14], "tx_carrier", path)?,
        tx_compressed: parse_u64(fields[15], "tx_compressed", path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_NET_DEV: &str = r#"Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 123456    1000    0    0    0     0          0         0   123456    1000    0    0    0     0       0          0
  eth0: 987654321 5000000  10   5    0     0          0       100 456789012 3000000  2    1    0    50      10          0
"#;

    #[test]
    fn test_parse_net_dev_line() {
        let path = PathBuf::from("/proc/net/dev");
        let line = "  eth0: 987654321 5000000  10   5    0     0          0       100 456789012 3000000  2    1    0    50      10          0";
        let stats = parse_net_dev_line(line, &path).unwrap();

        assert_eq!(stats.interface, "eth0");
        assert_eq!(stats.rx_bytes, 987654321);
        assert_eq!(stats.rx_packets, 5000000);
        assert_eq!(stats.rx_errors, 10);
        assert_eq!(stats.tx_bytes, 456789012);
        assert_eq!(stats.tx_packets, 3000000);
        assert_eq!(stats.tx_collisions, 50);
    }

    #[test]
    fn test_parse_net_dev_content() {
        let path = PathBuf::from("/proc/net/dev");
        let interfaces = parse_net_dev_content(TEST_NET_DEV, &path).unwrap();

        assert_eq!(interfaces.len(), 2);
        assert!(interfaces.contains_key("lo"));
        assert!(interfaces.contains_key("eth0"));

        let lo = &interfaces["lo"];
        assert_eq!(lo.rx_bytes, 123456);
        assert_eq!(lo.tx_bytes, 123456);

        let eth0 = &interfaces["eth0"];
        assert_eq!(eth0.rx_bytes, 987654321);
        assert_eq!(eth0.tx_bytes, 456789012);
    }

    #[test]
    fn test_invalid_line_no_colon() {
        let path = PathBuf::from("/proc/net/dev");
        let line = "eth0 123 456";
        let result = parse_net_dev_line(line, &path);
        assert!(result.is_err());
    }
}
