// Disk metrics collector

use crate::error::Result;
use crate::parsers::diskstats;
use crate::snapshot::{DiskStats, Snapshot};
use nix::sys::statvfs::statvfs;
use std::path::Path;

const SECTOR_SIZE: u64 = 512;

/// Collect disk metrics from /proc/diskstats and statvfs
pub fn collect_disk(prev: Option<&Snapshot>) -> Result<Vec<DiskStats>> {
    let disk_stats_map = diskstats::parse_proc_diskstats()?;
    let mut disks = Vec::new();

    // Get filesystem stats for root
    let root_fs = collect_filesystem_stats(Path::new("/")).ok();

    // Measure interval in seconds
    let interval_secs = if let Some(prev_snapshot) = prev {
        (prev_snapshot.measured_interval_ms as f64) / 1000.0
    } else {
        1.0 // Default to 1 second for first sample
    };

    for (device_name, stats) in disk_stats_map {
        if !is_physical_device(&device_name) {
            continue;
        }

        let bytes_read = stats.sectors_read * SECTOR_SIZE;
        let bytes_written = stats.sectors_written * SECTOR_SIZE;

        // Calculate rates if we have previous sample
        let (read_rate, write_rate, read_iops, write_iops) = if let Some(prev_snapshot) = prev {
            if let Some(prev_disk) = prev_snapshot.disks.iter().find(|d| d.device == device_name) {
                let prev_reads = prev_disk.read_bytes_per_sec.unwrap_or(0.0) * interval_secs;
                let prev_writes = prev_disk.write_bytes_per_sec.unwrap_or(0.0) * interval_secs;
                
                let read_bytes_delta = bytes_read.saturating_sub(prev_reads as u64);
                let write_bytes_delta = bytes_written.saturating_sub(prev_writes as u64);
                
                let read_rate = read_bytes_delta as f64 / interval_secs;
                let write_rate = write_bytes_delta as f64 / interval_secs;
                
                // IOPS calculation
                let read_iops = stats.reads_completed as f64 / interval_secs;
                let write_iops = stats.writes_completed as f64 / interval_secs;
                
                (Some(read_rate), Some(write_rate), Some(read_iops), Some(write_iops))
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        let is_primary = is_primary_device(&device_name);
        let (total_bytes, used_bytes, available_bytes) = if is_primary && root_fs.is_some() {
            let fs = root_fs.as_ref().unwrap();
            (Some(fs.0), Some(fs.1), Some(fs.2))
        } else {
            (None, None, None)
        };

        disks.push(DiskStats {
            device: device_name,
            mount_point: if is_primary { Some("/".to_string()) } else { None },
            fs_type: None, // TODO: Parse from /proc/mounts
            total_bytes,
            used_bytes,
            available_bytes,
            read_bytes_per_sec: read_rate,
            write_bytes_per_sec: write_rate,
            read_iops,
            write_iops,
            utilization_percent: None, // TODO: Calculate from time_io_ms
            is_whole_disk: true,
            is_readonly: false, // TODO: Parse from /sys/block
        });
    }

    Ok(disks)
}

fn collect_filesystem_stats(path: &Path) -> Result<(u64, u64, u64)> {
    let stats = statvfs(path)?;
    let block_size = stats.block_size();
    let total_bytes = stats.blocks() * block_size;
    let available_bytes = stats.blocks_available() * block_size;
    let free_bytes = stats.blocks_free() * block_size;
    let used_bytes = total_bytes.saturating_sub(free_bytes);
    Ok((total_bytes, used_bytes, available_bytes))
}

fn is_physical_device(name: &str) -> bool {
    if name.starts_with("loop") || name.starts_with("dm-") || name.starts_with("ram") {
        return false;
    }
    if name.starts_with("sd") || name.starts_with("vd") || name.starts_with("xvd") || name.starts_with("hd") {
        return !name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false);
    }
    if name.starts_with("nvme") {
        return name.contains("n") && !name.contains("p");
    }
    false
}

fn is_primary_device(name: &str) -> bool {
    matches!(name, "sda" | "nvme0n1" | "vda" | "xvda")
}
