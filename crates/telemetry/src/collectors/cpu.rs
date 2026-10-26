// CPU metrics collector

use crate::error::Result;
use crate::parsers::stat;
use crate::snapshot::{CpuStats, CpuCoreStats, Snapshot};
use crate::SystemConfig;

/// Collect CPU metrics from /proc/stat
pub fn collect_cpu(config: &SystemConfig, prev: Option<&Snapshot>) -> Result<CpuStats> {
    let (cpu_stat, _boot_time) = stat::parse_proc_stat(config.clock_ticks_per_sec as u64)?;

    // Calculate percentages if we have a previous sample
    let aggregate = if let Some(prev_snapshot) = prev {
        let prev_cpu = &prev_snapshot.cpu.aggregate;
        calculate_cpu_percentages(&cpu_stat, prev_cpu, config.clock_ticks_per_sec)
    } else {
        // First sample - no percentages yet
        CpuCoreStats {
            core_id: None,
            user_percent: 0.0,
            nice_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 100.0,
            iowait_percent: 0.0,
            irq_percent: 0.0,
            steal_percent: 0.0,
            busy_percent: 0.0,
        }
    };

    Ok(CpuStats {
        aggregate,
        per_core: Vec::new(), // TODO: Parse per-core stats from /proc/stat
        context_switches_per_sec: None, // TODO: Parse from /proc/stat
        forks_per_sec: None, // TODO: Parse from /proc/stat
        procs_running: 0, // TODO: Parse from /proc/stat
        procs_blocked: 0, // TODO: Parse from /proc/stat
    })
}

/// Calculate CPU percentages from tick deltas
fn calculate_cpu_percentages(
    curr: &stat::CpuStat,
    prev: &CpuCoreStats,
    _clk_tck: i64,
) -> CpuCoreStats {
    // Calculate current totals
    let curr_total = curr.total();
    let curr_idle = curr.idle + curr.iowait;
    
    // Previous totals (reconstruct from percentages)
    // This is a simplification - ideally we'd store raw ticks
    let prev_total_approx = 1000u64; // Placeholder
    let prev_idle_approx = (prev.idle_percent * 10.0) as u64;
    
    let total_delta = curr_total.saturating_sub(prev_total_approx);
    let _idle_delta = curr_idle.saturating_sub(prev_idle_approx);
    
    if total_delta == 0 {
        return CpuCoreStats {
            core_id: None,
            user_percent: 0.0,
            nice_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 100.0,
            iowait_percent: 0.0,
            irq_percent: 0.0,
            steal_percent: 0.0,
            busy_percent: 0.0,
        };
    }

    let user_percent = (curr.user as f64 / total_delta as f64) * 100.0;
    let nice_percent = (curr.nice as f64 / total_delta as f64) * 100.0;
    let system_percent = (curr.system as f64 / total_delta as f64) * 100.0;
    let idle_percent = (curr.idle as f64 / total_delta as f64) * 100.0;
    let iowait_percent = (curr.iowait as f64 / total_delta as f64) * 100.0;
    let irq_percent = ((curr.irq + curr.softirq) as f64 / total_delta as f64) * 100.0;
    let steal_percent = (curr.steal as f64 / total_delta as f64) * 100.0;
    let busy_percent = 100.0 - idle_percent - iowait_percent;

    CpuCoreStats {
        core_id: None,
        user_percent,
        nice_percent,
        system_percent,
        idle_percent,
        iowait_percent,
        irq_percent,
        steal_percent,
        busy_percent,
    }
}
