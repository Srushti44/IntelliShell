// Process metrics collector

use crate::error::Result;
use crate::parsers::procstat;
use crate::snapshot::{ProcessInfo, ProcessState, Snapshot};
use crate::SystemConfig;
use std::fs;

const MAX_PROCESSES: usize = 10000;

/// Collect process metrics for all running processes
pub fn collect_processes(config: &SystemConfig, prev: Option<&Snapshot>) -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();
    
    let proc_entries = fs::read_dir("/proc")?;

    let interval_secs = if let Some(prev_snapshot) = prev {
        (prev_snapshot.measured_interval_ms as f64) / 1000.0
    } else {
        1.0
    };

    for entry in proc_entries {
        if processes.len() >= MAX_PROCESSES {
            break;
        }

        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if let Ok(pid) = name_str.parse::<i32>() {
            if let Ok(proc_stat) = procstat::parse_proc_pid_stat(pid) {
                let cpu_time_ticks = proc_stat.utime + proc_stat.stime;
                let memory_rss_bytes = (proc_stat.rss.max(0) as u64) * (config.page_size_bytes as u64);

                // Calculate CPU percentage if we have previous sample
                let (cpu_percent_of_core, cpu_percent_of_system) = if let Some(prev_snapshot) = prev {
                    if let Some(prev_proc) = prev_snapshot.processes.iter().find(|p| 
                        p.pid == pid as u32 && p.start_time_ticks == proc_stat.starttime
                    ) {
                        let cpu_delta_ticks = cpu_time_ticks.saturating_sub(
                            prev_proc.cpu_percent_of_core.unwrap_or(0.0) as u64
                        );
                        let elapsed_ticks = (interval_secs * config.clock_ticks_per_sec as f64) as u64;
                        
                        if elapsed_ticks > 0 {
                            let cpu_of_core = (cpu_delta_ticks as f64 / elapsed_ticks as f64) * 100.0;
                            let cpu_of_system = cpu_of_core / config.cpu_count as f64;
                            (Some(cpu_of_core), Some(cpu_of_system))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                // Read cmdline
                let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", pid))
                    .ok()
                    .map(|s| s.replace('\0', " ").trim().to_string())
                    .filter(|s| !s.is_empty());

                processes.push(ProcessInfo {
                    pid: pid as u32,
                    start_time_ticks: proc_stat.starttime,
                    name: proc_stat.comm.trim_matches(|c| c == '(' || c == ')').to_string(),
                    state: ProcessState::from_char(proc_stat.state),
                    ppid: proc_stat.ppid as u32,
                    is_kernel_thread: proc_stat.ppid == 2 || pid == 2,
                    cpu_percent_of_core,
                    cpu_percent_of_system,
                    memory_rss_bytes,
                    memory_vsize_bytes: proc_stat.vsize,
                    num_threads: proc_stat.num_threads as u32,
                    cmdline,
                    username: None, // TODO: Read from /proc/<pid>/status
                    read_bytes_per_sec: None, // TODO: Read from /proc/<pid>/io
                    write_bytes_per_sec: None, // TODO: Read from /proc/<pid>/io
                    cgroup_path: None, // TODO: Read from /proc/<pid>/cgroup
                    partial: false,
                });
            }
        }
    }

    Ok(processes)
}
