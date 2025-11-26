"""
Public Python interface for the dinja MDX renderer.

This shim keeps the top-level `dinja` package pure Python so we can ship
typing markers while the heavy lifting lives in the `_native` extension
module compiled with PyO3.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from importlib import import_module
from typing import Any, Literal

_native = import_module("dinja._native")

_NativeRenderer = _native.Renderer

# Type aliases matching Rust enums
OutputFormat = Literal["html", "javascript", "schema"]
RenderEngine = Literal["base", "custom"]


@dataclass
class ComponentDefinition:
    """Component definition with code and metadata.

    Matches the Rust `ComponentDefinition` struct.

    Attributes:
        code: Component code (JSX/TSX) - required
        name: Component name (optional, defaults to dict key)
        docs: Component documentation (metadata)
        args: Component arguments/props types (metadata, as JSON value)
    """

    code: str
    name: str | None = None
    docs: str | None = None
    args: Any | None = None  # serde_json::Value equivalent

    @classmethod
    def from_name_code(cls, name: str, code: str) -> ComponentDefinition:
        """Create a ComponentDefinition from name and code.

        Helper function to create a component definition from a simple
        name-code pair.

        Args:
            name: Component name
            code: Component code (JSX/TSX)

        Returns:
            ComponentDefinition instance with name and code set
        """
        return cls(code=code, name=name)

    @classmethod
    def from_dict(cls, components: dict[str, str]) -> dict[str, ComponentDefinition]:
        """Create a dictionary of ComponentDefinition from a dict of name->code.

        Helper function to convert a simple dict mapping component names to code
        strings into the proper format expected by Input.

        Args:
            components: Dictionary mapping component names to their code strings

        Returns:
            Dictionary mapping component names to ComponentDefinition instances

        Example:
            ```python
            from dinja import Input, Settings, ComponentDefinition

            # Simple dict: name -> code
            component_dict = {
                "Button": "function Component(props) { return <button>{props.children}</button>; }",
                "Card": "function Component(props) { return <div>{props.children}</div>; }",
            }

            # Convert to proper format
            components = ComponentDefinition.from_dict(component_dict)

            # Use in Input
            input_data = Input(
                mdx={"page.mdx": "# Hello <Button>Click</Button>"},
                settings=Settings(engine="custom"),
                components=components,
            )
            ```
        """
        return {
            name: cls.from_name_code(name, code) for name, code in components.items()
        }

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        result: dict[str, Any] = {"code": self.code}
        if self.name is not None:
            result["name"] = self.name
        if self.docs is not None:
            result["docs"] = self.docs
        if self.args is not None:
            result["args"] = self.args
        return result


@dataclass
class Settings:
    """Rendering settings.

    Matches the Rust `Settings` struct.

    Attributes:
        output: Output format (default: "html")
        minify: Enable minification (default: True)
        engine: Rendering engine selection (default: "base")
        components: Component names to autopopulate when using base engine (default: empty list)
    """

    output: OutputFormat = "html"
    minify: bool = True
    engine: RenderEngine = "base"
    components: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        result: dict[str, Any] = {
            "output": self.output,
            "minify": self.minify,
            "engine": self.engine,
        }
        if self.components:
            result["components"] = self.components
        return result


@dataclass
class Input:
    """Input structure for batch MDX rendering requests.

    Matches the Rust `NamedMdxBatchInput` struct (aliased as `Input` in Python).

    Attributes:
        mdx: Map of file names to MDX content strings - required
        settings: Rendering settings (default: Settings())
        components: Optional map of component names to their definitions.
            Can be a dict[str, str] (name -> code) or dict[str, ComponentDefinition].
            If dict[str, str] is provided, it will be automatically converted.
    """

    mdx: dict[str, str]
    settings: Settings = field(default_factory=Settings)
    components: dict[str, ComponentDefinition] | dict[str, str] | None = None

    def __post_init__(self) -> None:
        """Convert dict[str, str] components to dict[str, ComponentDefinition]."""
        if self.components is not None:
            # Check if all values are strings (simple name->code dict)
            if all(isinstance(v, str) for v in self.components.values()):
                # Convert dict[str, str] to dict[str, ComponentDefinition]
                self.components = ComponentDefinition.from_dict(
                    self.components  # type: ignore[arg-type]
                )

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        result: dict[str, Any] = {
            "settings": self.settings.to_dict(),
            "mdx": self.mdx,
        }
        if self.components is not None:
            result["components"] = {
                name: comp.to_dict() for name, comp in self.components.items()
            }
        return result


def _is_v8_isolate_error(error: Exception) -> bool:
    """Check if exception is a v8 isolate error.

    Args:
        error: Exception to check

    Returns:
        True if error is related to v8 isolate management
    """
    error_type = type(error).__name__
    error_str = str(error)
    error_lower = error_str.lower()
    return (
        error_type == "PanicException"
        or "PanicException" in error_str
        or "v8::OwnedIsolate" in error_str
        or "v8 isolate" in error_lower
        or (
            "isolate" in error_lower
            and ("panic" in error_lower or "runtime" in error_lower)
        )
    )


class Renderer:
    """A renderer with automatic retry logic for v8 isolate errors.

    This class wraps the native Renderer and automatically retries on transient
    v8 isolate errors, which can occur during rapid successive renders or mode switching.

    Example:
        ```python
        from dinja import Renderer, Input, Settings

        # Create a renderer with default retry settings (3 retries)
        renderer = Renderer()

        # Option 1: Use dataclasses (type-safe)
        input_data = Input(
            mdx={"page.mdx": "# Hello"},
            settings=Settings(output="html", minify=True),
        )
        result = renderer.render(input_data)

        # Option 2: Use dictionaries (backward compatible)
        result = renderer.render({
            "settings": {"output": "html"},
            "mdx": {"page.mdx": "# Hello"},
        })
        ```

    Args:
        max_retries: Maximum number of retry attempts (default: 3)
        retry_delay: Initial delay between retries in seconds (default: 0.05)
        backoff_factor: Multiplier for exponential backoff (default: 1.5)
    """

    def __init__(
        self,
        max_retries: int = 3,
        retry_delay: float = 0.05,
        backoff_factor: float = 1.5,
    ) -> None:
        """Initialize the renderer with retry configuration."""
        self._renderer = _NativeRenderer()
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.backoff_factor = backoff_factor

    def render(self, input: Input | dict[str, Any]) -> dict[str, Any]:
        """Render MDX content with automatic retry on v8 isolate errors.

        Args:
            input: Either a `Input` dataclass instance or a dictionary
                containing:
                - `settings`: Dictionary with `output` ("html", "javascript", or "schema"),
                  `minify` (bool), `engine` ("base" or "custom"), `components` (list of strings)
                - `mdx`: Dictionary mapping file names to MDX content strings
                - `components`: Optional dictionary mapping component names to component definitions

        Returns:
            Dictionary containing:
                - `total`: Total number of files processed
                - `succeeded`: Number of files that rendered successfully
                - `failed`: Number of files that failed to render
                - `errors`: List of error dictionaries with `file` and `message` keys
                - `files`: Dictionary mapping file names to render outcomes

        Raises:
            ValueError: If the request is invalid after all retries
            RuntimeError: If an internal error occurs after all retries
        """
        # Convert Input to dict if needed
        if isinstance(input, Input):
            input_dict = input.to_dict()
        else:
            input_dict = input

        last_error: Exception | None = None
        delay = self.retry_delay

        for attempt in range(self.max_retries + 1):
            try:
                return self._renderer.render(input_dict)
            except Exception as e:
                last_error = e
                # Only retry on v8 isolate errors
                if not _is_v8_isolate_error(e):
                    # Non-retryable error, raise immediately
                    raise

                # If this was the last attempt, raise the error
                if attempt >= self.max_retries:
                    break

                # Wait before retrying with exponential backoff
                time.sleep(delay)
                delay *= self.backoff_factor

        # All retries exhausted, raise the last error
        if last_error is not None:
            raise last_error

        # This should never happen, but satisfy type checker
        raise RuntimeError("Renderer failed without raising an error")


# Export all public types and classes
__all__ = [
    "Renderer",
    "_NativeRenderer",
    "OutputFormat",
    "RenderEngine",
    "ComponentDefinition",
    "Settings",
    "Input",
]
