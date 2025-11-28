"""
Dinja - MDX Rendering Client

HTTP client for the Dinja MDX rendering service.
Connect to the Dinja service running via Docker:
    docker pull ghcr.io/hlop3z/dinja:latest
    docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any, Literal
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError

# Type aliases
Output = Literal["html", "javascript", "schema", "json"]


@dataclass
class Component:
    """Component definition with code and metadata.

    Attributes:
        code: Component code (JSX/TSX) - required
        name: Component name (optional, defaults to dict key)
        docs: Component documentation (metadata)
        args: Component arguments/props types (metadata, as JSON value)
    """

    code: str
    name: str | None = None
    docs: str | None = None
    args: Any | None = None

    @classmethod
    def from_dict(cls, components: dict[str, str]) -> dict[str, Component]:
        """Create a dictionary of Component from a dict of name->code."""
        return {name: cls(code=code, name=name) for name, code in components.items()}

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
class Input:
    """Input structure for MDX rendering requests.

    Attributes:
        views: Map of view names to MDX content strings - required
        utils: Optional JavaScript snippet for global utilities (export default { ... })
        components: Optional map of component names to their definitions
        minify: Enable minification (default: True)
        directives: Optional list of directive prefixes for schema extraction
    """

    views: dict[str, str]
    utils: str | None = None
    components: dict[str, Component] | dict[str, str] | None = None
    minify: bool = True
    directives: list[str] | None = None

    def __post_init__(self) -> None:
        """Convert dict[str, str] components to dict[str, Component]."""
        if self.components is not None:
            if all(isinstance(v, str) for v in self.components.values()):
                self.components = Component.from_dict(
                    self.components  # type: ignore[arg-type]
                )

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        result: dict[str, Any] = {"mdx": self.views, "minify": self.minify}
        if self.utils is not None:
            result["utils"] = self.utils
        if self.directives is not None:
            result["directives"] = self.directives
        if self.components is not None:
            result["components"] = {
                name: comp.to_dict() for name, comp in self.components.items()
            }
        return result


@dataclass
class FileResult:
    """Result for a single rendered file.

    Attributes:
        success: Whether rendering succeeded
        metadata: Parsed YAML frontmatter (empty dict if none)
        output: Rendered output (HTML, JS, schema, or JSON depending on format)
        error: Error message if rendering failed
    """

    success: bool
    metadata: dict[str, Any] = field(default_factory=dict)
    output: str | None = None
    error: str | None = None


@dataclass
class Result:
    """Result of a batch render operation.

    Attributes:
        total: Total number of files processed
        succeeded: Number of files that rendered successfully
        failed: Number of files that failed to render
        files: Dictionary mapping file names to FileResult
        errors: List of error dictionaries with file and message
    """

    total: int
    succeeded: int
    failed: int
    files: dict[str, FileResult]
    errors: list[dict[str, str]] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Result:
        """Create Result from API response dictionary."""
        files = {}
        for name, file_data in data.get("files", {}).items():
            files[name] = FileResult(
                success=file_data.get("success", False),
                metadata=file_data.get("result", {}).get("metadata", {}),
                output=file_data.get("result", {}).get("output"),
                error=file_data.get("error"),
            )
        return cls(
            total=data.get("total", 0),
            succeeded=data.get("succeeded", 0),
            failed=data.get("failed", 0),
            files=files,
            errors=data.get("errors", []),
        )

    def is_all_success(self) -> bool:
        """Check if all files rendered successfully."""
        return self.failed == 0 and self.succeeded == self.total

    def get_output(self, filename: str) -> str | None:
        """Get output for a specific file."""
        if filename in self.files:
            return self.files[filename].output
        return None

    def get_metadata(self, filename: str) -> dict[str, Any]:
        """Get metadata for a specific file."""
        if filename in self.files:
            return self.files[filename].metadata
        return {}


def _build_request_data(
    views: dict[str, str],
    components: dict[str, Component] | dict[str, str] | None = None,
    minify: bool = True,
    utils: str | None = None,
    directives: list[str] | None = None,
) -> dict[str, Any]:
    """Build request data dictionary from parameters."""
    # Convert simple component dict to Component
    if components is not None:
        if all(isinstance(v, str) for v in components.values()):
            components = Component.from_dict(components)  # type: ignore

    result: dict[str, Any] = {"mdx": views, "minify": minify}
    if utils is not None:
        result["utils"] = utils
    if directives is not None:
        result["directives"] = directives
    if components is not None:
        result["components"] = {
            name: comp.to_dict() for name, comp in components.items()  # type: ignore
        }
    return result


class Renderer:
    """HTTP client for the Dinja MDX rendering service.

    Example:
        ```python
        from dinja import Renderer

        # Connect to local Docker service
        renderer = Renderer("http://localhost:8080")

        # Render MDX to HTML
        result = renderer.html(
            mdx={"page.mdx": "# Hello World"},
            utils="export default { greeting: 'Hello' }",
        )

        print(result.get_output("page.mdx"))
        ```

    Args:
        base_url: Base URL of the Dinja service (default: "http://localhost:8080")
        timeout: Request timeout in seconds (default: 30)
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        timeout: float = 30.0,
    ) -> None:
        """Initialize the renderer client."""
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def _request(self, endpoint: str, data: dict[str, Any]) -> dict[str, Any]:
        """Make HTTP POST request to the service."""
        url = f"{self.base_url}{endpoint}"
        body = json.dumps(data).encode("utf-8")
        request = Request(
            url,
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urlopen(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except HTTPError as e:
            error_body = e.read().decode("utf-8")
            try:
                error_data = json.loads(error_body)
                raise RuntimeError(error_data.get("error", str(e))) from e
            except json.JSONDecodeError:
                raise RuntimeError(f"HTTP {e.code}: {error_body}") from e
        except URLError as e:
            raise ConnectionError(f"Failed to connect to {url}: {e.reason}") from e

    def html(
        self,
        views: dict[str, str],
        components: dict[str, Component] | dict[str, str] | None = None,
        minify: bool = True,
        utils: str | None = None,
        directives: list[str] | None = None,
    ) -> Result:
        """Render MDX to HTML.

        Args:
            views: Map of view names to MDX content strings
            components: Optional map of component names to their definitions
            minify: Enable minification (default: True)
            utils: Optional JavaScript snippet for global utilities
            directives: Optional list of directive prefixes for schema extraction

        Returns:
            Result with rendered HTML output
        """
        data = _build_request_data(views, components, minify, utils, directives)
        response = self._request("/render/html", data)
        return Result.from_dict(response)

    def javascript(
        self,
        views: dict[str, str],
        components: dict[str, Component] | dict[str, str] | None = None,
        minify: bool = True,
        utils: str | None = None,
        directives: list[str] | None = None,
    ) -> Result:
        """Render MDX to JavaScript.

        Args:
            views: Map of view names to MDX content strings
            components: Optional map of component names to their definitions
            minify: Enable minification (default: True)
            utils: Optional JavaScript snippet for global utilities
            directives: Optional list of directive prefixes for schema extraction

        Returns:
            Result with JavaScript output
        """
        data = _build_request_data(views, components, minify, utils, directives)
        response = self._request("/render/javascript", data)
        return Result.from_dict(response)

    def schema(
        self,
        views: dict[str, str],
        components: dict[str, Component] | dict[str, str] | None = None,
        minify: bool = True,
        utils: str | None = None,
        directives: list[str] | None = None,
    ) -> Result:
        """Extract schema from MDX (component names).

        Args:
            views: Map of view names to MDX content strings
            components: Optional map of component names to their definitions
            minify: Enable minification (default: True)
            utils: Optional JavaScript snippet for global utilities
            directives: Optional list of directive prefixes for schema extraction

        Returns:
            Result with schema output
        """
        data = _build_request_data(views, components, minify, utils, directives)
        response = self._request("/render/schema", data)
        return Result.from_dict(response)

    def json(
        self,
        views: dict[str, str],
        components: dict[str, Component] | dict[str, str] | None = None,
        minify: bool = True,
        utils: str | None = None,
        directives: list[str] | None = None,
    ) -> Result:
        """Render MDX to JSON tree.

        Args:
            views: Map of view names to MDX content strings
            components: Optional map of component names to their definitions
            minify: Enable minification (default: True)
            utils: Optional JavaScript snippet for global utilities
            directives: Optional list of directive prefixes for schema extraction

        Returns:
            Result with JSON tree output
        """
        data = _build_request_data(views, components, minify, utils, directives)
        response = self._request("/render/json", data)
        return Result.from_dict(response)

    def render(
        self,
        output: Output,
        views: dict[str, str],
        components: dict[str, Component] | dict[str, str] | None = None,
        minify: bool = True,
        utils: str | None = None,
        directives: list[str] | None = None,
    ) -> Result:
        """Render MDX with specified output format.

        Args:
            output: Output format ("html", "javascript", "schema", "json")
            views: Map of view names to MDX content strings
            components: Optional map of component names to their definitions
            minify: Enable minification (default: True)
            utils: Optional JavaScript snippet for global utilities
            directives: Optional list of directive prefixes for schema extraction

        Returns:
            Result with rendered output
        """
        data = _build_request_data(views, components, minify, utils, directives)
        response = self._request(f"/render/{output}", data)
        return Result.from_dict(response)

    def health(self) -> bool:
        """Check if the service is healthy.

        Returns:
            True if service is healthy, False otherwise
        """
        try:
            url = f"{self.base_url}/health"
            request = Request(url, method="GET")
            with urlopen(request, timeout=self.timeout) as response:
                data = json.loads(response.read().decode("utf-8"))
                return data.get("status") == "ok"
        except Exception:
            return False


# Export all public types and classes
__all__ = [
    "Renderer",
    "Input",
    "Result",
    "FileResult",
    "Component",
    "Output",
]
