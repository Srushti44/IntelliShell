#!/bin/bash
cd ~/projects/intellishell
export PATH="$HOME/.local/bin:$PATH"

# Load environment variables
export $(cat ~/projects/intellishell/.env | grep -v '^#' | xargs)

# Start service
cd ai-service/src
uvicorn intellishell_ai_service.main:app --host 0.0.0.0 --port 8000
