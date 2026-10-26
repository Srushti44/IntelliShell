mod cgroup;
mod namespace;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

// ============ Phase 1: Monitoring ============

fn parse_meminfo_value(contents: &str, key: &str) -> Option<u64> {
    for line in contents.lines() {
        if line.starts_with(key) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse::<u64>().ok();
            }
        }
    }
    None
}

struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    memory_kb: u64,
    cpu_percent: f64,
}

struct CpuSnapshot {
    idle: u64,
    total: u64,
}

fn read_system_cpu_snapshot() -> Option<CpuSnapshot> {
    let contents = fs::read_to_string("/proc/stat").ok()?;
    let first_line = contents.lines().next()?;

    let fields: Vec<u64> = first_line
        .split_whitespace()
        .skip(1)
        .filter_map(|f| f.parse::<u64>().ok())
        .collect();

    if fields.len() < 4 {
        return None;
    }

    let idle = fields[3] + fields.get(4).unwrap_or(&0);
    let total: u64 = fields.iter().sum();

    Some(CpuSnapshot { idle, total })
}

fn read_cpu_ticks(pid: u32) -> Option<u64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let contents = fs::read_to_string(&stat_path).ok()?;

    let last_paren = contents.rfind(')')?;
    let after_name = &contents[last_paren + 1..];
    let fields: Vec<&str> = after_name.split_whitespace().collect();

    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;

    Some(utime + stime)
}

fn read_process_info(pid: u32, cpu_percent: f64) -> Option<ProcessInfo> {
    let status_path = format!("/proc/{}/status", pid);
    let contents = fs::read_to_string(&status_path).ok()?;

    let mut name = String::new();
    let mut state = String::new();
    let mut memory_kb: u64 = 0;

    for line in contents.lines() {
        if line.starts_with("Name:") {
            name = line.split_whitespace().nth(1).unwrap_or("?").to_string();
        } else if line.starts_with("State:") {
            state = line
                .split_once(':')
                .map(|x| x.1)
                .unwrap_or("?")
                .trim()
                .to_string();
        } else if line.starts_with("VmRSS:") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                memory_kb = value_str.parse::<u64>().unwrap_or(0);
            }
        }
    }

    Some(ProcessInfo {
        pid,
        name,
        state,
        memory_kb,
        cpu_percent,
    })
}

fn list_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    let entries = fs::read_dir("/proc").expect("Failed to read /proc");
    for entry in entries.flatten() {
        if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
            pids.push(pid);
        }
    }
    pids
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn run_one_frame(interval_ms: u64) {
    let mem_contents = fs::read_to_string("/proc/meminfo").expect("Failed to read /proc/meminfo");
    let total_kb = parse_meminfo_value(&mem_contents, "MemTotal:").expect("no MemTotal");
    let available_kb =
        parse_meminfo_value(&mem_contents, "MemAvailable:").expect("no MemAvailable");
    let mem_used_percent = ((total_kb - available_kb) as f64 / total_kb as f64) * 100.0;

    let sys_snap1 = read_system_cpu_snapshot().expect("failed to read /proc/stat");
    let pids = list_pids();
    let mut snapshot1: HashMap<u32, u64> = HashMap::new();
    for &pid in &pids {
        if let Some(ticks) = read_cpu_ticks(pid) {
            snapshot1.insert(pid, ticks);
        }
    }

    thread::sleep(Duration::from_millis(interval_ms));

    let sys_snap2 = read_system_cpu_snapshot().expect("failed to read /proc/stat");
    let clock_ticks_per_sec: f64 = 100.0;
    let mut processes = Vec::new();

    for &pid in &pids {
        if let Some(ticks2) = read_cpu_ticks(pid) {
            let ticks1 = *snapshot1.get(&pid).unwrap_or(&ticks2);
            let delta_ticks = ticks2.saturating_sub(ticks1);
            let delta_seconds = delta_ticks as f64 / clock_ticks_per_sec;
            let elapsed_seconds = interval_ms as f64 / 1000.0;
            let cpu_percent = (delta_seconds / elapsed_seconds) * 100.0;

            if let Some(info) = read_process_info(pid, cpu_percent) {
                processes.push(info);
            }
        }
    }

    let idle_delta = sys_snap2.idle.saturating_sub(sys_snap1.idle);
    let total_delta = sys_snap2.total.saturating_sub(sys_snap1.total);
    let system_cpu_percent = if total_delta > 0 {
        (1.0 - (idle_delta as f64 / total_delta as f64)) * 100.0
    } else {
        0.0
    };

    processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());

    clear_screen();
    println!("IntelliShell Core — live system monitor  (Ctrl+C to quit)\n");
    println!("System Memory Used: {:.1}%", mem_used_percent);
    println!("System CPU Used:    {:.1}%\n", system_cpu_percent);
    println!("=== Top Processes ===");
    println!("Total processes: {}\n", processes.len());

    for p in processes.iter().take(15) {
        println!(
            "PID {:<8} CPU {:>5.1}%  MEM {:>8} kB  STATE {:<20} NAME {}",
            p.pid, p.cpu_percent, p.memory_kb, p.state, p.name
        );
    }
}

// ============ Phase 2: Process control ============

// Refuse to touch PID 1 (init/systemd — killing it takes down the whole OS)
// or our own PID (acting on yourself mid-command is just confusing, not useful).
fn is_protected_pid(pid: u32) -> bool {
    pid == 1 || pid == std::process::id()
}

// Sends a raw Unix signal to a PID via libc's kill() — the real syscall wrapper,
// same mechanism the `kill` shell command itself uses.
fn send_signal(pid: u32, signal: i32) -> Result<(), String> {
    let result = unsafe { libc::kill(pid as libc::pid_t, signal) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

fn handle_command(command: &str, args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: {} <pid>", command);
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid PID: '{}'", args[0]);
            return;
        }
    };

    if is_protected_pid(pid) {
        eprintln!("Refusing to act on protected PID {} (init or self)", pid);
        return;
    }

    let signal = match command {
        "kill" => libc::SIGTERM,   // polite: ask the process to shut down
        "kill9" => libc::SIGKILL,  // forceful: kernel terminates it immediately, no cleanup
        "pause" => libc::SIGSTOP,  // freeze the process (can't be caught/ignored by the process)
        "resume" => libc::SIGCONT, // unfreeze it
        _ => unreachable!(),
    };

    match send_signal(pid, signal) {
        Ok(()) => println!("Sent {} to PID {}", command, pid),
        Err(e) => eprintln!("Failed to signal PID {}: {}", pid, e),
    }
}

// Reads a process's current niceness value (-20 to 19) via getpriority().
fn read_priority(pid: u32) -> Option<i32> {
    // getpriority() has a quirk: it can legitimately return -1 as a valid
    // priority value, not just as an error. To disambiguate, we must clear
    // errno first and check it after the call, rather than just checking
    // the return value.
    unsafe {
        *libc::__errno_location() = 0;
        let result = libc::getpriority(libc::PRIO_PROCESS, pid);
        if result == -1 && *libc::__errno_location() != 0 {
            None
        } else {
            Some(result)
        }
    }
}

fn handle_renice(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: renice <pid> <priority: -20 to 19>");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid PID: '{}'", args[0]);
            return;
        }
    };

    let priority: i32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Invalid priority: '{}'", args[1]);
            return;
        }
    };

    if !(-20..=19).contains(&priority) {
        eprintln!("Priority must be between -20 (highest) and 19 (lowest)");
        return;
    }

    if is_protected_pid(pid) {
        eprintln!("Refusing to act on protected PID {} (init or self)", pid);
        return;
    }

    let before = read_priority(pid);

    let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, priority) };

    if result == 0 {
        let after = read_priority(pid);
        println!("PID {}: priority changed {:?} -> {:?}", pid, before, after);
    } else {
        let err = std::io::Error::last_os_error();
        eprintln!(
            "Failed to renice PID {}: {} (lowering niceness below current value usually requires root)",
            pid, err
        );
    }
}
// Proves the sandbox works: spawns a real disposable process (a shell
// running `sleep 60`), puts it in a cgroup capped to 5 processes, then
// tries to fork-bomb it FROM CODE — same experiment we just did by hand,
// now automated and repeatable.
fn handle_sandbox_test() {
    if let Err(e) = cgroup::require_root() {
        eprintln!("{}", e);
        return;
    }

    println!("Creating sandbox (pids.max=5, memory.max=50MB)...");
    let sandbox = match cgroup::Sandbox::create("intellishell-autotest", 5, 50 * 1024 * 1024) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create sandbox: {}", e);
            return;
        }
    };

    println!("Spawning a disposable shell inside the sandbox...");
    let mut child = match Command::new("bash")
        .arg("-c")
        .arg("sleep 1; :(){ :|:& };:")
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn test process: {}", e);
            let _ = sandbox.remove();
            return;
        }
    };

    let child_pid = child.id();
    println!(
        "Spawned PID {}, adding it to the sandbox before it forks...",
        child_pid
    );

    if let Err(e) = sandbox.add_process(child_pid) {
        eprintln!("{}", e);
        let _ = child.kill();
        let _ = sandbox.remove();
        return;
    }

    println!("Waiting for the fork bomb to trigger inside the sandbox...");
    std::thread::sleep(Duration::from_millis(1500));

    match sandbox.current_pids() {
        Ok(count) => println!(
            "Processes in sandbox: {} (capped at 5 -- real containment, not simulated)",
            count
        ),
        Err(e) => eprintln!("Could not read process count: {}", e),
    }

    println!("Killing everything in the sandbox...");
    if let Err(e) = sandbox.kill_all() {
        eprintln!("{}", e);
    }

    let mut removed = false;
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(200));
        if sandbox.remove().is_ok() {
            removed = true;
            break;
        }
    }

    if removed {
        println!("Sandbox cleaned up successfully.");
    } else {
        eprintln!("Cleanup warning: sandbox directory still busy (processes may still be exiting)");
    }

    let _ = child.wait();
}

// Proves namespace isolation from our own code: runs `ps aux` inside a new
// PID+mount namespace and lets it print directly to our terminal. If
// isolation is real, we should see only 1-2 processes (itself), not the
// 40+ processes visible on the real system.
fn handle_isolate_test() {
    if let Err(e) = cgroup::require_root() {
        eprintln!("{}", e);
        return;
    }

    println!("Running 'ps aux' inside an isolated PID+mount namespace...");
    println!("(compare this to a normal 'ps aux' -- should show almost nothing)\n");

    match namespace::run_isolated("ps", &["aux".to_string()]) {
        Ok(exit_code) => println!("\nIsolated process exited with code {}", exit_code),
        Err(e) => eprintln!("Failed to run isolated process: {}", e),
    }
}

fn print_usage() {
    eprintln!("IntelliShell Core");
    eprintln!("Usage:");
    eprintln!("  cargo run                             -> live system monitor");
    eprintln!("  cargo run -- kill <pid>                -> terminate gracefully (SIGTERM)");
    eprintln!("  cargo run -- kill9 <pid>               -> force kill (SIGKILL)");
    eprintln!("  cargo run -- pause <pid>               -> suspend (SIGSTOP)");
    eprintln!("  cargo run -- resume <pid>              -> resume (SIGCONT)");
    eprintln!("  cargo run -- renice <pid> <priority>   -> change scheduling priority (-20 to 19)");
    eprintln!("  cargo run -- sandbox-test              -> prove cgroup containment against a real fork bomb");
    eprintln!("  cargo run -- isolate-test              -> prove PID namespace isolation");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command = args[1].as_str();
        match command {
            "kill" | "kill9" | "pause" | "resume" => {
                handle_command(command, &args[2..]);
            }
            "renice" => {
                handle_renice(&args[2..]);
            }
            "sandbox-test" => {
                handle_sandbox_test();
            }
            "isolate-test" => {
                handle_isolate_test();
            }
            _ => print_usage(),
        }
        return;
    }

    // No command given -> default to the live monitor
    let interval_ms: u64 = 1000;
    loop {
        run_one_frame(interval_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_meminfo_value() {
        // Realistic /proc/meminfo format
        let meminfo_sample = "MemTotal:       16384000 kB\n\
                              MemFree:         8192000 kB\n\
                              MemAvailable:    12288000 kB\n\
                              Buffers:          512000 kB\n";

        assert_eq!(
            parse_meminfo_value(meminfo_sample, "MemTotal:"),
            Some(16384000)
        );
        assert_eq!(
            parse_meminfo_value(meminfo_sample, "MemFree:"),
            Some(8192000)
        );
        assert_eq!(
            parse_meminfo_value(meminfo_sample, "MemAvailable:"),
            Some(12288000)
        );
        assert_eq!(
            parse_meminfo_value(meminfo_sample, "Buffers:"),
            Some(512000)
        );

        // Missing key
        assert_eq!(parse_meminfo_value(meminfo_sample, "NonExistent:"), None);

        // Edge case: malformed line (no value)
        let malformed = "MemTotal:\n";
        assert_eq!(parse_meminfo_value(malformed, "MemTotal:"), None);

        // Edge case: invalid number
        let invalid = "MemTotal:       notanumber kB\n";
        assert_eq!(parse_meminfo_value(invalid, "MemTotal:"), None);
    }

    // Helper function that extracts the parsing logic from read_cpu_ticks
    // to make it testable without requiring /proc filesystem access
    fn parse_proc_stat_line(contents: &str) -> Option<(u64, u64)> {
        let last_paren = contents.rfind(')')?;
        let after_name = &contents[last_paren + 1..];
        let fields: Vec<&str> = after_name.split_whitespace().collect();

        let utime: u64 = fields.get(11)?.parse().ok()?;
        let stime: u64 = fields.get(12)?.parse().ok()?;

        Some((utime, stime))
    }

    #[test]
    fn test_proc_stat_parsing() {
        // Simple process name
        let simple = "12345 (bash) S 1234 12345 12345 0 -1 4194304 1234 0 0 0 100 50 0 0 20 0 1 0 123456 12345678 1234 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let (utime, stime) = parse_proc_stat_line(simple).expect("Failed to parse simple stat");
        assert_eq!(utime, 100);
        assert_eq!(stime, 50);

        // Process name with spaces
        let with_spaces = "12345 (my process name) S 1234 12345 12345 0 -1 4194304 1234 0 0 0 200 100 0 0 20 0 1 0 123456 12345678 1234 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let (utime, stime) =
            parse_proc_stat_line(with_spaces).expect("Failed to parse stat with spaces");
        assert_eq!(utime, 200);
        assert_eq!(stime, 100);

        // Process name with parentheses (edge case that requires finding the LAST ')')
        let with_parens = "12345 (bash (shell)) S 1234 12345 12345 0 -1 4194304 1234 0 0 0 300 150 0 0 20 0 1 0 123456 12345678 1234 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let (utime, stime) =
            parse_proc_stat_line(with_parens).expect("Failed to parse stat with parens");
        assert_eq!(utime, 300);
        assert_eq!(stime, 150);

        // Edge case: malformed (missing fields)
        let malformed = "12345 (bash) S 1234";
        assert_eq!(parse_proc_stat_line(malformed), None);

        // Edge case: no closing paren
        let no_paren = "12345 (bash S 1234 12345";
        assert_eq!(parse_proc_stat_line(no_paren), None);
    }
}
