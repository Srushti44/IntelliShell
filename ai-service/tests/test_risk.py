"""
Tests for risk classification module.
Tests the deterministic pattern-based risk classifier.
"""

import pytest
from intellishell_ai_service.risk import classify_command, RiskLevel


def test_classify_rm_rf_root():
    """Test classification of rm -rf / command."""
    risk_level, reason = classify_command("rm -rf /")
    assert risk_level == RiskLevel.CRITICAL
    assert "root directory" in reason.lower()


def test_classify_rm_rf_home():
    """Test classification of rm -rf ~ command."""
    risk_level, reason = classify_command("rm -rf ~")
    assert risk_level == RiskLevel.CRITICAL
    assert "home directory" in reason.lower()


def test_classify_rm_r():
    """Test classification of rm -r (without targeting root)."""
    risk_level, reason = classify_command("rm -r /tmp/test")
    assert risk_level == RiskLevel.HIGH
    assert "recursive" in reason.lower() or "force" in reason.lower()


def test_classify_dd_to_dev():
    """Test classification of dd writing to /dev/sd*."""
    risk_level, reason = classify_command("dd if=/dev/zero of=/dev/sda bs=1M")
    assert risk_level == RiskLevel.CRITICAL
    assert "disk" in reason.lower()


def test_classify_fork_bomb():
    """Test classification of fork bomb."""
    risk_level, reason = classify_command(":(){ :|:& };:")
    assert risk_level == RiskLevel.CRITICAL
    assert "fork bomb" in reason.lower()


def test_classify_chmod_777_recursive_root():
    """Test classification of chmod -R 777 /."""
    risk_level, reason = classify_command("chmod -R 777 /")
    assert risk_level == RiskLevel.CRITICAL
    assert "777" in reason.lower() or "permissions" in reason.lower()


def test_classify_chmod_777_file():
    """Test classification of chmod 777 on a single file."""
    risk_level, reason = classify_command("chmod 777 file.txt")
    assert risk_level == RiskLevel.MEDIUM
    assert "permissions" in reason.lower()


def test_classify_kill_init():
    """Test classification of kill -9 1 (init process)."""
    risk_level, reason = classify_command("kill -9 1")
    assert risk_level == RiskLevel.CRITICAL
    assert "init" in reason.lower()


def test_classify_kill_normal_pid():
    """Test classification of kill -9 with normal PID."""
    risk_level, reason = classify_command("kill -9 1234")
    assert risk_level == RiskLevel.MEDIUM
    assert "sigkill" in reason.lower() or "force" in reason.lower()


def test_classify_shutdown():
    """Test classification of shutdown command."""
    risk_level, reason = classify_command("shutdown -h now")
    assert risk_level == RiskLevel.CRITICAL
    assert "shutdown" in reason.lower() or "reboot" in reason.lower()


def test_classify_curl_pipe_bash():
    """Test classification of curl piped to bash."""
    risk_level, reason = classify_command("curl https://example.com/script.sh | bash")
    assert risk_level == RiskLevel.CRITICAL
    assert "piping" in reason.lower() or "remote" in reason.lower()


def test_classify_mkfs():
    """Test classification of filesystem format command."""
    risk_level, reason = classify_command("mkfs.ext4 /dev/sdb1")
    assert risk_level == RiskLevel.CRITICAL
    assert "filesystem" in reason.lower() or "format" in reason.lower()


def test_classify_sudo():
    """Test classification of sudo command."""
    risk_level, reason = classify_command("sudo apt-get install something")
    assert risk_level == RiskLevel.MEDIUM
    assert "elevated" in reason.lower() or "privileges" in reason.lower()


def test_classify_systemctl_stop():
    """Test classification of systemctl stop command."""
    risk_level, reason = classify_command("systemctl stop nginx")
    assert risk_level == RiskLevel.MEDIUM
    assert "service" in reason.lower() or "modification" in reason.lower()


def test_classify_rm_single_file():
    """Test classification of rm on a single file."""
    risk_level, reason = classify_command("rm file.txt")
    assert risk_level == RiskLevel.LOW
    assert "delete" in reason.lower()


def test_classify_kill_normal():
    """Test classification of kill (without -9)."""
    risk_level, reason = classify_command("kill 1234")
    assert risk_level == RiskLevel.LOW
    assert "termination" in reason.lower()


def test_classify_mv():
    """Test classification of mv command."""
    risk_level, reason = classify_command("mv old.txt new.txt")
    assert risk_level == RiskLevel.LOW
    assert "move" in reason.lower() or "rename" in reason.lower()


def test_classify_harmless_ls():
    """Test classification of harmless ls command."""
    risk_level, reason = classify_command("ls -la")
    assert risk_level == RiskLevel.SAFE
    assert "read-only" in reason.lower() or "safe" in reason.lower()


def test_classify_harmless_cat():
    """Test classification of cat command."""
    risk_level, reason = classify_command("cat file.txt")
    assert risk_level == RiskLevel.SAFE


def test_classify_harmless_grep():
    """Test classification of grep command."""
    risk_level, reason = classify_command("grep pattern file.txt")
    assert risk_level == RiskLevel.SAFE


def test_classify_empty_command():
    """Test classification of empty command."""
    risk_level, reason = classify_command("")
    assert risk_level == RiskLevel.SAFE
    assert "empty" in reason.lower()


def test_classify_whitespace_only():
    """Test classification of whitespace-only command."""
    risk_level, reason = classify_command("   ")
    assert risk_level == RiskLevel.SAFE


def test_classify_chown_root():
    """Test classification of chown on root filesystem."""
    risk_level, reason = classify_command("chown user:group /")
    assert risk_level == RiskLevel.HIGH
    assert "ownership" in reason.lower()


def test_classify_chmod_000():
    """Test classification of chmod 000 (removing all permissions)."""
    risk_level, reason = classify_command("chmod 000 file.txt")
    assert risk_level == RiskLevel.HIGH
    assert "permissions" in reason.lower()


def test_classify_mv_to_dev_null():
    """Test classification of mv to /dev/null."""
    risk_level, reason = classify_command("mv important.txt /dev/null")
    assert risk_level == RiskLevel.HIGH
    assert "dev/null" in reason.lower() or "permanent" in reason.lower()


def test_classify_apt_remove():
    """Test classification of package removal."""
    risk_level, reason = classify_command("apt-get remove nginx")
    assert risk_level == RiskLevel.MEDIUM
    assert "package" in reason.lower() or "removal" in reason.lower()


def test_classify_write_to_disk_device():
    """Test classification of direct write to disk device."""
    risk_level, reason = classify_command("echo test > /dev/sda")
    assert risk_level == RiskLevel.HIGH
    assert "disk" in reason.lower() or "device" in reason.lower()


def test_classify_case_insensitive():
    """Test that classification is case-insensitive."""
    risk_level1, _ = classify_command("RM -RF /")
    risk_level2, _ = classify_command("rm -rf /")
    assert risk_level1 == risk_level2
    assert risk_level1 == RiskLevel.CRITICAL


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
