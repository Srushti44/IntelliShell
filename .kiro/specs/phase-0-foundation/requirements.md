# Phase 0: Foundation — Requirements Specification

This document specifies the requirements for Phase 0 using EARS (Easy Approach to Requirements Syntax) notation.

## R1. Environment Verification

### R1.1 Doctor Script Execution

**WHEN** the user runs `make doctor`,
**THE SYSTEM SHALL** execute `scripts/doctor.sh` and perform all environment health checks.

### R1.2 Check Output Format

**WHEN** `scripts/doctor.sh` runs without the `--json` flag,
**THE SYSTEM SHALL** print each check result with a status of PASS, WARN, or FAIL followed by descriptive text.

**WHEN** `scripts/doctor.sh` runs with the `--json` flag,
**THE SYSTEM SHALL** output results in valid JSON format with fields: `check`, `status`, `message`, and optional `remediation`.

### R1.3 Exit Code

**WHEN** any check returns FAIL status,
**THE SYSTEM SHALL** exit with a non-zero exit code.

**WHEN** all checks return PASS or WARN status,
**THE SYSTEM SHALL** exit with exit code 0.

### R1.4 FAIL Checks (Critical Requirements)

**WHEN** `scripts/doctor.sh` runs,
**THE SYSTEM SHALL** perform the following FAIL-level checks:

1. **WSL2 Detection**: Verify the system is running under WSL2 by checking `/proc/version` contains "microsoft"
2. **Kernel Version**: Verify kernel version >= 5.15 (required for cgroup.kill and pidfd_open)
3. **Systemd Running**: Verify systemd is active as PID 1
4. **Cgroup v2 Enabled**: Verify `stat -fc %T /sys/fs/cgroup` returns `cgroup2fs`
5. **Cgroup Controllers**: Verify cpu, memory, and pids controllers are listed in `/sys/fs/cgroup/cgroup.controllers`
6. **Overlayfs Support**: Verify overlayfs appears in `/proc/filesystems`
7. **/proc Health**: Verify `readlink /proc/self/exe` resolves successfully and `/proc/mounts` is readable
8. **Filesystem Check**: Verify repository is not on 9p or drvfs filesystem (must be under `/home/`, not `/mnt/*`)
9. **Disk Space**: Verify free disk space >= 5 GB
10. **Git Identity**: Verify git user.name and user.email are configured
11. **Toolchain Versions**: Verify Rust, Node, and Python versions match pins in rust-toolchain.toml, .nvmrc, and .python-version

### R1.5 WARN Checks (Best Practices)

**WHEN** `scripts/doctor.sh` runs,
**THE SYSTEM SHALL** perform the following WARN-level checks:

1. **RAM Check**: Warn if available RAM < 4 GB
2. **Disk Space**: Warn if free disk space < 20 GB
3. **Port Conflicts**: Warn if ports 3000, 8000, 5173, or 11434 are held by unexpected processes
4. **Windows Ollama**: Warn if a Windows-side Ollama instance is detected competing for port 11434
5. **GPU Detection**: Warn if no NVIDIA GPU is visible (report "CPU-only inference")
6. **Network Binding**: Warn if any IntelliShell service is listening on a non-loopback address

### R1.6 Configuration Source

**WHEN** `scripts/doctor.sh` reads configuration,
**THE SYSTEM SHALL** read ports and paths from `config/default.env` with optional overrides from `config/local.env` if present.

### R1.7 Remediation Messages

**WHEN** a check fails,
**THE SYSTEM SHALL** print or include remediation instructions explaining how to fix the issue.

### R1.8 Self-Test

**WHEN** the doctor script includes a self-test mode,
**THE SYSTEM SHALL** force at least one check to fail, verify the exit code is non-zero, and verify the failure message is present.

### R1.9 Shellcheck Compliance

**WHEN** `scripts/doctor.sh` is validated,
**THE SYSTEM SHALL** pass shellcheck with no errors or warnings.

## R2. Pinned Toolchains

### R2.1 Rust Toolchain Pin

**WHEN** the repository is cloned,
**THE SYSTEM SHALL** include a `rust-toolchain.toml` file that pins one specific stable Rust version with rustfmt and clippy components.

**WHEN** rustup reads `rust-toolchain.toml`,
**THE SYSTEM SHALL** install Rust 1.87.0 (or a newer stable version if verified by `make check`) with edition 2024.

### R2.2 Node Version Pin

**WHEN** the repository is cloned,
**THE SYSTEM SHALL** include a `.nvmrc` file containing `24` (Node.js LTS through 2028-04-30).

**WHEN** nvm or node installers read `.nvmrc`,
**THE SYSTEM SHALL** install the latest Node 24.x patch version.

### R2.3 Python Version Pin

**WHEN** the repository is cloned,
**THE SYSTEM SHALL** include `ai-service/.python-version` containing `3.13` and `ai-service/pyproject.toml` specifying Python 3.13.

**WHEN** uv reads these files,
**THE SYSTEM SHALL** install Python 3.13.x and create a reproducible environment from `ai-service/uv.lock`.

### R2.4 Python Dependency Migration

**WHEN** migrating from venv to uv,
**THE SYSTEM SHALL** replace the hand-made venv and empty requirements.txt with:

- `pyproject.toml` listing dependencies: fastapi, uvicorn, requests, pydantic
- `pyproject.toml` listing dev dependencies: pytest, ruff
- Committed `uv.lock` file

### R2.5 Dashboard Reproducibility

**WHEN** the dashboard uses npm,
**THE SYSTEM SHALL** commit `package-lock.json`, add an `engines` field specifying Node 24.x, and ensure `npm ci` produces reproducible builds.

### R2.6 Bootstrap Idempotency

**WHEN** `scripts/bootstrap.sh` runs,
**THE SYSTEM SHALL** install the following dependencies idempotently (safe to run multiple times):

- System packages (apt): build-essential, pkg-config, libssl-dev, curl, git, zstd, shellcheck, jq
- Toolchains: rustup, Node.js, uv, gh (GitHub CLI)

### R2.7 Verified Versions Documentation

**WHEN** `docs/ENVIRONMENT.md` is read,
**THE SYSTEM SHALL** list the exact commands used by bootstrap.sh and the verified versions of all toolchains.

## R3. Local LLM Runtime

### R3.1 Ollama Systemd Service

**WHEN** Ollama is installed,
**THE SYSTEM SHALL** run as a systemd service inside WSL2 (either user service or system service).

### R3.2 Localhost Binding

**WHEN** Ollama is running,
**THE SYSTEM SHALL** listen only on 127.0.0.1:11434, verified by `ss -ltnp | grep 11434` showing no `0.0.0.0:11434` binding.

### R3.3 GPU Passthrough

**WHEN** an NVIDIA GPU is present on the Windows host,
**THE SYSTEM SHALL** rely only on the Windows driver for GPU access inside WSL2, verified by running `nvidia-smi` inside WSL.

**WHEN** `ollama ps` shows running models,
**THE SYSTEM SHALL** display the GPU model in the PROCESSOR column if GPU is being used.

### R3.4 CPU Fallback

**WHEN** no GPU is detected,
**THE SYSTEM SHALL** run Ollama inference on CPU, documented as the fallback behavior in `docs/ENVIRONMENT.md`.

### R3.5 LLM Smoke Test

**WHEN** `make llm-smoke` runs,
**THE SYSTEM SHALL**:

1. Send a fixed prompt to the model specified in OLLAMA_MODEL (default: phi3:mini)
2. Use JSON schema format for the response
3. Validate the response against the schema
4. Print tokens per second (calculated from eval_count and eval_duration)
5. Report whether inference ran on CPU or GPU

## R4. Repository Structure and Hygiene

### R4.1 Directory Layout

**WHEN** the repository is structured,
**THE SYSTEM SHALL** contain the following top-level directories: `.kiro/`, `.github/`, `core-engine/`, `ai-service/`, `dashboard/`, `docs/`, `scripts/`, `config/`.

### R4.2 Gitignore

**WHEN** files are tracked by git,
**THE SYSTEM SHALL** exclude the following via `.gitignore`:

- Build artifacts: `target/`, `__pycache__/`, `.venv/`, `node_modules/`, `dist/`
- Environment: `.env`, `config/local.env`
- Logs: `*.log`
- Caches: `.ruff_cache/`, `.pytest_cache/`

### R4.3 Editorconfig

**WHEN** an editor reads `.editorconfig`,
**THE SYSTEM SHALL** apply consistent formatting rules (indent style, line endings, charset).

### R4.4 License

**WHEN** the repository includes a LICENSE file,
**THE SYSTEM SHALL** use MIT license unless otherwise specified by the project owner.

### R4.5 Git Authentication

**WHEN** git credentials are configured,
**THE SYSTEM SHALL** use `gh auth login` and `gh auth setup-git` (or SSH keys) instead of `credential.helper store`.

**WHEN** the git remote URL is checked,
**THE SYSTEM SHALL** be a plain HTTPS URL with no embedded token.

### R4.6 Commit Convention

**WHEN** contributors commit changes,
**THE SYSTEM SHALL** follow conventional commit format: `<type>(<scope>): <description>`.

### R4.7 Push Local Commits

**WHEN** Phase 0 work is complete,
**THE SYSTEM SHALL** push all local commits to the remote repository.

## R5. Quality Gates

### R5.1 Makefile Targets

**WHEN** `make help` runs,
**THE SYSTEM SHALL** list all available targets with descriptions.

**WHEN** Makefile targets exist,
**THE SYSTEM SHALL** include: help, bootstrap, doctor, fmt, lint, test, check (lint + test + build), llm-smoke, clean, and run targets for each service (run-core, run-ai, run-dashboard).

### R5.2 Service Binding

**WHEN** run targets execute services,
**THE SYSTEM SHALL** bind all services to 127.0.0.1 only (never 0.0.0.0).

### R5.3 Rust Quality Checks

**WHEN** Rust code is validated,
**THE SYSTEM SHALL** run:

1. `cargo fmt --check` (formatting)
2. `cargo clippy --all-targets -- -D warnings` (linting, warnings as errors)
3. `cargo test` (testing)

### R5.4 Python Quality Checks

**WHEN** Python code is validated,
**THE SYSTEM SHALL** run:

1. `ruff format --check` (formatting)
2. `ruff check` (linting)
3. `pytest` (testing)

### R5.5 Web Quality Checks

**WHEN** web code is validated,
**THE SYSTEM SHALL** run:

1. `npm run lint` (linting)
2. `npm run build` (build verification)

### R5.6 Warning Fixes

**WHEN** existing code warnings are found (e.g., Sandbox.name unused field),
**THE SYSTEM SHALL** fix them properly (e.g., prefix with underscore or use field-level allow), not with blanket allow directives.

### R5.7 Rust Tests

**WHEN** Rust tests run,
**THE SYSTEM SHALL** include tests for:

1. `parse_meminfo_value()` with valid /proc/meminfo lines
2. `/proc/<pid>/stat` parsing including process names with spaces and parentheses

### R5.8 Python Tests

**WHEN** Python tests run,
**THE SYSTEM SHALL** include tests for:

1. `classify_command()` covering: rm -rf /, rm -rf ~/tmp, rm -r, dd to /dev, fork bomb, chmod -R 777 /, kill -9 1 vs normal PID, shutdown, curl | bash, harmless commands
2. `suggest_safer_alternative()` for known dangerous patterns
3. `extract_json()` with both fenced (```json) and unfenced output
4. `analyze_impact()` against real files under pytest's tmp_path

### R5.9 Pre-commit Hooks

**WHEN** `git commit` runs,
**THE SYSTEM SHALL** execute pre-commit hooks that:

1. Fix trailing whitespace and EOF newlines
2. Validate TOML, YAML, JSON syntax
3. Check for merge conflict markers
4. Run gitleaks (secret detection)
5. Run shellcheck on shell scripts
6. Run ruff format and check on Python
7. Run cargo fmt on Rust

### R5.10 CI Pipeline

**WHEN** code is pushed or a PR is created,
**THE SYSTEM SHALL** run GitHub Actions workflow `.github/workflows/ci.yml` with jobs:

1. `rust`: cargo fmt, clippy, test (exclude privileged tests)
2. `python`: ruff format check, ruff check, pytest
3. `web`: npm lint, npm build
4. `shellcheck`: validate all shell scripts
5. `gitleaks`: scan for secrets

**WHEN** CI uses third-party actions,
**THE SYSTEM SHALL** pin them to full commit SHAs, not tags or branches.

### R5.11 Dependency Updates

**WHEN** dependencies are tracked,
**THE SYSTEM SHALL** configure dependabot.yml for: cargo, pip, npm, github-actions with weekly update frequency.

### R5.12 Security Audits

**WHEN** CI security jobs run,
**THE SYSTEM SHALL** execute:

1. `cargo audit` (or `cargo deny`)
2. `pip-audit`
3. `npm audit`

**WHEN** high or critical vulnerabilities are found,
**THE SYSTEM SHALL** fail the CI build.

## R6. Security and Privilege Baseline

### R6.1 Security Documentation

**WHEN** `docs/SECURITY.md` is read,
**THE SYSTEM SHALL** document:

1. Assets: Linux system, user files, system processes
2. Actors: user, dashboard, ai-service, core-engine, kernel
3. Trust boundaries: browser ↔ dashboard ↔ ai-service ↔ core-engine ↔ kernel
4. Privilege model: only sandbox/process-control may be privileged; ai-service and dashboard run unprivileged

### R6.2 Localhost Binding

**WHEN** the core-engine HTTP server runs (future feature),
**THE SYSTEM SHALL** bind to 127.0.0.1:3000, not 0.0.0.0:3000.

### R6.3 CORS Policy

**WHEN** core-engine and ai-service respond to HTTP requests,
**THE SYSTEM SHALL** restrict CORS to `http://localhost:5173` and `http://127.0.0.1:5173` (no wildcard `*`).

### R6.4 Secret Generation

**WHEN** `scripts/bootstrap.sh` runs,
**THE SYSTEM SHALL** create `~/.config/intellishell/secret` containing a 32-byte hex value (`openssl rand -hex 32`) with file mode `0600`.

### R6.5 Secret Verification

**WHEN** `make doctor` verifies secrets,
**THE SYSTEM SHALL**:

1. Check that `~/.config/intellishell/secret` exists
2. Verify file mode is `0600`
3. Verify git does not track the secret file

### R6.6 Known Issues Documentation

**WHEN** `docs/SECURITY.md` lists known issues,
**THE SYSTEM SHALL** include the following with their phase owners:

1. `/execute` trusts caller's risk_level (Phase 5)
2. core-engine runs as root with no auth (Phase 5)
3. Sandbox shares host filesystem (Phase 3)
4. /proc mounts may leak to host namespace (Phase 3)
5. Signal safety issues (Phase 2)
6. Ollama has no authentication (Phase 4)
7. No audit logging (Phase 6)

**WHEN** known issues are documented,
**THE SYSTEM SHALL NOT** fix them in Phase 0; they are deferred to their respective phase owners.

## R7. Documentation

### R7.1 Steering Files

**WHEN** `.kiro/steering/` is inspected,
**THE SYSTEM SHALL** contain: product.md, tech.md, structure.md, security.md, workflow.md.

### R7.2 Specifications

**WHEN** `.kiro/specs/phase-0-foundation/` is inspected,
**THE SYSTEM SHALL** contain: requirements.md (this file, in EARS notation), design.md, tasks.md.

### R7.3 Architecture Documentation

**WHEN** `docs/ARCHITECTURE.md` is read,
**THE SYSTEM SHALL** include a Mermaid diagram showing:

1. Components: core-engine, ai-service, dashboard, Ollama
2. Data flow: user → dashboard → ai-service → Ollama; ai-service → core-engine
3. Ports: 3000 (core), 8000 (ai), 5173 (dashboard), 11434 (Ollama)
4. Binding: all on 127.0.0.1

### R7.4 Security Documentation

**WHEN** `docs/SECURITY.md` is read,
**THE SYSTEM SHALL** document assets, actors, trust boundaries, privilege model, and known issues with phase owners.

### R7.5 Phases Roadmap

**WHEN** `docs/PHASES.md` is read,
**THE SYSTEM SHALL** list phases: Phase 0 (Foundation), Phase 1-6 (future work with brief descriptions).

### R7.6 Architecture Decision Records

**WHEN** `docs/adr/` is inspected,
**THE SYSTEM SHALL** contain four ADRs:

1. 0001-record-decisions.md (use Markdown ADRs)
2. 0002-rust-core-engine.md (Rust for system-level work)
3. 0003-uv-python-package-manager.md (uv instead of pip)
4. 0004-rule-based-risk.md (pattern-based classification in Phase 0)

### R7.7 README

**WHEN** `README.md` is read,
**THE SYSTEM SHALL** include:

1. Project name and description
2. Current status: Phase 0 - Foundation
3. Quickstart: prerequisites, clone, bootstrap, doctor, check
4. Components table: core-engine (Rust 1.87.0, port 3000), ai-service (Python 3.13, port 8000), dashboard (Node 24, port 5173)
5. Development workflow: make targets, conventional commits, pre-commit hooks
6. Documentation links: ARCHITECTURE, SECURITY, ENVIRONMENT, PHASES

**WHEN** README prerequisites are listed,
**THE SYSTEM SHALL** clearly state: "Clone to /home/<user>/, not /mnt/*".

## Non-Functional Requirements

### NFR1. Reproducibility

**WHEN** a fresh Windows + WSL2 machine runs the documented setup,
**THE SYSTEM SHALL** produce a passing `make doctor` and passing `make check` using only documented commands.

### NFR2. No Placeholders

**WHEN** files are added,
**THE SYSTEM SHALL NOT** include TODO stubs, placeholder comments, mock endpoints, or empty smoke tests; every file must be used by a command in this phase.

### NFR3. No Feature Changes

**WHEN** Phase 0 work is performed,
**THE SYSTEM SHALL NOT** delete or rewrite existing features; the prototype code is verified and functional.

### NFR4. No Sudo Abuse

**WHEN** sudo is used,
**THE SYSTEM SHALL** only use it for: apt commands, systemd service management, and /etc/wsl.conf edits, all documented in docs/ENVIRONMENT.md.

### NFR5. No Windows Changes

**WHEN** Phase 0 setup is performed,
**THE SYSTEM SHALL NOT** modify `.wslconfig`, Windows drivers, firewall rules, or install NVIDIA drivers inside WSL.
