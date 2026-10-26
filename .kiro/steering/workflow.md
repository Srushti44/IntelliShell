# Development Workflow

## Getting Started

### Prerequisites

Before contributing to IntelliShell, ensure your environment meets the requirements:

- **Platform**: Windows with WSL2 (Windows 11 or Windows 10 with WSL2 enabled)
- **Distribution**: Ubuntu 22.04 or later inside WSL2
- **Kernel**: Linux kernel >= 5.15.0
- **Init**: systemd enabled in WSL2
- **Cgroup**: cgroup v2 with cpu, memory, pids controllers

### Initial Setup

1. **Clone to the correct location**:
   ```bash
   # IMPORTANT: Clone to /home/<user>/, never /mnt/*
   cd ~
   git clone <repository-url> intellishell
   cd intellishell
   ```

2. **Run bootstrap**:
   ```bash
   ./scripts/bootstrap.sh
   ```
   This installs all dependencies (Rust, Node, Python/uv, system packages).

3. **Install Ollama**:
   ```bash
   curl -fsSL https://ollama.com/install.sh | sh
   ollama pull phi3:mini
   ```

4. **Verify environment**:
   ```bash
   make doctor
   ```
   All checks should show `PASS` or `WARN` (no `FAIL`).

5. **Build and test**:
   ```bash
   make check
   ```
   This runs linting, testing, and building for all three components.

## Branching Strategy

### Branch Types

- **`main`**: Stable, release-ready code
- **`phase/*`**: Long-lived phase branches (e.g., `phase/0-foundation`)
  - All feature work for a phase starts from and merges back to the phase branch
  - Phase branch merges to `main` when phase is complete
- **`feature/*`**: Short-lived branches for specific features
  - Example: `feature/add-filesystem-check`
- **`fix/*`**: Bug fix branches
  - Example: `fix/meminfo-parsing-edge-case`

### Creating a Feature Branch

```bash
# Start from the current phase branch
git checkout phase/0-foundation
git pull origin phase/0-foundation

# Create feature branch
git checkout -b feature/my-feature-name

# Work on your feature...
# Commit regularly with conventional commit messages

# Push to remote
git push -u origin feature/my-feature-name
```

## Commit Conventions

All commits use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Commit Types

- **feat**: New feature
  - Example: `feat(ai-service): add filesystem impact analysis`
- **fix**: Bug fix
  - Example: `fix(core): handle process names with parentheses`
- **docs**: Documentation changes
  - Example: `docs: add ADR for uv adoption`
- **refactor**: Code refactoring (no behavior change)
  - Example: `refactor(core): extract meminfo parsing to separate module`
- **test**: Add or update tests
  - Example: `test(ai-service): add tests for classify_command`
- **chore**: Build, tooling, dependencies
  - Example: `chore: pin Rust to 1.87.0`
- **style**: Code style (formatting, whitespace)
- **perf**: Performance improvements
- **ci**: CI/CD changes

### Scope (Optional)

- `core`: core-engine
- `ai`: ai-service
- `dashboard`: dashboard
- `scripts`: scripts/
- `docs`: documentation
- Omit scope for changes spanning multiple components

### Examples

```bash
git commit -m "feat(core): implement cgroup v2 memory limit enforcement"
git commit -m "fix(ai): correct regex for fork bomb detection"
git commit -m "docs: update ENVIRONMENT.md with Ollama GPU setup"
git commit -m "chore(deps): bump fastapi from 0.104.0 to 0.104.1"
git commit -m "test(core): add test for /proc/stat parsing with spaces"
```

## Development Loop

### Daily Workflow

1. **Pull latest changes**:
   ```bash
   git checkout phase/0-foundation
   git pull origin phase/0-foundation
   ```

2. **Create or switch to feature branch**:
   ```bash
   git checkout -b feature/my-work
   # or
   git checkout feature/my-work
   ```

3. **Make changes**, then verify locally:
   ```bash
   make fmt      # Format code
   make lint     # Check for issues
   make test     # Run tests
   make check    # All of the above + build
   ```

4. **Commit with conventional commit message**:
   ```bash
   git add <files>
   git commit -m "feat(scope): description"
   ```
   Pre-commit hooks will run automatically.

5. **Push to remote**:
   ```bash
   git push origin feature/my-work
   ```

6. **Create pull request** to the phase branch (e.g., `phase/0-foundation`)

### Before Every Commit

Run `make check` to ensure:
- Code is formatted (`make fmt`)
- No linting errors (`make lint`)
- All tests pass (`make test`)
- All components build successfully

Pre-commit hooks will also run:
- Whitespace and EOF fixers
- TOML/YAML/JSON validation
- Merge conflict detection
- gitleaks (secret detection)
- shellcheck (shell scripts)
- ruff (Python formatting and linting)
- cargo fmt (Rust formatting)

If pre-commit hooks fail, **fix the issues and commit again** (do not use `--no-verify`).

## Make Targets Reference

All development tasks use `make`:

### Essential Targets

| Target | Description |
|--------|-------------|
| `make help` | Show all available targets with descriptions |
| `make bootstrap` | Run scripts/bootstrap.sh (install dependencies) |
| `make doctor` | Verify environment setup |
| `make check` | Run lint + test + build (run this before every commit) |

### Code Quality

| Target | Description |
|--------|-------------|
| `make fmt` | Format code (cargo fmt, ruff format, prettier) |
| `make lint` | Lint code (clippy, ruff check, eslint) |
| `make test` | Run all tests (cargo test, pytest, vitest) |
| `make build` | Build all components |
| `make clean` | Remove build artifacts |

### Running Services Locally

| Target | Description |
|--------|-------------|
| `make run-core` | Run core-engine (binds 127.0.0.1:3000) |
| `make run-ai` | Run ai-service (binds 127.0.0.1:8000) |
| `make run-dashboard` | Run dashboard dev server (binds 127.0.0.1:5173) |

### Testing

| Target | Description |
|--------|-------------|
| `make llm-smoke` | Test Ollama integration (sends sample prompt) |

## Pull Request Process

1. **Ensure your branch is up to date**:
   ```bash
   git checkout phase/0-foundation
   git pull origin phase/0-foundation
   git checkout feature/my-work
   git rebase phase/0-foundation
   ```

2. **Run full check**:
   ```bash
   make check
   ```

3. **Push and create PR**:
   ```bash
   git push origin feature/my-work
   gh pr create --base phase/0-foundation --title "feat: description" --body "Details..."
   ```

4. **CI must pass**: All CI jobs (lint, test, build, audit) must pass before merge

5. **Review and merge**: Wait for review, address feedback, then merge to phase branch

## CI/CD Pipeline

Continuous integration runs on every push and pull request:

### CI Jobs

1. **rust**: Lint and test core-engine
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

2. **python**: Lint and test ai-service
   - `ruff format --check`
   - `ruff check`
   - `pytest`

3. **web**: Lint and build dashboard
   - `npm run lint`
   - `npm run build`

4. **shellcheck**: Validate all shell scripts

5. **gitleaks**: Scan for secrets

6. **audit**: Security vulnerability scans
   - `cargo audit` (fails on high/critical)
   - `pip-audit` (fails on high/critical)
   - `npm audit` (fails on high/critical)

### Caching

CI uses aggressive caching to speed up builds:
- Rust: `~/.cargo` and `target/`
- Python: `~/.cache/uv`
- Node: `~/.npm` and `node_modules/`

### Dependabot

Automated dependency updates via GitHub Dependabot:
- **cargo**: Rust dependencies (weekly)
- **pip**: Python dependencies (weekly)
- **npm**: Node dependencies (weekly)
- **github-actions**: CI action updates (weekly)

Dependabot creates PRs automatically; review and merge after CI passes.

## Testing Standards

### Test Coverage

Every new feature must include tests:

- **Rust**: Unit tests in module files, integration tests in `tests/`
- **Python**: pytest tests in `tests/`, one test file per module
- **JavaScript**: Vitest tests co-located with components (`.test.jsx`)

### What to Test

- **Logic**: All non-trivial functions
- **Edge cases**: Empty input, invalid input, boundary conditions
- **Error handling**: Ensure errors are caught and reported
- **Parsing**: Validate parsing of /proc files, command patterns, JSON

### What NOT to Test

- **Third-party libraries**: Trust that FastAPI, React, etc., work
- **Mocks of the thing under test**: Test real logic, not mocks

### Running Tests Locally

```bash
# All tests
make test

# Rust only
cd core-engine && cargo test

# Python only
cd ai-service && pytest

# JavaScript only
cd dashboard && npm run test
```

## Debugging

### Core Engine (Rust)

```bash
# Build with debug symbols
cd core-engine
cargo build

# Run with RUST_LOG for detailed logs
RUST_LOG=debug cargo run

# Or attach a debugger (gdb, lldb, VS Code)
```

### AI Service (Python)

```bash
# Run with uvicorn reload for hot-reloading
cd ai-service
uv run uvicorn src.main:app --reload --host 127.0.0.1 --port 8000

# Or with debugger (pdb, VS Code)
```

### Dashboard (React)

```bash
# Vite dev server includes hot module replacement
cd dashboard
npm run dev

# Browser DevTools for debugging
```

## Common Issues

### Repository on /mnt/c or /mnt/d

**Symptom**: `make doctor` fails with "Repository on drvfs filesystem"

**Solution**: Move to `/home/<user>/`:
```bash
cd ~
mv /mnt/d/path/to/intellishell .
cd ~/intellishell
```

### Cgroup v2 not enabled

**Symptom**: `make doctor` fails with "cgroup v2 not enabled"

**Solution**: Enable systemd in `/etc/wsl.conf` and restart WSL:
```bash
sudo nano /etc/wsl.conf
# Add:
# [boot]
# systemd=true

# Then from PowerShell:
wsl --shutdown
wsl
```

### Ollama not responding

**Symptom**: `make llm-smoke` fails

**Solution**: Restart Ollama service:
```bash
systemctl --user restart ollama
systemctl --user status ollama
```

### Pre-commit hooks fail

**Symptom**: Commit rejected by pre-commit

**Solution**: Fix the issues reported and commit again:
```bash
# Hooks auto-fix formatting issues
git add <files>
git commit -m "message"

# If issues remain, fix manually and retry
```

---

For more details, see:
- [docs/ENVIRONMENT.md](../../docs/ENVIRONMENT.md) - Setup and toolchain
- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - System design
- [docs/SECURITY.md](../../docs/SECURITY.md) - Security guidelines
