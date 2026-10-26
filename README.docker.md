# IntelliShell Docker Setup

This guide explains how to build and run IntelliShell using Docker, avoiding WSL filesystem issues.

## Prerequisites

1. **Docker Desktop for Windows** - [Download here](https://www.docker.com/products/docker-desktop/)
   - Make sure it's running (you should see the Docker icon in system tray)
   
2. **Groq API Key** - Get from [console.groq.com](https://console.groq.com/)

3. **Ollama (optional)** - If you want local AI models
   - Install from [ollama.ai](https://ollama.ai/)
   - Run: `ollama pull phi3:mini`

## Quick Start

### Option 1: PowerShell Script (Easiest)

```powershell
# From project root (d:\3rd year\os project\IntelliShell)
.\docker-run.ps1
```

The script will:
1. Check if Docker is running
2. Create `.env` if needed
3. Build both services
4. Start containers
5. Show service URLs

### Option 2: Manual Commands

```powershell
# 1. Create .env file
Copy-Item .env.docker.example .env
# Edit .env and add your GROQ_API_KEY

# 2. Build images
docker-compose build

# 3. Start services
docker-compose up -d

# 4. View logs
docker-compose logs -f
```

## Service URLs

Once running:
- **Rust Server:** http://localhost:3000
- **Python AI Service:** http://localhost:8000

## Testing

### Health Checks
```powershell
# Test Rust server
curl http://localhost:3000/health

# Test Python AI service
curl http://localhost:8000/health
```

### Generate AI Command
```powershell
curl -X POST http://localhost:8000/generate-command `
  -H "Content-Type: application/json" `
  -d '{"task": "list all files in current directory"}'
```

### Get System Snapshot
```powershell
curl http://localhost:3000/snapshot
```

## Docker Commands

```powershell
# View logs (all services)
docker-compose logs -f

# View logs (specific service)
docker-compose logs -f rust-server
docker-compose logs -f ai-service

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Rebuild after code changes
docker-compose up -d --build

# Restart a service
docker-compose restart rust-server

# Check running containers
docker-compose ps

# Enter a container shell
docker-compose exec rust-server /bin/bash
docker-compose exec ai-service /bin/bash
```

## Troubleshooting

### Docker not running
```
Error: Cannot connect to Docker daemon
```
**Solution:** Start Docker Desktop from Windows Start menu

### Build fails with "Rust version not found"
**Solution:** The Dockerfile uses `rust:1.88-slim` - if this version doesn't exist yet, change to `rust:latest` in `Dockerfile.rust`

### AI service can't reach Ollama
**Solution:** Make sure Ollama is running on host machine:
```powershell
ollama serve
```

### Permission denied errors
**Solution:** The containers run as non-root. Check `data/` directory permissions:
```powershell
# Give everyone write access to data directory
icacls "data" /grant Everyone:F /T
```

### Port already in use
```
Error: port 3000 is already allocated
```
**Solution:** Stop the conflicting service or change ports in `docker-compose.yml`

## Architecture

```
┌─────────────────────┐
│   Windows Host      │
│                     │
│  Docker Desktop     │
│  ┌───────────────┐  │
│  │ rust-server   │  │  Port 3000
│  │  (Port 3000)  │◄─┼──────────── HTTP Requests
│  └───────┬───────┘  │
│          │          │
│          ▼          │
│  ┌───────────────┐  │
│  │ ai-service    │  │  Port 8000
│  │  (Port 8000)  │◄─┼──────────── HTTP Requests
│  └───────┬───────┘  │
│          │          │
│          ▼          │
│  ┌───────────────┐  │
│  │   Ollama      │  │  Port 11434
│  │ (host machine)│  │
│  └───────────────┘  │
│                     │
│  Shared Volume:     │
│  ./data → /app/data │
└─────────────────────┘
```

## Database Persistence

The SQLite database is stored in `./data/intellishell.db` on your Windows host. This means:
- ✅ Data persists across container restarts
- ✅ You can access the database directly from Windows
- ✅ Backups are easy (just copy the `data/` folder)

## Development Workflow

### After changing Rust code:
```powershell
docker-compose up -d --build rust-server
```

### After changing Python code:
```powershell
docker-compose up -d --build ai-service
```

### After changing both:
```powershell
docker-compose up -d --build
```

## Performance Notes

- **First build:** 5-10 minutes (downloads Rust toolchain, compiles dependencies)
- **Subsequent builds:** 1-2 minutes (uses Docker layer cache)
- **Startup time:** < 5 seconds

## Security

The Docker setup:
- ✅ Runs as non-root user inside containers
- ✅ Only exposes necessary ports
- ✅ Isolates services in a private network
- ✅ Keeps secrets in `.env` (not committed to git)

## Next Steps

After successful deployment:
1. Run integration tests: See `tests/phase5_integration_test.sh`
2. Test Phase 0-4 features: Use CLI inside `rust-server` container
3. Monitor system: Check `/snapshot` endpoint
4. Review logs: `docker-compose logs -f`
