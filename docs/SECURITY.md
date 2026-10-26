# IntelliShell Security Model

This document describes the security architecture, threat model, trust boundaries, and known security issues in IntelliShell.

## Security Assets

### System Assets

1. **Linux System Stability**: Kernel and system processes must remain stable
2. **User Files**: Home directory and data files must be protected from unintended modification
3. **System Processes**: Critical processes (systemd, kernel threads, etc.) must not be terminated
4. **Resource Availability**: CPU, memory, disk, network must be shared fairly

### Data Assets

1. **User Commands**: Command history and patterns (privacy-sensitive)
2. **Execution Results**: Output and exit codes
3. **System Metrics**: Process stats, resource usage
4. **Configuration**: Secrets, API keys, model configurations

## Threat Actors

### External Actors

- **Attacker on local network**: Cannot reach services (localhost-only bindings)
- **Attacker with remote access**: Out of scope (assume WSL2 host is secure)

### Internal Actors

- **User**: Trusted to use the tool appropriately
- **Dashboard**: Trusted static assets, browser sandbox
- **AI Service**: Trusted classification logic
- **Core Engine**: Privileged, must not be abused
- **Ollama**: Trusted LLM runtime

### Threats

1. **Malicious command execution**: User accidentally or intentionally runs destructive commands
2. **Privilege escalation**: Unprivileged component gains elevated access
3. **Resource exhaustion**: Runaway process consumes all CPU/memory
4. **Information disclosure**: Sensitive data leaks via logs or API responses
5. **Sandbox escape**: Process breaks out of isolation
6. **Supply chain compromise**: Malicious dependencies introduced

## Trust Boundaries

```
┌─────────────────────────────────────────────────────┐
│ Browser (Untrusted User Input)                      │
│                                                     │
│   ┌─────────────────────────────────────────────┐ │
│   │ Dashboard (Trusted Static Assets)           │ │
│   └─────────────────────────────────────────────┘ │
│                      ↓ HTTP                         │
│   ┌─────────────────────────────────────────────┐ │
│   │ AI Service (Unprivileged Business Logic)    │ │
│   └─────────────────────────────────────────────┘ │
│              ↓ HTTP           ↓ HTTP               │
│   ┌──────────────────┐   ┌──────────────────┐    │
│   │ Core Engine      │   │ Ollama           │    │
│   │ (Privileged)     │   │ (Unprivileged)   │    │
│   └──────────────────┘   └──────────────────┘    │
│         ↓ syscalls                                 │
│   ┌─────────────────────────────────────────────┐ │
│   │ Linux Kernel (cgroups, namespaces, /proc)   │ │
│   └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Boundary 1: Browser ↔ Dashboard

- **Protocol**: HTTPS (future production), HTTP (dev)
- **Trust**: Browser is untrusted; user input is validated by Dashboard before sending to backend
- **Threats**: XSS, CSRF, injection attacks
- **Mitigations**: React's automatic escaping, Content-Security-Policy headers (future), CORS restrictions

### Boundary 2: Dashboard ↔ AI Service / Core Engine

- **Protocol**: HTTP on localhost (127.0.0.1)
- **Trust**: Dashboard is trusted; API calls are authenticated (future)
- **Threats**: SSRF (mitigated by localhost binding), command injection
- **Mitigations**: Localhost-only bindings, input validation, CORS restrictions, authentication (Phase 5)

### Boundary 3: AI Service ↔ Ollama

- **Protocol**: HTTP on localhost (127.0.0.1)
- **Trust**: Both trusted; communication is not encrypted (no need on localhost)
- **Threats**: Prompt injection (adversarial inputs)
- **Mitigations**: Prompt sanitization, output validation, rate limiting (Phase 4)

### Boundary 4: Core Engine ↔ Kernel

- **Protocol**: System calls (clone3, setns, cgroup API, read /proc)
- **Trust**: Core Engine is privileged; kernel enforces isolation
- **Threats**: Kernel exploits, cgroup misconfiguration, namespace escape
- **Mitigations**: Use stable kernel features only, capability dropping, seccomp (Phase 5)

## Privilege Model

### Component Privileges

| Component | User | Effective Privileges | Required Capabilities | Rationale |
|-----------|------|---------------------|----------------------|-----------|
| **Dashboard** | Regular user | None | None | Static web assets |
| **AI Service** | Regular user | None | None | Classification logic requires no privileges |
| **Ollama** | Regular user | None | None | LLM inference is unprivileged |
| **Core Engine** | root (or regular user with capabilities) | Elevated | CAP_SYS_ADMIN (cgroup delegation), CAP_SYS_CHROOT (future) | Cgroup creation and namespace entry require privileges |

### Principle of Least Privilege

**Current State (Phase 0)**:
- Core Engine runs as root (inherited from prototype)
- No capability dropping
- No seccomp filters

**Future Hardening (Phase 5)**:
- Core Engine starts as root, drops to regular user after cgroup setup
- Retain only CAP_SYS_ADMIN for cgroup operations
- Apply seccomp filter to sandboxed processes (whitelist syscalls)
- SELinux or AppArmor profile for Core Engine

### Privilege Separation

Only the sandbox/process-control component in core-engine may be privileged:
- **Privileged**: Cgroup creation, namespace entry, resource limit enforcement
- **Unprivileged**: Command parsing, HTTP API handling, logging, metrics

Future refactoring (Phase 5) will split core-engine into:
- **core-api**: HTTP server (unprivileged)
- **core-sandbox**: Privilege manager (minimal privileges)

## Security Controls

### Network Isolation

#### Localhost-Only Bindings

**Rule**: All services bind to `127.0.0.1` exclusively, never `0.0.0.0`.

**Enforcement**:
- AI Service: `uvicorn --host 127.0.0.1 --port 8000`
- Dashboard: `vite --host 127.0.0.1 --port 5173`
- Core Engine: Bind address hardcoded to `127.0.0.1:3000` (future)
- Ollama: Service configuration verified by `make doctor`

**Verification**:
```bash
ss -ltnp | grep -E '3000|8000|5173|11434'
# All should show 127.0.0.1, not 0.0.0.0
```

#### CORS Restrictions

**Rule**: Only localhost origins are allowed for CORS requests.

**Configuration**:
- AI Service (FastAPI):
  ```python
  from fastapi.middleware.cors import CORSMiddleware
  app.add_middleware(
      CORSMiddleware,
      allow_origins=["http://localhost:5173", "http://127.0.0.1:5173"],
      allow_credentials=True,
      allow_methods=["*"],
      allow_headers=["*"],
  )
  ```

- Core Engine (future): Same allowed origins in HTTP server configuration

**NO wildcard origins**: `allow_origins=["*"]` is forbidden.

### Secret Management

#### Secret Storage

**Location**: `~/.config/intellishell/secret`

**Format**: 64-character hexadecimal string (32 bytes from `openssl rand -hex 32`)

**Permissions**: `0600` (read/write by owner only)

**Generation**: Automatic during `scripts/bootstrap.sh` run

**Usage** (future phases):
- Phase 4: Ollama API authentication
- Phase 5: JWT signing for user sessions
- Phase 6: Audit log encryption

#### Environment Variables

**Rule**: Sensitive configuration never in repository.

**Implementation**:
- `config/default.env`: Committed, contains non-sensitive defaults
- `config/local.env`: Git-ignored, contains overrides and secrets
- `.gitignore` explicitly excludes `.env`, `*.env` (except defaults), `config/local.env`

**Verification**:
```bash
git ls-files | grep -E '\.env$|local\.env'  # Should be empty
```

#### Detection

**Pre-commit hook**: gitleaks scans for secrets before every commit

**CI job**: gitleaks runs on every push and PR

**Failure response**: If secret detected, commit is blocked; developer must remove secret and rewrite history

### Sudo Usage

**Rule**: `sudo` is restricted to essential operations only, all documented.

**Allowed operations** (documented in [ENVIRONMENT.md](./ENVIRONMENT.md)):

1. **apt-get update** and **apt-get install**: Install system packages
2. **systemctl** commands: Manage Ollama systemd service (if not using `--user` mode)
3. **/etc/wsl.conf** edits: Enable systemd in WSL2 (one-time configuration)

**Forbidden operations**:
- File operations within repository (use regular user)
- Running application code with sudo
- Installing language-specific packages (cargo, uv, npm manage these)
- chmod 777 (never use world-writable permissions)

**Enforcement**:
- Code review: Reject PRs with undocumented sudo usage
- Makefile targets: Never use sudo
- Scripts: Include comment explaining each sudo usage

### Filesystem Permissions

**Rules**:

| File Type | Mode | Owner | Rationale |
|-----------|------|-------|-----------|
| Secrets | `0600` | User | Owner-only read/write |
| Config files | `0644` | User | Owner write, world read |
| Scripts | `0755` | User | Owner write, world execute |
| Private scripts | `0700` | User | Owner-only execute |
| Directories | `0755` | User | Standard directory permissions |

**Never use**:
- `chmod 777` (world-writable)
- `chmod 666` (world-writable files)
- `chown root` (unless specifically required for privilege component)

### Input Validation

#### Command Validation (AI Service)

**Threats**: Command injection, shell metacharacter abuse

**Mitigations**:
- Classify before execution
- Warn on dangerous patterns (`;`, `|`, `&&`, backticks, `$(...)`)
- Suggest quoted alternatives

#### API Input Validation (FastAPI)

**Threats**: JSON injection, oversized payloads

**Mitigations**:
- Pydantic schemas enforce structure and types
- Max request size: 1 MB
- Rate limiting (future)

### Audit and Logging

**Current State (Phase 0)**:
- No structured audit log
- Component logs to stdout/stderr
- No log retention policy

**Future (Phase 6)**:
- Structured audit log (JSON lines)
- Log events: command execution, risk classification, sandbox creation, authentication
- Log rotation and retention (30 days default)
- Log integrity (append-only, HMAC signatures)

## Known Security Issues

The following issues are documented and assigned to future phases for remediation. **Do not fix these in Phase 0.**

| Issue | Description | Risk Level | Impact | Phase Owner |
|-------|-------------|------------|--------|-------------|
| `/execute` trusts caller's `risk_level` | The Core Engine `/execute` endpoint (future) accepts a `risk_level` parameter from the caller without server-side validation. An attacker controlling the Dashboard could mark a dangerous command as "low" risk to bypass sandboxing. | **High** | Dangerous commands execute without isolation | **Phase 5** |
| Core Engine runs as root with no authentication | Core Engine currently runs as root and has no authentication. Any process on localhost can send commands. An attacker with local access can execute arbitrary commands with root privileges via the API. | **High** | Full system compromise | **Phase 5** |
| Sandbox shares host filesystem | The sandbox does not use mount namespaces yet. Sandboxed processes can access the entire host filesystem, including `/`, `/home`, `/etc`. | **Medium** | Data exfiltration, unauthorized file access | **Phase 3** |
| `/proc` mounts may leak to host namespace | The `/proc` filesystem in sandboxed processes currently shows host processes. This leaks information about the host and may allow sandbox escape via /proc/self/exe or /proc/sys manipulation. | **Medium** | Information disclosure, potential escape | **Phase 3** |
| Signal handlers are not async-signal-safe | Current signal handlers in core-engine may call non-async-signal-safe functions (e.g., logging, allocations). This can cause deadlocks or undefined behavior. | **Low** | Core Engine crashes or hangs | **Phase 2** |
| Ollama has no authentication | Ollama listens on localhost without authentication. Any local process can send prompts and consume resources. | **Low** | Resource exhaustion, prompt injection | **Phase 4** |
| No audit logging | There is no record of who executed what command, when, or with what result. This hinders forensics and compliance. | **Medium** | No accountability, compliance violations | **Phase 6** |

### Why These Are Deferred

- **Phase 0 is foundation**: Focus is on reproducibility, tooling, and environment setup
- **Prototype verification**: These issues exist in the current working prototype and have not caused problems during development
- **Incremental hardening**: Security improves phase-by-phase with explicit milestones
- **Clear ownership**: Each issue has a designated phase owner responsible for design and implementation

### What NOT to Do in Phase 0

- Do NOT add authentication to Core Engine (Phase 5)
- Do NOT implement mount namespace isolation (Phase 3)
- Do NOT add audit logging (Phase 6)
- Do NOT refactor signal handlers (Phase 2)

**Only implement the security baseline** documented in Phase 0 requirements (localhost bindings, CORS restrictions, secret management).

## Security Checklist for New Code

Before merging any pull request:

- [ ] Service binds to `127.0.0.1` only (not `0.0.0.0`)
- [ ] CORS restricted to `localhost:5173` and `127.0.0.1:5173` (no wildcard)
- [ ] No hardcoded secrets, API keys, tokens, or passwords
- [ ] Sensitive configuration in `config/local.env` (git-ignored)
- [ ] No `chmod 777` or world-writable files
- [ ] Sudo usage documented in ENVIRONMENT.md
- [ ] Pre-commit hooks pass (including gitleaks secret scan)
- [ ] CI security audit jobs pass (cargo audit, pip-audit, npm audit)
- [ ] New dependencies reviewed for typosquatting and trustworthiness
- [ ] Input validation for all API endpoints
- [ ] Error messages do not leak sensitive information

## Incident Response

### Reporting Security Issues

**DO NOT create a public GitHub issue for security vulnerabilities.**

**Instead**:
1. Email project maintainers directly (contact info in README.md)
2. Provide detailed information:
   - Steps to reproduce
   - Impact assessment
   - Affected versions
   - Suggested remediation (optional)
3. Allow time for coordinated disclosure

### Response Process

1. **Acknowledge**: Maintainers acknowledge receipt within 48 hours
2. **Assess**: Determine severity and impact (CVSS score)
3. **Fix**: Develop and test patch
4. **Coordinate**: Agree on disclosure timeline with reporter
5. **Disclose**: Publish security advisory and patched version
6. **Document**: Add to known issues table with phase owner if not immediately fixable

## Security Roadmap

### Phase 0 (Current)

- ✅ Localhost-only bindings
- ✅ CORS restrictions
- ✅ Secret management infrastructure
- ✅ gitleaks pre-commit and CI checks
- ✅ Dependency audit (cargo/pip/npm)
- ✅ Known issues documented

### Phase 1-2

- Signal handler safety
- Improved error handling
- Rate limiting on APIs

### Phase 3

- Mount namespace isolation
- Private /proc for sandboxed processes
- Filesystem access control (read-only, overlayfs)

### Phase 4

- Ollama authentication
- Advanced LLM prompt validation
- Model output sanitization

### Phase 5

- Core Engine authentication (JWT or similar)
- Server-side risk classification validation
- Capability-based privilege model
- Seccomp filters

### Phase 6

- Structured audit logging
- Log integrity (HMAC, append-only)
- Compliance reporting (who ran what, when)

---

For more details, see:
- [docs/ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture and components
- [docs/ENVIRONMENT.md](./ENVIRONMENT.md) - Environment setup and toolchain
- [.kiro/steering/security.md](../.kiro/steering/security.md) - Security rules and guidelines
