# Phase 0: Foundation — Design Document

This document describes the design decisions and implementation approach for Phase 0 requirements.

## Overview

Phase 0 establishes a reproducible development environment, toolchain, and quality infrastructure. It does not add new features or change product behavior. The goal is to transition from a prototype with unpushed commits to a production-ready foundation that can be cloned and built on any compatible machine.

## Design Principles

1. **Reproducibility first**: Every dependency pinned, every step documented
2. **Fail fast**: Environment issues caught by `make doctor` before wasting developer time
3. **Idempotent operations**: Scripts safe to run multiple times
4. **Platform constraints respected**: No workarounds that break on other machines
5. **Security by default**: Localhost-only bindings, secret management, no credential leaks

## Component Designs

### Environment Verification (R1)

**Purpose**: Catch environment issues early with actionable remediation messages.

**Design decisions**:

- **Bash script with functions**: Each check is a separate function returning 0 (pass), 1 (warn), or 2 (fail)
- **Structured output**: Both human-readable and JSON formats supported via `--json` flag
- **Configuration-driven**: Ports and paths read from `config/default.env` with overrides from `config/local.env`
- **Self-test**: Include a `--self-test` mode that deliberately breaks a check to verify error handling

**FAIL vs WARN distinction**:
- **FAIL**: Blocker that will cause build/runtime failures (missing cgroup, wrong filesystem, kernel too old)
- **WARN**: Best practice violation or suboptimal configuration (low RAM, CPU-only inference)

**Key checks**:

1. **WSL2 detection**: `grep -q microsoft /proc/version`
2. **Kernel version**: Parse `uname -r` and compare against 5.15.0
3. **Systemd**: `systemctl --version` succeeds and PID 1 is systemd
4. **Cgroup v2**: `stat -fc %T /sys/fs/cgroup` outputs `cgroup2fs`
5. **Controllers**: `cat /sys/fs/cgroup/cgroup.controllers` includes cpu, memory, pids
6. **Overlayfs**: `grep -q overlay /proc/filesystems`
7. **/proc health**: Test `readlink /proc/self/exe` and `cat /proc/mounts`
8. **Filesystem**: Use `stat -f -c %T .` to detect drvfs/9p; repo must be on ext4 or similar native FS
9. **Disk space**: `df -BG . | tail -1 | awk '{print $4}'` >= 5
10. **Git identity**: `git config user.name` and `git config user.email` are set
11. **Toolchain versions**: Parse version strings and compare against pins

**Shellcheck compliance**: Script passes `shellcheck -x scripts/doctor.sh` with no errors.

### Pinned Toolchains (R2)

**Purpose**: Eliminate "works on my machine" problems by pinning every toolchain version.

**Design decisions**:

#### Rust

- **rust-toolchain.toml**: Specifies channel (stable), version (1.87.0), components (rustfmt, clippy)
- **Edition 2024**: Keep the current edition for latest language features
- **Rationale**: rustup reads this file automatically; zero configuration for developers

#### Node

- **`.nvmrc`**: Contains `24` (LTS major version)
- **Engines field in package.json**: `"engines": {"node": "24.x"}` enforces version in CI
- **Rationale**: Node 24 is LTS until 2028-04-30; security patches through 2026-07-27

#### Python

- **uv migration**: Replace venv + requirements.txt with uv for speed and reproducibility
- **`.python-version`**: Contains `3.13`
- **`pyproject.toml`**: Defines dependencies using PEP 621 standard
- **`uv.lock`**: Committed lock file with exact versions and hashes
- **Rationale**: uv is 10-100x faster than pip; lock file ensures identical installs across machines

**Migration steps**:
1. Create `pyproject.toml` with current dependencies (fastapi, uvicorn, requests, pydantic)
2. Add dev dependencies (pytest, ruff)
3. Run `uv sync` to generate `uv.lock`
4. Delete `requirements.txt` and `.venv/`
5. Update Makefile and CI to use `uv run` instead of `python -m`

#### Dashboard

- **`package-lock.json`**: Committed to repo for reproducible `npm ci`
- **`engines` field**: Enforce Node 24.x in package.json
- **Rationale**: `npm ci` installs exact versions from lock file; faster and safer than `npm install`

#### Bootstrap Script

- **Idempotent**: Check for existing installations before installing
- **System packages**: Install via `apt-get` with `-y` flag for non-interactive
- **Toolchains**:
  - rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`
  - Node: Download and install from NodeSource or use nvm
  - uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
  - gh: `(type -p wget >/dev/null || (sudo apt update && sudo apt-get install wget -y)) && sudo mkdir -p -m 755 /etc/apt/keyrings && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null && sudo apt update && sudo apt install gh -y`

**Secret generation**:
```bash
mkdir -p ~/.config/intellishell
if [ ! -f ~/.config/intellishell/secret ]; then
  openssl rand -hex 32 > ~/.config/intellishell/secret
  chmod 0600 ~/.config/intellishell/secret
fi
```

### Local LLM Runtime (R3)

**Purpose**: Provide local, private AI inference for command analysis.

**Design decisions**:

#### Ollama Installation

- **Systemd service**: Ollama's official installer sets up a systemd service automatically
- **User service preferred**: `systemctl --user` avoids root for service management
- **Localhost binding**: Verify with `ss -ltnp | grep 11434` that it's `127.0.0.1:11434`, not `0.0.0.0:11434`

#### GPU Passthrough

- **Windows driver only**: Rely on NVIDIA driver installed on Windows host
- **No WSL driver**: Do NOT install NVIDIA drivers or CUDA inside WSL (causes conflicts)
- **Verification**: `nvidia-smi` inside WSL should work if GPU passthrough is functional
- **Processor detection**: `ollama ps` shows GPU model in PROCESSOR column when active

#### CPU Fallback

- **Automatic**: Ollama falls back to CPU if no GPU is detected
- **Performance**: Document that CPU is 10-50x slower than GPU
- **Detection**: PROCESSOR column in `ollama ps` is empty or shows "CPU"

#### LLM Smoke Test

**Script**: `scripts/llm-smoke.sh`

**Flow**:
1. Read OLLAMA_MODEL from config/default.env (default: phi3:mini)
2. Construct a JSON schema for a simple classification task
3. Send prompt to `http://127.0.0.1:11434/api/generate` with `format` parameter
4. Parse response JSON
5. Validate response against schema (use jq or Python)
6. Extract `eval_count` and `eval_duration` from response
7. Calculate tokens/sec: `eval_count / (eval_duration / 1e9)`
8. Report speed and processor type

**Example JSON schema**:
```json
{
  "type": "object",
  "properties": {
    "risk_level": {"type": "string", "enum": ["low", "medium", "high"]},
    "reason": {"type": "string"}
  },
  "required": ["risk_level", "reason"]
}
```

### Repository Structure (R4)

**Purpose**: Organize code and documentation for easy navigation and contribution.

**Design decisions**:

#### Directory Layout

- **`.kiro/`**: Project steering (product, tech, security, workflow, structure) and specs (requirements, design, tasks per phase)
- **`.github/`**: CI workflows and dependabot config
- **`core-engine/`**: Rust binary crate (Cargo.toml, src/, tests/)
- **`ai-service/`**: Python ASGI app (pyproject.toml, src/, tests/)
- **`dashboard/`**: React SPA (package.json, src/, public/)
- **`docs/`**: Architecture, environment, security, phases, ADRs
- **`scripts/`**: Bash scripts for bootstrap, doctor, llm-smoke
- **`config/`**: default.env (committed), local.env (git-ignored)

#### Gitignore

**Committed artifacts**:
- Lock files: Cargo.lock, uv.lock, package-lock.json
- Pin files: rust-toolchain.toml, .nvmrc, .python-version

**Ignored artifacts**:
- Build outputs: target/, dist/, __pycache__/, node_modules/
- Environment: .env, config/local.env
- Logs: *.log
- Caches: .ruff_cache/, .pytest_cache/, .vite/

#### Editorconfig

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 2

[*.rs]
indent_size = 4

[*.py]
indent_size = 4

[*.md]
trim_trailing_whitespace = false

[Makefile]
indent_style = tab
```

#### License

- **MIT**: Permissive, widely compatible, simple
- **Copyright**: Assign to project owner

#### Git Authentication

- **Preferred**: `gh auth login` (OAuth device flow) + `gh auth setup-git` (credential helper)
- **Alternative**: SSH key (`ssh-keygen -t ed25519 -C "email@example.com"` + add to GitHub)
- **Forbidden**: `credential.helper store` (stores plaintext tokens in ~/.git-credentials)

### Quality Gates (R5)

**Purpose**: Catch bugs, style issues, and regressions before they reach production.

**Design decisions**:

#### Makefile Design

- **Phony targets**: All targets are `.PHONY` since they don't produce files
- **Parallel execution**: Use `&&` for sequential deps, `&` for parallel where safe
- **Error propagation**: Use `set -e` in multi-command targets
- **Help target**: Parse comments in Makefile to generate usage text

**Example Makefile structure**:
```make
.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: check
check: lint test build ## Run all checks (lint, test, build)

.PHONY: lint
lint: ## Lint all components
	cd core-engine && cargo clippy --all-targets -- -D warnings
	cd ai-service && uv run ruff check
	cd dashboard && npm run lint

.PHONY: test
test: ## Run tests in all components
	cd core-engine && cargo test
	cd ai-service && uv run pytest
	cd dashboard && npm run test
```

#### Pre-commit Hooks

**Configuration**: `.pre-commit-config.yaml`

**Hooks**:
1. **Trailing whitespace**: `trailing-whitespace` (built-in)
2. **EOF newlines**: `end-of-file-fixer` (built-in)
3. **TOML/YAML/JSON**: `check-toml`, `check-yaml`, `check-json` (built-in)
4. **Merge conflicts**: `check-merge-conflict` (built-in)
5. **gitleaks**: Secret scanning (third-party hook)
6. **shellcheck**: Shell script linting (third-party or local)
7. **ruff**: Python formatting and linting (third-party hook)
8. **cargo fmt**: Rust formatting (local hook: `cargo fmt -- --check`)

**Installation**: `pre-commit install` (run by bootstrap.sh)

#### CI Pipeline

**Workflow file**: `.github/workflows/ci.yml`

**Triggers**: `on: [push, pull_request]`

**Jobs**:

1. **rust**:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test` (exclude privileged tests with `#[cfg(not(ci))]` or `#[ignore]`)
   - Cache: `~/.cargo`, `target/`

2. **python**:
   - `uv sync`
   - `uv run ruff format --check`
   - `uv run ruff check`
   - `uv run pytest`
   - Cache: `~/.cache/uv`

3. **web**:
   - `npm ci`
   - `npm run lint`
   - `npm run build`
   - Cache: `~/.npm`

4. **shellcheck**:
   - `shellcheck scripts/*.sh`

5. **gitleaks**:
   - `gitleaks detect --source . --no-git`

6. **audit-rust**:
   - `cargo audit --deny high --deny critical`

7. **audit-python**:
   - `uv run pip-audit`

8. **audit-web**:
   - `npm audit --audit-level high`

**Action pinning**: Use full commit SHA for security:
```yaml
- uses: actions/checkout@8f4b7f84864f9b28f5d9d8b1e9c0f2a3b4c5d6e7
```

#### Dependabot

**Configuration**: `.github/dependabot.yml`

**Ecosystems**: cargo, pip, npm, github-actions

**Schedule**: Weekly updates

**Auto-merge strategy**: Dependabot creates PRs; CI must pass before merge

#### Test Design

**Rust tests** (core-engine):

- **Meminfo parsing**:
  ```rust
  #[test]
  fn test_parse_meminfo_value() {
      assert_eq!(parse_meminfo_value("MemTotal:       16384000 kB"), Some(16384000));
      assert_eq!(parse_meminfo_value("MemFree:        8192000 kB"), Some(8192000));
      assert_eq!(parse_meminfo_value("Invalid line"), None);
  }
  ```

- **/proc/stat parsing**:
  ```rust
  #[test]
  fn test_parse_proc_stat_with_spaces() {
      let line = "1234 (name with spaces) S 1 1234 1234 0 -1 4194304 100 0 0 0 10 5 0 0 20 0 1 0 12345 1024000 100 18446744073709551615";
      let parsed = parse_proc_stat(line).unwrap();
      assert_eq!(parsed.comm, "name with spaces");
  }

  #[test]
  fn test_parse_proc_stat_with_parens() {
      let line = "5678 ((nested) (parens)) R 1 5678 5678 0 -1 4194304 200 0 0 0 20 10 0 0 20 0 1 0 54321 2048000 200 18446744073709551615";
      let parsed = parse_proc_stat(line).unwrap();
      assert_eq!(parsed.comm, "(nested) (parens)");
  }
  ```

**Python tests** (ai-service):

- **classify_command**:
  ```python
  def test_classify_command_critical():
      assert classify_command("rm -rf /") == "critical"
      assert classify_command("dd if=/dev/zero of=/dev/sda") == "critical"
      assert classify_command(":(){ :|:& };:") == "critical"

  def test_classify_command_high():
      assert classify_command("rm -rf ~/tmp") == "high"
      assert classify_command("chmod -R 777 /") == "high"
      assert classify_command("kill -9 1") == "high"

  def test_classify_command_medium():
      assert classify_command("rm -r somedir") == "medium"
      assert classify_command("shutdown -h now") == "medium"

  def test_classify_command_low():
      assert classify_command("ls -la") == "low"
      assert classify_command("cat file.txt") == "low"
  ```

- **suggest_safer_alternative**:
  ```python
  def test_suggest_safer_rm():
      alt = suggest_safer_alternative("rm -rf /")
      assert "trash" in alt or "specific path" in alt
  ```

- **extract_json**:
  ```python
  def test_extract_json_fenced():
      response = '```json\n{"key": "value"}\n```'
      assert extract_json(response) == {"key": "value"}

  def test_extract_json_unfenced():
      response = '{"key": "value"}'
      assert extract_json(response) == {"key": "value"}
  ```

- **analyze_impact**:
  ```python
  def test_analyze_impact_with_files(tmp_path):
      (tmp_path / "file.txt").write_text("content")
      impact = analyze_impact("rm file.txt", tmp_path)
      assert "1 file" in impact["message"]
  ```

### Security Baseline (R6)

**Purpose**: Establish security defaults and document known issues with phase owners.

**Design decisions**:

#### Localhost Binding

**Current state**: Prototype likely has `0.0.0.0` bindings

**Changes**:
- core-engine: Change bind address to `127.0.0.1:3000` (when HTTP server is added in future phase)
- ai-service: Ensure uvicorn runs with `--host 127.0.0.1`
- dashboard: Vite dev server uses `--host 127.0.0.1`

#### CORS Policy

**Current state**: Prototype may allow `*` origin

**Changes**:
- core-engine: `allowed_origins = ["http://localhost:5173", "http://127.0.0.1:5173"]`
- ai-service: Same allowed origins in FastAPI CORS middleware

#### Secret Management

**Generation**: bootstrap.sh creates `~/.config/intellishell/secret`

**Verification**: doctor.sh checks:
1. File exists
2. Mode is `0600` (use `stat -c %a`)
3. Git does not track it (check `.git/info/exclude` or global `.gitignore`)

**Usage**: Future phases use secret for signing, encryption, or authentication

#### Known Issues Table

**Format** (in SECURITY.md):

| Issue | Description | Risk | Phase Owner |
|-------|-------------|------|-------------|
| `/execute` trusts `risk_level` | Caller provides risk level; no server-side validation | High | Phase 5 |
| Core-engine runs as root with no auth | Any process on localhost can send commands | High | Phase 5 |
| Sandbox shares host filesystem | No mount namespace isolation yet | Medium | Phase 3 |
| `/proc` mounts may leak | Host /proc visible in sandbox | Medium | Phase 3 |
| Signal safety issues | Signal handlers not async-signal-safe | Low | Phase 2 |
| Ollama has no authentication | Anyone on localhost can use LLM | Low | Phase 4 |
| No audit logging | No record of who executed what | Medium | Phase 6 |

**Importance**: These issues are *documented* but *not fixed* in Phase 0. Each has a designated future phase responsible for remediation.

## Non-Functional Design

### Reproducibility

**Goal**: Fresh Windows + WSL2 machine goes from clone to passing checks

**Approach**:
1. Pin all toolchain versions
2. Commit all lock files
3. Document every setup step
4. Test on a clean VM or container

### Idempotency

**Goal**: Scripts safe to run multiple times

**Approach**:
- Check before install: `command -v cargo || install_rust`
- Skip if already configured: `[ -f ~/.config/intellishell/secret ] || generate_secret`
- Use `apt-get install -y` with explicit package names

### Fail Fast

**Goal**: Detect environment issues before wasting developer time

**Approach**:
- `make doctor` runs before any build or test
- CI runs doctor as first step
- Clear remediation messages for each failure

## Verification Strategy

After implementing all requirements:

1. **Fresh clone**: Delete local repo, clone again, verify all files present
2. **Bootstrap**: Run `./scripts/bootstrap.sh`, verify no errors
3. **Doctor**: Run `make doctor`, verify all PASS or WARN (no FAIL)
4. **Build**: Run `make check`, verify all components build, lint, and test
5. **LLM smoke**: Run `make llm-smoke`, verify prompt/response cycle
6. **Pre-commit**: Run `git commit` with a test change, verify hooks execute
7. **CI**: Push branch, verify all CI jobs pass

## Future Phase Integration

Phase 0 provides the foundation for:

- **Phase 1**: TBD
- **Phase 2**: Signal safety (fix signal handlers)
- **Phase 3**: Filesystem isolation (mount namespaces)
- **Phase 4**: LLM integration (advanced classification, context-aware suggestions)
- **Phase 5**: Authentication and authorization (secure /execute endpoint)
- **Phase 6**: Audit logging (who ran what, when, result)

Each future phase assumes Phase 0 is complete: reproducible environment, passing tests, working CI.
