# Phase 0: Foundation — Task Checklist

This document provides a detailed task breakdown for implementing Phase 0 requirements.

## Task Overview

Phase 0 is organized into 8 feature groups (FEAT-001 through FEAT-008):

1. **FEAT-001**: Environment setup (repo location, WSL2 verification)
2. **FEAT-002**: Git setup (branch, tag, auth, commit conventions)
3. **FEAT-003**: Toolchain pins (rust-toolchain.toml, .nvmrc, uv migration)
4. **FEAT-004**: Bootstrap script and doctor script
5. **FEAT-005**: Quality gates (Makefile, pre-commit, CI)
6. **FEAT-006**: Tests (Rust, Python with real implementations)
7. **FEAT-007**: Security baseline (localhost bindings, CORS, secrets)
8. **FEAT-008**: Documentation (.kiro/steering, specs, docs/, ADRs, README)

---

## FEAT-001: Environment Setup

### Tasks

- [ ] Verify repository is under `/home/<user>/`, not `/mnt/*`
  - If on `/mnt/*`, document as blocker (user must move manually)
- [ ] Verify kernel version: `uname -r` >= 5.15
  - Command: `uname -r`
  - Expected: `6.18.33.2` or similar >= 5.15
- [ ] Verify WSL2: `/proc/version` contains "microsoft"
  - Command: `cat /proc/version | grep -i microsoft`
- [ ] Verify systemd running
  - Command: `systemctl --version` and check PID 1
- [ ] Verify cgroup v2
  - Command: `stat -fc %T /sys/fs/cgroup` should return `cgroup2fs`
- [ ] Verify cgroup controllers
  - Command: `cat /sys/fs/cgroup/cgroup.controllers` should list cpu, memory, pids

### Verification

```bash
uname -r
cat /proc/version
systemctl --version
stat -fc %T /sys/fs/cgroup
cat /sys/fs/cgroup/cgroup.controllers
pwd | grep -E '^/home/'
```

---

## FEAT-002: Git Setup

### Tasks

- [ ] Tag current HEAD as `prototype-v0`
  - Command: `git tag prototype-v0`
- [ ] Create branch `phase/0-foundation`
  - Command: `git checkout -b phase/0-foundation`
- [ ] Remove credential.helper store if present
  - Command: `git config --global --unset credential.helper` (if set to store)
- [ ] Configure gh auth
  - Command: `gh auth login` and `gh auth setup-git`
  - Alternative: SSH key setup
- [ ] Verify remote URL is HTTPS (no embedded token)
  - Command: `git remote get-url origin`
- [ ] Document conventional commit format in docs/

### Verification

```bash
git tag | grep prototype-v0
git branch | grep phase/0-foundation
git config --get credential.helper  # should not be 'store'
gh auth status
git remote get-url origin  # should be https://github.com/... with no token
```

---

## FEAT-003: Toolchain Pins

### Tasks

#### Rust
- [ ] Create `rust-toolchain.toml`
  - Channel: stable
  - Version: 1.87.0
  - Components: rustfmt, clippy
  - Edition: Keep 2024 in Cargo.toml
- [ ] Verify Rust version
  - Command: `rustc --version` should show 1.87.0

#### Node
- [ ] Create `.nvmrc` with content `24`
- [ ] Add `engines` field to `dashboard/package.json`: `"engines": {"node": "24.x"}`
- [ ] Commit `dashboard/package-lock.json`
- [ ] Verify Node version
  - Command: `node --version` should show v24.x.x

#### Python
- [ ] Create `ai-service/.python-version` with content `3.13`
- [ ] Create `ai-service/pyproject.toml` with:
  - Dependencies: fastapi, uvicorn, requests, pydantic
  - Dev dependencies: pytest, ruff
- [ ] Run `uv sync` in ai-service/ to generate uv.lock
- [ ] Delete old `ai-service/requirements.txt` and `.venv/`
- [ ] Commit `ai-service/uv.lock`
- [ ] Verify Python version
  - Command: `python3 --version` should show Python 3.13.x

### Verification

```bash
rustc --version  # 1.87.0
cargo --version
node --version   # v24.x.x
npm --version
python3 --version  # Python 3.13.x
uv --version
test -f rust-toolchain.toml
test -f .nvmrc
test -f ai-service/.python-version
test -f ai-service/uv.lock
```

---

## FEAT-004: Bootstrap and Doctor Scripts

### Tasks

#### Bootstrap Script
- [ ] Create `scripts/bootstrap.sh`
- [ ] Make idempotent (check before installing)
- [ ] Install system packages:
  - build-essential, pkg-config, libssl-dev, curl, git, zstd, shellcheck, jq
- [ ] Install rustup (if not present)
- [ ] Install Node.js (via nvm or direct, if not present)
- [ ] Install uv (if not present)
- [ ] Install gh CLI (if not present)
- [ ] Generate `~/.config/intellishell/secret` (if not present)
  - Command: `openssl rand -hex 32 > ~/.config/intellishell/secret`
  - Mode: 0600
- [ ] Install pre-commit hooks
  - Command: `pre-commit install`
- [ ] Make script executable: `chmod +x scripts/bootstrap.sh`

#### Doctor Script
- [ ] Create `scripts/doctor.sh`
- [ ] Implement FAIL checks:
  - WSL2 detected
  - Kernel >= 5.15
  - Systemd running
  - Cgroup v2 enabled
  - Cgroup controllers (cpu, memory, pids)
  - Overlayfs in /proc/filesystems
  - /proc health (readlink /proc/self/exe, read /proc/mounts)
  - Repo not on drvfs (not under /mnt/*)
  - Free disk >= 5 GB
  - Git identity set
  - Toolchain versions match pins
- [ ] Implement WARN checks:
  - Available RAM < 4 GB
  - Free disk < 20 GB
  - Port conflicts (3000, 8000, 5173, 11434)
  - Windows Ollama competing for 11434
  - No GPU (CPU-only inference)
  - Non-loopback bindings
- [ ] Support `--json` flag for structured output
- [ ] Include remediation messages for each failure
- [ ] Add `--self-test` mode
- [ ] Pass shellcheck: `shellcheck -x scripts/doctor.sh`
- [ ] Make script executable: `chmod +x scripts/doctor.sh`

#### LLM Smoke Test
- [ ] Create `scripts/llm-smoke.sh`
- [ ] Read OLLAMA_MODEL from config (default: phi3:mini)
- [ ] Send prompt with JSON schema format
- [ ] Validate response against schema
- [ ] Calculate and print tokens/sec
- [ ] Report CPU vs GPU inference
- [ ] Make script executable: `chmod +x scripts/llm-smoke.sh`

### Verification

```bash
./scripts/bootstrap.sh  # Should complete without errors
./scripts/doctor.sh     # All PASS or WARN, no FAIL
./scripts/doctor.sh --json | jq .  # Valid JSON
./scripts/doctor.sh --self-test  # Exit code non-zero
shellcheck scripts/*.sh  # No errors
make doctor  # Calls doctor.sh
make llm-smoke  # Tests Ollama
```

---

## FEAT-005: Quality Gates

### Tasks

#### Makefile
- [ ] Create Makefile in repo root
- [ ] Add target: `help` (list all targets)
- [ ] Add target: `bootstrap` (run scripts/bootstrap.sh)
- [ ] Add target: `doctor` (run scripts/doctor.sh)
- [ ] Add target: `fmt` (format all: cargo fmt, ruff format, prettier)
- [ ] Add target: `lint` (lint all: clippy, ruff check, eslint)
- [ ] Add target: `test` (test all: cargo test, pytest, vitest)
- [ ] Add target: `build` (build all: cargo build, vite build)
- [ ] Add target: `check` (lint + test + build)
- [ ] Add target: `clean` (remove target/, dist/, etc.)
- [ ] Add target: `llm-smoke` (run scripts/llm-smoke.sh)
- [ ] Add target: `run-core` (run core-engine, bind 127.0.0.1:3000)
- [ ] Add target: `run-ai` (run ai-service, bind 127.0.0.1:8000)
- [ ] Add target: `run-dashboard` (run dashboard, bind 127.0.0.1:5173)

#### Pre-commit Hooks
- [ ] Create `.pre-commit-config.yaml`
- [ ] Add hooks:
  - trailing-whitespace
  - end-of-file-fixer
  - check-toml
  - check-yaml
  - check-json
  - check-merge-conflict
  - gitleaks
  - shellcheck
  - ruff (format + check)
  - cargo fmt (local hook)
- [ ] Install hooks: `pre-commit install`
- [ ] Test hooks: Make a commit and verify they run

#### CI Pipeline
- [ ] Create `.github/workflows/ci.yml`
- [ ] Add jobs:
  - rust (fmt, clippy, test)
  - python (ruff format check, ruff check, pytest)
  - web (npm lint, npm build)
  - shellcheck
  - gitleaks
  - audit-rust (cargo audit)
  - audit-python (pip-audit)
  - audit-web (npm audit)
- [ ] Pin third-party actions to full commit SHAs
- [ ] Enable caching for cargo, uv, npm
- [ ] Exclude privileged tests from CI

#### Dependabot
- [ ] Create `.github/dependabot.yml`
- [ ] Configure ecosystems: cargo, pip, npm, github-actions
- [ ] Set schedule: weekly

### Verification

```bash
make help  # Lists all targets
make fmt   # Formats code
make lint  # Lints code (no errors)
make test  # Runs tests (all pass)
make check # Full check passes
git commit -m "test" --allow-empty  # Pre-commit hooks run
# Push and verify CI passes
```

---

## FEAT-006: Tests

### Tasks

#### Rust Tests (core-engine)
- [ ] Extract `parse_meminfo_value()` if needed (currently inline)
- [ ] Write test: valid /proc/meminfo line
- [ ] Write test: invalid line returns None
- [ ] Extract `/proc/stat` parsing (handle spaces and parens)
- [ ] Write test: process name with spaces
- [ ] Write test: process name with parentheses
- [ ] Run: `cargo test` (all pass)

#### Python Tests (ai-service)
- [ ] Implement `classify_command()` in `src/classifier.py`
- [ ] Write tests for classify_command:
  - `rm -rf /` → critical
  - `rm -rf ~/tmp` → high
  - `rm -r somedir` → medium
  - `dd if=/dev/zero of=/dev/sda` → critical
  - `:(){ :|:& };:` (fork bomb) → critical
  - `chmod -R 777 /` → high
  - `kill -9 1` → high
  - `kill -9 <pid>` → medium
  - `shutdown -h now` → medium
  - `curl ... | bash` → high
  - `ls -la` → low
  - `cat file.txt` → low
- [ ] Implement `suggest_safer_alternative()` in `src/classifier.py`
- [ ] Write tests for suggest_safer_alternative
- [ ] Implement `extract_json()` in `src/analyzer.py`
- [ ] Write tests for extract_json (fenced and unfenced)
- [ ] Implement `analyze_impact()` in `src/analyzer.py`
- [ ] Write tests for analyze_impact (use tmp_path)
- [ ] Run: `pytest` (all pass)

#### Dashboard Tests
- [ ] (Optional for Phase 0, or add placeholder test suite)

### Verification

```bash
cd core-engine && cargo test
cd ai-service && uv run pytest
cd dashboard && npm run test  # If tests exist
make test  # All tests pass
```

---

## FEAT-007: Security Baseline

### Tasks

#### Localhost Binding
- [ ] Core-engine: Ensure bind to 127.0.0.1:3000 (if HTTP server exists)
- [ ] AI-service: Run uvicorn with `--host 127.0.0.1 --port 8000`
- [ ] Dashboard: Vite dev server with `--host 127.0.0.1 --port 5173`
- [ ] Update Makefile run-* targets with localhost binds

#### CORS Policy
- [ ] Core-engine: Set allowed_origins to localhost:5173 and 127.0.0.1:5173
- [ ] AI-service: Add CORS middleware with same origins
- [ ] No wildcard `*` origins

#### Secret Management
- [ ] Bootstrap script creates `~/.config/intellishell/secret`
- [ ] Secret mode: 0600
- [ ] Doctor script verifies secret exists, mode is 0600, not tracked by git

#### Repository Hygiene
- [ ] Verify `.gitignore` excludes: .env, config/local.env, *.log, etc.
- [ ] Verify no secrets in git history: `git log -p | grep -i 'password\|secret\|token'`
- [ ] Run gitleaks: `gitleaks detect --source . --no-git`

### Verification

```bash
make doctor  # Verifies secret management
ss -ltnp | grep -E '3000|8000|5173|11434'  # All show 127.0.0.1
git log -p | grep -iE 'password|secret|api.*key|token' | wc -l  # Should be 0 or only references to example/docs
gitleaks detect --source . --no-git  # No findings
```

---

## FEAT-008: Documentation

### Tasks

#### .kiro/steering/
- [ ] Create `product.md`: Project name, purpose, users, safety philosophy
- [ ] Create `tech.md`: Stack, toolchain pins, architecture principles
- [ ] Create `structure.md`: Directories, conventions, make targets
- [ ] Create `security.md`: Security rules, localhost bindings, no sudo abuse
- [ ] Create `workflow.md`: Dev workflow, branching, commits, CI

#### .kiro/specs/phase-0-foundation/
- [ ] Create `requirements.md`: EARS notation requirements (R1-R7)
- [ ] Create `design.md`: Design decisions, verification strategy
- [ ] Create `tasks.md`: This file (task checklist)

#### docs/
- [ ] Create `ARCHITECTURE.md`: Components, data flow, Mermaid diagram
- [ ] Update `SECURITY.md`: Assets, actors, trust boundaries, privilege model, known issues table
- [ ] Create `PHASES.md`: Phase 0-6 roadmap

#### docs/adr/
- [ ] Create `0001-record-decisions.md`: ADR format decision
- [ ] Create `0002-rust-core-engine.md`: Why Rust for core-engine
- [ ] Create `0003-uv-python-package-manager.md`: Why uv over pip
- [ ] Create `0004-rule-based-risk.md`: Rule-based classification in Phase 0

#### Root
- [ ] Create `README.md`: Overview, quickstart, components, development, docs links
- [ ] Ensure README mentions: "Repo must be in /home/<user>/, not /mnt/*"

### Verification

```bash
ls -la .kiro/steering/ | grep -E 'product|tech|structure|security|workflow'
ls -la .kiro/specs/phase-0-foundation/ | grep -E 'requirements|design|tasks'
ls -la docs/ | grep -E 'ARCHITECTURE|SECURITY|PHASES|ENVIRONMENT'
ls docs/adr/ | grep -E '0001|0002|0003|0004'
test -f README.md
grep '/home/' README.md
grep -v '/mnt/' README.md || true
```

---

## Final Integration Tasks

### Tasks

- [ ] Commit all changes with conventional commits
  - `git commit -m "chore: add rust-toolchain pin"`
  - `git commit -m "feat(scripts): add bootstrap and doctor scripts"`
  - `git commit -m "docs: complete documentation foundation"`
- [ ] Run full check: `make check`
- [ ] Run doctor: `make doctor`
- [ ] Run LLM smoke test: `make llm-smoke` (if Ollama installed)
- [ ] Push branch: `git push -u origin phase/0-foundation`
- [ ] Verify CI passes on GitHub

### Verification

```bash
git status  # No untracked files, no uncommitted changes
make check  # All components pass
make doctor  # All PASS or WARN
git log --oneline | head -20  # Conventional commit messages
git branch -vv | grep phase/0-foundation  # Pushed to remote
# Check GitHub Actions: all jobs green
```

---

## Summary

Phase 0 is complete when:

1. ✅ Repository is on /home/ (or documented as blocker)
2. ✅ All toolchains pinned (rust-toolchain.toml, .nvmrc, .python-version)
3. ✅ Bootstrap script installs dependencies idempotently
4. ✅ Doctor script reports all PASS or WARN (no FAIL)
5. ✅ Makefile provides all essential targets
6. ✅ Pre-commit hooks installed and working
7. ✅ CI pipeline runs on push and PR
8. ✅ Tests pass for Rust and Python
9. ✅ Services bind localhost only, CORS restricted
10. ✅ Documentation complete (.kiro/, docs/, README.md)
11. ✅ All work committed with conventional commits
12. ✅ Phase branch pushed to remote
13. ✅ CI green on GitHub

**Next**: Merge `phase/0-foundation` to `main` when all acceptance criteria met.
