"""
IntelliShell AI Service - Complete Phase 5 Integration

FastAPI service that generates Linux commands from natural language using AI,
with independent risk classification, impact analysis, and execution integration
with the Rust core engine.

Architecture:
- Primary: Local Ollama (phi3:mini)
- Fallback: Groq API (if configured)
- Risk Classification: Deterministic rule-based patterns
- Impact Analysis: Real filesystem/process inspection
- Execution: Integrated with Rust core engine via HTTP
- Persistence: Command history in Rust database
"""

import os
import sys
import time
import uuid
from typing import Optional

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

import requests
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Local imports
from intellishell_ai_service.risk import classify_command, suggest_safer_alternative, RiskLevel
from intellishell_ai_service.impact import analyze_impact

# Provider imports (if available)
try:
    from intellishell_ai_service.providers import (
        ProviderError,
        ProviderMalformedOutput,
        ProviderTimeout,
        ProviderUnavailable,
        ProviderManager,
        OllamaProvider,
        GroqProvider,
    )
    PROVIDERS_AVAILABLE = True
except ImportError:
    PROVIDERS_AVAILABLE = False
    print("Warning: Provider module not available, AI features will be limited")

app = FastAPI(
    title="IntelliShell AI Service",
    description="AI-powered Linux command generation with safety analysis",
    version="0.5.0",
)

# CORS for dashboard integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://127.0.0.1:5173", "http://localhost:3000"],
    allow_methods=["*"],
    allow_headers=["*"],
)

# Rust core engine URL
CORE_ENGINE_URL = os.getenv("CORE_ENGINE_URL", "http://localhost:3000")

# Token management for confirmation flow
PENDING_COMMANDS = {}
TOKEN_TTL_SECONDS = 300  # 5 minutes

# Activity audit log (in-memory, capped at 100)
ACTIVITY_LOG = []
ACTIVITY_LOG_MAX = 100

# Initialize provider manager if available
provider_manager = None
if PROVIDERS_AVAILABLE:
    try:
        # Use Groq as PRIMARY provider (no Ollama)
        groq_key = os.getenv("GROQ_API_KEY")
        groq_model = os.getenv("GROQ_MODEL", "openai/gpt-oss-120b")
        
        if groq_key:
            print(f"✅ Initializing Groq provider with model: {groq_model}")
            groq_provider = GroqProvider(api_key=groq_key, model_name=groq_model)
            provider_manager = ProviderManager(primary=groq_provider, fallback=None)
            print("✅ Groq provider initialized successfully")
        else:
            print("❌ GROQ_API_KEY not found in environment")
    except Exception as e:
        print(f"❌ Failed to initialize provider manager: {e}")

SYSTEM_INSTRUCTIONS = """You are an expert Linux command generator and system administrator. Your job is to translate natural language requests into precise, safe, and effective Linux commands.

CRITICAL RULES:
1. Respond with ONLY a JSON object - no markdown, no code blocks, no extra text
2. The JSON must have exactly these two fields:
   {"command": "<single shell command>", "explanation": "<clear one-sentence explanation>"}
3. Use ONLY real, standard Linux commands that actually exist (bash, coreutils, util-linux, etc.)
4. Generate commands that are safe by default - avoid destructive operations unless explicitly requested
5. Prefer non-destructive alternatives when possible (e.g., ls instead of rm, mv instead of dd)
6. Keep commands simple and focused on the specific task requested
7. Use proper quoting and escaping for file paths with spaces

COMMAND GENERATION GUIDELINES:
- File operations: Use ls, cat, head, tail, grep, find, cp, mv (avoid rm unless explicitly requested)
- Process management: Use ps, top, kill (with proper signal), systemctl status
- System info: Use df, du, free, uptime, uname, hostname
- Text processing: Use grep, sed, awk, cut, sort, uniq, wc
- Network: Use curl, wget, ping, netstat, ss, ip (avoid iptables/firewall changes)
- Permissions: Use chmod, chown (avoid recursive changes to system directories)
- Package management: Use apt list, dpkg -l (avoid install/remove unless explicitly requested)

SAFETY PRINCIPLES:
- Never generate commands that:
  * Delete system directories (/, /etc, /bin, /usr, /var)
  * Modify critical system files without explicit request
  * Run fork bombs or infinite loops
  * Write to raw disk devices (/dev/sda, etc.)
  * Shutdown/reboot without explicit request
  * Download and execute untrusted code
- For potentially destructive operations, use the safest minimal approach
- Add appropriate flags for safety (e.g., -i for interactive confirmation)

WSL/WINDOWS PATH RULES:
This system runs in WSL2 on Windows. Commands execute inside WSL (Ubuntu/Linux shell).
- Windows drives are mounted under /mnt/<lowercase-drive-letter>/ (e.g. C: → /mnt/c/, D: → /mnt/d/)
- CRITICAL: Always use lowercase drive letters: /mnt/d/ NOT /mnt/D/
- CRITICAL: Paths containing spaces MUST be wrapped in double quotes: ls "/mnt/d/3rd year" NOT ls /mnt/d/3rd year
- When user mentions 'D drive', 'C drive', 'Windows drive', map to /mnt/d/, /mnt/c/ etc.
- When user mentions 'Documents', 'Desktop', 'Downloads', use /mnt/c/Users/$USER/<FolderName> with quotes if needed
- The current workspace is at /mnt/d/3rd year/os project/IntelliShell

EXAMPLES:
Request: "show me the largest files in my home directory"
{"command": "du -ah ~ | sort -rh | head -20", "explanation": "Lists the 20 largest files/directories in your home folder sorted by size"}

Request: "find all log files modified today"
{"command": "find /var/log -name '*.log' -mtime 0 -type f", "explanation": "Searches /var/log for files with .log extension modified in the last 24 hours"}

Request: "check memory usage"
{"command": "free -h", "explanation": "Displays system memory usage in human-readable format"}

Request: "list files in D drive 3rd year folder"
{"command": "ls \"/mnt/d/3rd year\"", "explanation": "Lists files in the Windows D drive '3rd year' directory, accessed via WSL mount with proper quoting for the space"}

Request: "show my Windows documents"
{"command": "ls \"/mnt/c/Users/$USER/Documents\"", "explanation": "Lists files in your Windows Documents folder via WSL's C drive mount"}

Request: "open D drive"
{"command": "ls /mnt/d/", "explanation": "Lists the contents of the Windows D drive via WSL mount point"}

Request: "find large files on C drive"
{"command": "du -ah \"/mnt/c/Users/$USER\" | sort -rh | head -20", "explanation": "Finds the 20 largest files/folders in your Windows user profile via WSL"}

Now respond to the user's request with ONLY the JSON object:"""


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
    duration_ms: Optional[int] = None


class TroubleshootRequest(BaseModel):
    """Troubleshooting analysis request."""
    issue: str


class TroubleshootResponse(BaseModel):
    """Troubleshooting analysis result."""
    issue: str
    analysis: str
    evidence: dict
    recommendations: list
    provider_used: str


# ===== API Endpoints =====


@app.get("/health")
def health_check():
    """
    Check service health and AI provider availability.
    
    Returns:
        status: "ok" if service is healthy
        ai_available: bool
        core_engine_reachable: bool
    """
    # Check AI provider
    ai_available = False
    if provider_manager:
        ai_available = provider_manager.primary.health_check()
    
    # Check core engine
    core_reachable = False
    try:
        response = requests.get(f"{CORE_ENGINE_URL}/health", timeout=2)
        core_reachable = response.status_code == 200
    except:
        pass
    
    status = "ok" if ai_available and core_reachable else "degraded"
    
    return {
        "status": status,
        "ai_available": ai_available,
        "core_engine_reachable": core_reachable,
        "providers_loaded": PROVIDERS_AVAILABLE,
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
    if not provider_manager:
        raise HTTPException(
            status_code=503,
            detail="AI provider not available - check service configuration"
        )
    
    full_prompt = f"{SYSTEM_INSTRUCTIONS}\n\nRequest: {request.query}"
    
    # Generate using provider manager (handles fallback automatically)
    try:
        response = provider_manager.generate(full_prompt, timeout_seconds=120)
    except ProviderTimeout as e:
        raise HTTPException(status_code=504, detail=f"AI provider timed out: {e}")
    except ProviderUnavailable as e:
        raise HTTPException(status_code=502, detail=f"AI provider unavailable: {e}")
    except ProviderMalformedOutput as e:
        raise HTTPException(status_code=502, detail=f"AI provider returned invalid output: {e}")
    except ProviderError as e:
        raise HTTPException(status_code=502, detail=f"AI provider error: {e}")
    
    parsed = response.parsed_json
    raw_output = response.raw_output
    
    # Extract required fields with validation
    command = parsed.get("command")
    if not command:
        raise HTTPException(
            status_code=502,
            detail=f"Model did not return a 'command' field. Raw output: {raw_output!r}",
        )
    
    # Get explanation (with fallback)
    explanation = parsed.get("explanation", "(model did not provide an explanation)")
    
    # Deterministic safer alternative (rule-based, not AI-generated)
    safer_alternative = suggest_safer_alternative(command)
    
    # Independent risk classification (deterministic, not AI-generated)
    risk_level_enum, risk_reason = classify_command(command)
    risk_level = risk_level_enum.value
    
    # Real impact analysis (filesystem/process inspection)
    impact_context = analyze_impact(command)
    
    # Generate one-time confirmation token
    token = str(uuid.uuid4())
    PENDING_COMMANDS[token] = {
        "command": command,
        "risk_level": risk_level,
        "query": request.query,
        "created_at": time.time(),
    }
    
    # Log activity
    log_activity("generated", request.query, command, risk_level)
    
    return CommandResponse(
        query=request.query,
        command=command,
        explanation=explanation,
        safer_alternative=safer_alternative,
        impact_context=impact_context,
        risk_level=risk_level,
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
    
    # Forward to core-engine for execution
    try:
        response = requests.post(
            f"{CORE_ENGINE_URL}/execute",
            json={
                "command": entry["command"],
                "risk_level": entry["risk_level"],
                "timeout_seconds": 30,
            },
            timeout=35,
        )
        response.raise_for_status()
    except requests.exceptions.RequestException as e:
        raise HTTPException(status_code=502, detail=f"Failed to reach core-engine: {e}")
    
    result = response.json()
    
    # Log confirmation/block event
    event_type = "blocked" if result.get("method") == "blocked" else "confirmed"
    log_activity(
        event_type, entry["query"], entry["command"], entry["risk_level"], result.get("message", "")
    )
    
    return ConfirmResponse(**result)


@app.post("/troubleshoot", response_model=TroubleshootResponse)
def troubleshoot_system(request: TroubleshootRequest):
    """
    Analyze system issues using real telemetry and AI.
    
    Workflow:
    1. Fetch real system snapshot from core-engine
    2. Send telemetry + user issue to AI provider
    3. AI analyzes actual data
    4. Return findings with evidence
    
    Args:
        request: TroubleshootRequest with issue description
    
    Returns:
        TroubleshootResponse with analysis and recommendations
    """
    if not provider_manager:
        raise HTTPException(
            status_code=503,
            detail="AI provider not available"
        )
    
    # Fetch real system snapshot
    try:
        response = requests.get(f"{CORE_ENGINE_URL}/snapshot", timeout=5)
        response.raise_for_status()
        snapshot = response.json()
    except requests.exceptions.RequestException as e:
        raise HTTPException(
            status_code=502,
            detail=f"Failed to fetch system snapshot: {e}"
        )
    
    # Build analysis prompt
    analysis_prompt = f"""You are a system administrator analyzing performance issues.

User reports: {request.issue}

Real system telemetry:
- CPU Usage: {snapshot['cpu_percent']:.1f}%
- Memory: {snapshot['memory_used_bytes'] / (1024**3):.1f}GB / {snapshot['memory_total_bytes'] / (1024**3):.1f}GB ({snapshot['memory_used_percent']:.1f}% used)
- Top processes by CPU:
"""
    
    # Add top 5 processes
    for proc in snapshot['processes'][:5]:
        analysis_prompt += f"  * PID {proc['pid']} ({proc['name']}): {proc['cpu_percent']:.1f}% CPU, {proc['memory_kb']/1024:.0f}MB RAM\n"
    
    analysis_prompt += """
Based on this ACTUAL telemetry data (not hypothetical):
1. What is the most likely cause of the reported issue?
2. What evidence supports this diagnosis?
3. What are 2-3 specific actions to investigate or resolve?

Respond in JSON format:
{
  "diagnosis": "<primary likely cause>",
  "confidence": "<high/medium/low>",
  "evidence": ["<fact 1 from telemetry>", "<fact 2>"],
  "recommendations": ["<action 1>", "<action 2>", "<action 3>"]
}"""
    
    # Get AI analysis
    try:
        ai_response = provider_manager.generate(analysis_prompt, timeout_seconds=60)
        analysis_data = ai_response.parsed_json
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"AI analysis failed: {e}")
    
    # Extract fields
    diagnosis = analysis_data.get("diagnosis", "Unable to determine root cause")
    confidence = analysis_data.get("confidence", "low")
    evidence = analysis_data.get("evidence", [])
    recommendations = analysis_data.get("recommendations", [])
    
    # Build analysis text
    analysis_text = f"Diagnosis: {diagnosis} (confidence: {confidence})"
    if evidence:
        analysis_text += "\n\nEvidence:\n" + "\n".join(f"• {e}" for e in evidence)
    
    return TroubleshootResponse(
        issue=request.issue,
        analysis=analysis_text,
        evidence=snapshot,
        recommendations=recommendations,
        provider_used=ai_response.provider.value,
    )


@app.get("/activity")
def get_activity():
    """
    Get audit log of all command generation/confirmation/block events.
    
    Returns:
        events: List of activity events in reverse chronological order
    """
    return {"events": ACTIVITY_LOG}


@app.get("/history")
def get_history():
    """
    Get command history from core-engine database.
    
    Returns:
        history: List of CommandHistoryRecord from core-engine
    """
    try:
        response = requests.get(f"{CORE_ENGINE_URL}/history", timeout=5)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        raise HTTPException(
            status_code=502,
            detail=f"Failed to fetch history from core-engine: {e}"
        )


# ===== Legacy Endpoints (for compatibility) =====

class ClassifyRequest(BaseModel):
    command: str

@app.post("/classify")
def classify_endpoint(request: ClassifyRequest):
    """Classify command risk level (legacy endpoint)."""
    risk_level, risk_reason = classify_command(request.command)
    return {
        "risk_level": risk_level.value,
        "reason": risk_reason,
        "command": request.command,
    }


class SuggestRequest(BaseModel):
    command: str

@app.post("/suggest")
def suggest_endpoint(request: SuggestRequest):
    """Suggest safer alternatives (legacy endpoint)."""
    suggestion = suggest_safer_alternative(request.command)
    return {
        "command": request.command,
        "safer_alternative": suggestion if suggestion else "Command is already reasonably safe",
    }


if __name__ == "__main__":
    import uvicorn
    
    # Bind to 127.0.0.1:8000 only
    uvicorn.run(app, host="127.0.0.1", port=8000)
