# Security Rules

## Core Security Principles

IntelliShell handles privileged operations and system-level access. Security is embedded in every design decision.

### Principle of Least Privilege

- **Only the sandbox component may be privileged**: The cgroup/namespace isolation code in core-engine requires elevated permissions
- **AI service runs unprivileged**: Command classification and LLM interaction require no special privileges
- **Dashboard runs unprivileged**: Web UI is a standard user-space application
- **Sandbox uses capability dropping**: Even privileged components drop unnecessary capabilities

### Network Isolation

- **All services bind 127.0.0.1 only**: No service listens on `0.0.0.0` or external interfaces
- **No external communication**: IntelliShell never sends data outside the local machine
- **Ollama localhost-only**: LLM inference happens locally on `127.0.0.1:11434`
- **CORS restrictions**: Core-engine and ai-service restrict CORS to:
  - `http://localhost:5173`
  - `http://127.0.0.1:5173`
  - No wildcard origins

### Secret Management

- **Secrets never in repository**: No API keys, tokens, passwords, or cryptographic keys are committed
- **Per-machine secrets**: `~/.config/intellishell/secret` generated at bootstrap time
- **File permissions**: All secrets have mode `0600` (owner read/write only)
- **Environment variables**: Sensitive config in `config/local.env` (git-ignored)
- **Detection**: gitleaks runs in pre-commit hooks and CI to catch accidental commits

## Privilege Model

### Core Engine (Privileged Component)

The core-engine binary may run as root or with specific capabilities:

**Required privileges**:
- Create and manage cgroup hierarchies
- Delegate cgroup controllers to sandboxed processes
- Create and enter namespaces (mount, PID, IPC, network, UTS)
- Send signals to processes (including in other namespaces)

**Privilege boundaries**:
- No network access (sandbox uses network namespace)
- No access to user's home directory from within sandbox (Phase 3+)
- Resource limits enforced by cgroup (memory, CPU, process count)

**Future hardening** (Phase 5):
- Capability-based model (drop CAP_SYS_ADMIN after setup)
- Seccomp filters on sandboxed processes
- SELinux or AppArmor profiles

### AI Service (Unprivileged)

- Runs as regular user
- No sudo or elevated permissions
- Reads only: command strings and configuration
- Writes only: logs and temporary analysis results
- HTTP client to Ollama (localhost)

### Dashboard (Unprivileged)

- Static web assets served by Vite dev server or nginx
- JavaScript runs in browser sandbox
- HTTP client to ai-service and core-engine (localhost)

## Sudo Usage

`sudo` is restricted to documented, essential operations only:

**Allowed sudo commands** (all documented in docs/ENVIRONMENT.md):

1. **apt-get update** and **apt-get install**: Install system packages during bootstrap
2. **systemctl commands**: Manage Ollama systemd service (if not using `--user`)
3. **/etc/wsl.conf edits**: Enable systemd in WSL2 (one-time configuration)

**Never use sudo for**:
- File operations within the repository
- Running application code
- Installing language-specific packages (cargo, uv, npm handle these)

## File Permissions

- **Never use `chmod 777`**: World-writable files are forbidden
- **Secret files**: Mode `0600` (owner only)
- **Config files**: Mode `0644` (owner write, world read)
- **Executables**: Mode `0755` (owner write, world execute)
- **Scripts**: Mode `0755` or `0700` depending on sensitivity

## Windows-Side Restrictions

To maintain reproducibility and avoid platform-specific configuration:

**Do not modify**:
- `.wslconfig` (Windows WSL global settings)
- Windows firewall rules
- Windows network settings
- Host file mappings

**Do not install inside WSL**:
- NVIDIA driver (rely on Windows driver passthrough)
- CUDA toolkit with bundled driver
- Any kernel modules

## Supply Chain Security

### Dependency Management

- **Pin versions**: All dependencies locked (Cargo.lock, uv.lock, package-lock.json)
- **Audit regularly**: CI runs `cargo audit`, `pip-audit`, `npm audit`
- **Fail on high/critical**: Builds fail if high or critical vulnerabilities found
- **Dependabot**: Automated PR creation for dependency updates
- **Pin third-party CI actions**: Use full commit SHAs, not tags (e.g., `actions/checkout@abc123def...`)

### Repository Security

- **No embedded credentials**: Git config uses `gh auth` or SSH keys, never `credential.helper store`
- **HTTPS or SSH**: Remote URLs use HTTPS (with gh auth) or SSH (with keys)
- **Signed commits** (optional): Encourage GPG signing for accountability

## Known Security Issues

The following issues are documented but **NOT fixed in Phase 0**. Each has a phase owner responsible for remediation.

| Issue | Description | Risk | Phase Owner |
|-------|-------------|------|-------------|
| `/execute` trusts `risk_level` | Caller provides risk level; no server-side validation | High | Phase 5 |
| Core-engine runs as root with no auth | Any process on localhost can send commands | High | Phase 5 |
| Sandbox shares host filesystem | No mount namespace isolation yet | Medium | Phase 3 |
| `/proc` mounts may leak | Host /proc visible in sandbox | Medium | Phase 3 |
| Signal safety issues | Signal handlers not async-signal-safe | Low | Phase 2 |
| Ollama has no authentication | Anyone on localhost can use LLM | Low | Phase 4 |
| No audit logging | No record of who executed what | Medium | Phase 6 |

### Why These Are Deferred

- **Phase 0 is foundation**: Environment setup, tooling, and reproducibility
- **Prototype verification**: These issues exist in the current working prototype
- **Incremental hardening**: Security improves phase-by-phase
- **Explicit ownership**: Each issue has a designated phase for resolution

## Security Checklist for New Code

Before merging any feature:

- [ ] Service binds `127.0.0.1` only (not `0.0.0.0`)
- [ ] CORS restricted to localhost origins (no `*`)
- [ ] No hardcoded secrets, tokens, or keys
- [ ] Sensitive config in `config/local.env` (git-ignored)
- [ ] No `chmod 777` or world-writable files
- [ ] Sudo usage documented in ENVIRONMENT.md
- [ ] Pre-commit hooks pass (including gitleaks)
- [ ] CI security audit jobs pass
- [ ] New dependencies reviewed (not typosquatting)

## Incident Response

If a security issue is discovered:

1. **Do not create a public issue**: Email project maintainers directly
2. **Provide details**: Steps to reproduce, impact assessment, affected versions
3. **Coordinate disclosure**: Agree on fix timeline and disclosure date
4. **Document in SECURITY.md**: Add to known issues table with phase owner
