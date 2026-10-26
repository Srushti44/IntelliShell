//! Snapshot data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_VERSION: &str = "1.0.0";

/// Complete system snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: String,
    pub sample_id: u64,
    pub timestamp_unix_ms: u64,
    pub measured_interval_ms: u64,
    pub host_info: HostInfo,
    pub cpu: CpuStats,
    pub load: LoadAverage,
    pub memory: MemoryStats,
    pub swap: SwapStats,
    pub pressure: Option<PressureStats>,
    pub disks: Vec<DiskStats>,
    pub network: NetworkStats,
    pub processes: Vec<ProcessInfo>,
    pub health: HealthInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub kernel_version: String,
    pub cpu_count: usize,
    pub clock_ticks_per_sec: i64,
    pub page_size_bytes: i64,
    pub boot_time_unix_secs: u64,
    pub uptime_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub aggregate: CpuCoreStats,
    pub per_core: Vec<CpuCoreStats>,
    pub context_switches_per_sec: Option<f64>,
    pub forks_per_sec: Option<f64>,
    pub procs_running: u32,
    pub procs_blocked: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreStats {
    pub core_id: Option<usize>,
    pub user_percent: f64,
    pub nice_percent: f64,
    pub system_percent: f64,
    pub idle_percent: f64,
    pub iowait_percent: f64,
    pub irq_percent: f64,
    pub steal_percent: f64,
    pub busy_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub shmem_bytes: u64,
    pub dirty_bytes: u64,
    pub writeback_bytes: u64,
    pub anon_bytes: u64,
    pub slab_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStats {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub swap_in_per_sec: Option<f64>,
    pub swap_out_per_sec: Option<f64>,
    pub oom_kill_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureStats {
    pub cpu: Option<PressureMetric>,
    pub memory: Option<PressureMetric>,
    pub io: Option<PressureMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureMetric {
    pub some_avg10: f64,
    pub some_avg60: f64,
    pub some_avg300: f64,
    pub some_total_us: u64,
    pub full_avg10: Option<f64>,
    pub full_avg60: Option<f64>,
    pub full_avg300: Option<f64>,
    pub full_total_us: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    pub device: String,
    pub mount_point: Option<String>,
    pub fs_type: Option<String>,
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub read_bytes_per_sec: Option<f64>,
    pub write_bytes_per_sec: Option<f64>,
    pub read_iops: Option<f64>,
    pub write_iops: Option<f64>,
    pub utilization_percent: Option<f64>,
    pub is_whole_disk: bool,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterface>,
    pub tcp_states: HashMap<String, u32>,
    pub aggregate_rx_bytes_per_sec: f64,
    pub aggregate_tx_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub kind: InterfaceKind,
    pub state: String,
    pub speed_mbps: Option<u64>,
    pub mtu: Option<u32>,
    pub rx_bytes_per_sec: Option<f64>,
    pub tx_bytes_per_sec: Option<f64>,
    pub rx_packets_per_sec: Option<f64>,
    pub tx_packets_per_sec: Option<f64>,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterfaceKind {
    Loopback,
    Physical,
    Virtual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub start_time_ticks: u64,
    pub name: String,
    pub state: ProcessState,
    pub ppid: u32,
    pub is_kernel_thread: bool,
    pub cpu_percent_of_core: Option<f64>,
    pub cpu_percent_of_system: Option<f64>,
    pub memory_rss_bytes: u64,
    pub memory_vsize_bytes: u64,
    pub num_threads: u32,
    pub cmdline: Option<String>,
    pub username: Option<String>,
    pub read_bytes_per_sec: Option<f64>,
    pub write_bytes_per_sec: Option<f64>,
    pub cgroup_path: Option<String>,
    pub partial: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    Stopped,
    TracingStop,
    Dead,
    Unknown,
}

impl ProcessState {
    pub fn from_char(c: char) -> Self {
        match c {
            'R' => Self::Running,
            'S' => Self::Sleeping,
            'D' => Self::DiskSleep,
            'Z' => Self::Zombie,
            'T' => Self::Stopped,
            't' => Self::TracingStop,
            'X' | 'x' => Self::Dead,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    pub degraded: bool,
    pub collection_duration_ms: u64,
    pub errors: HashMap<String, String>,
}

impl Snapshot {
    pub fn new(sample_id: u64, config: &crate::SystemConfig) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            sample_id,
            timestamp_unix_ms: now.as_millis() as u64,
            measured_interval_ms: 0,
            host_info: HostInfo {
                kernel_version: String::new(),
                cpu_count: config.cpu_count,
                clock_ticks_per_sec: config.clock_ticks_per_sec,
                page_size_bytes: config.page_size_bytes,
                boot_time_unix_secs: config.boot_time_secs,
                uptime_secs: 0.0,
            },
            cpu: CpuStats {
                aggregate: CpuCoreStats::default(),
                per_core: Vec::new(),
                context_switches_per_sec: None,
                forks_per_sec: None,
                procs_running: 0,
                procs_blocked: 0,
            },
            load: LoadAverage {
                load1: 0.0,
                load5: 0.0,
                load15: 0.0,
            },
            memory: MemoryStats {
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
            },
            swap: SwapStats {
                total_bytes: 0,
                free_bytes: 0,
                used_bytes: 0,
                swap_in_per_sec: None,
                swap_out_per_sec: None,
                oom_kill_count: None,
            },
            pressure: None,
            disks: Vec::new(),
            network: NetworkStats {
                interfaces: Vec::new(),
                tcp_states: HashMap::new(),
                aggregate_rx_bytes_per_sec: 0.0,
                aggregate_tx_bytes_per_sec: 0.0,
            },
            processes: Vec::new(),
            health: HealthInfo {
                degraded: false,
                collection_duration_ms: 0,
                errors: HashMap::new(),
            },
        }
    }
}

impl Default for CpuCoreStats {
    fn default() -> Self {
        Self {
            core_id: None,
            user_percent: 0.0,
            nice_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 0.0,
            iowait_percent: 0.0,
            irq_percent: 0.0,
            steal_percent: 0.0,
            busy_percent: 0.0,
        }
    }
}
