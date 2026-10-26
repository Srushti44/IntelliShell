use std::env;
use std::fs;
use std::path::PathBuf;

// Checks if we're running with root privileges — required because cgroup
// delegation only grants write access within a user's own systemd slice,
// and we proved by hand that moving a process INTO that slice from the
// true root cgroup requires root.
pub fn require_root() -> Result<(), String> {
    let euid = unsafe { libc::geteuid() };
    if euid != 0 {
        Err("IntelliShell's sandbox features require root. Run with: sudo ./target/debug/intellishell-core ...".to_string())
    } else {
        Ok(())
    }
}

// When running under `sudo`, geteuid()/getuid() both report 0 (root),
// but we still want the ORIGINAL user's delegated cgroup slice
// (e.g. user-1000.slice), not root's. sudo sets the SUDO_UID env var
// to the invoking user's real UID specifically for this purpose.
fn invoking_uid() -> u32 {
    env::var("SUDO_UID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| unsafe { libc::getuid() })
}

// Public version for sandbox module
pub fn invoking_uid_pub() -> u32 {
    invoking_uid()
}

fn app_slice_base() -> PathBuf {
    let uid = invoking_uid();
    PathBuf::from(format!(
        "/sys/fs/cgroup/user.slice/user-{}.slice/user@{}.service/app.slice",
        uid, uid
    ))
}

pub struct Sandbox {
    // Kept for future observability and debugging output
    #[allow(dead_code)]
    name: String,
    path: PathBuf,
}

impl Sandbox {
    // Public accessor for sandbox name (needed by sandbox module)
    pub fn name_pub(&self) -> &str {
        &self.name
    }
    
    // Creates a new cgroup with hard limits on process count and memory,
    // exactly the two commands we ran by hand: mkdir, then write pids.max
    // and memory.max. The kernel auto-populates every other control file.
    pub fn create(name: &str, pids_max: u32, memory_max_bytes: u64) -> Result<Self, String> {
        let path = app_slice_base().join(name);

        fs::create_dir(&path)
            .map_err(|e| format!("Failed to create cgroup at {:?}: {}", path, e))?;

        fs::write(path.join("pids.max"), pids_max.to_string())
            .map_err(|e| format!("Failed to set pids.max: {}", e))?;

        fs::write(path.join("memory.max"), memory_max_bytes.to_string())
            .map_err(|e| format!("Failed to set memory.max: {}", e))?;

        Ok(Sandbox {
            name: name.to_string(),
            path,
        })
    }

    // Moves an already-running process into this cgroup by writing its PID
    // into cgroup.procs — same mechanism as `echo <pid> > cgroup.procs`.
    pub fn add_process(&self, pid: u32) -> Result<(), String> {
        fs::write(self.path.join("cgroup.procs"), pid.to_string())
            .map_err(|e| format!("Failed to add PID {} to sandbox: {}", pid, e))
    }

    // Reads the current live process count inside this cgroup.
    pub fn current_pids(&self) -> Result<u32, String> {
        let contents = fs::read_to_string(self.path.join("pids.current"))
            .map_err(|e| format!("Failed to read pids.current: {}", e))?;
        contents
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse pids.current: {}", e))
    }

    // The emergency stop — instantly kills every process in the cgroup,
    // exactly what we tested against the real fork bomb.
    pub fn kill_all(&self) -> Result<(), String> {
        fs::write(self.path.join("cgroup.kill"), "1")
            .map_err(|e| format!("Failed to kill sandbox: {}", e))
    }

    // Removes the (now-empty) cgroup directory. Fails if processes remain,
    // which is a useful safety property, not a bug — call kill_all() first.
    pub fn remove(&self) -> Result<(), String> {
        fs::remove_dir(&self.path)
            .map_err(|e| format!("Failed to remove sandbox (must be empty first): {}", e))
    }
}
