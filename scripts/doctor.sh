#!/usr/bin/env bash
# IntelliShell environment doctor - verify system meets all requirements
set -euo pipefail

# Parse arguments
JSON_OUTPUT=false
if [ "${1:-}" = "--json" ]; then
    JSON_OUTPUT=true
fi

# Get script directory and repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load configuration
CONFIG_FILE="$REPO_ROOT/config/default.env"
LOCAL_CONFIG="$REPO_ROOT/config/local.env"

if [ -f "$LOCAL_CONFIG" ]; then
    # shellcheck disable=SC1090
    source "$LOCAL_CONFIG"
elif [ -f "$CONFIG_FILE" ]; then
    # shellcheck disable=SC1090
    source "$CONFIG_FILE"
else
    echo "ERROR: config/default.env not found"
    exit 1
fi

# Initialize counters and results
FAIL_COUNT=0
WARN_COUNT=0
declare -a RESULTS=()

# Helper functions
version_ge() {
    # Compare versions: returns 0 if $1 >= $2
    printf '%s\n%s\n' "$2" "$1" | sort -V -C
}

add_result() {
    local status=$1
    local check=$2
    local message=$3

    if [ "$JSON_OUTPUT" = true ]; then
        # Escape quotes in message
        local escaped_msg="${message//\"/\\\"}"
        RESULTS+=("{\"check\":\"$check\",\"status\":\"$status\",\"message\":\"$escaped_msg\"}")
    else
        case $status in
            PASS) echo "✓ PASS: $check" ;;
            WARN) echo "⚠ WARN: $check - $message"; WARN_COUNT=$((WARN_COUNT + 1)) ;;
            FAIL) echo "✗ FAIL: $check - $message"; FAIL_COUNT=$((FAIL_COUNT + 1)) ;;
        esac
    fi
}

# FAIL checks (critical requirements)

check_wsl2() {
    if grep -qi microsoft /proc/version; then
        add_result "PASS" "WSL2 detected" ""
    else
        add_result "FAIL" "WSL2 detected" "Not running in WSL2. This project requires WSL2 on Windows."
    fi
}

check_kernel_version() {
    local kernel_version
    kernel_version=$(uname -r | cut -d'-' -f1)
    if version_ge "$kernel_version" "5.15"; then
        add_result "PASS" "Kernel version >= 5.15" ""
    else
        add_result "FAIL" "Kernel version >= 5.15" "Kernel $kernel_version is too old. Need >= 5.15 for cgroup.kill and pidfd. Run: wsl --update"
    fi
}

check_systemd() {
    if pidof systemd >/dev/null 2>&1 || [ "$(ps -p 1 -o comm=)" = "systemd" ]; then
        add_result "PASS" "systemd running" ""
    else
        add_result "FAIL" "systemd running" "systemd is not running. Add [boot] systemd=true to /etc/wsl.conf and restart WSL."
    fi
}

check_cgroup_v2() {
    local fstype
    fstype=$(stat -fc %T /sys/fs/cgroup 2>/dev/null || echo "unknown")

    if [ "$fstype" = "cgroup2fs" ]; then
        # Check for required controllers
        local controllers
        controllers=$(cat /sys/fs/cgroup/cgroup.controllers 2>/dev/null || echo "")

        if echo "$controllers" | grep -q "cpu" && \
           echo "$controllers" | grep -q "memory" && \
           echo "$controllers" | grep -q "pids"; then
            add_result "PASS" "cgroup v2 with cpu, memory, pids" ""
        else
            add_result "FAIL" "cgroup v2 with cpu, memory, pids" "cgroup v2 detected but missing controllers: $controllers. Need cpu, memory, pids."
        fi
    else
        add_result "FAIL" "cgroup v2 with cpu, memory, pids" "Filesystem type is $fstype, not cgroup2fs. systemd.unified_cgroup_hierarchy=1 may be needed."
    fi
}

check_overlayfs() {
    if grep -q overlay /proc/filesystems; then
        add_result "PASS" "overlayfs available" ""
    else
        add_result "FAIL" "overlayfs available" "overlayfs not in /proc/filesystems. Kernel module may need loading."
    fi
}

check_proc_health() {
    local exe_link
    exe_link=$(readlink /proc/self/exe 2>/dev/null || echo "")

    if [ -n "$exe_link" ] && [ -r /proc/mounts ]; then
        add_result "PASS" "/proc health" ""
    else
        add_result "FAIL" "/proc health" "/proc appears corrupted (mount issue detected in prototype). Restart WSL: wsl --shutdown"
    fi
}

check_repo_filesystem() {
    local repo_path
    repo_path=$(git -C "$REPO_ROOT" rev-parse --show-toplevel 2>/dev/null || echo "$REPO_ROOT")

    local fstype
    fstype=$(stat -f -c %T "$repo_path" 2>/dev/null || echo "unknown")

    if [ "$fstype" = "9p" ] || [ "$fstype" = "v9fs" ] || [ "$fstype" = "drvfs" ]; then
        add_result "FAIL" "repo not on 9p/drvfs" "Repository is on $fstype filesystem (Windows mount at /mnt/*). Clone/move repo to /home/<user>/ for correct permissions and performance."
    else
        add_result "PASS" "repo not on 9p/drvfs" ""
    fi
}

check_disk_space() {
    local free_gb
    free_gb=$(df -BG "$REPO_ROOT" | awk 'NR==2 {print $4}' | sed 's/G//')

    if [ "$free_gb" -ge "$MIN_DISK_GB" ]; then
        add_result "PASS" "free disk >= ${MIN_DISK_GB}GB" ""
    else
        add_result "FAIL" "free disk >= ${MIN_DISK_GB}GB" "Only ${free_gb}GB free. Need at least ${MIN_DISK_GB}GB. Free up space or move to larger partition."
    fi
}

check_git_identity() {
    local git_name git_email
    git_name=$(git config user.name 2>/dev/null || echo "")
    git_email=$(git config user.email 2>/dev/null || echo "")

    if [ -n "$git_name" ] && [ -n "$git_email" ]; then
        add_result "PASS" "git identity set" ""
    else
        add_result "FAIL" "git identity set" "Run: git config --global user.name \"Your Name\" && git config --global user.email \"you@example.com\""
    fi
}

check_toolchains() {
    local all_ok=true
    local messages=()

    # Load nvm if available (needed for Node check)
    if [ -s "$HOME/.nvm/nvm.sh" ]; then
        # shellcheck disable=SC1091
        source "$HOME/.nvm/nvm.sh"
    fi

    # Check Rust
    if command -v rustc >/dev/null 2>&1; then
        local rust_version
        rust_version=$(rustc --version | awk '{print $2}')
        local expected_rust="1.87.0"
        if [ "$rust_version" = "$expected_rust" ]; then
            : # OK
        else
            all_ok=false
            messages+=("rustc is $rust_version, expected $expected_rust")
        fi
    else
        all_ok=false
        messages+=("rustc not found. Run: source ~/.cargo/env or ./scripts/bootstrap.sh")
    fi

    # Check Node
    if command -v node >/dev/null 2>&1; then
        local node_version
        node_version=$(node --version | sed 's/v//' | cut -d'.' -f1)
        if [ "$node_version" = "24" ]; then
            : # OK
        else
            all_ok=false
            messages+=("node is v$node_version, expected v24")
        fi
    else
        all_ok=false
        messages+=("node not found. Run: source ~/.nvm/nvm.sh && nvm use 24")
    fi

    # Check Python (prefer uv-managed Python if available, fallback to system)
    local python_cmd="python3"
    if command -v uv >/dev/null 2>&1 && [ -f "$REPO_ROOT/ai-service/.python-version" ]; then
        # Check uv-managed Python version
        local python_version
        python_version=$(cd "$REPO_ROOT/ai-service" && uv run python --version 2>/dev/null | awk '{print $2}' | cut -d'.' -f1,2)
        if [ -n "$python_version" ] && version_ge "$python_version" "3.13"; then
            : # OK
        else
            all_ok=false
            messages+=("uv-managed Python is ${python_version:-missing}, expected >= 3.13")
        fi
    elif command -v python3 >/dev/null 2>&1; then
        local python_version
        python_version=$(python3 --version | awk '{print $2}' | cut -d'.' -f1,2)
        if version_ge "$python_version" "3.13"; then
            : # OK
        else
            all_ok=false
            messages+=("python3 is $python_version, expected >= 3.13")
        fi
    else
        all_ok=false
        messages+=("python3 not found")
    fi

    if [ "$all_ok" = true ]; then
        add_result "PASS" "toolchain versions match pins" ""
    else
        local msg
        msg=$(IFS="; "; echo "${messages[*]}")
        add_result "FAIL" "toolchain versions match pins" "$msg"
    fi
}

check_secret_file() {
    local secret_file="$HOME/.config/intellishell/secret"

    if [ ! -f "$secret_file" ]; then
        add_result "FAIL" "machine secret exists" "Run: ./scripts/bootstrap.sh to create $secret_file"
        return
    fi

    local perms
    perms=$(stat -c %a "$secret_file" 2>/dev/null || echo "unknown")

    if [ "$perms" = "600" ]; then
        # Check if tracked by git
        if git -C "$REPO_ROOT" ls-files --error-unmatch "$secret_file" >/dev/null 2>&1; then
            add_result "FAIL" "machine secret not tracked by git" "Secret file is tracked by git! Run: git rm --cached $secret_file"
        else
            add_result "PASS" "machine secret secure" ""
        fi
    else
        add_result "FAIL" "machine secret has mode 0600" "Current mode: $perms. Run: chmod 0600 $secret_file"
    fi
}

# WARN checks (warnings that don't block)

check_ram() {
    local total_ram_gb
    total_ram_gb=$(free -g | awk '/^Mem:/ {print $2}')

    if [ "$total_ram_gb" -lt "$MIN_RAM_GB" ]; then
        add_result "WARN" "available RAM >= ${MIN_RAM_GB}GB" "Only ${total_ram_gb}GB RAM available. ${MIN_RAM_GB}GB recommended. Adjust in .wslconfig if needed."
    else
        add_result "PASS" "available RAM >= ${MIN_RAM_GB}GB" ""
    fi
}

check_disk_warn() {
    local free_gb
    free_gb=$(df -BG "$REPO_ROOT" | awk 'NR==2 {print $4}' | sed 's/G//')

    # Skip if already failed the minimum check
    if [ "$free_gb" -ge "$MIN_DISK_GB" ]; then
        if [ "$free_gb" -lt "$WARN_DISK_GB" ]; then
            add_result "WARN" "free disk >= ${WARN_DISK_GB}GB" "Only ${free_gb}GB free. ${WARN_DISK_GB}GB recommended for builds and LLM models."
        else
            add_result "PASS" "free disk >= ${WARN_DISK_GB}GB" ""
        fi
    fi
}

check_ports() {
    local listening_ports
    listening_ports=$(ss -ltn 2>/dev/null | grep -E ":(${CORE_ENGINE_PORT}|${AI_SERVICE_PORT}|${DASHBOARD_PORT}|${OLLAMA_PORT})" || echo "")

    if [ -n "$listening_ports" ]; then
        local port_list
        port_list=$(echo "$listening_ports" | awk '{print $4}' | cut -d':' -f2 | sort -u | tr '\n' ',' | sed 's/,$//')
        add_result "WARN" "ports available" "Ports in use: $port_list. Stop conflicting services or adjust config/local.env."
    else
        add_result "PASS" "ports available" ""
    fi
}

check_gpu() {
    if command -v nvidia-smi >/dev/null 2>&1; then
        if nvidia-smi >/dev/null 2>&1; then
            add_result "PASS" "GPU available" ""
        else
            add_result "WARN" "GPU available" "nvidia-smi found but failed. Check Windows NVIDIA driver."
        fi
    else
        add_result "WARN" "GPU available" "No NVIDIA GPU detected. CPU-only inference will be slower."
    fi
}

check_nonloopback_bindings() {
    local non_loopback
    non_loopback=$(ss -ltn 2>/dev/null | awk '$4 !~ /127\.0\.0\.1/ && $4 !~ /\[::\]/ && $4 ~ /:(3000|8000|5173|11434)$/' || echo "")

    if [ -n "$non_loopback" ]; then
        local bindings
        bindings=$(echo "$non_loopback" | awk '{print $4}' | tr '\n' ',' | sed 's/,$//')
        add_result "WARN" "services bind loopback only" "Non-loopback bindings detected: $bindings. Security risk: services should bind 127.0.0.1 only."
    else
        add_result "PASS" "services bind loopback only" ""
    fi
}

# Run all checks
if [ "$JSON_OUTPUT" != true ]; then
    echo "=== IntelliShell Environment Doctor ==="
    echo
    echo "FAIL checks (critical):"
fi

check_wsl2
check_kernel_version
check_systemd
check_cgroup_v2
check_overlayfs
check_proc_health
check_repo_filesystem
check_disk_space
check_git_identity
check_toolchains
check_secret_file

if [ "$JSON_OUTPUT" != true ]; then
    echo
    echo "WARN checks (recommended):"
fi

check_ram
check_disk_warn
check_ports
check_gpu
check_nonloopback_bindings

# Output results
if [ "$JSON_OUTPUT" = true ]; then
    printf "["
    first=true
    for result in "${RESULTS[@]}"; do
        if [ "$first" = true ]; then
            first=false
        else
            printf ","
        fi
        printf "\n  %s" "$result"
    done
    printf "\n]\n"
else
    echo
    echo "=== Summary ==="
    echo "Failures: $FAIL_COUNT"
    echo "Warnings: $WARN_COUNT"
    echo
    if [ "$FAIL_COUNT" -eq 0 ]; then
        echo "✓ All critical checks passed!"
    else
        echo "✗ Fix the failures above before proceeding."
    fi
fi

exit "$FAIL_COUNT"
