# PowerShell script to run IntelliShell with Docker
# Run this from Windows PowerShell in the project root

Write-Host "IntelliShell Docker Setup" -ForegroundColor Cyan
Write-Host ""

# Check if Docker is running
try {
    docker info | Out-Null
} catch {
    Write-Host "Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Check if .env exists
if (-Not (Test-Path ".env")) {
    Write-Host ".env file not found. Creating from template..." -ForegroundColor Yellow
    Copy-Item ".env.docker.example" ".env"
    Write-Host "Please edit .env and add your GROQ_API_KEY" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Press Enter after editing .env to continue..." -ForegroundColor Yellow
    Read-Host
}

# Create data directory
if (-Not (Test-Path "data")) {
    New-Item -ItemType Directory -Path "data" | Out-Null
    Write-Host "Created data directory" -ForegroundColor Green
}

Write-Host "Building Docker images..." -ForegroundColor Cyan
docker-compose build

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Starting services..." -ForegroundColor Cyan
    docker-compose up -d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Services started!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Service URLs:" -ForegroundColor Cyan
        Write-Host "   Rust Server:  http://localhost:3000" -ForegroundColor White
        Write-Host "   AI Service:   http://localhost:8000" -ForegroundColor White
        Write-Host ""
        Write-Host "Useful commands:" -ForegroundColor Cyan
        Write-Host "   View logs:    docker-compose logs -f" -ForegroundColor White
        Write-Host "   Stop:         docker-compose down" -ForegroundColor White
        Write-Host "   Rebuild:      docker-compose up -d --build" -ForegroundColor White
        Write-Host ""
        Write-Host "Test health:" -ForegroundColor Cyan
        Write-Host "   curl http://localhost:3000/health" -ForegroundColor White
        Write-Host "   curl http://localhost:8000/health" -ForegroundColor White
    } else {
        Write-Host "Failed to start services" -ForegroundColor Red
        docker-compose logs
        exit 1
    }
} else {
    Write-Host "Build failed. Check output above." -ForegroundColor Red
    exit 1
}
