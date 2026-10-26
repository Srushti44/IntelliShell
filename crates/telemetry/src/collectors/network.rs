// Network metrics collector

use crate::error::Result;
use crate::parsers::netdev;
use crate::snapshot::{NetworkStats, NetworkInterface, InterfaceKind, Snapshot};

/// Collect network metrics from /proc/net/dev
pub fn collect_network(prev: Option<&Snapshot>) -> Result<NetworkStats> {
    let interfaces_map = netdev::parse_proc_net_dev()?;
    
    let interval_secs = if let Some(prev_snapshot) = prev {
        (prev_snapshot.measured_interval_ms as f64) / 1000.0
    } else {
        1.0
    };

    let mut interfaces = Vec::new();
    let mut aggregate_rx = 0.0;
    let mut aggregate_tx = 0.0;

    for (name, stats) in interfaces_map {
        let kind = classify_interface(&name);
        
        // Calculate rates if we have previous sample
        let (rx_rate, tx_rate, rx_pps, tx_pps) = if let Some(prev_snapshot) = prev {
            if let Some(prev_iface) = prev_snapshot.network.interfaces.iter().find(|i| i.name == name) {
                let rx_bytes_delta = stats.rx_bytes.saturating_sub(
                    (prev_iface.rx_bytes_per_sec.unwrap_or(0.0) * interval_secs) as u64
                );
                let tx_bytes_delta = stats.tx_bytes.saturating_sub(
                    (prev_iface.tx_bytes_per_sec.unwrap_or(0.0) * interval_secs) as u64
                );
                let rx_packets_delta = stats.rx_packets.saturating_sub(
                    (prev_iface.rx_packets_per_sec.unwrap_or(0.0) * interval_secs) as u64
                );
                let tx_packets_delta = stats.tx_packets.saturating_sub(
                    (prev_iface.tx_packets_per_sec.unwrap_or(0.0) * interval_secs) as u64
                );
                
                let rx_rate = rx_bytes_delta as f64 / interval_secs;
                let tx_rate = tx_bytes_delta as f64 / interval_secs;
                let rx_pps = rx_packets_delta as f64 / interval_secs;
                let tx_pps = tx_packets_delta as f64 / interval_secs;
                
                (Some(rx_rate), Some(tx_rate), Some(rx_pps), Some(tx_pps))
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        if let Some(r) = rx_rate {
            aggregate_rx += r;
        }
        if let Some(t) = tx_rate {
            aggregate_tx += t;
        }

        interfaces.push(NetworkInterface {
            name,
            kind,
            state: "up".to_string(), // TODO: Read from /sys/class/net
            speed_mbps: None, // TODO: Read from /sys/class/net
            mtu: None, // TODO: Read from /sys/class/net
            rx_bytes_per_sec: rx_rate,
            tx_bytes_per_sec: tx_rate,
            rx_packets_per_sec: rx_pps,
            tx_packets_per_sec: tx_pps,
            rx_errors: stats.rx_errors,
            tx_errors: stats.tx_errors,
        });
    }

    Ok(NetworkStats {
        interfaces,
        tcp_states: std::collections::HashMap::new(), // TODO: Parse from /proc/net/tcp
        aggregate_rx_bytes_per_sec: aggregate_rx,
        aggregate_tx_bytes_per_sec: aggregate_tx,
    })
}

fn classify_interface(name: &str) -> InterfaceKind {
    if name == "lo" {
        InterfaceKind::Loopback
    } else if name.starts_with("eth") || name.starts_with("en") || name.starts_with("wlan") {
        InterfaceKind::Physical
    } else {
        InterfaceKind::Virtual
    }
}
