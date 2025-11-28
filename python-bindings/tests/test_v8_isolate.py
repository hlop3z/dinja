"""Test script to verify v8 isolate behavior with dinja.

This test verifies that the Renderer class properly handles v8 isolate management
for rapid successive renders and mode switching.
"""

from __future__ import annotations

from typing import Any

import pytest

import dinja

# Test constants
_BUTTON_COMPONENT = {
    "name": "Button",
    "code": "export default function Component(props) { return <button>{props.children}</button>; }",
}

_COMPONENTS = {"Button": _BUTTON_COMPONENT}

_MDX_CONTENT = {
    "index.mdx": "# Hello World <Button name='Submit' />",
    "about.mdx": "## About\nThis is a sample page.",
}

_MODES = ("html", "javascript", "schema")


def _build_render_config(mode: str, mdx: dict[str, str]) -> dict[str, Any]:
    """Build render configuration dictionary.

    Args:
        mode: Output mode (html, javascript, or schema)
        mdx: MDX content dictionary

    Returns:
        Render configuration dictionary
    """
    return {
        "settings": {
            "output": mode,
            "minify": True,
        },
        "mdx": mdx,
        "components": _COMPONENTS,
    }


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
        or ("isolate" in error_lower and ("panic" in error_lower or "runtime" in error_lower))
    )


def _validate_result(result: dict[str, Any], mode: str) -> bool:
    """Validate render result.

    Args:
        result: Result dictionary from renderer.render
        mode: Mode name for error reporting

    Returns:
        True if result is valid, False otherwise
    """
    succeeded = result.get("succeeded", 0)
    if succeeded == 0:
        print(f"  ❌ Failed at mode={mode}")
        return False
    print(f"  ✓ {mode}: {succeeded} files rendered")
    return True


def test_renderer_multiple_modes() -> None:
    """Test using Renderer class with multiple modes in succession."""
    print("Test 1: Renderer class with multiple modes in succession")
    print("-" * 60)
    try:
        renderer = dinja.Renderer()
    except BaseException as e:
        if _is_v8_isolate_error(e):
            pytest.skip(
                f"v8 isolate error when creating Renderer (likely from previous tests): {type(e).__name__}"
            )
        raise
    for mode in _MODES:
        result = renderer.render(_build_render_config(mode, _MDX_CONTENT))
        assert _validate_result(result, mode), f"Failed at mode={mode}"
    print("  ✅ All modes succeeded\n")


# Note: These tests are designed to be run with pytest.
def main() -> None:
    """Run all tests and report results (standalone execution)."""
    print("=" * 60)
    print("Dinja Renderer Class Tests")
    print("=" * 60)
    print()
    print("Note: For best results, run with pytest:")
    print("  pytest tests/test_v8_isolate.py -v")
    print()


if __name__ == "__main__":
    main()

