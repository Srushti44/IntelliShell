# ADR 0002: Use Rust for Core Engine

**Status**: Accepted

**Date**: 2024-01-15

## Context

IntelliShell's core-engine component requires low-level system access to Linux kernel features:

- **cgroup v2 API**: Create cgroups, set resource limits, delegate controllers
- **Namespaces**: Create and enter mount, PID, network, IPC, UTS namespaces
- **/proc filesystem**: Parse `/proc/<pid>/stat`, `/proc/meminfo`, `/proc/<pid>/status`
- **Signal handling**: Send signals (SIGTERM, SIGKILL) to processes, including in other namespaces
- **Process management**: Fork, exec, wait, reap zombies

The language choice must balance:
1. **Low-level access**: Direct system calls and libc bindings
2. **Memory safety**: Avoid buffer overflows, use-after-free, data races
3. **Performance**: Minimal overhead for process monitoring (runs continuously)
4. **Ergonomics**: Good tooling, package management, testing

## Decision

We will use **Rust** for the core-engine component.

- **Edition**: 2024 (latest stable features)
- **Toolchain**: Rust 1.87.0 (pinned via rust-toolchain.toml)
- **libc bindings**: Direct use of `libc` crate for system calls
- **No unsafe abstractions**: Use `unsafe` only where necessary, with clear invariants

## Consequences

### Positive

- **Memory safety**: Rust's ownership system prevents entire classes of bugs (buffer overflows, use-after-free, data races)
- **Zero-cost abstractions**: Safe code compiles to efficient machine code
- **Strong ecosystem**: Crates for common tasks (clap for CLI, serde for serialization, tokio for async if needed later)
- **Excellent tooling**: cargo (build, test, package), rustfmt (formatting), clippy (linting), rust-analyzer (IDE support)
- **Long-term maintainability**: Type system catches bugs at compile time, refactoring is safer

### Negative

- **Learning curve**: Rust's ownership and lifetime system is complex for newcomers
- **Compile times**: Rust compiles slower than C or Go (but incremental compilation helps)
- **Unsafe code required**: Some system calls require `unsafe` blocks (e.g., pointer casts, raw syscalls)
- **Smaller talent pool**: Fewer developers know Rust compared to C or Go

### Mitigation

- **Good documentation**: Comment unsafe blocks with safety invariants
- **Minimize unsafe**: Use safe wrappers around unsafe operations
- **Incremental compilation**: Use `cargo check` for fast feedback during development
- **Training resources**: Point contributors to Rust Book and Rustlings

## Alternatives Considered

### 1. Go

**Pros**:
- Simpler language (no lifetimes, no unsafe)
- Fast compilation
- Built-in concurrency (goroutines)
- Large standard library

**Cons**:
- Garbage collector (pauses, unpredictable latency)
- Less control over memory layout
- CGO overhead for system calls (or complex syscall wrappers)
- No compile-time memory safety guarantees

**Verdict**: Go's GC is a deal-breaker for a low-level process monitor. Rust's deterministic memory management is preferable.

### 2. C

**Pros**:
- Maximum control over hardware
- Direct system call access (no wrapper overhead)
- Universally available
- Minimal runtime

**Cons**:
- No memory safety (buffer overflows, use-after-free)
- Manual memory management (error-prone)
- Poor tooling (no standard package manager, testing is harder)
- Hard to refactor safely

**Verdict**: C gives control but sacrifices safety. For a security-sensitive tool, memory safety is critical.

### 3. Python (with C extensions)

**Pros**:
- Fast development
- Easy to prototype
- Good ecosystem

**Cons**:
- Python for low-level system programming is awkward
- C extensions reintroduce memory safety issues
- GIL limits concurrency
- Slower than compiled languages

**Verdict**: Python is great for ai-service, but not suitable for core-engine.

## References

- [Rust for Linux kernel development](https://rust-for-linux.com/)
- [Facebook's use of Rust for low-level systems](https://engineering.fb.com/2021/04/29/developer-tools/rust/)
- [Rust vs C performance comparison](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)

## Notes

This decision applies to **core-engine only**. Other components (ai-service in Python, dashboard in JavaScript) are not affected.
