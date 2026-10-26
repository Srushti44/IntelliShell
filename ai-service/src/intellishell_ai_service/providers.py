"""
AI Provider Abstraction Layer

Supports multiple AI providers (Ollama primary, Groq fallback) with a common interface.
Provider selection, timeouts, and fallback behavior are configurable.
"""

import json
import os
import re
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Optional

import requests


class ProviderType(str, Enum):
    """Available AI providers."""

    OLLAMA = "ollama"
    GROQ = "groq"


class ProviderError(Exception):
    """Base exception for provider failures."""

    pass


class ProviderUnavailable(ProviderError):
    """Provider service is not reachable."""

    pass


class ProviderTimeout(ProviderError):
    """Provider request exceeded timeout."""

    pass


class ProviderAuthError(ProviderError):
    """Provider authentication failed."""

    pass


class ProviderRateLimitError(ProviderError):
    """Provider rate limit exceeded."""

    pass


class ProviderMalformedOutput(ProviderError):
    """Provider returned invalid or unparseable output."""

    pass


@dataclass
class ProviderResponse:
    """Standardized response from any provider."""

    raw_output: str
    parsed_json: dict
    provider: ProviderType
    latency_ms: int
    model_name: str


class AIProvider(ABC):
    """Abstract base class for AI providers."""

    @abstractmethod
    def generate(self, prompt: str, timeout_seconds: int) -> ProviderResponse:
        """
        Generate a response from the AI model.

        Args:
            prompt: The complete prompt including system instructions
            timeout_seconds: Maximum time to wait for response

        Returns:
            ProviderResponse with raw output and parsed JSON

        Raises:
            ProviderUnavailable: Service not reachable
            ProviderTimeout: Request exceeded timeout
            ProviderAuthError: Authentication failed
            ProviderRateLimitError: Rate limit exceeded
            ProviderMalformedOutput: Invalid or unparseable output
        """
        pass

    @abstractmethod
    def health_check(self) -> bool:
        """
        Check if the provider is currently available.

        Returns:
            True if provider is healthy, False otherwise
        """
        pass


class OllamaProvider(AIProvider):
    """Local Ollama provider."""

    def __init__(
        self,
        base_url: str = "http://localhost:11434",
        model_name: str = "phi3:mini",
    ):
        self.base_url = base_url
        self.model_name = model_name
        self.generate_url = f"{base_url}/api/generate"
        self.tags_url = f"{base_url}/api/tags"

    def health_check(self) -> bool:
        """Check if Ollama is running and has the model."""
        try:
            response = requests.get(self.tags_url, timeout=3)
            response.raise_for_status()
            tags = response.json()
            models = [m.get("name", "") for m in tags.get("models", [])]
            return any(self.model_name in model for model in models)
        except requests.exceptions.RequestException:
            return False

    def generate(self, prompt: str, timeout_seconds: int) -> ProviderResponse:
        """Generate response using Ollama."""
        start_time = time.time()

        try:
            response = requests.post(
                self.generate_url,
                json={
                    "model": self.model_name,
                    "prompt": prompt,
                    "stream": False,
                },
                timeout=timeout_seconds,
            )
            response.raise_for_status()
        except requests.exceptions.Timeout as e:
            raise ProviderTimeout(f"Ollama request timed out after {timeout_seconds}s") from e
        except requests.exceptions.ConnectionError as e:
            raise ProviderUnavailable(f"Cannot connect to Ollama at {self.base_url}") from e
        except requests.exceptions.RequestException as e:
            raise ProviderUnavailable(f"Ollama request failed: {e}") from e

        latency_ms = int((time.time() - start_time) * 1000)

        try:
            result = response.json()
            raw_output = result.get("response", "")
        except json.JSONDecodeError as e:
            raise ProviderMalformedOutput(f"Ollama returned invalid JSON: {e}") from e

        if not raw_output:
            raise ProviderMalformedOutput("Ollama returned empty response")

        # Extract and parse JSON from model output
        try:
            parsed_json = self._extract_json(raw_output)
        except (ValueError, json.JSONDecodeError) as e:
            raise ProviderMalformedOutput(f"Could not parse JSON from Ollama output: {e}") from e

        return ProviderResponse(
            raw_output=raw_output,
            parsed_json=parsed_json,
            provider=ProviderType.OLLAMA,
            latency_ms=latency_ms,
            model_name=self.model_name,
        )

    def _extract_json(self, text: str) -> dict:
        """Extract JSON object from model output, handling markdown fences."""
        text = text.strip()
        # Remove markdown code fences if present
        text = re.sub(r"^```(?:json)?\s*", "", text)
        text = re.sub(r"\s*```$", "", text)
        text = text.strip()

        # Find JSON object boundaries
        start = text.find("{")
        end = text.rfind("}")

        if start == -1 or end == -1 or end < start:
            raise ValueError(f"No JSON object found in output: {text!r}")

        json_str = text[start : end + 1]
        return json.loads(json_str)


class GroqProvider(AIProvider):
    """Cloud Groq provider as fallback."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model_name: str = "openai/gpt-oss-20b",
    ):
        self.api_key = api_key or os.getenv("GROQ_API_KEY")
        if not self.api_key:
            raise ValueError("GROQ_API_KEY environment variable not set")

        self.model_name = model_name
        self.base_url = "https://api.groq.com/openai/v1/chat/completions"

    def health_check(self) -> bool:
        """Check if Groq API key is configured."""
        return bool(self.api_key)

    def generate(self, prompt: str, timeout_seconds: int) -> ProviderResponse:
        """Generate response using Groq API."""
        if not self.api_key:
            raise ProviderAuthError("GROQ_API_KEY not configured")

        start_time = time.time()

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        # Groq uses OpenAI-compatible API
        payload = {
            "model": self.model_name,
            "messages": [
                {
                    "role": "system",
                    "content": (
                        "You are a Linux command generator. "
                        "Always respond with ONLY a JSON object in this exact format: "
                        '{"command": "the linux command", "explanation": "what it does"}. '
                        "Do not include any other text, code blocks, or markdown."
                    ),
                },
                {"role": "user", "content": prompt},
            ],
            "temperature": 0.1,  # Low temperature for more deterministic output
            "max_tokens": 500,
        }

        try:
            response = requests.post(
                self.base_url,
                headers=headers,
                json=payload,
                timeout=timeout_seconds,
            )

            # Handle specific HTTP errors
            if response.status_code == 401:
                raise ProviderAuthError("Invalid Groq API key")
            elif response.status_code == 429:
                raise ProviderRateLimitError("Groq API rate limit exceeded")
            elif response.status_code >= 500:
                raise ProviderUnavailable(f"Groq API server error: {response.status_code}")

            response.raise_for_status()

        except requests.exceptions.Timeout as e:
            raise ProviderTimeout(f"Groq API request timed out after {timeout_seconds}s") from e
        except requests.exceptions.ConnectionError as e:
            raise ProviderUnavailable(f"Cannot connect to Groq API") from e
        except requests.exceptions.RequestException as e:
            raise ProviderUnavailable(f"Groq API request failed: {e}") from e

        latency_ms = int((time.time() - start_time) * 1000)

        try:
            result = response.json()
            raw_output = result["choices"][0]["message"]["content"]
        except (json.JSONDecodeError, KeyError, IndexError) as e:
            raise ProviderMalformedOutput(f"Groq returned invalid response structure: {e}") from e

        if not raw_output:
            raise ProviderMalformedOutput("Groq returned empty response")

        # Extract and parse JSON from model output
        try:
            parsed_json = self._extract_json(raw_output)
        except (ValueError, json.JSONDecodeError) as e:
            raise ProviderMalformedOutput(f"Could not parse JSON from Groq output: {e}") from e

        return ProviderResponse(
            raw_output=raw_output,
            parsed_json=parsed_json,
            provider=ProviderType.GROQ,
            latency_ms=latency_ms,
            model_name=self.model_name,
        )

    def _extract_json(self, text: str) -> dict:
        """Extract JSON object from model output."""
        text = text.strip()
        # Remove markdown code fences if present
        text = re.sub(r"^```(?:json)?\s*", "", text)
        text = re.sub(r"\s*```$", "", text)
        text = text.strip()

        # Find JSON object boundaries
        start = text.find("{")
        end = text.rfind("}")

        if start == -1 or end == -1 or end < start:
            raise ValueError(f"No JSON object found in output: {text!r}")

        json_str = text[start : end + 1]
        return json.loads(json_str)


class ProviderManager:
    """
    Manages multiple AI providers with fallback logic.

    Primary provider is always tried first. Falls back to secondary provider
    if primary fails, subject to configured retry limits.
    """

    def __init__(
        self,
        primary: AIProvider,
        fallback: Optional[AIProvider] = None,
        max_retries: int = 2,
    ):
        self.primary = primary
        self.fallback = fallback
        self.max_retries = max_retries

    def generate(self, prompt: str, timeout_seconds: int = 120) -> ProviderResponse:
        """
        Generate response using primary provider, falling back if necessary.

        Args:
            prompt: Complete prompt with system instructions
            timeout_seconds: Timeout per attempt

        Returns:
            ProviderResponse from whichever provider succeeded

        Raises:
            ProviderError: All providers failed
        """
        # Try primary provider with retries
        last_error = None
        for attempt in range(self.max_retries):
            try:
                return self.primary.generate(prompt, timeout_seconds)
            except (ProviderMalformedOutput, ProviderTimeout) as e:
                # These are retryable errors
                last_error = e
                continue
            except (ProviderUnavailable, ProviderAuthError) as e:
                # These are not retryable, go to fallback immediately
                last_error = e
                break

        # Try fallback provider if configured
        if self.fallback:
            try:
                response = self.fallback.generate(prompt, timeout_seconds)
                # Mark that fallback was used
                return response
            except ProviderError as e:
                raise ProviderError(
                    f"Primary provider failed ({last_error}), "
                    f"fallback provider also failed ({e})"
                ) from e

        # No fallback or fallback not configured
        raise ProviderError(f"All providers failed. Last error: {last_error}") from last_error


def create_provider_manager() -> ProviderManager:
    """
    Factory function to create ProviderManager with environment-based configuration.

    Reads configuration from environment variables:
    - OLLAMA_URL: Ollama server URL (default: http://localhost:11434)
    - OLLAMA_MODEL: Ollama model name (default: phi3:mini)
    - GROQ_API_KEY: Groq API key (optional, enables fallback if set)
    - GROQ_MODEL: Groq model name (default: llama-3.3-70b-versatile)
    - ENABLE_GROQ_FALLBACK: Enable Groq fallback (default: true if API key set)
    """
    ollama_url = os.getenv("OLLAMA_URL", "http://localhost:11434")
    ollama_model = os.getenv("OLLAMA_MODEL", "phi3:mini")
    groq_api_key = os.getenv("GROQ_API_KEY")
    groq_model = os.getenv("GROQ_MODEL", "llama-3.3-70b-versatile")
    enable_groq = os.getenv("ENABLE_GROQ_FALLBACK", "true").lower() == "true"

    primary = OllamaProvider(base_url=ollama_url, model_name=ollama_model)

    fallback = None
    if enable_groq and groq_api_key:
        try:
            fallback = GroqProvider(api_key=groq_api_key, model_name=groq_model)
        except ValueError:
            # API key not configured, fallback disabled
            pass

    return ProviderManager(primary=primary, fallback=fallback, max_retries=2)
