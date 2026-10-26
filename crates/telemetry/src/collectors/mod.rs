// Collector modules that build the complete Snapshot from /proc parsers
// Handles rate calculations between samples

pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod process;

use crate::error::Result;
use crate::snapshot::*;
use crate::SystemConfig;
use std::time::{SystemTime, UNIX_EPOCH};

/// Collect a complete system snapshot
pub fn collect_snapshot(
    sample_id: u64,
    config: &SystemConfig,
    prev: Option<&Snapshot>,
) -> Result<Snapshot> {
    let start = std::time::Instant::now();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    // Calculate measured interval from previous sample
    let measured_interval_ms = if let Some(prev_snapshot) = prev {
        timestamp.as_millis().saturating_sub(prev_snapshot.timestamp_unix_ms as u128) as u64
    } else {
        0
    };

    // Collect all metrics
    let mut errors = std::collections::HashMap::new();
    
    let cpu_stats = match cpu::collect_cpu(config, prev) {
        Ok(stats) => stats,
        Err(e) => {
            errors.insert("cpu".to_string(), e.to_string());
            CpuStats::default()
        }
    };

    let load = match crate::parsers::loadavg::parse_proc_loadavg() {
        Ok(la) => LoadAverage {
            load1: la.load1,
            load5: la.load5,
            load15: la.load15,
        },
        Err(e) => {
            errors.insert("load".to_string(), e.to_string());
            LoadAverage::default()
        }
    };

    let (memory_stats, swap_stats) = match memory::collect_memory() {
        Ok((mem, swap)) => (mem, swap),
        Err(e) => {
            errors.insert("memory".to_string(), e.to_string());
            (MemoryStats::default(), SwapStats::default())
        }
    };

    let pressure_stats = match memory::collect_pressure() {
        Ok(p) => Some(p),
        Err(_) => None, // Pressure may not be available
    };

    let disks = match disk::collect_disk(prev) {
        Ok(d) => d,
        Err(e) => {
            errors.insert("disk".to_string(), e.to_string());
            vec![]
        }
    };

    let network_stats = match network::collect_network(prev) {
        Ok(n) => n,
        Err(e) => {
            errors.insert("network".to_string(), e.to_string());
            NetworkStats::default()
        }
    };

    let processes = match process::collect_processes(config, prev) {
        Ok(p) => p,
        Err(e) => {
            errors.insert("processes".to_string(), e.to_string());
            vec![]
        }
    };

    // Calculate kernel version and uptime
    let kernel_version = std::fs::read_to_string("/proc/version")
        .unwrap_or_else(|_| String::from("unknown"));
    let uptime_secs = timestamp.as_secs().saturating_sub(config.boot_time_secs) as f64;

    let collection_duration_ms = start.elapsed().as_millis() as u64;
    let degraded = !errors.is_empty();

    Ok(Snapshot {
        schema_version: SCHEMA_VERSION.to_string(),
        sample_id,
        timestamp_unix_ms: timestamp.as_millis() as u64,
        measured_interval_ms,
        host_info: HostInfo {
            kernel_version: kernel_version.lines().next().unwrap_or("unknown").to_string(),
            cpu_count: config.cpu_count,
            clock_ticks_per_sec: config.clock_ticks_per_sec,
            page_size_bytes: config.page_size_bytes,
            boot_time_unix_secs: config.boot_time_secs,
            uptime_secs,
        },
        cpu: cpu_stats,
        load,
        memory: memory_stats,
        swap: swap_stats,
        pressure: pressure_stats,
        disks,
        network: network_stats,
        processes,
        health: HealthInfo {
            degraded,
            collection_duration_ms,
            errors,
        },
    })
}

// Default implementations
impl Default for CpuStats {
    fn default() -> Self {
        Self {
            aggregate: CpuCoreStats::default(),
            per_core: Vec::new(),
            context_switches_per_sec: None,
            forks_per_sec: None,
            procs_running: 0,
            procs_blocked: 0,
        }
    }
}

impl Default for LoadAverage {
    fn default() -> Self {
        Self {
            load1: 0.0,
            load5: 0.0,
            load15: 0.0,
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            free_bytes: 0,
            available_bytes: 0,
            used_bytes: 0,
            buffers_bytes: 0,
            cached_bytes: 0,
            shmem_bytes: 0,
            dirty_bytes: 0,
            writeback_bytes: 0,
            anon_bytes: 0,
            slab_bytes: 0,
        }
    }
}

impl Default for SwapStats {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            free_bytes: 0,
            used_bytes: 0,
            swap_in_per_sec: None,
            swap_out_per_sec: None,
            oom_kill_count: None,
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            tcp_states: std::collections::HashMap::new(),
            aggregate_rx_bytes_per_sec: 0.0,
            aggregate_tx_bytes_per_sec: 0.0,
        }
    }
}
