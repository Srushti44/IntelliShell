// Memory metrics collector

use crate::error::Result;
use crate::parsers::{meminfo, pressure};
use crate::snapshot::{MemoryStats, SwapStats, PressureStats, PressureMetric};
use std::path::Path;

/// Collect memory and swap metrics from /proc/meminfo
pub fn collect_memory() -> Result<(MemoryStats, SwapStats)> {
    let fields = meminfo::parse_proc_meminfo()?;
    
    // Extract memory fields (all in bytes now)
    let total_bytes = *fields.get("MemTotal").unwrap_or(&0);
    let free_bytes = *fields.get("MemFree").unwrap_or(&0);
    let available_bytes = *fields.get("MemAvailable").unwrap_or(&0);
    let buffers_bytes = *fields.get("Buffers").unwrap_or(&0);
    let cached_bytes = *fields.get("Cached").unwrap_or(&0);
    let shmem_bytes = *fields.get("Shmem").unwrap_or(&0);
    let dirty_bytes = *fields.get("Dirty").unwrap_or(&0);
    let writeback_bytes = *fields.get("Writeback").unwrap_or(&0);
    let anon_bytes = *fields.get("AnonPages").unwrap_or(&0);
    let slab_bytes = *fields.get("Slab").unwrap_or(&0);

    let used_bytes = total_bytes.saturating_sub(free_bytes + buffers_bytes + cached_bytes);

    // Extract swap fields
    let swap_total_bytes = *fields.get("SwapTotal").unwrap_or(&0);
    let swap_free_bytes = *fields.get("SwapFree").unwrap_or(&0);
    let swap_used_bytes = swap_total_bytes.saturating_sub(swap_free_bytes);

    let memory = MemoryStats {
        total_bytes,
        free_bytes,
        available_bytes,
        used_bytes,
        buffers_bytes,
        cached_bytes,
        shmem_bytes,
        dirty_bytes,
        writeback_bytes,
        anon_bytes,
        slab_bytes,
    };

    let swap = SwapStats {
        total_bytes: swap_total_bytes,
        free_bytes: swap_free_bytes,
        used_bytes: swap_used_bytes,
        swap_in_per_sec: None,
        swap_out_per_sec: None,
        oom_kill_count: None,
    };

    Ok((memory, swap))
}

/// Collect PSI (Pressure Stall Information) metrics
pub fn collect_pressure() -> Result<PressureStats> {
    let cpu_path = Path::new("/proc/pressure/cpu");
    let memory_path = Path::new("/proc/pressure/memory");
    let io_path = Path::new("/proc/pressure/io");

    let cpu = pressure::parse_pressure_file(cpu_path)
        .ok()
        .map(|p| PressureMetric {
            some_avg10: p.some_avg10,
            some_avg60: p.some_avg60,
            some_avg300: p.some_avg300,
            some_total_us: p.some_total,
            full_avg10: p.full_avg10,
            full_avg60: p.full_avg60,
            full_avg300: p.full_avg300,
            full_total_us: p.full_total,
        });

    let memory = pressure::parse_pressure_file(memory_path)
        .ok()
        .map(|p| PressureMetric {
            some_avg10: p.some_avg10,
            some_avg60: p.some_avg60,
            some_avg300: p.some_avg300,
            some_total_us: p.some_total,
            full_avg10: p.full_avg10,
            full_avg60: p.full_avg60,
            full_avg300: p.full_avg300,
            full_total_us: p.full_total,
        });

    let io = pressure::parse_pressure_file(io_path)
        .ok()
        .map(|p| PressureMetric {
            some_avg10: p.some_avg10,
            some_avg60: p.some_avg60,
            some_avg300: p.some_avg300,
            some_total_us: p.some_total,
            full_avg10: p.full_avg10,
            full_avg60: p.full_avg60,
            full_avg300: p.full_avg300,
            full_total_us: p.full_total,
        });

    Ok(PressureStats { cpu, memory, io })
}
