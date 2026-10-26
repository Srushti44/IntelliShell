from intellishell_ai_service.main import extract_json


def test_extract_json_fenced():
    """Test extracting JSON from fenced code block."""
    text = """
Here is some JSON:
```json
{"key": "value", "number": 42}
```
    """
    result = extract_json(text)
    assert "error" not in result
    assert result["key"] == "value"
    assert result["number"] == 42


def test_extract_json_fenced_no_lang():
    """Test extracting JSON from fenced code block without language specifier."""
    text = """
```
{"key": "value", "number": 42}
```
    """
    result = extract_json(text)
    assert "error" not in result
    assert result["key"] == "value"
    assert result["number"] == 42


def test_extract_json_unfenced():
    """Test extracting plain JSON without fencing."""
    text = '{"key": "value", "number": 42}'
    result = extract_json(text)
    assert "error" not in result
    assert result["key"] == "value"
    assert result["number"] == 42


def test_extract_json_unfenced_with_whitespace():
    """Test extracting plain JSON with surrounding whitespace."""
    text = """

    {"key": "value", "number": 42}

    """
    result = extract_json(text)
    assert "error" not in result
    assert result["key"] == "value"
    assert result["number"] == 42


def test_extract_json_invalid():
    """Test extracting invalid JSON."""
    text = '{"key": "value", invalid}'
    result = extract_json(text)
    assert "error" in result
    assert result["error"] == "Invalid JSON"
    assert "message" in result


def test_extract_json_complex():
    """Test extracting complex nested JSON."""
    text = """
```json
{
    "nested": {
        "array": [1, 2, 3],
        "object": {"a": "b"}
    },
    "bool": true,
    "null": null
}
```
    """
    result = extract_json(text)
    assert "error" not in result
    assert result["nested"]["array"] == [1, 2, 3]
    assert result["nested"]["object"]["a"] == "b"
    assert result["bool"] is True
    assert result["null"] is None
