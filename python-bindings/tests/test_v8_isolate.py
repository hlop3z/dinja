"""Test script to verify HTTP client behavior with dinja service.

This test verifies that the Renderer class properly handles multiple
rendering operations in succession with different output modes.
"""

from __future__ import annotations

import pytest

import dinja

# Test constants
_COMPONENTS = {
    "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
}

_MDX_CONTENT = {
    "index.mdx": "# Hello World <Button name='Submit' />",
    "about.mdx": "## About\nThis is a sample page.",
}

_MODES = ("html", "javascript", "schema")


def test_renderer_multiple_modes() -> None:
    """Test using Renderer class with multiple output modes in succession."""
    print("Test 1: Renderer class with multiple modes in succession")
    print("-" * 60)

    try:
        renderer = dinja.Renderer("http://localhost:8080")
        if not renderer.health():
            pytest.skip("Dinja service not running")
    except ConnectionError:
        pytest.skip("Dinja service not running")

    for mode in _MODES:
        result = renderer.render(
            output=mode,  # type: ignore
            mdx=_MDX_CONTENT,
            components=_COMPONENTS,
        )

        assert result.is_all_success(), f"Failed at mode={mode}"
        print(f"  ✓ {mode}: {result.succeeded} files rendered")

    print("  ✅ All modes succeeded\n")


def test_renderer_rapid_renders() -> None:
    """Test rapid successive renders to verify connection pooling."""
    print("Test 2: Rapid successive renders")
    print("-" * 60)

    try:
        renderer = dinja.Renderer("http://localhost:8080")
        if not renderer.health():
            pytest.skip("Dinja service not running")
    except ConnectionError:
        pytest.skip("Dinja service not running")

    # Perform 10 rapid renders
    for i in range(10):
        result = renderer.html(mdx={"test.mdx": f"# Test {i}"})
        assert result.is_all_success(), f"Failed at iteration {i}"

    print("  ✅ All rapid renders succeeded\n")


# Note: These tests are designed to be run with pytest.
def main() -> None:
    """Run all tests and report results (standalone execution)."""
    print("=" * 60)
    print("Dinja Renderer HTTP Client Tests")
    print("=" * 60)
    print()
    print("Note: For best results, run with pytest:")
    print("  pytest tests/test_v8_isolate.py -v")
    print()


if __name__ == "__main__":
    main()
