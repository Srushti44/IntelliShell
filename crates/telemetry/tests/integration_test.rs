//! Integration tests: End-to-end workflow verification

use intellishell_telemetry::{Sampler, SystemConfig};
use std::time::Duration;
use std::thread;

#[test]
fn test_sampler_lifecycle() {
    let config = SystemConfig {
        sample_interval: Duration::from_millis(100),
        history_size: 10,
        collect_processes: true,
    };

    // Start sampler
    let sampler = Sampler::start(config);

    // Wait for a few samples
    thread::sleep(Duration::from_millis(350));

    // Get latest
    let latest = sampler.latest();
    assert!(latest.is_some(), "Should have collected at least one sample");

    let collection = latest.unwrap();
    assert!(collection.snapshot.cpu.clk_tck > 0);
    assert!(collection.snapshot.memory.total_bytes > 0);

    // Get history
    let history = sampler.history();
    assert!(!history.is_empty(), "History should have samples");
    assert!(history.len() <= 10, "History should respect size limit");

    // Stop sampler
    sampler.stop();
}

#[test]
fn test_cpu_metrics() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    let collection = sampler.latest().expect("Should have sample");
    let cpu = &collection.snapshot.cpu;

    // Validate CPU metrics
    assert!(cpu.clk_tck > 0, "CLK_TCK should be positive");
    assert!(cpu.user_ticks > 0, "User ticks should be positive");
    assert!(cpu.idle_ticks > 0, "Idle ticks should be positive");
    assert!(cpu.load_avg_1 >= 0.0, "Load average should be non-negative");
    assert!(cpu.boot_time_unix > 1000000000, "Boot time should be reasonable");

    sampler.stop();
}

#[test]
fn test_memory_metrics() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    let collection = sampler.latest().expect("Should have sample");
    let mem = &collection.snapshot.memory;

    // Validate memory metrics
    assert!(mem.total_bytes > 0, "Total memory should be positive");
    assert!(mem.used_bytes <= mem.total_bytes, "Used <= total");
    assert!(mem.available_bytes <= mem.total_bytes, "Available <= total");
    assert!(mem.free_bytes <= mem.total_bytes, "Free <= total");

    sampler.stop();
}

#[test]
fn test_disk_metrics() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    let collection = sampler.latest().expect("Should have sample");
    let disks = &collection.snapshot.disk;

    // Should have at least one disk
    if !disks.is_empty() {
        let disk = &disks[0];
        assert!(!disk.device.is_empty(), "Device name should not be empty");
        assert!(disk.bytes_read >= 0, "Bytes read should be non-negative");
        assert!(disk.bytes_written >= 0, "Bytes written should be non-negative");
    }

    sampler.stop();
}

#[test]
fn test_network_metrics() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    let collection = sampler.latest().expect("Should have sample");
    let networks = &collection.snapshot.network;

    // Should have at least one network interface (besides lo)
    assert!(!networks.is_empty(), "Should have network interfaces");

    for net in networks {
        assert!(!net.interface.is_empty(), "Interface name should not be empty");
        assert!(net.interface != "lo", "Should not include loopback");
    }

    sampler.stop();
}

#[test]
fn test_process_metrics() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    let collection = sampler.latest().expect("Should have sample");
    let processes = &collection.snapshot.processes;

    // Should have many processes
    assert!(processes.len() > 10, "Should have many processes");

    // Validate a few processes
    for proc in processes.iter().take(5) {
        assert!(proc.pid > 0, "PID should be positive");
        assert!(!proc.comm.is_empty(), "Command should not be empty");
        assert!(proc.memory_bytes >= 0, "Memory should be non-negative");
    }

    sampler.stop();
}

#[test]
fn test_rate_calculations() {
    let config = SystemConfig {
        sample_interval: Duration::from_millis(500),
        history_size: 5,
        collect_processes: false, // Faster without processes
    };

    let sampler = Sampler::start(config);
    
    // Wait for two samples
    thread::sleep(Duration::from_secs(1));

    let history = sampler.history();
    assert!(history.len() >= 2, "Need at least 2 samples for rate calculation");

    // Test CPU usage calculation
    use intellishell_telemetry::collectors::cpu::calculate_cpu_usage;
    let prev = &history[0].snapshot.cpu;
    let curr = &history[1].snapshot.cpu;
    let cpu_usage = calculate_cpu_usage(prev, curr);
    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0, "CPU usage should be 0-100%");

    sampler.stop();
}

#[test]
fn test_concurrent_access() {
    let sampler = Sampler::start(SystemConfig::default());
    thread::sleep(Duration::from_millis(200));

    // Spawn multiple threads reading concurrently
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let sampler_ref = &sampler;
            thread::spawn(move || {
                for _ in 0..100 {
                    let _ = sampler_ref.latest();
                    let _ = sampler_ref.history();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    sampler.stop();
}
