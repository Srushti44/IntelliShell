//! Oracle tests: Compare our parsers against procfs/sysinfo crates
//! These crates are ONLY used in tests, not in production code

use intellishell_telemetry::collectors;
use std::thread;
use std::time::Duration;

#[test]
fn test_cpu_vs_sysinfo() {
    let our_cpu = collectors::cpu::collect_cpu().expect("Failed to collect CPU");
    
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    
    // Compare CLK_TCK
    let sysinfo_clk_tck = 100; // sysinfo uses 100
    // Our implementation gets it from sysconf, which might differ
    assert!(our_cpu.clk_tck > 0);
    println!("Our CLK_TCK: {}, sysinfo assumes: {}", our_cpu.clk_tck, sysinfo_clk_tck);
    
    // Compare load average (should be similar within small delta)
    let sysinfo_load = sys.load_average();
    let delta_1 = (our_cpu.load_avg_1 - sysinfo_load.one).abs();
    let delta_5 = (our_cpu.load_avg_5 - sysinfo_load.five).abs();
    let delta_15 = (our_cpu.load_avg_15 - sysinfo_load.fifteen).abs();
    
    assert!(delta_1 < 0.5, "Load avg 1min differs too much: ours={}, sysinfo={}", 
        our_cpu.load_avg_1, sysinfo_load.one);
    assert!(delta_5 < 0.5, "Load avg 5min differs too much");
    assert!(delta_15 < 0.5, "Load avg 15min differs too much");
    
    println!("✓ CPU metrics match sysinfo within tolerance");
}

#[test]
fn test_memory_vs_sysinfo() {
    let our_mem = collectors::memory::collect_memory().expect("Failed to collect memory");
    
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    
    let sysinfo_total = sys.total_memory();
    let sysinfo_used = sys.used_memory();
    let sysinfo_available = sys.available_memory();
    
    // Allow 5% tolerance (sysinfo might calculate differently)
    let tolerance = 0.05;
    
    let total_diff = (our_mem.total_bytes as f64 - sysinfo_total as f64).abs() 
        / our_mem.total_bytes as f64;
    assert!(total_diff < tolerance, 
        "Total memory differs: ours={}, sysinfo={}", 
        our_mem.total_bytes, sysinfo_total);
    
    let available_diff = (our_mem.available_bytes as f64 - sysinfo_available as f64).abs() 
        / our_mem.available_bytes as f64;
    assert!(available_diff < tolerance,
        "Available memory differs: ours={}, sysinfo={}",
        our_mem.available_bytes, sysinfo_available);
    
    println!("✓ Memory metrics match sysinfo within {}% tolerance", tolerance * 100.0);
}

#[test]
fn test_process_vs_procfs() {
    // Get our own PID
    let our_pid = std::process::id() as i32;
    
    // Parse with our implementation
    let our_proc = intellishell_telemetry::parsers::procstat::parse_proc_pid_stat(our_pid)
        .expect("Failed to parse our process");
    
    // Parse with procfs crate
    let procfs_proc = procfs::process::Process::new(our_pid)
        .expect("Failed to get process from procfs");
    let procfs_stat = procfs_proc.stat().expect("Failed to get stat");
    
    // Compare key fields
    assert_eq!(our_proc.pid, procfs_stat.pid);
    assert_eq!(our_proc.ppid, procfs_stat.ppid);
    assert_eq!(our_proc.state, procfs_stat.state().to_char());
    
    // CPU times should match
    assert_eq!(our_proc.utime, procfs_stat.utime);
    assert_eq!(our_proc.stime, procfs_stat.stime);
    
    // Memory (RSS in pages)
    assert_eq!(our_proc.rss, procfs_stat.rss);
    
    // Virtual memory size
    assert_eq!(our_proc.vsize, procfs_stat.vsize);
    
    println!("✓ Process stat matches procfs crate exactly");
}

#[test]
fn test_boot_time_vs_procfs() {
    use intellishell_telemetry::parsers::stat;
    
    let our_cpu = collectors::cpu::collect_cpu().expect("Failed to collect CPU");
    let our_boot_time = our_cpu.boot_time_unix;
    
    let procfs_boot_time = procfs::boot_time_secs().expect("Failed to get boot time");
    
    assert_eq!(our_boot_time, procfs_boot_time,
        "Boot time should match exactly");
    
    println!("✓ Boot time matches procfs: {}", our_boot_time);
}

#[test]
fn test_disk_stats_vs_procfs() {
    let our_disks = collectors::disk::collect_disk().expect("Failed to collect disk");
    
    let procfs_diskstats = procfs::diskstats().expect("Failed to read diskstats");
    
    // Find matching device
    if let Some(our_sda) = our_disks.iter().find(|d| d.device == "sda") {
        if let Some(procfs_sda) = procfs_diskstats.iter().find(|d| d.name == "sda") {
            // Compare key metrics
            assert_eq!(our_sda.reads_completed, procfs_sda.reads);
            assert_eq!(our_sda.writes_completed, procfs_sda.writes);
            
            // Sectors to bytes (512 bytes per sector)
            let our_sectors_read = our_sda.bytes_read / 512;
            let our_sectors_written = our_sda.bytes_written / 512;
            assert_eq!(our_sectors_read, procfs_sda.sectors_read);
            assert_eq!(our_sectors_written, procfs_sda.sectors_written);
            
            println!("✓ Disk stats match procfs for sda");
        }
    }
}

#[test]
fn test_network_stats_vs_procfs() {
    let our_net = collectors::network::collect_network().expect("Failed to collect network");
    
    let procfs_net = procfs::net::dev_status().expect("Failed to read /proc/net/dev");
    
    // Find matching interface (eth0 or first non-loopback)
    if let Some(our_iface) = our_net.first() {
        if let Some(procfs_iface) = procfs_net.get(&our_iface.interface) {
            // Compare metrics
            assert_eq!(our_iface.rx_bytes, procfs_iface.recv_bytes);
            assert_eq!(our_iface.tx_bytes, procfs_iface.sent_bytes);
            assert_eq!(our_iface.rx_packets, procfs_iface.recv_packets);
            assert_eq!(our_iface.tx_packets, procfs_iface.sent_packets);
            
            println!("✓ Network stats match procfs for {}", our_iface.interface);
        }
    }
}

#[test]
fn test_monotonicity_over_time() {
    // Collect initial snapshot
    let snapshot1 = collectors::collect_snapshot().expect("Failed to collect snapshot 1");
    
    // Do some work
    thread::sleep(Duration::from_millis(500));
    let _work: u64 = (0..1000000).sum();
    
    // Collect second snapshot
    let snapshot2 = collectors::collect_snapshot().expect("Failed to collect snapshot 2");
    
    // CPU counters should increase (monotonic)
    assert!(snapshot2.snapshot.cpu.user_ticks >= snapshot1.snapshot.cpu.user_ticks);
    assert!(snapshot2.snapshot.cpu.system_ticks >= snapshot1.snapshot.cpu.system_ticks);
    
    // Time should advance
    assert!(snapshot2.timestamp > snapshot1.timestamp);
    assert!(snapshot2.wall_time > snapshot1.wall_time);
    
    println!("✓ All counters are monotonic");
}
