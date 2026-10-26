# IntelliShell 🚀

**AI-Powered System Intent Engine for Safe Linux Command Execution**

IntelliShell is an intelligent command-line assistant that translates natural language into safe, executable Linux commands using AI. It features real-time system monitoring, risk assessment, and execution controls designed for both technical and non-technical users.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.10+-blue.svg)](https://www.python.org/)
[![React](https://img.shields.io/badge/react-18.0+-61DAFB.svg)](https://reactjs.org/)

---

## 🎯 Features

### 🤖 AI Command Generation
- **Natural Language to Commands**: Type what you want in plain English, get the right Linux command
- **Multi-Provider AI**: Groq API (cloud) with Ollama (local) fallback support
- **Smart Risk Classification**: Automatic safety analysis (SAFE → LOW → MEDIUM → HIGH → CRITICAL)
- **Safer Alternatives**: Suggests less destructive options when available

### 🛡️ Safety & Security
- **Risk-Based Execution Routing**:
  - 🟢 SAFE/LOW: Direct execution
  - 🟡 MEDIUM: Sandboxed execution (cgroups)
  - 🔴 HIGH/CRITICAL: Blocked with explanation
- **Command History**: Persistent SQLite database tracks all executions
- **Execution Confirmation**: Two-step process (generate → review → execute)
- **Impact Analysis**: Filesystem and process inspection before execution

### 📊 Real-Time System Monitoring
- **CPU, Memory, Disk, Network**: Live telemetry updated every 2 seconds
- **Process Management**: View, search, and control running processes
- **Swiss-Minimalist Design**: Clean, professional interface with brown/terracotta/olive color palette
- **Interactive Charts**: Real-time visualization using Recharts

### 🎨 Modern Tech Stack
- **Backend**: Rust (Axum) - High-performance core engine
- **AI Service**: Python (FastAPI) - Groq/Ollama integration
- **Frontend**: React 18 + Vite - Fast, responsive dashboard
- **Database**: SQLite - Persistent command history

---

## 🏗️ Architectureer

```
┌─────────────────────────────────────────────────────────┐
│                   React Dashboard                        │
│              (Port 5173 - Vite Dev)                     │
│  • AI Command Assistant  • System Monitoring             │
│  • Process Manager       • Command History              │
└────────────┬────────────────────────────┬───────────────┘
             │                            │
             ▼                            ▼
┌────────────────────────┐  ┌────────────────────────────┐
│   Python AI Service    │  │    Rust Core Engine        │
│   (Port 8000 FastAPI)  │  │   (Port 3000 Axum)         │
│                        │  │                            │
│  • Groq API Client     │  │  • Command Execution       │
│  • Ollama Fallback     │  │  • Risk Routing            │
│  • Risk Classifier     │  │  • Sandbox (cgroups)       │
│  • Token Management    │  │  • System Telemetry        │
└────────────────────────┘  └──────────┬─────────────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │   SQLite Database    │
                            │  (Command History)   │
                            └──────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.88+**: [Install Rust](https://rustup.rs/)
- **Python 3.10+**: [Install Python](https://www.python.org/downloads/)
- **Node.js 18+**: [Install Node.js](https://nodejs.org/)
- **WSL2** (Windows users): [Install WSL2](https://docs.microsoft.com/en-us/windows/wsl/install)
- **Groq API Key**: [Get free key](https://console.groq.com/) (or use Ollama locally)

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/Srushti44/IntelliShell.git
cd IntelliShell
```

2. **Configure AI Service**

Create `ai-service/.env`:
```bash
AI_PROVIDER=groq
GROQ_API_KEY=your_groq_api_key_here
GROQ_MODEL=openai/gpt-oss-120b
CORE_ENGINE_URL=http://localhost:3000
PROVIDER_TIMEOUT=120
TOKEN_TTL_SECONDS=300
```

Or for local Ollama:
```bash
AI_PROVIDER=ollama
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=phi3:mini
```

3. **Configure Core Engine**

Create `core-engine/.env`:
```bash
DATABASE_URL=sqlite:./intellishell.db
LOG_LEVEL=info
BIND_ADDRESS=127.0.0.1:3000
```

### Running IntelliShell

Open **3 terminal windows**:

**Terminal 1 - Rust Core Engine** (WSL/Linux):
```bash
cd core-engine
cargo run --release --bin intellishell-server
```

**Terminal 2 - Python AI Service** (WSL/Linux):
```bash
cd ai-service
# With environment variables
export GROQ_API_KEY='your_key_here'
export GROQ_MODEL='openai/gpt-oss-120b'
python3 -m uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000
```

**Terminal 3 - Frontend** (Windows/Mac/Linux):
```bash
cd dashboard
npm install
npm run dev
```

**Access**: Open http://localhost:5173 in your browser

---

## 📖 Usage

### AI Command Assistant

1. Navigate to **"AI Command Assistant"** in the sidebar
2. Type your request in plain English:
   - ✅ "list all files in the current directory"
   - ✅ "find large files bigger than 100MB"
   - ✅ "show disk usage"
3. Click **"Generate Command"**
4. Review the command, risk level, and explanation
5. Click **"Execute Command"** to run it

### System Monitoring

- **Overview**: Quick snapshot of CPU, Memory, Disk, Network
- **Monitoring**: Detailed real-time charts and metrics
- **Process Manager**: View and control running processes
- **Command History**: Audit trail of all executed commands

---

## 🧪 Testing

### Rust Core Engine
```bash
cd core-engine
cargo test -- --test-threads=1
```

**Test Coverage**: 41 tests including:
- Execution routing (LOW, MEDIUM, HIGH, CRITICAL)
- History persistence
- System telemetry (CPU, memory, disk, network)
- Sandbox execution

### Python AI Service
```bash
cd ai-service
python3 -m pytest tests/test_risk.py -v
```

**Test Coverage**: 28 tests including:
- Risk classification patterns
- CRITICAL commands (rm -rf /, dd, fork bombs)
- HIGH commands (recursive deletion, chmod 777 /)
- MEDIUM commands (kill, sudo, systemctl)
- SAFE commands (ls, cat, grep)

---

## 🔒 Security Model

### Risk Levels

| Level | Examples | Action |
|-------|----------|--------|
| 🟢 **SAFE** | `ls`, `cat`, `grep` | Execute directly |
| 🟢 **LOW** | `echo`, `pwd`, `whoami` | Execute directly |
| 🟡 **MEDIUM** | `kill PID`, `sudo`, `systemctl` | Sandbox or graceful degradation |
| 🔴 **HIGH** | `rm -r`, `chmod 777 /` | **BLOCKED** |
| 🔴 **CRITICAL** | `rm -rf /`, `dd of=/dev/sda`, `:(){:\|:&};:` | **BLOCKED** |

### Execution Flow

```
Natural Language Query
        ↓
   AI Generation
        ↓
Risk Classification (Deterministic Rules)
        ↓
    ┌───────────────────────┐
    │  Risk Level Check     │
    └───────┬───────────────┘
            ↓
    ┌───────┴────────┬──────────┬─────────┐
    │                │          │         │
  SAFE/LOW       MEDIUM      HIGH     CRITICAL
    │                │          │         │
    ↓                ↓          ↓         ↓
Direct Exec    Sandbox Exec  BLOCK    BLOCK
    │                │          │         │
    └────────┬───────┴──────────┴─────────┘
             ↓
      Record to History
```

---

## 🛠️ Development

### Project Structure

```
IntelliShell/
├── core-engine/           # Rust backend
│   ├── src/
│   │   ├── server.rs     # Axum HTTP server, telemetry
│   │   ├── database.rs   # SQLite operations
│   │   ├── process.rs    # Process management
│   │   ├── sandbox.rs    # Cgroup sandboxing
│   │   └── lib.rs
│   └── Cargo.toml
│
├── ai-service/            # Python AI service
│   ├── src/intellishell_ai_service/
│   │   ├── main.py       # FastAPI app, endpoints
│   │   ├── providers.py  # Groq/Ollama clients
│   │   ├── risk.py       # Risk classification
│   │   └── impact.py     # Impact analysis
│   ├── tests/
│   └── pyproject.toml
│
├── dashboard/             # React frontend
│   ├── src/
│   │   ├── pages/        # Page components
│   │   ├── components/   # Reusable UI components
│   │   ├── hooks/        # React Query hooks
│   │   ├── lib/          # API client, utilities
│   │   └── styles/       # Swiss design system
│   └── package.json
│
└── .agents/               # Development artifacts
    └── tasks/            # Phase completion reports
```

### Building for Production

```bash
# Rust (optimized)
cd core-engine
cargo build --release

# Python (with uv)
cd ai-service
uv sync

# Frontend (static build)
cd dashboard
npm run build
```

---

## 📊 Phase Completion Status

IntelliShell has completed **8 phases** of development:

- ✅ **Phase 0**: Foundation (Rust + Python + React scaffold)
- ✅ **Phase 1**: Core execution engine with risk routing
- ✅ **Phase 2**: AI command generation (Groq + Ollama)
- ✅ **Phase 3**: System monitoring and telemetry
- ✅ **Phase 4**: Database persistence and history
- ✅ **Phase 5**: Impact analysis and safer alternatives
- ✅ **Phase 6**: Process management and control
- ✅ **Phase 7**: Swiss-style frontend redesign
- ✅ **Phase 8**: Bug fixes, Windows/WSL path support, comprehensive testing

**Test Results**: 69/69 tests passing (41 Rust + 28 Python)

See `.agents/tasks/PHASE8_FINAL_RELEASE_REPORT.md` for detailed audit.

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Coding Standards

- **Rust**: Run `cargo fmt` and `cargo clippy`
- **Python**: Run `ruff check` and `ruff format`
- **JavaScript**: Run `npm run lint`
- **Tests**: Add tests for new features

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- **Rust Community**: For the amazing ecosystem (Axum, Tokio, SQLite)
- **Groq**: For fast, free LLM API
- **Ollama**: For local LLM capabilities
- **React Team**: For the fantastic UI framework
- **WSL Team**: For seamless Linux integration on Windows

---

## 📧 Contact

**Project Maintainer**: Srushti  
**GitHub**: [@Srushti44](https://github.com/Srushti44)  
**Repository**: [IntelliShell](https://github.com/Srushti44/IntelliShell)

---

## 🗺️ Roadmap

### Planned Features
- [ ] Windows native monitoring (host system metrics)
- [ ] Multi-user support with authentication
- [ ] Command templates and favorites
- [ ] Shell script generation
- [ ] Advanced sandbox with network isolation
- [ ] Docker/Kubernetes integration
- [ ] Cloud deployment guides

---

**Built with ❤️ using Rust, Python, and React**
