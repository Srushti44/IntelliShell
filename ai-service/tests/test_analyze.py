from intellishell_ai_service.main import analyze_impact


def test_analyze_impact_with_real_files(tmp_path):
    """Test analyze_impact with real files created in tmp_path."""
    # Create some test files and directories
    test_file = tmp_path / "testfile.txt"
    test_file.write_text("test content")

    test_dir = tmp_path / "testdir"
    test_dir.mkdir()

    nested_file = test_dir / "nested.txt"
    nested_file.write_text("nested content")

    # Test rm command on existing file
    command = f"rm {test_file}"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert result["command"] == command
    assert result["cwd"] == str(tmp_path)
    assert "delete" in result["operations"]
    assert str(test_file) in result["affected_paths"]
    assert str(test_file) in result["existing_paths"]


def test_analyze_impact_rm_nonexistent(tmp_path):
    """Test analyze_impact with rm on non-existent file."""
    command = "rm nonexistent.txt"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert "delete" in result["operations"]
    assert len(result["affected_paths"]) > 0
    # Non-existent file should not be in existing_paths
    assert len(result["existing_paths"]) == 0


def test_analyze_impact_chmod(tmp_path):
    """Test analyze_impact with chmod command."""
    test_file = tmp_path / "file.txt"
    test_file.write_text("content")

    command = f"chmod 755 {test_file}"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert "permission_change" in result["operations"]
    assert str(test_file) in result["affected_paths"]
    assert str(test_file) in result["existing_paths"]


def test_analyze_impact_mv(tmp_path):
    """Test analyze_impact with mv command."""
    command = "mv source.txt dest.txt"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert "move" in result["operations"]


def test_analyze_impact_cp(tmp_path):
    """Test analyze_impact with cp command."""
    command = "cp source.txt dest.txt"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert "copy" in result["operations"]


def test_analyze_impact_critical_command(tmp_path):
    """Test analyze_impact includes risk level for critical commands."""
    command = "rm -rf /"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert result["risk_level"] == "critical"
    assert "delete" in result["operations"]


def test_analyze_impact_relative_path(tmp_path):
    """Test analyze_impact converts relative paths to absolute."""
    test_file = tmp_path / "relative.txt"
    test_file.write_text("content")

    command = "rm relative.txt"
    context = {"cwd": str(tmp_path)}
    result = analyze_impact(command, context)

    assert "delete" in result["operations"]
    # Should contain absolute path
    assert any(str(tmp_path) in path for path in result["affected_paths"])


def test_analyze_impact_absolute_path(tmp_path):
    """Test analyze_impact with absolute path."""
    test_file = tmp_path / "absolute.txt"
    test_file.write_text("content")

    command = f"rm {test_file}"
    context = {"cwd": "/some/other/dir"}
    result = analyze_impact(command, context)

    assert str(test_file) in result["affected_paths"]
    assert str(test_file) in result["existing_paths"]
