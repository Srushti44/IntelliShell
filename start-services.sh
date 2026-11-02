#!/bin/bash
# IntelliShell Service Startup Script
# Run this in WSL/Linux: bash start-services.sh

export GROQ_API_KEY="your_groq_api_key_here"

echo "================================="
echo "Starting IntelliShell Services"
echo "================================="
echo ""

# Get the script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check if running in WSL
if grep -qi microsoft /proc/version; then
    echo "✓ Running in WSL"
else
    echo "⚠ Warning: Not running in WSL. This script is designed for WSL/Linux."
fi

echo ""
echo "Starting services in separate terminals..."
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "Checking prerequisites..."
if ! command_exists cargo; then
    echo "✗ Cargo not found. Please install Rust."
    exit 1
fi
if ! command_exists python3; then
    echo "✗ Python3 not found. Please install Python 3."
    exit 1
fi
if ! command_exists npm; then
    echo "✗ npm not found. Please install Node.js."
    exit 1
fi
echo "✓ All prerequisites found"
echo ""

# Start Rust backend
echo "1. Starting Rust Backend (Core Engine)..."
echo "   Command: cd core-engine && cargo run --release --bin intellishell-server"
cd "$SCRIPT_DIR/core-engine"
cargo run --release --bin intellishell-server &
RUST_PID=$!
echo "   PID: $RUST_PID"
echo ""

# Wait a bit for Rust to start
sleep 3

# Start Python AI service
echo "2. Starting Python AI Service..."
echo "   Command: cd ai-service && uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000"
cd "$SCRIPT_DIR/ai-service"
python3 -m uvicorn intellishell_ai_service.main:app --host 127.0.0.1 --port 8000 &
PYTHON_PID=$!
echo "   PID: $PYTHON_PID"
echo ""

# Wait a bit for Python to start
sleep 3

# Start React frontend
echo "3. Starting React Frontend (Dashboard)..."
echo "   Command: cd dashboard && npm run dev"
cd "$SCRIPT_DIR/dashboard"
npm run dev &
REACT_PID=$!
echo "   PID: $REACT_PID"
echo ""

echo "================================="
echo "Services Started Successfully!"
echo "================================="
echo ""
echo "Service URLs:"
echo "  - Frontend: http://localhost:5173"
echo "  - Rust API: http://localhost:3000"
echo "  - Python AI: http://localhost:8000"
echo ""
echo "Process IDs:"
echo "  - Rust:   $RUST_PID"
echo "  - Python: $PYTHON_PID"
echo "  - React:  $REACT_PID"
echo ""
echo "To stop all services, run:"
echo "  kill $RUST_PID $PYTHON_PID $REACT_PID"
echo ""
echo "Or press Ctrl+C to stop this script (services will continue running)"
echo ""

# Keep script running
wait
