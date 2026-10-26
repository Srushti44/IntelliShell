#!/usr/bin/env bash
# IntelliShell bootstrap script - idempotent installation of all dependencies
set -euo pipefail

echo "=== IntelliShell Bootstrap ==="
echo

# Check we're running in WSL2
if ! grep -qi microsoft /proc/version; then
    echo "ERROR: This script must run inside WSL2"
    exit 1
fi

echo "✓ Running in WSL2"
echo

# Update apt package list
echo "Updating apt package list..."
sudo apt-get update -qq

# Install system packages
echo "Installing system packages..."
PACKAGES=(
    build-essential
    pkg-config
    libssl-dev
    curl
    git
    zstd
    shellcheck
    jq
)

# Check which packages are missing
MISSING_PACKAGES=()
for pkg in "${PACKAGES[@]}"; do
    if ! dpkg -l | grep -q "^ii  $pkg "; then
        MISSING_PACKAGES+=("$pkg")
    fi
done

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
    echo "Installing: ${MISSING_PACKAGES[*]}"
    sudo apt-get install -y "${MISSING_PACKAGES[@]}"
else
    echo "All system packages already installed"
fi
echo

# Install rustup
if [ ! -f "$HOME/.cargo/bin/rustup" ]; then
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
else
    echo "✓ rustup already installed"
fi
echo

# Install nvm and Node 24
if [ ! -d "$HOME/.nvm" ]; then
    echo "Installing nvm..."
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
    export NVM_DIR="$HOME/.nvm"
    # shellcheck disable=SC1091
    [ -s "$NVM_DIR/nvm.sh" ] && source "$NVM_DIR/nvm.sh"
    echo "Installing Node 24..."
    nvm install 24
    nvm use 24
else
    echo "✓ nvm already installed"
    export NVM_DIR="$HOME/.nvm"
    # shellcheck disable=SC1091
    [ -s "$NVM_DIR/nvm.sh" ] && source "$NVM_DIR/nvm.sh"
    if ! nvm list | grep -q "v24"; then
        echo "Installing Node 24..."
        nvm install 24
    fi
    nvm use 24
fi
echo

# Install uv (Python package manager)
if ! command -v uv &> /dev/null; then
    echo "Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
else
    echo "✓ uv already installed"
fi
echo

# Install GitHub CLI
if ! command -v gh &> /dev/null; then
    echo "Installing GitHub CLI..."
    # Use official GitHub CLI installation for Ubuntu
    (type -p wget >/dev/null || (sudo apt-get update && sudo apt-get install -y wget)) \
    && sudo mkdir -p -m 755 /etc/apt/keyrings \
    && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
    && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && sudo apt-get update \
    && sudo apt-get install -y gh
else
    echo "✓ GitHub CLI already installed"
fi
echo

# Create config directory for IntelliShell secrets
CONFIG_DIR="$HOME/.config/intellishell"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Creating config directory..."
    mkdir -p "$CONFIG_DIR"
    chmod 0700 "$CONFIG_DIR"
else
    echo "✓ Config directory exists"
    chmod 0700 "$CONFIG_DIR"
fi

# Generate machine secret if it doesn't exist
SECRET_FILE="$CONFIG_DIR/secret"
if [ ! -f "$SECRET_FILE" ]; then
    echo "Generating machine secret..."
    openssl rand -hex 32 > "$SECRET_FILE"
    chmod 0600 "$SECRET_FILE"
else
    echo "✓ Machine secret already exists"
    chmod 0600 "$SECRET_FILE"
fi
echo

# Display installed versions
echo "=== Installed Versions ==="
echo "Rust toolchain: $(rustc --version 2>/dev/null || echo 'not in PATH - run: source ~/.cargo/env')"
echo "Node: $(node --version 2>/dev/null || echo 'not in PATH - run: source ~/.nvm/nvm.sh && nvm use 24')"
echo "Python: $(python3 --version)"
echo "uv: $(uv --version 2>/dev/null || echo 'not in PATH - run: source ~/.cargo/env')"
echo "gh: $(gh --version | head -n1)"
echo
echo "✓ Bootstrap complete!"
echo
echo "Next steps:"
echo "  1. Reload shell or run: source ~/.cargo/env && source ~/.nvm/nvm.sh"
echo "  2. Run: make doctor"
