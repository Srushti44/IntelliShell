//! Property-based tests using proptest

use intellishell_telemetry::parsers::*;
use proptest::prelude::*;
use std::path::PathBuf;

// Test that parser never panics on arbitrary input
proptest! {
    #[test]
    fn test_loadavg_never_panics(input in ".*") {
        let path = PathBuf::from("/test/loadavg");
        let _ = loadavg::parse_loadavg_content(&input, &path);
        // Success: no panic
    }

    #[test]
    fn test_meminfo_never_panics(input in ".*") {
        let path = PathBuf::from("/test/meminfo");
        let _ = meminfo::parse_meminfo_content(&input, &path);
    }

    #[test]
    fn test_stat_never_panics(input in ".*", clk_tck in 1u64..10000) {
        let path = PathBuf::from("/test/stat");
        let _ = stat::parse_stat_content(&input, clk_tck, &path);
    }

    #[test]
    fn test_diskstats_never_panics(input in ".*") {
        let path = PathBuf::from("/test/diskstats");
        let _ = diskstats::parse_diskstats_content(&input, &path);
    }

    #[test]
    fn test_netdev_never_panics(input in ".*") {
        let path = PathBuf::from("/test/net/dev");
        let _ = netdev::parse_net_dev_content(&input, &path);
    }

    #[test]
    fn test_procstat_never_panics(input in ".*") {
        let path = PathBuf::from("/test/proc/123/stat");
        let _ = procstat::parse_proc_stat_content(&input, &path);
    }

    #[test]
    fn test_pressure_never_panics(input in ".*") {
        let path = PathBuf::from("/test/pressure/memory");
        let _ = pressure::parse_pressure_content(&input, &path);
    }
}

// Property: Valid load average always parses correctly
proptest! {
    #[test]
    fn test_valid_loadavg_parses(
        load1 in 0.0f64..100.0,
        load5 in 0.0f64..100.0,
        load15 in 0.0f64..100.0,
        runnable in 1u32..1000,
        total in 1u32..10000,
    ) {
        prop_assume!(runnable <= total);
        
        let content = format!(
            "{:.2} {:.2} {:.2} {}/{} 12345",
            load1, load5, load15, runnable, total
        );
        
        let path = PathBuf::from("/proc/loadavg");
        let result = loadavg::parse_loadavg_content(&content, &path);
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert!((metrics.load1 - load1).abs() < 0.01);
        assert_eq!(metrics.runnable_entities, runnable);
        assert_eq!(metrics.total_entities, total);
    }
}

// Property: CPU stat total is sum of components
proptest! {
    #[test]
    fn test_cpu_stat_total_invariant(
        user in 0u64..1000000,
        nice in 0u64..100000,
        system in 0u64..500000,
        idle in 0u64..5000000,
        iowait in 0u64..100000,
    ) {
        let cpu_stat = stat::CpuStat {
            user,
            nice,
            system,
            idle,
            iowait,
            irq: 0,
            softirq: 0,
            steal: 0,
            guest: 0,
            guest_nice: 0,
        };
        
        let expected_total = user + nice + system + idle + iowait;
        assert_eq!(cpu_stat.total(), expected_total);
    }
}

// Property: Memory metrics must be consistent
proptest! {
    #[test]
    fn test_memory_consistency(
        total_kb in 1000000u64..64000000, // 1GB-64GB
        free_kb in 100000u64..32000000,   // 100MB-32GB
        buffers_kb in 10000u64..8000000,  // 10MB-8GB
        cached_kb in 100000u64..16000000, // 100MB-16GB
    ) {
        prop_assume!(free_kb + buffers_kb + cached_kb < total_kb);
        
        let content = format!(
            "MemTotal:       {} kB\n\
             MemFree:        {} kB\n\
             MemAvailable:   {} kB\n\
             Buffers:        {} kB\n\
             Cached:         {} kB\n\
             SwapTotal:      4096000 kB\n\
             SwapFree:       4096000 kB\n",
            total_kb, free_kb, total_kb - 1000000, buffers_kb, cached_kb
        );
        
        let path = PathBuf::from("/proc/meminfo");
        let result = meminfo::parse_meminfo_content(&content, &path);
        
        if let Ok(fields) = result {
            let total_bytes = fields.get("MemTotal").copied().unwrap();
            let free_bytes = fields.get("MemFree").copied().unwrap();
            
            // Consistency check
            assert!(free_bytes <= total_bytes);
            assert_eq!(total_bytes, total_kb * 1024);
        }
    }
}

// Property: Disk I/O counters are monotonic
proptest! {
    #[test]
    fn test_disk_io_monotonic(
        reads1 in 0u64..1000000,
        reads2 in 0u64..1000000,
        writes1 in 0u64..1000000,
        writes2 in 0u64..1000000,
    ) {
        prop_assume!(reads2 >= reads1);
        prop_assume!(writes2 >= writes1);
        
        let disk1 = diskstats::DiskStats {
            major: 8,
            minor: 0,
            device_name: "sda".to_string(),
            reads_completed: reads1,
            reads_merged: 0,
            sectors_read: reads1 * 8,
            time_reading_ms: 0,
            writes_completed: writes1,
            writes_merged: 0,
            sectors_written: writes1 * 8,
            time_writing_ms: 0,
            io_in_progress: 0,
            time_io_ms: 0,
            weighted_time_io_ms: 0,
        };
        
        let disk2 = diskstats::DiskStats {
            reads_completed: reads2,
            writes_completed: writes2,
            sectors_read: reads2 * 8,
            sectors_written: writes2 * 8,
            ..disk1.clone()
        };
        
        // Delta should be non-negative
        assert!(disk2.reads_completed >= disk1.reads_completed);
        assert!(disk2.writes_completed >= disk1.writes_completed);
        assert!(disk2.total_io() >= disk1.total_io());
    }
}
