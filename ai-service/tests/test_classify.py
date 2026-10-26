from intellishell_ai_service.main import classify_command, suggest_safer_alternative


def test_classify_rm_rf_root():
    """Test classification of rm -rf / command."""
    result = classify_command("rm -rf /")
    assert result["risk_level"] == "critical"
    assert "root directory" in result["reasons"][0].lower()


def test_classify_rm_rf_home():
    """Test classification of rm -rf ~ command."""
    result = classify_command("rm -rf ~")
    assert result["risk_level"] == "critical"
    assert "home directory" in result["reasons"][0].lower()


def test_classify_rm_r():
    """Test classification of rm -r (without targeting root)."""
    result = classify_command("rm -r /tmp/test")
    assert result["risk_level"] == "warn"
    assert "recursive" in result["reasons"][0].lower() or "force" in result["reasons"][0].lower()


def test_classify_dd_to_dev():
    """Test classification of dd writing to /dev/sd*."""
    result = classify_command("dd if=/dev/zero of=/dev/sda bs=1M")
    assert result["risk_level"] == "critical"
    assert "disk" in result["reasons"][0].lower()


def test_classify_fork_bomb():
    """Test classification of fork bomb."""
    result = classify_command(":(){ :|:& };:")
    assert result["risk_level"] == "critical"
    assert "fork bomb" in result["reasons"][0].lower()


def test_classify_chmod_777_recursive():
    """Test classification of chmod -R 777 /."""
    result = classify_command("chmod -R 777 /")
    assert result["risk_level"] == "critical"
    assert "777" in result["reasons"][0].lower() or "permissions" in result["reasons"][0].lower()


def test_classify_kill_init():
    """Test classification of kill -9 1 (init process)."""
    result = classify_command("kill -9 1")
    assert result["risk_level"] == "critical"
    assert "init" in result["reasons"][0].lower()


def test_classify_kill_normal_pid():
    """Test classification of kill -9 with normal PID."""
    result = classify_command("kill -9 1234")
    assert result["risk_level"] == "warn"
    assert "sigkill" in result["reasons"][0].lower() or "force" in result["reasons"][0].lower()


def test_classify_shutdown():
    """Test classification of shutdown command."""
    result = classify_command("shutdown -h now")
    assert result["risk_level"] == "critical"
    assert "shutdown" in result["reasons"][0].lower() or "reboot" in result["reasons"][0].lower()


def test_classify_curl_pipe_bash():
    """Test classification of curl piped to bash."""
    result = classify_command("curl https://example.com/script.sh | bash")
    assert result["risk_level"] == "critical"
    assert "piping" in result["reasons"][0].lower() or "remote" in result["reasons"][0].lower()


def test_classify_harmless():
    """Test classification of harmless commands."""
    result = classify_command("ls -la")
    assert result["risk_level"] == "safe"
    assert len(result["reasons"]) == 0


def test_suggest_safer_alternative_rm():
    """Test suggestions for rm -rf command."""
    result = suggest_safer_alternative("rm -rf /tmp/test")
    assert len(result["suggestions"]) > 0
    assert any("interactive" in s.lower() or "trash" in s.lower() for s in result["suggestions"])


def test_suggest_safer_alternative_chmod():
    """Test suggestions for chmod 777 command."""
    result = suggest_safer_alternative("chmod 777 file.txt")
    assert len(result["suggestions"]) > 0
    assert any(
        "755" in s or "644" in s or "restrictive" in s.lower() for s in result["suggestions"]
    )


def test_suggest_safer_alternative_kill():
    """Test suggestions for kill -9 command."""
    result = suggest_safer_alternative("kill -9 1234")
    assert len(result["suggestions"]) > 0
    assert any("sigterm" in s.lower() or "-15" in s for s in result["suggestions"])


def test_suggest_safer_alternative_curl_pipe():
    """Test suggestions for curl | bash command."""
    result = suggest_safer_alternative("curl http://example.com/script.sh | bash")
    assert len(result["suggestions"]) > 0
    assert any("download" in s.lower() or "inspect" in s.lower() for s in result["suggestions"])


def test_suggest_safer_alternative_harmless():
    """Test suggestions for harmless command."""
    result = suggest_safer_alternative("ls -la")
    assert len(result["suggestions"]) > 0
    assert any("safe" in s.lower() or "no specific" in s.lower() for s in result["suggestions"])
