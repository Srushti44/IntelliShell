#!/usr/bin/env python3
"""
Test script for dual-OS functionality
Tests command generation and risk classification for both Linux and Windows
"""

import sys
import os

# Add to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "ai-service", "src"))

from intellishell_ai_service.risk import classify_command as classify_linux
from intellishell_ai_service.windows_risk import classify_windows_command as classify_windows

print("=" * 80)
print("DUAL-OS FUNCTIONALITY TEST")
print("=" * 80)

# Test Linux classification
print("\n📌 LINUX COMMAND CLASSIFICATION:")
print("-" * 80)

linux_tests = [
    ("pwd", "safe"),
    ("ls -la", "safe"),
    ("rm test.txt", "low"),
    ("rm -rf /tmp/test", "medium"),
    ("rm -rf /", "critical"),
    ("sudo apt-get install vim", "medium"),
]

for cmd, expected_level in linux_tests:
    level, reason = classify_linux(cmd)
    status = "✅" if level.value == expected_level else "❌"
    print(f"{status} {cmd:30} -> {level.value:10} (expected: {expected_level:10})")
    if level.value != expected_level:
        print(f"   Reason: {reason}")

# Test Windows classification
print("\n📌 WINDOWS COMMAND CLASSIFICATION:")
print("-" * 80)

windows_tests = [
    ("Get-Location", "safe"),
    ("Get-Process", "safe"),
    ("Get-ChildItem", "safe"),
    ("New-Item test.txt", "low"),
    ("Remove-Item test.txt", "medium"),
    ("Stop-Service wuauserv", "medium"),
    ("Remove-Item C:\\ -Recurse -Force", "high"),
    ("Format-Volume C:", "critical"),
    ("Stop-Computer", "critical"),
]

for cmd, expected_level in windows_tests:
    level, reason = classify_windows(cmd)
    status = "✅" if level.value == expected_level else "❌"
    print(f"{status} {cmd:40} -> {level.value:10} (expected: {expected_level:10})")
    if level.value != expected_level:
        print(f"   Reason: {reason}")

print("\n" + "=" * 80)
print("TEST SUMMARY")
print("=" * 80)

# Count successes
linux_correct = sum(1 for cmd, expected in linux_tests if classify_linux(cmd)[0].value == expected)
windows_correct = sum(1 for cmd, expected in windows_tests if classify_windows(cmd)[0].value == expected)

total_tests = len(linux_tests) + len(windows_tests)
total_correct = linux_correct + windows_correct

print(f"Linux tests:   {linux_correct}/{len(linux_tests)} passed")
print(f"Windows tests: {windows_correct}/{len(windows_tests)} passed")
print(f"Overall:       {total_correct}/{total_tests} passed ({100*total_correct//total_tests}%)")

if total_correct == total_tests:
    print("\n🎉 ALL TESTS PASSED!")
    sys.exit(0)
else:
    print(f"\n⚠️  {total_tests - total_correct} test(s) failed")
    sys.exit(1)
