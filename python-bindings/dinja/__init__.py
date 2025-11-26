"""Dinja - MDX Rendering Library

A high-performance library for converting MDX (Markdown with JSX) to HTML and JavaScript.
"""

from typing import Dict, Any

try:
    from ._dinja import render as _render
except ImportError:
    # Fallback for development or different installation methods
    try:
        from dinja_python_bindings import render as _render
    except ImportError:
        raise ImportError(
            "Failed to import dinja. Make sure the package is properly installed. "
            "Run: maturin develop or pip install dinja"
        )


def render(input_dict: Dict[str, Any]) -> Dict[str, Any]:
    """Render MDX content to HTML, JavaScript, or schema format.

    This is a stateless function - no need to create service instances or manage
    static directories. All static files are embedded in the library.

    Args:
        input_dict: Dictionary containing:
            - settings: Dictionary with:
                - output: "html", "javascript", or "schema"
                - minify: bool (whether to minify output)
                - engine: "base" or "custom"
                - components: list of component names (for base engine)
            - mdx: Dictionary mapping file names to MDX content strings
            - components: Optional dictionary mapping component names to component definitions
                (required for custom engine, optional for base engine)

    Returns:
        Dictionary containing:
            - total: Total number of files processed
            - succeeded: Number of files that rendered successfully
            - failed: Number of files that failed to render
            - errors: List of error dictionaries with `file` and `message` keys
            - files: Dictionary mapping file names to render outcomes

    Raises:
        ValueError: If the request is invalid (e.g., resource limits exceeded, invalid input)
        RuntimeError: If an internal error occurs during rendering

    Example:
        >>> result = render({
        ...     "settings": {
        ...         "output": "html",
        ...         "minify": True,
        ...         "engine": "base",
        ...         "components": []
        ...     },
        ...     "mdx": {
        ...         "page1.mdx": "# Hello World"
        ...     },
        ...     "components": None
        ... })
        >>> print(result["succeeded"])
        1
    """
    return _render(input_dict)


__all__ = ["render"]
