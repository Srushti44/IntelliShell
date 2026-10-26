//! IntelliShell Telemetry Engine
//! 
//! Production-grade Linux system metrics collection with kernel-accurate data.
//! Native /proc parsing, zero unsafe code, comprehensive error handling.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

pub mod error;
pub mod snapshot;
pub mod parsers;
pub mod collectors;
pub mod sampler;

pub use error::{TelemetryError, ParseError, Result};
pub use snapshot::Snapshot;
pub use sampler::Sampler;

/// System configuration read once at startup
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub clock_ticks_per_sec: i64,
    pub page_size_bytes: i64,
    pub cpu_count: usize,
    pub boot_time_secs: u64,
}

impl SystemConfig {
    /// Initialize system configuration from kernel
    pub fn detect() -> Result<Self> {
        let clock_ticks_per_sec = nix::unistd::sysconf(nix::unistd::SysconfVar::CLK_TCK)?
            .ok_or_else(|| TelemetryError::Config("CLK_TCK unavailable".into()))?;
        
        let page_size_bytes = nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)?
            .ok_or_else(|| TelemetryError::Config("PAGE_SIZE unavailable".into()))?;
        
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        
        let boot_time_secs = parsers::stat::read_boot_time()?;
        
        Ok(SystemConfig {
            clock_ticks_per_sec,
            page_size_bytes,
            cpu_count,
            boot_time_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_config_detect() {
        let config = SystemConfig::detect().expect("Failed to detect system config");
        assert!(config.clock_ticks_per_sec > 0);
        assert!(config.page_size_bytes > 0);
        assert!(config.cpu_count > 0);
        assert!(config.boot_time_secs > 0);
    }
}
