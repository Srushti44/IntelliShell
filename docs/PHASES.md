# IntelliShell Development Phases

This document outlines the phased development roadmap for IntelliShell, from foundation to production-ready system.

## Overview

IntelliShell development is organized into phases, each with specific goals and acceptance criteria. Each phase builds on the previous phase's foundation.

---

## Phase 0: Foundation (Current)

**Status**: In Progress

**Goal**: Establish a reproducible development environment, toolchain, and quality infrastructure. After this phase, a fresh Windows + WSL2 machine goes from clone to passing doctor check, passing lint/test/build, and a green CI run.

### Objectives

1. **Reproducible environment**: Pin all toolchains, commit lock files, document setup
2. **Environment verification**: `make doctor` checks all prerequisites with actionable remediation
3. **Quality gates**: Linting, testing, pre-commit hooks, CI pipeline
4. **Repository hygiene**: .gitignore, .editorconfig, LICENSE, proper git auth
5. **Security baseline**: Localhost bindings, CORS restrictions, secret management
6. **Documentation foundation**: Steering files, specs, architecture, ADRs, README

### Key Deliverables

- **Toolchain pins**: rust-toolchain.toml (1.87.0), .nvmrc (24), .python-version (3.13)
- **Scripts**: bootstrap.sh, doctor.sh, llm-smoke.sh
- **Makefile**: All targets (bootstrap, doctor, fmt, lint, test, check, run-*)
- **Pre-commit hooks**: Formatting, linting, secret detection
- **CI pipeline**: Rust, Python, web jobs with security audits
- **Tests**: Real tests for meminfo parsing, /proc/stat parsing, command classification
- **Documentation**: .kiro/steering/, .kiro/specs/phase-0-foundation/, docs/, README.md

### Acceptance Criteria

- [ ] `make doctor` passes with all PASS or WARN (no FAIL)
- [ ] `make check` passes (lint + test + build for all three components)
- [ ] CI runs green on GitHub
- [ ] All documentation files present and complete
- [ ] Repo moved to /home/ (or documented as blocker)
- [ ] Secret in ~/.config/intellishell/secret with mode 0600
- [ ] All local commits pushed to remote

### What Phase 0 Does NOT Include

- ❌ New features or behavior changes
- ❌ HTTP API for core-engine (CLI only)
- ❌ LLM integration in ai-service (rule-based classification only)
- ❌ Dashboard functionality (skeleton only)
- ❌ Authentication or authorization
- ❌ Audit logging

---

## Phase 1: TBD

**Status**: Not Started

**Goal**: To be determined based on Phase 0 outcomes and user feedback.

### Potential Focus Areas

- Command execution API (core-engine HTTP server)
- Dashboard integration with ai-service and core-engine
- Basic process monitoring UI
- Enhanced error handling and user feedback

### Prerequisites

- Phase 0 complete (reproducible environment, passing tests, green CI)

---

## Phase 2: Signal Safety

**Status**: Not Started

**Goal**: Fix signal handling issues in core-engine to prevent crashes and undefined behavior.

### Known Issues (from Phase 0)

- Signal handlers call non-async-signal-safe functions
- Potential deadlocks or crashes under signal load

### Objectives

1. **Audit signal handlers**: Identify all signal handlers in core-engine
2. **Async-signal-safe refactor**: Replace unsafe functions with safe alternatives
3. **Self-pipe trick**: Use self-pipe or signalfd for safe signal delivery
4. **Signal handler tests**: Unit tests for signal scenarios (SIGTERM, SIGINT, SIGCHLD)

### Key Deliverables

- Refactored signal handlers (async-signal-safe)
- Signal delivery mechanism (self-pipe or signalfd)
- Tests for signal handling
- Updated docs/SECURITY.md (remove signal safety issue)

### Acceptance Criteria

- [ ] All signal handlers use only async-signal-safe functions
- [ ] Tests cover SIGTERM, SIGINT, SIGCHLD scenarios
- [ ] No known signal-related crashes or deadlocks
- [ ] Documentation updated

---

## Phase 3: Filesystem Isolation

**Status**: Not Started

**Goal**: Sandbox processes with mount namespaces to restrict filesystem access.

### Known Issues (from Phase 0)

- Sandboxed processes share host filesystem (full access to /, /home, /etc)
- Host /proc visible in sandbox (information leak)

### Objectives

1. **Mount namespace isolation**: Create private mount namespace for sandboxed processes
2. **Private /proc**: Mount private /proc for sandboxed processes
3. **Filesystem access control**:
   - Read-only bind mounts for safe directories (/usr, /bin)
   - Writable overlayfs for /tmp
   - No access to /home, /etc, /root
4. **Configurable isolation**: Per-command isolation policies

### Key Deliverables

- Mount namespace creation in core-engine
- Private /proc mounting
- Overlayfs setup for /tmp workspace
- Isolation policy configuration (YAML or TOML)
- Tests for filesystem isolation (verify no access to host files)
- Updated docs/SECURITY.md (remove filesystem issues)

### Acceptance Criteria

- [ ] Sandboxed processes run in mount namespace
- [ ] Sandboxed processes have private /proc (no host process visibility)
- [ ] Host /home and /etc are not accessible from sandbox
- [ ] Tests verify isolation effectiveness
- [ ] Documentation updated

---

## Phase 4: LLM Integration

**Status**: Not Started

**Goal**: Integrate Ollama for context-aware command analysis and intelligent suggestions.

### Known Issues (from Phase 0)

- Ollama has no authentication (any local process can use it)
- No LLM-based classification (rule-based only)

### Objectives

1. **Ollama authentication**: Add API key or token-based auth
2. **Context-aware classification**: Send command + context (cwd, recent history) to LLM
3. **Intelligent suggestions**: LLM-generated safer alternatives
4. **Prompt engineering**: Design prompts for safety analysis
5. **Model selection**: Evaluate models (phi3, llama3, codellama) for accuracy and speed
6. **Prompt injection defense**: Sanitize user input, validate LLM output

### Key Deliverables

- Ollama authentication implementation
- ai-service LLM client (prompt construction, response parsing)
- Context-aware classification endpoint
- Model evaluation report (accuracy, latency, resource usage)
- Prompt templates (command classification, safer alternatives)
- Tests for LLM integration (mock Ollama in tests)
- Updated docs/SECURITY.md (remove Ollama auth issue)

### Acceptance Criteria

- [ ] Ollama requires authentication (API key or token)
- [ ] ai-service uses LLM for classification (fallback to rules)
- [ ] LLM-generated suggestions are actionable and accurate
- [ ] Tests cover LLM integration scenarios
- [ ] Model choice documented with rationale
- [ ] Documentation updated

---

## Phase 5: Authentication and Authorization

**Status**: Not Started

**Goal**: Secure core-engine API with authentication and server-side risk validation.

### Known Issues (from Phase 0)

- `/execute` endpoint trusts caller's risk_level (no server-side validation)
- core-engine runs as root with no authentication (any localhost process can send commands)

### Objectives

1. **Authentication**: JWT or token-based auth for core-engine and ai-service APIs
2. **Server-side risk validation**: core-engine re-classifies commands (does not trust caller)
3. **Capability-based privileges**: Drop CAP_SYS_ADMIN after cgroup setup
4. **Seccomp filters**: Restrict syscalls in sandboxed processes
5. **Multi-user support**: Per-user secrets, cgroup hierarchies, audit logs

### Key Deliverables

- JWT authentication for HTTP APIs
- Server-side risk classification in core-engine (calls ai-service)
- Capability dropping after privilege operations
- Seccomp filter profiles (whitelist syscalls)
- Per-user secret management
- Tests for authentication and authorization
- Updated docs/SECURITY.md (remove auth issues)

### Acceptance Criteria

- [ ] core-engine requires JWT for API calls
- [ ] core-engine independently classifies commands (does not trust caller)
- [ ] core-engine drops unnecessary capabilities after setup
- [ ] Sandboxed processes run with seccomp filters
- [ ] Tests cover authentication and authorization scenarios
- [ ] Documentation updated

---

## Phase 6: Audit Logging

**Status**: Not Started

**Goal**: Implement structured audit logging for accountability and compliance.

### Known Issues (from Phase 0)

- No audit logging (no record of who executed what, when, result)

### Objectives

1. **Structured logging**: JSON-formatted audit log (one event per line)
2. **Log events**: Command execution, classification, sandbox creation, authentication, errors
3. **Log integrity**: Append-only log with HMAC signatures
4. **Log rotation**: Automatic rotation (size or time-based)
5. **Retention policy**: Configurable retention (default: 30 days)
6. **Compliance reporting**: Query interface for audit log (who ran what, when)

### Key Deliverables

- Audit logger module (Rust in core-engine, Python in ai-service)
- Log schema (JSON format with timestamps, user, command, result)
- HMAC signing for log integrity
- Log rotation configuration (logrotate or built-in)
- Query interface (CLI or API)
- Tests for audit logging
- Updated docs/SECURITY.md (remove audit logging issue)

### Acceptance Criteria

- [ ] All command executions logged
- [ ] All classification requests logged
- [ ] Logs are append-only with integrity checks
- [ ] Log rotation configured
- [ ] Query interface available
- [ ] Tests cover logging scenarios
- [ ] Documentation updated

---

## Phase Dependencies

```
Phase 0 (Foundation)
    ↓
Phase 1 (TBD)
    ↓
Phase 2 (Signal Safety) ──┐
    ↓                       ↓
Phase 3 (Filesystem)   Phase 4 (LLM)
    ↓                       ↓
    └───────→ Phase 5 (Auth) ←────┘
              ↓
          Phase 6 (Audit Logging)
```

**Notes**:
- Phase 2 and Phase 4 can proceed in parallel (no dependencies)
- Phase 3 and Phase 4 both feed into Phase 5
- Phase 6 depends on Phase 5 (multi-user auth needed for audit log)

---

## Future Considerations (Beyond Phase 6)

### Production Deployment

- Systemd service files for all components
- nginx reverse proxy configuration
- TLS certificate management (Let's Encrypt)
- Container image (Docker or Podman) for easier deployment
- Installation script for production environments

### Advanced Features

- **Dashboard enhancements**: Real-time metrics, charts, history
- **Command history**: Search and replay previous commands
- **Templates**: Pre-defined safe command templates
- **Learning mode**: Train custom models on user's command history
- **Integration**: Shell integration (zsh, bash plugins)
- **Notifications**: Slack, email, webhook notifications for high-risk commands

### Scalability

- Multi-tenancy (shared service for multiple users)
- Distributed execution (run sandboxes on remote machines)
- Model serving (dedicated model server separate from Ollama)

---

## Phase Completion Criteria

A phase is considered complete when:

1. ✅ All objectives achieved
2. ✅ All key deliverables implemented and tested
3. ✅ All acceptance criteria met
4. ✅ Documentation updated (remove known issues, add new features)
5. ✅ CI passes (all tests green)
6. ✅ Phase branch merged to main
7. ✅ Git tag created (e.g., `phase-0-complete`, `v0.1.0`)

---

## Revision History

| Date | Phase | Change | Author |
|------|-------|--------|--------|
| 2024-01-15 | Phase 0 | Initial roadmap created | Phase 0 team |

---

For current phase details, see:
- [.kiro/specs/phase-0-foundation/requirements.md](../.kiro/specs/phase-0-foundation/requirements.md)
- [.kiro/specs/phase-0-foundation/design.md](../.kiro/specs/phase-0-foundation/design.md)
- [.kiro/specs/phase-0-foundation/tasks.md](../.kiro/specs/phase-0-foundation/tasks.md)
