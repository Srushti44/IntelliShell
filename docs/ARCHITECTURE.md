# IntelliShell Architecture

This document describes the system architecture, components, data flow, and communication patterns of IntelliShell.

## System Overview

IntelliShell is an AI-assisted Linux command assistant composed of three independent components that communicate via HTTP APIs on localhost. All inference runs locally using Ollama for privacy and control.

## Architecture Diagram

```mermaid
graph TB
    subgraph Browser
        User[User]
    end

    subgraph "Localhost (127.0.0.1)"
        Dashboard[Dashboard<br/>React + Vite<br/>Port 5173]
        AIService[AI Service<br/>Python FastAPI<br/>Port 8000]
        CoreEngine[Core Engine<br/>Rust CLI<br/>Port 3000]
        Ollama[Ollama<br/>LLM Runtime<br/>Port 11434]
    end

    subgraph "Linux Kernel"
        CgroupV2[cgroup v2]
        Namespaces[Namespaces<br/>mount, PID, net, IPC, UTS]
        Procfs[/proc filesystem]
    end

    User -->|HTTPS| Dashboard
    Dashboard -->|HTTP POST /classify| AIService
    Dashboard -->|HTTP POST /execute| CoreEngine
    AIService -->|HTTP POST /api/generate| Ollama
    CoreEngine -->|cgroup API| CgroupV2
    CoreEngine -->|clone3, setns| Namespaces
    CoreEngine -->|read| Procfs

    style Dashboard fill:#e1f5ff
    style AIService fill:#fff4e6
    style CoreEngine fill:#ffe7e7
    style Ollama fill:#f3e5f5
    style CgroupV2 fill:#e8f5e9
    style Namespaces fill:#e8f5e9
    style Procfs fill:#e8f5e9
```

## Components

### Dashboard (React + Vite)

**Purpose**: User interface for command input, risk assessment display, and execution control.

**Technology**:
- React 19 (UI library)
- Vite (build tool and dev server)
- Node 24 LTS

**Port**: 5173 (dev server on 127.0.0.1)

**Responsibilities**:
- Accept command input from user
- Display command classification and risk level
- Show safer alternatives when available
- Provide execution controls (run, sandbox, cancel)
- Display execution results and resource metrics
- Show process monitoring data

**API Clients**:
- Calls AI Service `/classify` endpoint
- Calls Core Engine `/execute` endpoint (future)
- Polls Core Engine `/status` endpoint for monitoring (future)

### AI Service (Python + FastAPI)

**Purpose**: Command classification, risk analysis, impact assessment, and safer alternative suggestions.

**Technology**:
- Python 3.13
- FastAPI (async web framework)
- Pydantic (data validation)
- uvicorn (ASGI server)

**Port**: 8000 (on 127.0.0.1)

**Responsibilities**:
- Classify commands by risk level (low, medium, high, critical)
- Identify dangerous patterns (rm -rf, fork bombs, dd to devices)
- Suggest safer alternatives
- Analyze filesystem impact
- Extract structured data from LLM responses
- Communicate with Ollama for advanced analysis (Phase 4+)

**Endpoints** (current and planned):
- `POST /classify`: Classify a command and return risk assessment
- `POST /suggest`: Suggest safer alternatives
- `POST /analyze`: Analyze potential impact

**Dependencies**:
- Rule-based pattern matching (Phase 0)
- Ollama integration for context-aware suggestions (Phase 4+)

### Core Engine (Rust CLI)

**Purpose**: Process monitoring, resource control, and sandboxed execution using cgroups and namespaces.

**Technology**:
- Rust 1.87.0 (edition 2024)
- Direct libc bindings for system calls
- No HTTP server in current version (CLI monitor loop)

**Port**: 3000 (future HTTP API on 127.0.0.1)

**Responsibilities**:
- Monitor running processes (CPU, memory, I/O)
- Parse `/proc/<pid>/stat`, `/proc/meminfo`, `/proc/<pid>/status`
- Create and manage cgroup v2 hierarchies
- Enforce resource limits (memory, CPU, process count)
- Create isolated namespaces (mount, PID, network, IPC, UTS)
- Execute commands in sandboxed environments
- Handle signals and process lifecycle

**Current State** (Phase 0):
- CLI tool with monitor loop
- Parses /proc filesystem
- Basic cgroup and namespace scaffolding
- No HTTP server yet (added in later phase)

**Future** (Phase 1+):
- HTTP API on port 3000
- `/execute` endpoint for sandboxed command execution
- `/status/<pid>` endpoint for process monitoring
- WebSocket for real-time metrics

### Ollama (LLM Runtime)

**Purpose**: Local large language model inference for advanced command analysis.

**Technology**:
- Ollama (systemd service)
- Default model: phi3:mini (Phase 0), configurable in later phases

**Port**: 11434 (on 127.0.0.1)

**Responsibilities**:
- Generate command risk assessments
- Provide context-aware suggestions
- Explain command behavior in natural language
- Generate structured JSON responses

**GPU Acceleration**:
- Uses NVIDIA GPU via Windows driver passthrough (if available)
- Falls back to CPU if no GPU detected

**Security**:
- Runs on localhost only
- No authentication (Phase 4 will add auth)
- Model data stays local (no external API calls)

## Data Flow

### Command Classification Flow

1. User enters command in **Dashboard**
2. Dashboard sends `POST /classify` to **AI Service**
3. AI Service applies rule-based pattern matching
4. (Future) AI Service sends prompt to **Ollama** for context analysis
5. (Future) Ollama returns structured JSON response
6. AI Service combines rules + LLM insights
7. AI Service returns risk level, explanation, and safer alternatives to **Dashboard**
8. Dashboard displays results to user

### Command Execution Flow (Future)

1. User approves command in **Dashboard**
2. Dashboard sends `POST /execute` to **Core Engine** with command and risk level
3. Core Engine determines sandbox requirements based on risk level
4. For high/critical risk:
   - Create cgroup with resource limits
   - Create namespaces (mount, PID, network)
   - Execute command in isolated environment
5. For low/medium risk:
   - Execute with cgroup resource limits only
   - No namespace isolation
6. Core Engine monitors process:
   - Reads `/proc/<pid>/stat` for CPU time
   - Reads cgroup files for memory usage
   - Enforces limits, terminates on overrun
7. Core Engine streams status updates to Dashboard
8. Dashboard displays real-time metrics

### Process Monitoring Flow

1. **Core Engine** maintains list of monitored processes
2. Periodic loop (e.g., every 1 second):
   - For each PID: read `/proc/<pid>/stat`
   - Parse process name, state, CPU ticks
   - Read `/proc/<pid>/status` for memory
   - Read cgroup controller files
3. Update in-memory metrics
4. (Future) Push updates via WebSocket to Dashboard
5. Dashboard renders charts and tables

## Port Allocation

| Component | Port | Protocol | Binding |
|-----------|------|----------|---------|
| Dashboard | 5173 | HTTP | 127.0.0.1 |
| AI Service | 8000 | HTTP | 127.0.0.1 |
| Core Engine | 3000 | HTTP (future) | 127.0.0.1 |
| Ollama | 11434 | HTTP | 127.0.0.1 |

All services bind to `127.0.0.1` exclusively. No external access is permitted.

## Security Boundaries

### Trust Boundaries

```
[Browser] <--HTTPS--> [Dashboard] <--HTTP--> [AI Service] <--HTTP--> [Ollama]
                          |
                          v
                      [Core Engine] <--syscalls--> [Kernel]
```

**Trust levels**:
1. **Browser**: Untrusted user input, JavaScript sandbox
2. **Dashboard**: Trusted static assets, validates user input
3. **AI Service**: Trusted business logic, unprivileged
4. **Core Engine**: Privileged (root or capabilities), isolated from network
5. **Ollama**: Trusted LLM runtime, unprivileged
6. **Kernel**: Enforces cgroups, namespaces, syscall restrictions

### Privilege Model

| Component | User | Privileges | Rationale |
|-----------|------|------------|-----------|
| Dashboard | Regular user | None | Static assets, browser sandbox |
| AI Service | Regular user | None | Classification logic only |
| Ollama | Regular user | None | LLM inference requires no privileges |
| Core Engine | root (or capabilities) | CAP_SYS_ADMIN, cgroup delegation | Required for cgroup creation and namespace entry |

**Future hardening** (Phase 5):
- Core Engine drops capabilities after setup
- Seccomp filters on sandboxed processes
- SELinux or AppArmor profiles

### CORS Policy

Core Engine and AI Service restrict CORS to:
- `http://localhost:5173`
- `http://127.0.0.1:5173`

No wildcard origins (`*`) are permitted.

## Technology Stack Summary

| Layer | Technology | Version | Pin File |
|-------|------------|---------|----------|
| **Frontend** | React | 19 | package.json |
| | Vite | Latest | package.json |
| | Node.js | 24 LTS | .nvmrc |
| **Backend** | Python | 3.13 | .python-version |
| | FastAPI | Latest | pyproject.toml |
| | uvicorn | Latest | pyproject.toml |
| **System** | Rust | 1.87.0 | rust-toolchain.toml |
| **LLM** | Ollama | Latest | - (systemd service) |
| **Platform** | WSL2 | - | docs/ENVIRONMENT.md |
| | Kernel | >= 5.15 | doctor.sh check |
| | Cgroup | v2 | doctor.sh check |

## Deployment Architecture

### Development

```
Developer Workstation (Windows)
└── WSL2 (Ubuntu)
    ├── Dashboard dev server (npm run dev, port 5173)
    ├── AI Service dev server (uvicorn --reload, port 8000)
    ├── Core Engine (cargo run or compiled binary, port 3000)
    └── Ollama systemd service (port 11434)
```

**Commands**:
- `make run-dashboard`: Start Vite dev server
- `make run-ai`: Start ai-service with hot reload
- `make run-core`: Start core-engine monitor loop

### Production (Future)

```
Linux Server (Native or WSL2)
├── Dashboard (nginx serving static build, port 5173)
├── AI Service (systemd service, port 8000)
├── Core Engine (systemd service, port 3000)
└── Ollama (systemd service, port 11434)
```

**Key differences**:
- Dashboard: Pre-built static assets served by nginx
- AI Service: Managed by systemd, no --reload
- Core Engine: Managed by systemd, privileged user
- Reverse proxy (optional): nginx terminates TLS, proxies to services

## File System Layout

### Repository Structure

```
IntelliShell/
├── dashboard/          # React SPA
│   ├── src/            # React components
│   ├── public/         # Static assets
│   └── package.json
├── ai-service/         # Python FastAPI
│   ├── src/            # Python modules
│   ├── tests/          # pytest tests
│   └── pyproject.toml
├── core-engine/        # Rust binary
│   ├── src/            # Rust source
│   │   ├── main.rs
│   │   ├── cgroup.rs
│   │   └── namespace.rs
│   └── Cargo.toml
├── docs/               # Documentation
├── scripts/            # Utility scripts
└── config/             # Configuration files
```

### Runtime Filesystem

```
/home/<user>/intellishell/     # Repository (must be on native FS, not /mnt/*)
~/.config/intellishell/secret  # Per-machine secret (mode 0600)
/sys/fs/cgroup/               # Cgroup v2 filesystem
/proc/                        # Process information
/tmp/intellishell/            # Temporary sandbox workspaces
```

## Communication Protocols

### HTTP REST (Dashboard ↔ AI Service)

**Request**:
```http
POST /classify HTTP/1.1
Host: 127.0.0.1:8000
Content-Type: application/json

{
  "command": "rm -rf /tmp/old_data"
}
```

**Response**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "risk_level": "high",
  "reason": "Recursive delete of directory",
  "impact": "Will delete all files and subdirectories under /tmp/old_data",
  "safer_alternative": "Use 'rm -ri /tmp/old_data' for interactive deletion, or move to trash first"
}
```

### HTTP REST (Dashboard ↔ Core Engine, Future)

**Request**:
```http
POST /execute HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json

{
  "command": "find . -name '*.log' -delete",
  "risk_level": "medium",
  "sandbox": true,
  "limits": {
    "memory_mb": 256,
    "cpu_percent": 50,
    "timeout_sec": 300
  }
}
```

**Response**:
```http
HTTP/1.1 202 Accepted
Content-Type: application/json

{
  "pid": 12345,
  "status": "running",
  "monitor_url": "/status/12345"
}
```

### HTTP REST (AI Service ↔ Ollama)

**Request**:
```http
POST /api/generate HTTP/1.1
Host: 127.0.0.1:11434
Content-Type: application/json

{
  "model": "phi3:mini",
  "prompt": "Analyze this command for safety: rm -rf /tmp/test",
  "format": {
    "type": "object",
    "properties": {
      "risk_level": {"type": "string", "enum": ["low", "medium", "high", "critical"]},
      "reason": {"type": "string"}
    },
    "required": ["risk_level", "reason"]
  },
  "stream": false
}
```

**Response**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "model": "phi3:mini",
  "created_at": "2024-01-15T10:30:00Z",
  "response": "{\"risk_level\": \"high\", \"reason\": \"Recursive force delete without confirmation\"}",
  "done": true,
  "total_duration": 1234567890,
  "eval_count": 42,
  "eval_duration": 987654321
}
```

## Error Handling

### HTTP Status Codes

- **200 OK**: Successful classification or analysis
- **202 Accepted**: Command accepted for execution (async)
- **400 Bad Request**: Invalid command syntax or parameters
- **422 Unprocessable Entity**: Command validation failed
- **500 Internal Server Error**: Service error (classification engine failure)
- **503 Service Unavailable**: Ollama not responding

### Error Response Format

```json
{
  "error": "classification_failed",
  "message": "Unable to determine risk level for command",
  "details": "Pattern matching timed out",
  "remediation": "Try simplifying the command or contact support"
}
```

## Performance Considerations

### Latency Targets (Phase 0)

- Command classification (rule-based): < 100ms
- LLM classification (Phase 4+): < 2 seconds (GPU), < 10 seconds (CPU)
- Process monitoring update: < 50ms
- Dashboard render: < 100ms

### Throughput

- AI Service: 100+ req/sec (rule-based)
- Core Engine: 10-50 processes monitored concurrently
- Ollama: 1-5 req/sec (model-dependent)

### Resource Usage

- Dashboard: ~100 MB RAM (browser)
- AI Service: ~50 MB RAM
- Core Engine: ~10 MB RAM per monitored process
- Ollama: 1-8 GB VRAM (GPU) or 2-16 GB RAM (CPU), model-dependent

## Scalability and Future Work

### Phase 0 (Foundation)

- Single-user, single-machine
- No horizontal scaling
- Local development environment

### Future Phases

- **Phase 1-2**: Signal safety and process improvements
- **Phase 3**: Filesystem isolation (mount namespaces)
- **Phase 4**: Advanced LLM integration (context-aware suggestions)
- **Phase 5**: Authentication and multi-user support
- **Phase 6**: Audit logging and compliance

### Multi-User Considerations (Future)

- Per-user secrets in `~/.config/intellishell/`
- User-specific cgroup hierarchies
- Authentication token in HTTP headers
- Audit log per user

---

For implementation details, see:
- [docs/ENVIRONMENT.md](./ENVIRONMENT.md) - Setup and toolchain
- [docs/SECURITY.md](./SECURITY.md) - Security model and known issues
- [.kiro/specs/phase-0-foundation/](../.kiro/specs/phase-0-foundation/) - Phase 0 requirements and design
