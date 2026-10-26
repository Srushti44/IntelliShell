"""
Unit tests for AI provider abstraction layer.
"""

import json
import pytest
from unittest.mock import Mock, patch, MagicMock
import requests

from app.providers import (
    OllamaProvider,
    GroqProvider,
    ProviderManager,
    ProviderError,
    ProviderUnavailable,
    ProviderTimeout,
    ProviderAuthError,
    ProviderRateLimitError,
    ProviderMalformedOutput,
    ProviderType,
)


class TestOllamaProvider:
    """Test Ollama provider implementation."""

    def test_health_check_success(self):
        """Health check succeeds when model is available."""
        provider = OllamaProvider()

        with patch("requests.get") as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {
                "models": [{"name": "phi3:mini"}, {"name": "llama2"}]
            }
            mock_get.return_value = mock_response

            assert provider.health_check() is True

    def test_health_check_model_not_found(self):
        """Health check fails when model is not installed."""
        provider = OllamaProvider(model_name="nonexistent:model")

        with patch("requests.get") as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {"models": [{"name": "phi3:mini"}]}
            mock_get.return_value = mock_response

            assert provider.health_check() is False

    def test_health_check_service_down(self):
        """Health check fails when Ollama is not running."""
        provider = OllamaProvider()

        with patch("requests.get", side_effect=requests.exceptions.ConnectionError()):
            assert provider.health_check() is False

    def test_generate_success(self):
        """Successful command generation."""
        provider = OllamaProvider()

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {
                "response": '{"command": "ls -la", "explanation": "List files"}'
            }
            mock_post.return_value = mock_response

            result = provider.generate("test prompt", timeout_seconds=30)

            assert result.provider == ProviderType.OLLAMA
            assert result.parsed_json["command"] == "ls -la"
            assert result.parsed_json["explanation"] == "List files"
            assert result.latency_ms >= 0

    def test_generate_with_markdown_fences(self):
        """Handle model output with markdown code fences."""
        provider = OllamaProvider()

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {
                "response": '```json\n{"command": "pwd", "explanation": "Print working directory"}\n```'
            }
            mock_post.return_value = mock_response

            result = provider.generate("test prompt", timeout_seconds=30)

            assert result.parsed_json["command"] == "pwd"

    def test_generate_timeout(self):
        """Timeout exception is raised correctly."""
        provider = OllamaProvider()

        with patch("requests.post", side_effect=requests.exceptions.Timeout()):
            with pytest.raises(ProviderTimeout):
                provider.generate("test prompt", timeout_seconds=30)

    def test_generate_connection_error(self):
        """Connection error raises ProviderUnavailable."""
        provider = OllamaProvider()

        with patch("requests.post", side_effect=requests.exceptions.ConnectionError()):
            with pytest.raises(ProviderUnavailable):
                provider.generate("test prompt", timeout_seconds=30)

    def test_generate_malformed_json(self):
        """Malformed JSON raises ProviderMalformedOutput."""
        provider = OllamaProvider()

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {"response": "This is not JSON"}
            mock_post.return_value = mock_response

            with pytest.raises(ProviderMalformedOutput):
                provider.generate("test prompt", timeout_seconds=30)

    def test_generate_empty_response(self):
        """Empty response raises ProviderMalformedOutput."""
        provider = OllamaProvider()

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {"response": ""}
            mock_post.return_value = mock_response

            with pytest.raises(ProviderMalformedOutput):
                provider.generate("test prompt", timeout_seconds=30)


class TestGroqProvider:
    """Test Groq provider implementation."""

    def test_init_without_api_key(self):
        """Provider requires API key."""
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(ValueError, match="GROQ_API_KEY"):
                GroqProvider()

    def test_health_check_with_api_key(self):
        """Health check succeeds when API key is set."""
        provider = GroqProvider(api_key="test-key")
        assert provider.health_check() is True

    def test_health_check_without_api_key(self):
        """Health check fails when API key is not set."""
        with patch.dict("os.environ", {}, clear=True):
            with pytest.raises(ValueError):
                GroqProvider()

    def test_generate_success(self):
        """Successful Groq generation."""
        provider = GroqProvider(api_key="test-key")

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = {
                "choices": [
                    {
                        "message": {
                            "content": '{"command": "echo hello", "explanation": "Print message"}'
                        }
                    }
                ]
            }
            mock_post.return_value = mock_response

            result = provider.generate("test prompt", timeout_seconds=30)

            assert result.provider == ProviderType.GROQ
            assert result.parsed_json["command"] == "echo hello"

    def test_generate_auth_error(self):
        """Invalid API key raises ProviderAuthError."""
        provider = GroqProvider(api_key="invalid-key")

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 401
            mock_post.return_value = mock_response

            with pytest.raises(ProviderAuthError):
                provider.generate("test prompt", timeout_seconds=30)

    def test_generate_rate_limit(self):
        """Rate limit raises ProviderRateLimitError."""
        provider = GroqProvider(api_key="test-key")

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 429
            mock_post.return_value = mock_response

            with pytest.raises(ProviderRateLimitError):
                provider.generate("test prompt", timeout_seconds=30)

    def test_generate_server_error(self):
        """Server error raises ProviderUnavailable."""
        provider = GroqProvider(api_key="test-key")

        with patch("requests.post") as mock_post:
            mock_response = Mock()
            mock_response.status_code = 503
            mock_post.return_value = mock_response

            with pytest.raises(ProviderUnavailable):
                provider.generate("test prompt", timeout_seconds=30)


class TestProviderManager:
    """Test provider manager with fallback logic."""

    def test_primary_success_no_fallback(self):
        """Primary provider succeeds, no fallback needed."""
        primary = Mock()
        primary.generate.return_value = Mock(
            provider=ProviderType.OLLAMA, parsed_json={"command": "ls"}
        )

        fallback = Mock()

        manager = ProviderManager(primary=primary, fallback=fallback)
        result = manager.generate("test prompt")

        assert result.provider == ProviderType.OLLAMA
        primary.generate.assert_called_once()
        fallback.generate.assert_not_called()

    def test_primary_fails_fallback_succeeds(self):
        """Primary fails, fallback succeeds."""
        primary = Mock()
        primary.generate.side_effect = ProviderUnavailable("Ollama down")

        fallback = Mock()
        fallback.generate.return_value = Mock(
            provider=ProviderType.GROQ, parsed_json={"command": "pwd"}
        )

        manager = ProviderManager(primary=primary, fallback=fallback)
        result = manager.generate("test prompt")

        assert result.provider == ProviderType.GROQ
        primary.generate.assert_called_once()
        fallback.generate.assert_called_once()

    def test_primary_timeout_retry_then_success(self):
        """Primary times out once, retries and succeeds."""
        primary = Mock()
        primary.generate.side_effect = [
            ProviderTimeout("First attempt"),
            Mock(provider=ProviderType.OLLAMA, parsed_json={"command": "echo"}),
        ]

        fallback = Mock()

        manager = ProviderManager(primary=primary, fallback=fallback, max_retries=2)
        result = manager.generate("test prompt")

        assert result.provider == ProviderType.OLLAMA
        assert primary.generate.call_count == 2
        fallback.generate.assert_not_called()

    def test_primary_fails_all_retries_then_fallback(self):
        """Primary fails all retries, fallback succeeds."""
        primary = Mock()
        primary.generate.side_effect = [
            ProviderTimeout("Attempt 1"),
            ProviderTimeout("Attempt 2"),
        ]

        fallback = Mock()
        fallback.generate.return_value = Mock(
            provider=ProviderType.GROQ, parsed_json={"command": "date"}
        )

        manager = ProviderManager(primary=primary, fallback=fallback, max_retries=2)
        result = manager.generate("test prompt")

        assert result.provider == ProviderType.GROQ
        assert primary.generate.call_count == 2
        fallback.generate.assert_called_once()

    def test_both_providers_fail(self):
        """Both primary and fallback fail."""
        primary = Mock()
        primary.generate.side_effect = ProviderUnavailable("Ollama down")

        fallback = Mock()
        fallback.generate.side_effect = ProviderUnavailable("Groq down")

        manager = ProviderManager(primary=primary, fallback=fallback)

        with pytest.raises(ProviderError, match="All providers failed"):
            manager.generate("test prompt")

    def test_no_fallback_configured(self):
        """No fallback provider, primary failure raises error."""
        primary = Mock()
        primary.generate.side_effect = ProviderUnavailable("Ollama down")

        manager = ProviderManager(primary=primary, fallback=None, max_retries=2)

        with pytest.raises(ProviderError, match="All providers failed"):
            manager.generate("test prompt")
