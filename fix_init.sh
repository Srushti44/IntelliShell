#!/bin/bash
cat > ~/projects/intellishell/ai-service/src/intellishell_ai_service/__init__.py << 'INITEOF'
"""IntelliShell AI Service - AI-assisted Linux command classification and analysis."""

__version__ = "0.1.0"

from .main import app

__all__ = [
    "app",
]
INITEOF
echo "Fixed __init__.py"
