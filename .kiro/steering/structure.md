# Repository Structure

## Directory Layout

```
IntelliShell/
├── .github/              # GitHub-specific configuration
│   ├── workflows/        # CI/CD pipeline definitions
│   └── dependabot.yml    # Dependency update automation
├── .kiro/                # Project steering and specifications
│   ├── steering/         # Project guidance documents (this directory)
│   └── specs/            # Detailed specifications by phase
│       └── phase-0-foundation/
├── ai-service/           # Python FastAPI service
│   ├── src/              # Python source code
│   ├── tests/            # pytest test suite
│   ├── pyproject.toml    # Python project metadata and dependencies
│   ├── uv.lock           # Locked dependency versions
│   └── .python-version   # Python version pin (3.13)
├── core-engine/          # Rust process monitor and sandbox
│   ├── src/              # Rust source code
│   │   ├── main.rs
│   │   ├── cgroup.rs
│   │   └── namespace.rs
│   ├── tests/            # Integration tests
│   ├── Cargo.toml        # Rust package manifest
│   └── Cargo.lock        # Locked dependency versions
├── dashboard/            # React web interface
│   ├── src/              # React components and assets
│   ├── public/           # Static assets
│   ├── package.json      # npm package manifest
│   ├── package-lock.json # Locked dependency versions
│   └── vite.config.js    # Vite build configuration
├── config/               # Configuration files
│   ├── default.env       # Default environment variables (committed)
│   └── local.env         # Local overrides (git-ignored, never committed)
├── docs/                 # Documentation
│   ├── adr/              # Architecture Decision Records
│   ├── ARCHITECTURE.md   # System architecture overview
│   ├── ENVIRONMENT.md    # Setup and toolchain guide
│   ├── SECURITY.md       # Security model and known issues
│   └── PHASES.md         # Development phases roadmap
├── scripts/              # Utility scripts
│   ├── bootstrap.sh      # Environment setup (idempotent)
│   ├── doctor.sh         # Environment health checks
│   └── llm-smoke.sh      # LLM integration test
├── .editorconfig         # Editor configuration for consistent style
├── .gitignore            # Git ignore patterns
├── .nvmrc                # Node version pin (24)
├── .pre-commit-config.yaml  # Pre-commit hooks configuration
├── LICENSE               # MIT license
├── Makefile              # Build automation and task runner
├── README.md             # Project overview and quick start
└── rust-toolchain.toml   # Rust toolchain pin (1.87.0)
```

## Component Structure

### Core Engine (Rust)

- **Binary crate**: Produces a single executable
- **Entry point**: `src/main.rs`
- **Modules**: Organized by feature (cgroup, namespace, sandbox)
- **Tests**: Unit tests alongside code, integration tests in `tests/`

### AI Service (Python)

- **ASGI application**: Runs via uvicorn
- **Entry point**: `src/main.py` (FastAPI app)
- **Structure**:
  - `src/classifier.py`: Rule-based command classification
  - `src/analyzer.py`: Impact analysis and safer alternatives
  - `src/llm.py`: Ollama client and prompt handling
- **Tests**: pytest suite in `tests/`

### Dashboard (React)

- **SPA**: Single-page application built with Vite
- **Entry point**: `src/main.jsx`
- **Structure**:
  - `src/components/`: React components
  - `src/hooks/`: Custom React hooks
  - `src/api/`: HTTP client for backend services
  - `src/assets/`: Images, styles
- **Tests**: Vitest (or similar) in `src/**/*.test.jsx`

## Conventions

### Version Control

- **Branches**:
  - `main`: Stable release branch
  - `phase/*`: Long-lived phase branches (e.g., `phase/0-foundation`)
  - `feature/*`: Short-lived feature branches
  - `fix/*`: Bug fix branches
- **Commits**: Conventional commit format
  - `feat:` New features
  - `fix:` Bug fixes
  - `docs:` Documentation changes
  - `refactor:` Code refactoring
  - `test:` Test additions or fixes
  - `chore:` Build, tooling, dependencies
- **Tags**: Version tags (e.g., `v0.1.0`) and milestone tags (e.g., `prototype-v0`)

### Make Targets

All operations are accessible via `make`:

| Target | Description |
|--------|-------------|
| `make help` | List all available targets |
| `make bootstrap` | Run scripts/bootstrap.sh |
| `make doctor` | Run scripts/doctor.sh environment checks |
| `make fmt` | Format code in all components |
| `make lint` | Lint code in all components |
| `make test` | Run tests in all components |
| `make check` | Run lint + test + build (full CI locally) |
| `make build` | Build all components |
| `make clean` | Remove build artifacts |
| `make llm-smoke` | Test LLM integration |
| `make run-core` | Run core-engine locally |
| `make run-ai` | Run ai-service locally |
| `make run-dashboard` | Run dashboard dev server locally |

### Code Quality

- **Pre-commit hooks**: Run automatically on `git commit`
  - Whitespace and EOF fixes
  - TOML/YAML/JSON validation
  - Merge conflict detection
  - gitleaks (secret scanning)
  - shellcheck (shell script linting)
  - ruff (Python formatting and linting)
  - cargo fmt (Rust formatting)
- **CI validation**: All checks run on push and PR
  - Linting, testing, building
  - Security audit (cargo audit, pip-audit, npm audit)
  - Dependency updates tracked by dependabot

### File Naming

- **Rust**: `snake_case.rs`
- **Python**: `snake_case.py`
- **JavaScript**: `PascalCase.jsx` for components, `camelCase.js` for utilities
- **Config**: `kebab-case.yml`, `kebab-case.toml`
- **Docs**: `UPPERCASE.md` for top-level docs, `kebab-case.md` for ADRs

## Git Ignore Patterns

The following are never committed:

- Build artifacts: `target/`, `dist/`, `build/`
- Dependencies: `node_modules/`, `__pycache__/`, `.venv/`
- Environment: `.env`, `config/local.env`
- Logs: `*.log`
- Caches: `.ruff_cache/`, `.pytest_cache/`, `.vite/`
- Secrets: Any file in `~/.config/intellishell/` (outside repo)
- Editor: `.vscode/`, `.idea/` (optional, team-specific)
