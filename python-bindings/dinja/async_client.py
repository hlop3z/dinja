"""
Async HTTP client for the Dinja MDX rendering service.

Supports both httpx and aiohttp as backends. Install your preferred library:
    pip install dinja[httpx]   # for httpx
    pip install dinja[aiohttp] # for aiohttp

Usage:
    from dinja import AsyncRenderer

    async with AsyncRenderer("http://localhost:8080") as renderer:
        result = await renderer.html(views={"page.mdx": "# Hello"})
        print(result.get_output("page.mdx"))
"""

from __future__ import annotations

import json
from typing import Any, Literal, Protocol, runtime_checkable

from . import Component, Result

# Type aliases
Output = Literal["html", "javascript", "schema", "json"]


@runtime_checkable
class AsyncHTTPClient(Protocol):
    """Protocol for async HTTP clients."""

    async def post(self, url: str, data: dict[str, Any]) -> dict[str, Any]: ...
    async def get(self, url: str) -> dict[str, Any]: ...
    async def close(self) -> None: ...


class HttpxClient:
    """httpx-based async HTTP client."""

    def __init__(self, base_url: str, timeout: float) -> None:
        try:
            import httpx
        except ImportError as e:
            raise ImportError(
                "httpx is required for async support. "
                "Install it with: pip install dinja[httpx]"
            ) from e

        self._client = httpx.AsyncClient(
            base_url=base_url,
            timeout=timeout,
            headers={"Content-Type": "application/json"},
        )

    async def post(self, url: str, data: dict[str, Any]) -> dict[str, Any]:
        response = await self._client.post(url, json=data)
        response.raise_for_status()
        return response.json()

    async def get(self, url: str) -> dict[str, Any]:
        response = await self._client.get(url)
        response.raise_for_status()
        return response.json()

    async def close(self) -> None:
        await self._client.aclose()


class AiohttpClient:
    """aiohttp-based async HTTP client."""

    def __init__(self, base_url: str, timeout: float) -> None:
        try:
            import aiohttp
        except ImportError as e:
            raise ImportError(
                "aiohttp is required for async support. "
                "Install it with: pip install dinja[aiohttp]"
            ) from e

        self._base_url = base_url.rstrip("/")
        self._timeout = aiohttp.ClientTimeout(total=timeout)
        self._session: aiohttp.ClientSession | None = None

    async def _get_session(self) -> "aiohttp.ClientSession":
        import aiohttp

        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(
                timeout=self._timeout,
                headers={"Content-Type": "application/json"},
            )
        return self._session

    async def post(self, url: str, data: dict[str, Any]) -> dict[str, Any]:
        session = await self._get_session()
        async with session.post(f"{self._base_url}{url}", json=data) as response:
            response.raise_for_status()
            return await response.json()

    async def get(self, url: str) -> dict[str, Any]:
        session = await self._get_session()
        async with session.get(f"{self._base_url}{url}") as response:
            response.raise_for_status()
            return await response.json()

    async def close(self) -> None:
        if self._session and not self._session.closed:
            await self._session.close()


def _detect_backend() -> Literal["httpx", "aiohttp"]:
    """Detect which async HTTP library is available."""
    try:
        import httpx  # noqa: F401

        return "httpx"
    except ImportError:
        pass

    try:
        import aiohttp  # noqa: F401

        return "aiohttp"
    except ImportError:
        pass

    raise ImportError(
        "No async HTTP library found. Install one with:\n"
        "  pip install dinja[httpx]   # recommended\n"
        "  pip install dinja[aiohttp] # alternative"
    )


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


class AsyncRenderer:
    """Async HTTP client for the Dinja MDX rendering service.

    Supports both httpx and aiohttp backends. The library will auto-detect
    which one is installed, or you can specify explicitly.

    Example:
        ```python
        from dinja import AsyncRenderer

        # Auto-detect backend
        async with AsyncRenderer("http://localhost:8080") as renderer:
            result = await renderer.html(views={"page.mdx": "# Hello World"})
            print(result.get_output("page.mdx"))

        # Or specify backend explicitly
        renderer = AsyncRenderer(
            "http://localhost:8080",
            backend="httpx"  # or "aiohttp"
        )
        ```

    Args:
        base_url: Base URL of the Dinja service (default: "http://localhost:8080")
        timeout: Request timeout in seconds (default: 30)
        backend: HTTP backend to use ("httpx", "aiohttp", or None for auto-detect)
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        timeout: float = 30.0,
        backend: Literal["httpx", "aiohttp"] | None = None,
    ) -> None:
        """Initialize the async renderer client."""
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

        if backend is None:
            backend = _detect_backend()

        self._backend = backend
        self._client: AsyncHTTPClient | None = None

    def _get_client(self) -> AsyncHTTPClient:
        """Get or create the HTTP client."""
        if self._client is None:
            if self._backend == "httpx":
                self._client = HttpxClient(self.base_url, self.timeout)
            else:
                self._client = AiohttpClient(self.base_url, self.timeout)
        return self._client

    async def __aenter__(self) -> "AsyncRenderer":
        """Enter async context manager."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Exit async context manager."""
        await self.close()

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client is not None:
            await self._client.close()
            self._client = None

    async def html(
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
        response = await self._get_client().post("/render/html", data)
        return Result.from_dict(response)

    async def javascript(
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
        response = await self._get_client().post("/render/javascript", data)
        return Result.from_dict(response)

    async def schema(
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
        response = await self._get_client().post("/render/schema", data)
        return Result.from_dict(response)

    async def json(
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
        response = await self._get_client().post("/render/json", data)
        return Result.from_dict(response)

    async def render(
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
        response = await self._get_client().post(f"/render/{output}", data)
        return Result.from_dict(response)

    async def health(self) -> bool:
        """Check if the service is healthy.

        Returns:
            True if service is healthy, False otherwise
        """
        try:
            data = await self._get_client().get("/health")
            return data.get("status") == "ok"
        except Exception:
            return False


__all__ = ["AsyncRenderer"]
