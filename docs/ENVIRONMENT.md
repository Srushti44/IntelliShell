# IntelliShell Environment Setup

This document describes the environment requirements, toolchain setup, and bootstrap process for IntelliShell development.

## System Requirements

IntelliShell runs on Linux inside WSL2 on Windows. The following requirements must be met:

- **WSL2**: Windows Subsystem for Linux version 2
- **Distribution**: Ubuntu (tested on Ubuntu 22.04 or later)
- **Kernel**: Version >= 5.15.0 (required for cgroup.kill, pidfd_open, and other modern kernel features)
- **Init system**: systemd enabled
- **Cgroup**: cgroup v2 with cpu, memory, and pids controllers enabled
  - Verify with: `stat -fc %T /sys/fs/cgroup` should return `cgroup2fs`
  - Controllers: `cat /sys/fs/cgroup/cgroup.controllers` should list `cpu`, `memory`, and `pids`
- **Filesystem**: Repository must be under `/home/<user>/`, never on `/mnt/*` (9p/drvfs filesystems have limitations with overlayfs and cgroups)
- **RAM**: Minimum 4GB available (warning below this threshold)
- **Disk**: Minimum 5GB free (recommended 20GB+)

### Verifying WSL2

```bash
# Check kernel version
uname -r

# Verify WSL2 (should contain "microsoft")
cat /proc/version

# Check systemd is running
systemctl --version
```

## Toolchain Versions

All toolchain versions are pinned for reproducibility:

### Rust

- **Version**: 1.87.0 (stable)
- **Edition**: 2024
- **Components**: rustfmt, clippy
- **Pin file**: `rust-toolchain.toml` in repository root
- **Installation**: Managed by rustup (installed via bootstrap.sh)

```bash
# Verify Rust version
rustc --version  # Should show: rustc 1.87.0
cargo --version
```

### Node.js

- **Version**: 24.x LTS (long-term support through 2028-04-30)
- **Pin file**: `.nvmrc` in repository root (contains `24`)
- **Installation**: Use nvm or install directly via NodeSource

```bash
# Verify Node version
node --version  # Should show: v24.x.x
npm --version
```

### Python

- **Version**: 3.13.x
- **Package manager**: uv (modern, fast Python package manager)
- **Pin files**: `.python-version` and `pyproject.toml` in ai-service/
- **Lock file**: `ai-service/uv.lock` (committed to repository)

```bash
# Verify Python version
python3 --version  # Should show: Python 3.13.x
uv --version
```

## Ollama Setup

IntelliShell uses Ollama for local LLM inference. Ollama runs as a systemd service inside WSL2.

### Installation

```bash
# Install Ollama (run inside WSL)
curl -fsSL https://ollama.com/install.sh | sh
```

This script installs Ollama and sets up the systemd service automatically.

### Verify Service Status

```bash
# Check Ollama systemd service
systemctl --user status ollama

# Verify Ollama is listening on localhost only
ss -ltnp | grep 11434
# Should show: 127.0.0.1:11434 (NOT 0.0.0.0:11434)
```

### Pull Required Model

```bash
# Pull the default model (phi3:mini)
ollama pull phi3:mini

# List installed models
ollama list
```

### GPU Passthrough

**IMPORTANT**: IntelliShell relies on GPU passthrough from the Windows NVIDIA driver to WSL2. **DO NOT** install an NVIDIA driver inside WSL.

#### With NVIDIA GPU

If you have an NVIDIA GPU on Windows with the appropriate driver installed:

```bash
# Verify GPU is visible inside WSL
nvidia-smi

# Check which device Ollama uses
ollama ps
# The PROCESSOR column should show your GPU model
```

When running models, Ollama will automatically detect and use the GPU via CUDA.

#### CPU Fallback

Without a GPU, Ollama runs inference on CPU. This works but is significantly slower.

```bash
# When ollama ps shows empty or "CPU" in PROCESSOR column,
# inference is running on CPU
```

Performance expectations:
- **With GPU**: 50-200+ tokens/second (model and GPU dependent)
- **With CPU**: 5-20 tokens/second (CPU dependent)

## Bootstrap Process

The bootstrap script (`scripts/bootstrap.sh`) installs all required dependencies and configures the environment. It is idempotent and safe to run multiple times.

```bash
# Run bootstrap
./scripts/bootstrap.sh
```

### What Bootstrap Does

1. **System packages** (requires sudo for apt):
   - `build-essential` - C/C++ compiler toolchain
   - `pkg-config` - Helper tool for compiling applications
   - `libssl-dev` - OpenSSL development files
   - `curl` - HTTP client
   - `git` - Version control
   - `zstd` - Compression utility
   - `shellcheck` - Shell script linter
   - `jq` - JSON processor

2. **Toolchains**:
   - rustup (Rust toolchain installer)
   - Node.js (via nvm or direct installation)
   - uv (Python package manager)
   - gh (GitHub CLI)

3. **Secret generation**:
   - Creates `~/.config/intellishell/secret` with a 32-byte random hex value
   - Sets file mode to `0600` (owner read/write only)

### Sudo Commands Used

The bootstrap script uses `sudo` only for the following operations (all documented here):

1. **apt-get update**: Update package lists
2. **apt-get install**: Install system packages listed above
3. **(Optional) systemctl commands**: If manually managing Ollama service
4. **(Optional) /etc/wsl.conf edits**: If configuring WSL boot settings for systemd

No other privileged operations are performed. File permissions are managed with regular `chmod` (never `chmod 777`).

## Secret Management

IntelliShell generates a per-machine secret for cryptographic operations.

### Location and Permissions

- **Path**: `~/.config/intellishell/secret`
- **Permissions**: `0600` (read/write by owner only)
- **Format**: 64-character hexadecimal string (32 bytes)
- **Generation**: Created automatically by `scripts/bootstrap.sh`

```bash
# Verify secret exists with correct permissions
ls -l ~/.config/intellishell/secret
# Should show: -rw------- 1 <user> <group> 65 ...
```

### Security Requirements

- The secret file must **never** be committed to git
- `config/local.env` is git-ignored and should never be tracked
- No secrets, tokens, or credentials belong in the repository
- The `.gitignore` file explicitly excludes:
  - `.env` and `*.env` files (except documented defaults)
  - `config/local.env`
  - `~/.config/intellishell/secret` is outside the repository entirely

### Verification

The `make doctor` command verifies:
- Secret file exists
- Permissions are exactly `0600`
- Git does not track the secret
- No `.env` files are staged or committed

## Quick Start

After cloning the repository to `/home/<user>/intellishell`:

```bash
# 1. Run bootstrap (installs dependencies)
./scripts/bootstrap.sh

# 2. Install Ollama if not already installed
curl -fsSL https://ollama.com/install.sh | sh

# 3. Pull the default model
ollama pull phi3:mini

# 4. Verify environment
make doctor

# 5. Build and test all components
make check

# 6. Test LLM integration
make llm-smoke
```

## Troubleshooting

### Ollama not responding

```bash
# Check if service is running
systemctl --user status ollama

# Start if stopped
systemctl --user start ollama

# Check logs
journalctl --user -u ollama -f
```

### Repository on /mnt/d or /mnt/c

The repository **must** be under `/home/<user>/`. Clone it to your home directory:

```bash
# From /mnt/d (wrong location)
cd ~
git clone <url> intellishell
cd intellishell
```

### Cgroup v2 not enabled

Edit `/etc/wsl.conf` (requires sudo):

```ini
[boot]
systemd=true
```

Then restart WSL from PowerShell:

```powershell
wsl --shutdown
wsl
```

### GPU not detected

Ensure:
1. Windows has an NVIDIA GPU driver installed (version 510.06 or later)
2. WSL2 is up to date: `wsl --update`
3. No CUDA or NVIDIA driver packages are installed inside WSL
4. Run `nvidia-smi` inside WSL to verify GPU visibility

---

For more details, see:
- [docs/SECURITY.md](./SECURITY.md) - Security architecture and privilege model
- [scripts/doctor.sh](../scripts/doctor.sh) - Environment verification checks
- [.kiro/specs/phase-0-foundation/](../.kiro/specs/phase-0-foundation/) - Phase 0 requirements and design
