"""
IntelliShell AI Service - Phase 4: Command Generation and Risk Classification

FastAPI service that generates Linux commands from natural language using AI,
with independent risk classification and impact analysis.

Architecture:
- Primary: Local Ollama (phi3:mini)
- Fallback: Groq API (if configured)
- Risk Classification: Rule-based deterministic patterns
- Impact Analysis: Real filesystem/process inspection
"""

import json
import re
import time
import uuid
from typing import Optional

import requests
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from app.impact import analyze_impact
from app.providers import (
    ProviderError,
    ProviderMalformedOutput,
    ProviderTimeout,
    ProviderType,
    ProviderUnavailable,
    create_provider_manager,
)
from app.risk import classify_command, suggest_safer_alternative

app = FastAPI(
    title="IntelliShell AI Service",
    description="AI-powered Linux command generation with safety analysis",
    version="0.2.0",
)

# CORS for dashboard integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://127.0.0.1:5173"],
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize provider manager (primary Ollama, fallback Groq if configured)
provider_manager = create_provider_manager()

CORE_ENGINE_URL = "http://localhost:3000"

# Token management for confirmation flow
PENDING_COMMANDS = {}
TOKEN_TTL_SECONDS = 300  # 5 minutes

# Activity audit log (in-memory, capped)
ACTIVITY_LOG = []
ACTIVITY_LOG_MAX = 100

SYSTEM_INSTRUCTIONS = """You are a Linux command generator. Given a request, respond with ONLY a JSON object, nothing else -- no markdown, no code fences, no explanation outside the JSON.

The JSON must have exactly these fields:
{"command": "<the single shell command>", "explanation": "<one sentence explaining what it does>", "safer_alternative": "<a less destructive way to do the same thing, or empty string if the command is already safe>"}

Rules:
- Use only real, standard Linux/bash commands and flags that actually exist.
- The command must be a single line, no pipes to external scripts.
- For safer_alternative: if the command deletes, overwrites, or force-modifies something, suggest a safer version (e.g. moving to trash instead of rm -rf, using -i for interactive confirmation, dry-run flags where they exist). If the command is already safe (read-only, listing, etc.), set safer_alternative to an empty string.
- Do not wrap your response in ```json or ``` or any other formatting.
- Respond with the raw JSON object and nothing else."""


def log_activity(event_type: str, query: str, command: str, risk_level: str, detail: str = ""):
    """Log generation/confirmation/block events for audit trail."""
    ACTIVITY_LOG.insert(
        0,
        {
            "timestamp": time.time(),
            "event_type": event_type,  # "generated" | "confirmed" | "blocked"
            "query": query,
            "command": command,
            "risk_level": risk_level,
            "detail": detail,
        },
    )
    if len(ACTIVITY_LOG) > ACTIVITY_LOG_MAX:
        ACTIVITY_LOG.pop()


# ===== Request/Response Models =====


class CommandRequest(BaseModel):
    """Natural language command request."""

    query: str


class CommandResponse(BaseModel):
    """Structured command generation response."""

    query: str
    command: str
    explanation: str
    safer_alternative: str
    impact_context: str
    risk_level: str
    risk_reason: str
    raw_model_output: str
    confirmation_token: str
    provider_used: str
    generation_latency_ms: int


class ConfirmRequest(BaseModel):
    """Confirmation token for execution."""

    token: str


class ConfirmResponse(BaseModel):
    """Execution result."""

    executed: bool
    method: str
    exit_code: Optional[int]
    stdout: str
    stderr: str
    message: str


# ===== API Endpoints =====


@app.get("/health")
def health_check():
    """
    Check service health and AI provider availability.

    Returns:
        status: "ok" if both Ollama and service are healthy
        ollama_reachable: bool
        groq_configured: bool
    """
    ollama_healthy = provider_manager.primary.health_check()

    groq_configured = (
        provider_manager.fallback is not None and provider_manager.fallback.health_check()
    )

    status = "ok" if ollama_healthy else "degraded"

    return {
        "status": status,
        "ollama_reachable": ollama_healthy,
        "groq_configured": groq_configured,
    }


@app.post("/generate-command", response_model=CommandResponse)
def generate_command(request: CommandRequest):
    """
    Generate a Linux command from natural language query.

    Process:
    1. Send query to AI provider (Ollama primary, Groq fallback)
    2. Extract and validate structured JSON response
    3. Run independent risk classification
    4. Analyze filesystem/process impact
    5. Generate deterministic safer alternative
    6. Issue one-time confirmation token

    Args:
        request: CommandRequest with natural language query

    Returns:
        CommandResponse with command, explanation, risk assessment, and token

    Raises:
        HTTPException 502: Provider unavailable or malformed output
        HTTPException 504: Provider timeout
    """
    full_prompt = f"{SYSTEM_INSTRUCTIONS}\n\nRequest: {request.query}"

    # Generate using provider manager (handles fallback automatically)
    try:
        response = provider_manager.generate(full_prompt, timeout_seconds=120)
    except ProviderTimeout as e:
        raise HTTPException(
            status_code=504, detail=f"AI provider timed out: {e}"
        )
    except ProviderUnavailable as e:
        raise HTTPException(
            status_code=502, detail=f"AI provider unavailable: {e}"
        )
    except ProviderMalformedOutput as e:
        raise HTTPException(
            status_code=502,
            detail=f"AI provider returned invalid output: {e}",
        )
    except ProviderError as e:
        raise HTTPException(
            status_code=502, detail=f"AI provider error: {e}"
        )

    parsed = response.parsed_json
    raw_output = response.raw_output

    # Extract required fields with validation
    command = parsed.get("command")
    if not command:
        raise HTTPException(
            status_code=502,
            detail=f"Model did not return a 'command' field. Raw output: {raw_output!r}",
        )

    # Explanation is nice-to-have, gracefully handle missing/misnamed fields
    explanation = parsed.get("explanation")
    if not explanation:
        # Try to find explanation-like field
        for key, value in parsed.items():
            if key.lower().startswith("explan") and isinstance(value, str) and value.strip():
                explanation = value
                break

    if not explanation:
        explanation = "(model did not provide an explanation)"

    # Deterministic safer alternative (rule-based, not AI-generated)
    # We don't trust the model's safer_alternative field because testing showed
    # it was unreliable (correctly identified rm -rf as dangerous but returned
    # empty alternative). Our rule-based version is verifiable.
    safer_alternative = suggest_safer_alternative(command)

    # Independent risk classification (deterministic, not AI-generated)
    risk_level, risk_reason = classify_command(command)

    # Real impact analysis (filesystem/process inspection)
    impact_context = analyze_impact(command)

    # Generate one-time confirmation token
    token = str(uuid.uuid4())
    PENDING_COMMANDS[token] = {
        "command": command,
        "risk_level": risk_level.value,
        "created_at": time.time(),
    }

    # Log activity
    log_activity("generated", request.query, command, risk_level.value)

    return CommandResponse(
        query=request.query,
        command=command,
        explanation=explanation,
        safer_alternative=safer_alternative,
        impact_context=impact_context,
        risk_level=risk_level.value,
        risk_reason=risk_reason,
        raw_model_output=raw_output,
        confirmation_token=token,
        provider_used=response.provider.value,
        generation_latency_ms=response.latency_ms,
    )


@app.post("/confirm-execute", response_model=ConfirmResponse)
def confirm_execute(request: ConfirmRequest):
    """
    Execute a command using a valid confirmation token.

    Token-based confirmation ensures:
    - Only generated commands can be executed
    - Commands must be explicitly confirmed
    - Tokens are one-time use only
    - Tokens expire after TTL

    Args:
        request: ConfirmRequest with confirmation token

    Returns:
        ConfirmResponse with execution result from core-engine

    Raises:
        HTTPException 404: Invalid or used token
        HTTPException 410: Expired token
        HTTPException 502: Core-engine unavailable
    """
    # Pop token (one-time use)
    entry = PENDING_COMMANDS.pop(request.token, None)

    if entry is None:
        raise HTTPException(
            status_code=404, detail="Invalid or already-used confirmation token"
        )

    # Check expiration
    if time.time() - entry["created_at"] > TOKEN_TTL_SECONDS:
        raise HTTPException(
            status_code=410,
            detail="Confirmation token expired -- please regenerate the command",
        )

    # Forward to core-engine for sandboxed execution
    try:
        response = requests.post(
            f"{CORE_ENGINE_URL}/execute",
            json={"command": entry["command"], "risk_level": entry["risk_level"]},
            timeout=30,
        )
        response.raise_for_status()
    except requests.exceptions.RequestException as e:
        raise HTTPException(status_code=502, detail=f"Failed to reach core-engine: {e}")

    result = response.json()

    # Log confirmation/block event
    event_type = "blocked" if result.get("method") == "blocked" else "confirmed"
    log_activity(
        event_type, "", entry["command"], entry["risk_level"], result.get("message", "")
    )

    return ConfirmResponse(**result)


@app.get("/activity")
def get_activity():
    """
    Get audit log of all command generation/confirmation/block events.

    Returns:
        events: List of activity events in reverse chronological order
    """
    return {"events": ACTIVITY_LOG}


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="127.0.0.1", port=8000)
