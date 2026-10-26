// Background sampler with lock-free latest() access
// Spawns a background thread that collects snapshots at regular intervals

use crate::snapshot::Snapshot;
use crate::SystemConfig;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Sampler configuration
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    pub sample_interval: Duration,
    pub history_size: usize,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            sample_interval: Duration::from_secs(5),
            history_size: 100,
        }
    }
}

/// Background sampler handle
pub struct Sampler {
    handle: Option<thread::JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    latest: Arc<parking_lot::RwLock<Option<Snapshot>>>,
    history: Arc<parking_lot::RwLock<HistoryRing>>,
    sample_count: Arc<AtomicU64>,
}

/// Ring buffer for historical samples
struct HistoryRing {
    buffer: Vec<Snapshot>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl HistoryRing {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            size: 0,
        }
    }

    fn push(&mut self, item: Snapshot) {
        if self.size < self.capacity {
            self.buffer.push(item);
            self.size += 1;
        } else {
            self.buffer[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &Snapshot> + '_> {
        if self.size < self.capacity {
            Box::new(self.buffer[..self.size].iter())
        } else {
            Box::new(
                self.buffer[self.head..]
                    .iter()
                    .chain(self.buffer[..self.head].iter())
            )
        }
    }
}

impl Sampler {
    /// Start background sampler
    pub fn start(system_config: SystemConfig, sampler_config: SamplerConfig) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let latest = Arc::new(parking_lot::RwLock::new(None));
        let history = Arc::new(parking_lot::RwLock::new(HistoryRing::new(
            sampler_config.history_size,
        )));
        let sample_count = Arc::new(AtomicU64::new(0));

        let stop_flag_clone = Arc::clone(&stop_flag);
        let latest_clone = Arc::clone(&latest);
        let history_clone = Arc::clone(&history);
        let sample_count_clone = Arc::clone(&sample_count);
        let system_config_clone = system_config.clone();

        let handle = thread::Builder::new()
            .name("intellishell-telemetry".to_string())
            .spawn(move || {
                sampler_loop(
                    system_config_clone,
                    sampler_config.sample_interval,
                    stop_flag_clone,
                    latest_clone,
                    history_clone,
                    sample_count_clone,
                );
            })
            .expect("Failed to spawn telemetry sampler thread");

        Self {
            handle: Some(handle),
            stop_flag,
            latest,
            history,
            sample_count,
        }
    }

    /// Get latest snapshot (lock-free read)
    /// Returns None if no sample collected yet
    pub fn latest(&self) -> Option<Snapshot> {
        self.latest.read().clone()
    }

    /// Get historical samples (up to history_size)
    pub fn history(&self) -> Vec<Snapshot> {
        self.history.read().iter().cloned().collect()
    }

    /// Get number of samples collected
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }

    /// Stop the sampler and wait for thread to exit
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Main sampler loop
fn sampler_loop(
    system_config: SystemConfig,
    interval: Duration,
    stop_flag: Arc<AtomicBool>,
    latest: Arc<parking_lot::RwLock<Option<Snapshot>>>,
    history: Arc<parking_lot::RwLock<HistoryRing>>,
    sample_count: Arc<AtomicU64>,
) {
    let mut prev_snapshot: Option<Snapshot> = None;
    
    while !stop_flag.load(Ordering::Relaxed) {
        let sample_id = sample_count.fetch_add(1, Ordering::SeqCst);
        
        match crate::collectors::collect_snapshot(sample_id, &system_config, prev_snapshot.as_ref()) {
            Ok(snapshot) => {
                // Update latest
                *latest.write() = Some(snapshot.clone());

                // Update history
                history.write().push(snapshot.clone());
                
                // Keep for next iteration
                prev_snapshot = Some(snapshot);
            }
            Err(e) => {
                // Log error but continue sampling
                eprintln!("Telemetry collection error: {}", e);
            }
        }

        // Sleep for interval, checking stop flag periodically
        let sleep_chunks = 10;
        let chunk_duration = interval / sleep_chunks;
        for _ in 0..sleep_chunks {
            if stop_flag.load(Ordering::Relaxed) {
                return;
            }
            thread::sleep(chunk_duration);
        }
    }
}
