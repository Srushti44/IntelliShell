# Technology Stack

## Architecture Overview

IntelliShell consists of three independent components that communicate via HTTP APIs on localhost:

1. **core-engine**: Rust CLI process monitor with cgroup and namespace isolation
2. **ai-service**: Python FastAPI service for command classification and analysis
3. **dashboard**: React single-page application for user interaction

## Technology Choices

### Core Engine (Rust)

- **Language**: Rust (edition 2024)
- **Toolchain pin**: Rust 1.87.0 via `rust-toolchain.toml`
- **Purpose**: Process monitoring, cgroup/namespace isolation, resource control
- **Rationale**: Memory safety with low-level system access to Linux kernel features
- **Key capabilities**:
  - Direct libc bindings for cgroup v2, namespaces, and signals
  - High performance with minimal overhead
  - Safe concurrent process monitoring

### AI Service (Python)

- **Language**: Python 3.13
- **Framework**: FastAPI for HTTP API
- **Package manager**: uv (modern, fast alternative to pip)
- **Toolchain pins**:
  - `.python-version` file pins Python 3.13
  - `pyproject.toml` defines dependencies
  - `uv.lock` provides reproducible installs
- **Purpose**: Command classification, risk analysis, pattern matching, LLM integration
- **Rationale**: Rich ecosystem for AI/ML libraries, rapid development, easy integration with Ollama

### Dashboard (React)

- **Language**: JavaScript/TypeScript
- **Framework**: React 19
- **Build tool**: Vite
- **Toolchain pin**: Node 24 LTS via `.nvmrc` (supported through 2028-04-30)
- **Purpose**: User interface for command input, risk visualization, and result display
- **Rationale**: Modern, responsive UI with excellent developer experience

### LLM Runtime (Ollama)

- **Runtime**: Ollama (systemd service inside WSL2)
- **Purpose**: Local large language model inference
- **Interface**: HTTP API on localhost:11434
- **Models**: Configurable (default: phi3:mini in Phase 0)

## Toolchain Pins

Reproducibility is critical for a system-level tool. All toolchains are pinned:

| Component | Version | Pin File | Lock File |
|-----------|---------|----------|-----------|
| Rust | 1.87.0 stable | `rust-toolchain.toml` | `Cargo.lock` |
| Node | 24.x LTS | `.nvmrc` | `package-lock.json` |
| Python | 3.13.x | `.python-version` | `uv.lock` |

## Architecture Principles

### Independent Components

- **No monorepo workspace**: Each component (`core-engine/`, `ai-service/`, `dashboard/`) is self-contained
- **Separate build systems**: cargo, uv, npm operate independently
- **Loose coupling**: Components communicate only via HTTP APIs
- **Independent deployability**: Each component can be updated without rebuilding others

### Communication

- **Protocol**: HTTP/REST on localhost
- **Ports**:
  - `3000`: core-engine API
  - `8000`: ai-service API
  - `5173`: dashboard dev server
  - `11434`: Ollama API
- **Security**: All services bind to `127.0.0.1` only (never `0.0.0.0`)

### Data Flow

```
User → Dashboard → AI Service → Ollama (LLM)
              ↓
         Core Engine (sandbox/monitor)
```

1. User enters command in dashboard
2. Dashboard sends command to ai-service for classification
3. AI service uses Ollama for advanced analysis (Phase 4+)
4. AI service returns risk assessment and recommendations
5. If user proceeds, dashboard instructs core-engine to execute (sandbox if high-risk)
6. Core engine monitors execution and reports metrics

## Development Constraints

- **Platform**: Linux only (WSL2 on Windows)
- **Kernel requirements**: >= 5.15 (for cgroup.kill, pidfd_open)
- **Init system**: systemd required
- **Cgroup version**: v2 with cpu, memory, pids controllers
- **Filesystem**: Repository must be under `/home/`, never on `/mnt/*` (9p/drvfs limitations)
