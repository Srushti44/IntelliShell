# IntelliShell Phase 0 Foundation — Implementation Plan

## Critical Prerequisites

**REPOSITORY LOCATION ISSUE**: The repo is currently on `/mnt/d/3rd year/os project/IntelliShell` (a 9p/drvfs filesystem). R1 requires the repo to be under `/home/<user>/`, never `/mnt/*`. This is a manual prerequisite — the user must clone or move the repo to `/home/<user>/IntelliShell` before implementation begins. Doctor script will FAIL this check until fixed.

## Overview

Phase 0 establishes reproducible environment from clone to passing doctor, lint, test, build, and CI. Three components: core-engine (Rust CLI), ai-service (Python FastAPI), dashboard (React/Vite). Work on branch `phase/0-foundation` after tagging current HEAD as `prototype-v0`.

## Key Findings from Exploration

- **Rust**: core-engine/Cargo.toml has `edition = "2024"` (must fix to "2021"), Sandbox.name field unused (warning), parse_meminfo_value() and read_cpu_ticks() exist with /proc parsing logic, NO HTTP server (CLI monitor only, no 0.0.0.0 binding to fix)
- **Python**: ai-service/ has ONLY empty requirements.txt — needs complete implementation from scratch
- **Node**: dashboard/package.json has React 19 + Vite 8, no package-lock.json
- **Git**: remote is HTTPS (correct: https://github.com/Srushti44/IntelliShell.git), has unpushed local commits
- **Structure**: missing .kiro/, .github/, docs/, scripts/, config/, LICENSE, README, .editorconfig, rust-toolchain.toml, .nvmrc, pre-commit config

---

## Implementation Plan

### FEAT-001: Repository Structure and Git Baseline

- [ ] 1. **Tag prototype and create branch**
      Current HEAD becomes `prototype-v0` tag. Create and checkout branch `phase/0-foundation`.
      Files: .git/refs/tags/prototype-v0, .git/refs/heads/phase/0-foundation
      Verify: `git tag -l | grep prototype-v0 && git branch --show-current` returns phase/0-foundation

- [ ] 2. **Create directory structure**
      mkdir -p .kiro/steering .kiro/specs/phase-0-foundation .github/workflows docs/adr scripts config
      Files: .kiro/, .github/, docs/, scripts/, config/ directories
      Verify: `ls -d .kiro .github docs scripts config` all exist

- [ ] 3. **Complete .gitignore**
      Add: target/, __pycache__/, .venv/, node_modules/, dist/, .env, config/local.env, *.log, .ruff_cache/, .pytest_cache/, *.pyc, .DS_Store (uv.lock should be committed, not ignored)
      Files: .gitignore
      Verify: `grep -E 'target/|__pycache__|node_modules|config/local.env' .gitignore`

- [ ] 4. **Create .editorconfig**
      Standard rules: root=true, [*] indent_style=space, indent_size=2, end_of_line=lf, charset=utf-8, trim_trailing_whitespace=true, insert_final_newline=true, [*.{rs,toml}] indent_size=4, [*.md] trim_trailing_whitespace=false, [Makefile] indent_style=tab
      Files: .editorconfig
      Verify: `test -f .editorconfig && grep 'root = true' .editorconfig`

- [ ] 5. **Create MIT LICENSE**
      Copyright holder: IntelliShell Contributors
      Files: LICENSE
      Verify: `test -f LICENSE && grep MIT LICENSE`

- [ ] 6. **Commit structure changes**
      git add .gitignore .editorconfig LICENSE && git commit -m "chore: repository structure and hygiene"
      Files: committed changes
      Verify: `git log --oneline -1` shows the chore commit

---

### FEAT-002: Rust Toolchain and Core-Engine Fixes

- [ ] 7. **Pin Rust toolchain**
      Create rust-toolchain.toml: [toolchain] channel = "1.87.0", components = ["rustfmt", "clippy"]
      Files: rust-toolchain.toml
      Verify: `cat rust-toolchain.toml | grep 1.87.0`

- [ ] 8. **Fix Cargo.toml edition**
      Change edition = "2024" to edition = "2021" in core-engine/Cargo.toml
      Files: core-engine/Cargo.toml
      Verify: `grep 'edition = "2021"' core-engine/Cargo.toml`

- [ ] 9. **Fix unused Sandbox.name field**
      In core-engine/src/cgroup.rs, rename field `name: String` to `_name: String` (underscore prefix indicates intentionally unused) OR add field-level `#[allow(dead_code)]` with comment explaining kept for future observability
      Files: core-engine/src/cgroup.rs
      Verify: `cargo clippy --all-targets -- -D warnings` in core-engine/ passes with zero warnings

- [ ] 10. **Extract testable parsing functions**
      In core-engine/src/main.rs, parse_meminfo_value is already pub. For /proc/stat parsing in read_cpu_ticks(), extract the logic after finding last ')' into a pub function like `parse_proc_stat_fields(contents: &str) -> Option<(u64, u64)>` that returns (utime, stime). This allows testing process names with spaces and parentheses.
      Files: core-engine/src/main.rs
      Verify: `grep 'pub fn parse_proc_stat_fields' core-engine/src/main.rs`

- [ ] 11. **Add Rust tests**
      Create core-engine/src/tests.rs (or inline #[cfg(test)] mod tests in main.rs) with: test_parse_meminfo_value (test with "MemTotal: 8192 kB\n", "MemAvailable: 4096 kB", missing key), test_proc_stat_parsing (test parse_proc_stat_fields with process name containing spaces and parentheses like "(bash script)" and edge cases)
      Files: core-engine/src/tests.rs or inline in main.rs
      Verify: `cd core-engine && cargo test` passes

- [ ] 12. **Format and lint Rust**
      Run cargo fmt, then cargo clippy --all-targets -- -D warnings
      Files: core-engine/src/*.rs (formatted)
      Verify: `cd core-engine && cargo fmt --check && cargo clippy --all-targets -- -D warnings`

- [ ] 13. **Commit Rust fixes**
      git add rust-toolchain.toml core-engine/ && git commit -m "fix(core): pin Rust 1.87.0, fix edition 2021, fix unused field warning, add tests"
      Files: committed changes
      Verify: `git log --oneline -1` shows the fix(core) commit

---

### FEAT-003: Python AI-Service Implementation

- [ ] 14. **Create Python version pin**
      Create ai-service/.python-version containing: 3.13
      Files: ai-service/.python-version
      Verify: `cat ai-service/.python-version` returns 3.13

- [ ] 15. **Create pyproject.toml**
      [project] name="intellishell-ai-service", version="0.1.0", requires-python=">=3.13", dependencies=["fastapi>=0.115.0", "uvicorn[standard]>=0.32.0", "requests>=2.32.0", "pydantic>=2.10.0"]; [build-system] requires=["hatchling"], build-backend="hatchling.build"; [tool.uv] dev-dependencies=["pytest>=8.3.0", "ruff>=0.8.0"]; [tool.ruff] line-length=100; [tool.ruff.lint] select=["E", "F", "I"]; [tool.pytest.ini_options] testpaths=["tests"]
      Files: ai-service/pyproject.toml
      Verify: `grep 'fastapi>=0.115.0' ai-service/pyproject.toml`

- [ ] 16. **Generate uv.lock**
      Run `uv lock` in ai-service/ (requires uv installed via bootstrap.sh first — if not available, this step defers to after bootstrap or creates placeholder)
      Files: ai-service/uv.lock
      Verify: `test -f ai-service/uv.lock`

- [ ] 17. **Implement classify_command**
      In ai-service/main.py, create function classify_command(command: str) -> dict with pattern matching: rm -rf / (exact or with whitespace), rm -rf ~/* or ~/., dd if=... of=/dev/sd*, fork bomb :(){:|:&};:, chmod -R 777 / or chmod -R 777 .., kill -9 1 (PID 1), shutdown/reboot/halt/poweroff, curl|wget ... | bash|sh. Return {risk_level: "critical"|"high"|"medium"|"low", reason: str, detected_patterns: list}
      Files: ai-service/main.py
      Verify: `grep 'def classify_command' ai-service/main.py`

- [ ] 18. **Implement suggest_safer_alternative**
      In ai-service/main.py, create function suggest_safer_alternative(command: str) -> dict. For dangerous patterns detected by classify_command, suggest safer versions: rm -rf / -> "Do you mean to delete specific files? Use rm -rf /path/to/specific/dir", dd to /dev/sd* -> "Double-check device with lsblk first, use conv=fsync", etc. Return {has_suggestion: bool, suggestion: str, explanation: str}
      Files: ai-service/main.py
      Verify: `grep 'def suggest_safer_alternative' ai-service/main.py`

- [ ] 19. **Implement extract_json**
      In ai-service/main.py, create function extract_json(text: str) -> dict. Handle fenced JSON (```json\n{...}\n```) and unfenced ({...}). Use regex to extract, parse with json.loads, return parsed dict or raise ValueError.
      Files: ai-service/main.py
      Verify: `grep 'def extract_json' ai-service/main.py`

- [ ] 20. **Implement analyze_impact**
      In ai-service/main.py, create function analyze_impact(command: str, context: dict) -> dict. Parse command for file/directory targets, check if paths exist (using context like cwd), estimate impact (files affected, directories, whether recursive). Return {files_affected: int, directories_affected: int, is_recursive: bool, targets: list}
      Files: ai-service/main.py
      Verify: `grep 'def analyze_impact' ai-service/main.py`

- [ ] 21. **Implement FastAPI app**
      In ai-service/main.py, create FastAPI app with CORSMiddleware allowing origins ['http://localhost:5173', 'http://127.0.0.1:5173']. Endpoints: POST /classify (body: {command: str}), POST /suggest (body: {command: str}), POST /analyze (body: {command: str, context: dict}), GET /health (returns {status: "ok"}). Use uvicorn to run on 127.0.0.1:8000.
      Files: ai-service/main.py
      Verify: `grep 'CORSMiddleware' ai-service/main.py && grep '@app.post.*classify' ai-service/main.py`

- [ ] 22. **Create Python tests for classify**
      Create ai-service/tests/test_classify.py with pytest tests: test_classify_rm_rf_root, test_classify_rm_rf_home, test_classify_rm_r, test_classify_dd_to_dev, test_classify_fork_bomb, test_classify_chmod_777_recursive, test_classify_kill_init, test_classify_kill_normal_pid (like kill -9 1234), test_classify_shutdown, test_classify_curl_pipe_bash (e.g. curl http://example.com/script.sh | bash), test_classify_harmless (like ls -la), test_suggest_safer_alternative_cases
      Files: ai-service/tests/test_classify.py
      Verify: `cd ai-service && pytest tests/test_classify.py`

- [ ] 23. **Create Python tests for extract_json**
      Create ai-service/tests/test_extract.py with tests: test_extract_json_fenced (with ```json wrapper), test_extract_json_unfenced (plain JSON string), test_extract_json_invalid (malformed JSON raises error)
      Files: ai-service/tests/test_extract.py
      Verify: `cd ai-service && pytest tests/test_extract.py`

- [ ] 24. **Create Python tests for analyze_impact**
      Create ai-service/tests/test_analyze.py with test_analyze_impact_with_real_files using pytest tmp_path fixture to create temporary files/directories, then test analyze_impact returns correct counts
      Files: ai-service/tests/test_analyze.py
      Verify: `cd ai-service && pytest tests/test_analyze.py`

- [ ] 25. **Format and lint Python**
      Run ruff format ai-service/, then ruff check ai-service/
      Files: ai-service/*.py (formatted)
      Verify: `cd ai-service && ruff format --check . && ruff check .`

- [ ] 26. **Run all Python tests**
      Run pytest in ai-service/
      Files: test results
      Verify: `cd ai-service && pytest` all pass

- [ ] 27. **Remove old requirements.txt**
      Delete ai-service/requirements.txt (replaced by pyproject.toml)
      Files: ai-service/requirements.txt (removed)
      Verify: `test ! -f ai-service/requirements.txt`

- [ ] 28. **Commit Python implementation**
      git add ai-service/ && git rm ai-service/requirements.txt && git commit -m "feat(ai): full FastAPI service with uv, classify/suggest/extract/analyze, tests"
      Files: committed changes
      Verify: `git log --oneline -1` shows the feat(ai) commit

---

### FEAT-004: Node/Dashboard Setup

- [ ] 29. **Create .nvmrc**
      Create .nvmrc at repo root containing: 24
      Files: .nvmrc
      Verify: `cat .nvmrc` returns 24

- [ ] 30. **Add engines field to package.json**
      In dashboard/package.json, add: "engines": {"node": ">=24.0.0", "npm": ">=10.0.0"}
      Files: dashboard/package.json
      Verify: `grep engines dashboard/package.json`

- [ ] 31. **Generate package-lock.json**
      Run npm install in dashboard/
      Files: dashboard/package-lock.json
      Verify: `test -f dashboard/package-lock.json`

- [ ] 32. **Verify dashboard lint and build**
      Run npm run lint and npm run build in dashboard/
      Files: dashboard/dist/
      Verify: `cd dashboard && npm run lint && npm run build`

- [ ] 33. **Commit dashboard setup**
      git add .nvmrc dashboard/package.json dashboard/package-lock.json && git commit -m "chore(dashboard): pin Node 24, add engines, commit package-lock.json"
      Files: committed changes
      Verify: `git log --oneline -1` shows the chore(dashboard) commit

---

### FEAT-005: Doctor and Bootstrap Scripts

- [ ] 34. **Create config/default.env**
      Define: CORE_ENGINE_PORT=3000, AI_SERVICE_PORT=8000, DASHBOARD_PORT=5173, OLLAMA_PORT=11434, OLLAMA_MODEL=phi3:mini, MIN_DISK_GB=5, WARN_DISK_GB=20, MIN_RAM_GB=4
      Files: config/default.env
      Verify: `cat config/default.env | grep OLLAMA_MODEL`

- [ ] 35. **Create scripts/bootstrap.sh structure**
      Shebang #!/usr/bin/env bash, set -euo pipefail. Idempotent checks for each tool. Install apt packages: sudo apt-get update && sudo apt-get install -y build-essential pkg-config libssl-dev curl git zstd shellcheck jq. Install rustup if not present: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y. Install nvm + Node 24 if not present: curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash, then source ~/.nvm/nvm.sh && nvm install 24. Install uv if not present: curl -LsSf https://astral.sh/uv/install.sh | sh. Install gh if not present: follow GitHub CLI install for Ubuntu. Create ~/.config/intellishell/ with mode 0700. Generate ~/.config/intellishell/secret if not exists: openssl rand -hex 32 > ~/.config/intellishell/secret && chmod 0600 ~/.config/intellishell/secret. Echo versions at end.
      Files: scripts/bootstrap.sh
      Verify: `test -x scripts/bootstrap.sh && shellcheck scripts/bootstrap.sh`

- [ ] 36. **Create scripts/doctor.sh FAIL checks**
      Source config/default.env (with optional local.env override). Parse --json flag. Initialize FAIL_COUNT=0. Implement functions: check_wsl2 (grep -qi microsoft /proc/version), check_kernel_version (uname -r >= 5.15), check_systemd (pidof systemd), check_cgroup_v2 (stat -fc %T /sys/fs/cgroup returns cgroup2fs AND grep -q cpu /sys/fs/cgroup/cgroup.controllers AND memory AND pids), check_overlayfs (grep -q overlay /proc/filesystems), check_proc_health (readlink /proc/self/exe resolves AND test -r /proc/mounts), check_repo_filesystem (repo=$(git rev-parse --show-toplevel); fstype=$(stat -f -c %T "$repo"); test "$fstype" != "9p" && test "$fstype" != "drvfs"), check_disk_space (df -BG . | awk 'NR==2 {print $4}' >= MIN_DISK_GB), check_git_identity (git config user.name && git config user.email), check_toolchains (rustc --version matches rust-toolchain.toml, node --version matches .nvmrc, python3 --version >= 3.13). Each prints PASS or FAIL with remediation. Increment FAIL_COUNT on FAIL.
      Files: scripts/doctor.sh
      Verify: `bash scripts/doctor.sh` runs and shows PASS/FAIL for each check

- [ ] 37. **Create scripts/doctor.sh WARN checks**
      Add functions: check_ram (free -g | awk '/^Mem:/ {print $2}' < MIN_RAM_GB), check_disk_warn (free space < WARN_DISK_GB), check_ports (ss -ltn | grep -E ':3000|:8000|:5173|:11434' and identify unexpected processes), check_gpu (nvidia-smi || echo no GPU), check_nonloopback_bindings (ss -ltn | grep -v 127.0.0.1 | grep intellishell). Each prints WARN with message.
      Files: scripts/doctor.sh
      Verify: `bash scripts/doctor.sh` shows WARN checks

- [ ] 38. **Add doctor.sh JSON mode and exit code**
      If --json flag set, collect all check results in array and print JSON at end: [{check: "wsl2", status: "PASS", message: "..."}]. Exit with code FAIL_COUNT (non-zero if any FAIL).
      Files: scripts/doctor.sh
      Verify: `bash scripts/doctor.sh --json | jq .` produces valid JSON

- [ ] 39. **Create scripts/doctor-selftest.sh**
      Override a check to force failure (e.g., MIN_DISK_GB=999999 bash scripts/doctor.sh), capture exit code and output, assert exit code != 0 and output contains "FAIL" and specific check name. Exit 0 if self-test passes, exit 1 if not.
      Files: scripts/doctor-selftest.sh
      Verify: `bash scripts/doctor-selftest.sh` exits 0

- [ ] 40. **Make scripts executable and shellcheck**
      chmod +x scripts/*.sh, run shellcheck scripts/*.sh and fix all issues
      Files: scripts/*.sh (executable, shellcheck-clean)
      Verify: `shellcheck scripts/*.sh` returns zero warnings

- [ ] 41. **Commit scripts and config**
      git add config/ scripts/ && git commit -m "feat(scripts): doctor and bootstrap with full environment checks"
      Files: committed changes
      Verify: `git log --oneline -1` shows the feat(scripts) commit

---

### FEAT-006: LLM Smoke Test and Ollama Docs

- [ ] 42. **Document Ollama setup in docs/ENVIRONMENT.md**
      Create docs/ENVIRONMENT.md with sections: System Requirements (WSL2, Ubuntu, kernel >= 5.15, systemd, cgroup v2, 4GB+ RAM, 20GB+ disk), Toolchain Versions (Rust 1.87.0, Node 24, Python 3.13), Ollama Setup (install: curl -fsSL https://ollama.com/install.sh | sh, verify systemd: systemctl --user status ollama, verify bind: ss -ltnp | grep 11434, pull model: ollama pull phi3:mini, GPU: nvidia-smi inside WSL via Windows driver, CPU fallback documented), Bootstrap Process (run scripts/bootstrap.sh, list sudo commands: apt-get, systemctl if needed, wsl.conf edits), Secret Management (location ~/.config/intellishell/secret, mode 0600, never in git, bootstrap generates)
      Files: docs/ENVIRONMENT.md
      Verify: `test -f docs/ENVIRONMENT.md && grep 'Ollama Setup' docs/ENVIRONMENT.md`

- [ ] 43. **Create scripts/llm-smoke.sh structure**
      Shebang #!/usr/bin/env bash, set -euo pipefail. Source config/default.env for OLLAMA_MODEL. Check Ollama running: curl -s http://127.0.0.1:11434/api/tags || exit 1. Define fixed prompt (e.g., "Is this command safe: ls -la") and JSON schema (e.g., {type: "object", properties: {safe: {type: "boolean"}, reason: {type: "string"}}, required: ["safe", "reason"]}).
      Files: scripts/llm-smoke.sh
      Verify: `grep 'OLLAMA_MODEL' scripts/llm-smoke.sh`

- [ ] 44. **Implement llm-smoke.sh Ollama API call**
      Send POST to http://127.0.0.1:11434/api/generate with body: {model: "$OLLAMA_MODEL", prompt: "$FIXED_PROMPT", format: $SCHEMA_JSON, stream: false}. Parse response JSON, extract .response field, validate against schema using jq. Extract .eval_count and .eval_duration, calculate tokens/sec: eval_count / (eval_duration / 1e9). Detect GPU: check if response metadata contains cuda/gpu indicators. Print: Model: <model>, Tokens/sec: <tps>, Device: <CPU|GPU>. Exit 0 if valid, 1 if not.
      Files: scripts/llm-smoke.sh
      Verify: `bash scripts/llm-smoke.sh` runs (if Ollama available) or shows clear error

- [ ] 45. **Make llm-smoke.sh executable and shellcheck**
      chmod +x scripts/llm-smoke.sh, shellcheck scripts/llm-smoke.sh
      Files: scripts/llm-smoke.sh (executable, shellcheck-clean)
      Verify: `shellcheck scripts/llm-smoke.sh` passes

- [ ] 46. **Commit LLM smoke test and docs**
      git add docs/ENVIRONMENT.md scripts/llm-smoke.sh && git commit -m "feat(llm): smoke test script and Ollama documentation"
      Files: committed changes
      Verify: `git log --oneline -1` shows the feat(llm) commit

---

### FEAT-007: Quality Gates (Makefile, Pre-commit, CI)

- [ ] 47. **Create Makefile with all targets**
      .PHONY: help bootstrap doctor fmt lint test check llm-smoke clean run-core run-api run-dashboard. help: list targets. bootstrap: bash scripts/bootstrap.sh. doctor: bash scripts/doctor.sh. fmt: (cd core-engine && cargo fmt) && (cd ai-service && ruff format .) && (cd dashboard && npm run lint --fix || true). lint: (cd core-engine && cargo clippy --all-targets -- -D warnings) && (cd ai-service && ruff check .) && (cd dashboard && npm run lint). test: (cd core-engine && cargo test) && (cd ai-service && pytest) && (cd dashboard && npm run test || echo 'No dashboard tests yet'). build: (cd core-engine && cargo build --release) && (cd dashboard && npm run build). check: $(MAKE) lint && $(MAKE) test && $(MAKE) build. llm-smoke: bash scripts/llm-smoke.sh. clean: (cd core-engine && cargo clean) && rm -rf ai-service/.venv ai-service/__pycache__ ai-service/.pytest_cache ai-service/.ruff_cache && rm -rf dashboard/node_modules dashboard/dist. run-core: cd core-engine && cargo run. run-api: cd ai-service && uvicorn main:app --host 127.0.0.1 --port 8000 --reload. run-dashboard: cd dashboard && npm run dev -- --host 127.0.0.1 --port 5173.
      Files: Makefile
      Verify: `make help`

- [ ] 48. **Create .pre-commit-config.yaml**
      repos: pre-commit/pre-commit-hooks (trailing-whitespace, end-of-file-fixer, check-yaml, check-toml, check-json, check-merge-conflict), gitleaks/gitleaks (gitleaks detect), shellcheck-py/shellcheck-py (shellcheck on scripts/), astral-sh/ruff-pre-commit (ruff format, ruff check), doublify/pre-commit-rust (rustfmt, cargo-check). Use specific rev tags.
      Files: .pre-commit-config.yaml
      Verify: `test -f .pre-commit-config.yaml`

- [ ] 49. **Create .github/workflows/ci.yml rust job**
      name: CI, on: [push, pull_request]. Job rust: runs-on ubuntu-latest, steps: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683, actions/cache@5a3ec84eff668545956fd18022155c47e93e2684 with key based on core-engine/Cargo.lock hash, setup rust via rustup using rust-toolchain.toml file, cargo fmt --check in core-engine/, cargo clippy --all-targets -- -D warnings in core-engine/, cargo test in core-engine/ (exclude privileged tests with -- --skip sandbox_test --skip isolate_test or use #[cfg(not(feature = "ci"))] attribute).
      Files: .github/workflows/ci.yml
      Verify: `grep 'actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683' .github/workflows/ci.yml`

- [ ] 50. **Create .github/workflows/ci.yml python job**
      Job python: runs-on ubuntu-latest, steps: checkout, actions/setup-python@0b93645e9fea7318ecaed2b359559ac225c90a2b with python-version 3.13, cache .venv with key based on ai-service/uv.lock hash, install uv (curl -LsSf https://astral.sh/uv/install.sh | sh), cd ai-service && uv sync, ruff format --check ., ruff check ., pytest.
      Files: .github/workflows/ci.yml
      Verify: `grep 'setup-python@0b93645e9fea7318ecaed2b359559ac225c90a2b' .github/workflows/ci.yml`

- [ ] 51. **Create .github/workflows/ci.yml web job**
      Job web: runs-on ubuntu-latest, steps: checkout, actions/setup-node@39370e3970a6d050c480ffad4ff0ed4d3fdee5af with node-version 24, cache npm with path dashboard/node_modules, cd dashboard && npm ci, npm run lint, npm run build.
      Files: .github/workflows/ci.yml
      Verify: `grep 'setup-node@39370e3970a6d050c480ffad4ff0ed4d3fdee5af' .github/workflows/ci.yml`

- [ ] 52. **Create .github/workflows/ci.yml shellcheck and gitleaks jobs**
      Job shellcheck: runs-on ubuntu-latest, steps: checkout, run shellcheck scripts/*.sh. Job gitleaks: runs-on ubuntu-latest, steps: checkout, uses gitleaks/gitleaks-action@v2 (or pinned SHA).
      Files: .github/workflows/ci.yml
      Verify: `grep shellcheck .github/workflows/ci.yml && grep gitleaks .github/workflows/ci.yml`

- [ ] 53. **Create .github/workflows/ci.yml audit jobs**
      Job cargo-audit: runs-on ubuntu-latest, steps: checkout, install cargo-audit (cargo install cargo-audit), cd core-engine && cargo audit --deny warnings. Job pip-audit: runs-on ubuntu-latest, steps: checkout, setup-python, install pip-audit, cd ai-service && pip-audit. Job npm-audit: runs-on ubuntu-latest, steps: checkout, setup-node, cd dashboard && npm audit --audit-level=high.
      Files: .github/workflows/ci.yml
      Verify: `grep 'cargo-audit' .github/workflows/ci.yml && grep 'pip-audit' .github/workflows/ci.yml && grep 'npm-audit' .github/workflows/ci.yml`

- [ ] 54. **Create .github/dependabot.yml**
      version: 2, updates: [{package-ecosystem: "cargo", directory: "/core-engine", schedule: {interval: "daily"}}, {package-ecosystem: "pip", directory: "/ai-service", schedule: {interval: "daily"}}, {package-ecosystem: "npm", directory: "/dashboard", schedule: {interval: "daily"}}, {package-ecosystem: "github-actions", directory: "/", schedule: {interval: "weekly"}}]
      Files: .github/dependabot.yml
      Verify: `test -f .github/dependabot.yml && grep cargo .github/dependabot.yml`

- [ ] 55. **Install pre-commit hooks**
      Run pre-commit install in repo root (requires pre-commit to be installed, can be done via uv tool install pre-commit or pip install pre-commit)
      Files: .git/hooks/pre-commit
      Verify: `test -x .git/hooks/pre-commit`

- [ ] 56. **Test Makefile targets**
      Run make fmt, make lint (fix any issues found), make test
      Files: formatted and linted code
      Verify: `make lint && make test` both pass

- [ ] 57. **Commit quality gates**
      git add Makefile .pre-commit-config.yaml .github/ && git commit -m "feat(ci): Makefile, pre-commit, CI workflows, dependabot, audit jobs"
      Files: committed changes
      Verify: `git log --oneline -1` shows the feat(ci) commit

---

### FEAT-008: Documentation

- [ ] 58. **Create .kiro/steering/ files**
      product.md (project purpose, user, safety philosophy), tech.md (stack, pins, architecture), structure.md (directories, conventions), security.md (bind 127.0.0.1, secrets, sudo usage), workflow.md (clone to /home/, bootstrap, make targets, pre-commit, CI)
      Files: .kiro/steering/*.md (5 files)
      Verify: `ls .kiro/steering/*.md | wc -l` returns 5

- [ ] 59. **Create .kiro/specs/phase-0-foundation/ files**
      requirements.md (EARS notation: WHEN <condition> THE SYSTEM SHALL <requirement> for R1-R7), design.md (design decisions: uv for Python, rust-toolchain.toml, doctor checks, bootstrap idempotence, pre-commit, CI excludes privileged tests, dependabot, audit), tasks.md (checklist mirroring FEAT-001 through FEAT-008)
      Files: .kiro/specs/phase-0-foundation/*.md (3 files)
      Verify: `ls .kiro/specs/phase-0-foundation/*.md | wc -l` returns 3

- [ ] 60. **Create docs/ARCHITECTURE.md**
      Mermaid diagram showing: components (core-engine CLI, ai-service FastAPI, dashboard React, Ollama LLM), data flow (user -> dashboard -> ai-service -> Ollama, ai-service -> core-engine for sandbox), ports (3000 core, 8000 ai, 5173 dashboard, 11434 ollama), all on 127.0.0.1
      Files: docs/ARCHITECTURE.md
      Verify: `test -f docs/ARCHITECTURE.md && grep -i mermaid docs/ARCHITECTURE.md`

- [ ] 61. **Create docs/SECURITY.md**
      Assets (Linux system, user files, processes), Actors (user, dashboard, ai-service, core-engine, kernel), Trust boundaries (browser <-> dashboard <-> ai-service <-> core-engine <-> kernel). Privilege model: only sandbox/process-control may be privileged, others unprivileged, all bind 127.0.0.1, secrets never in repo. Known issues with phase owners: /execute trusts risk_level (Phase 5), core-engine no auth (Phase 5), sandbox shares filesystem (Phase 3), /proc mount leak (Phase 3), signal safety (Phase 2), Ollama auth (Phase 4), no audit (Phase 6)
      Files: docs/SECURITY.md
      Verify: `test -f docs/SECURITY.md && grep 'Phase 5' docs/SECURITY.md`

- [ ] 62. **Create docs/PHASES.md**
      List phases: Phase 0 (Foundation - current), Phase 1 (TBD), Phase 2 (Signal safety), Phase 3 (Filesystem isolation), Phase 4 (LLM integration), Phase 5 (Auth), Phase 6 (Audit)
      Files: docs/PHASES.md
      Verify: `test -f docs/PHASES.md`

- [ ] 63. **Create docs/adr/ files**
      0001-record-decisions.md (use ADRs), 0002-rust-core-engine.md (Rust for system-level control), 0003-uv-python-package-manager.md (uv for reproducible Python), 0004-rule-based-risk.md (pattern-matching for Phase 0, LLM deferred to Phase 4)
      Files: docs/adr/*.md (4 files)
      Verify: `ls docs/adr/*.md | wc -l` returns 4

- [ ] 64. **Create README.md**
      # IntelliShell, AI-Based System Intent Engine for Safe Linux Command Execution. Status: Phase 0 - Foundation. Description: three components. Quickstart: Prerequisites (WSL2, kernel >= 5.15, systemd, cgroup v2, repo in /home/ NOT /mnt/*), Clone, Run scripts/bootstrap.sh, make doctor, make check. Components table: core-engine (Rust 1.87.0, monitor + sandbox, 3000), ai-service (Python 3.13, classification, 8000), dashboard (Node 24 + React 19, UI, 5173). Development: make help, feature branches, conventional commits, pre-commit. Documentation links. License: MIT.
      Files: README.md
      Verify: `test -f README.md && grep IntelliShell README.md && grep '/home/' README.md`

- [ ] 65. **Commit documentation**
      git add .kiro/ docs/ README.md && git commit -m "docs: complete documentation foundation (steering, specs, architecture, ADRs, README)"
      Files: committed changes
      Verify: `git log --oneline -1` shows the docs commit

---

## Final Verification

- [ ] 66. **Run make check**
      Verify all three components build, lint, and test successfully
      Files: build artifacts
      Verify: `make check` exits 0

- [ ] 67. **Run make doctor**
      Verify environment checks (will FAIL on /mnt/* filesystem until user moves repo)
      Files: doctor output
      Verify: `make doctor` shows expected PASS/FAIL/WARN results

- [ ] 68. **Push branch to remote**
      Push phase/0-foundation branch and tags to origin
      Files: remote refs
      Verify: `git push -u origin phase/0-foundation && git push --tags`

---

## Notes

- **Repo location**: Current location /mnt/d is UNSUPPORTED. User must manually move/clone to /home/<user>/IntelliShell before running doctor.
- **Bootstrap dependencies**: scripts/bootstrap.sh must run before most other steps (needs uv, rustup, node). Some steps may need adjustment if bootstrap hasn't run yet (e.g., uv lock in step 16).
- **Ollama**: Required for llm-smoke test but not for core functionality. CI may skip llm-smoke.
- **Privileged tests**: Cgroup tests (sandbox-test, isolate-test) require root. CI excludes them. Local testing with sudo works.
- **CI action SHAs**: Pinned SHAs in plan are examples. Verify current pinned SHAs for actions/checkout@v4, actions/cache@v4, actions/setup-python@v5, actions/setup-node@v4 at implementation time.


---

## Phase 2: Process Control Engine - COMPLETED 2026-10-19

**Status**: ✅ COMPLETE  
**Implementation**: `core-engine/src/lib.rs`, `core-engine/src/process.rs`, `core-engine/tests/process_integration.rs`  
**Build**: ✅ PASSED (release mode, cargo build --lib --release)  
**Tests**: ✅ PASSED (11/11 unit tests)  
**Documentation**: See `.agents/tasks/PHASE2_COMPLETION.md`

### Deliverables Completed

- [x] Production-grade Rust library for process control
- [x] Graceful kill with SIGTERM→SIGKILL timeout escalation
- [x] Pause/resume with state verification via `/proc/<pid>/stat`
- [x] Priority adjustment (renice) with readback verification
- [x] PID-reuse safety via (PID, start_time) tuple validation
- [x] Typed error system with `thiserror` (`ProcessError` enum)
- [x] Comprehensive unit tests (11 tests, 100% pass rate)
- [x] Integration test suite with real child processes
- [x] Extensive rustdoc documentation
- [x] Zero unsafe code (`#![deny(unsafe_code)]`)

### API Summary

```rust
// Create process identity
let proc_id = ProcessId::from_pid(pid)?;

// Graceful kill with 5s timeout
let opts = KillOptions::default(); // 5000ms timeout
let outcome = kill_process(proc_id, opts)?;

// Pause and resume
pause_process(proc_id)?;  // SIGSTOP + state verification
resume_process(proc_id)?; // SIGCONT + state verification

// Renice
renice_process(proc_id, 10)?; // Lower priority
```

### Build Commands

```bash
# Build library in release mode
cd ~/projects/intellishell/core-engine
cargo build --lib --release

# Run unit tests
cargo test --lib

# View library API docs
cargo doc --lib --no-deps --open
```

### Known Limitations

1. **No pidfd support yet**: Future enhancement for kernel 5.1+ to eliminate TOCTOU race
2. **renice uses external command**: Could be replaced with direct `libc::setpriority` call
3. **Integration tests blocked**: Existing `main.rs` missing dependencies (axum, tokio)
4. **Doc warnings**: 10 cosmetic warnings for missing field/variant documentation

### Next Steps (Out of Scope for Phase 2)

1. Update `main.rs` (lines 11-113) to delegate to new library functions
2. Add missing dependencies for full project build (axum, tokio, serde, tower-http)
3. Implement pidfd_open/pidfd_send_signal for kernels >= 5.1
4. Replace renice command with direct libc::setpriority
5. Document all struct fields and enum variants to eliminate warnings

**Location**: Windows staging: `D:\3rd year\os project\IntelliShell\core-engine`  
**Location**: WSL production: `~/projects/intellishell/core-engine`  
**Completion Report**: `.agents/tasks/PHASE2_COMPLETION.md`
