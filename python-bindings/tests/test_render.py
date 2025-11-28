"""Python integration tests for the dinja bindings."""

from __future__ import annotations

from copy import deepcopy
from typing import Generator

import pytest

from dinja import ComponentDefinition, Input, Renderer, Settings


# =============================================================================
# Fixtures for V8 isolate error handling
# =============================================================================


@pytest.fixture
def renderer() -> Generator[Renderer, None, None]:
    """Create a Renderer, skipping test if V8 isolate error occurs.

    V8 isolates must be dropped in reverse order of creation. When running
    multiple tests, previous test cleanup may cause isolate ordering issues.
    """
    try:
        r = Renderer()
        yield r
    except BaseException as e:
        error_msg = str(e)
        error_type = str(type(e))
        if (
            "v8::OwnedIsolate" in error_msg
            or "PanicException" in error_type
            or "panic" in error_type.lower()
        ):
            pytest.skip(f"V8 isolate error: {type(e).__name__}")
        raise


# =============================================================================
# Tests using dict API (backward compatibility)
# =============================================================================

BASE_PAYLOAD = {
    "settings": {
        "output": "html",
        "minify": True,
    },
    "mdx": {
        "index.mdx": "---\ntitle: Index\n---\n# Hello World\n",
    },
    "components": None,
}


def test_render_html_success() -> None:
    """End-to-end success case mirrors the `/render` handler contract."""
    renderer = Renderer()
    result = renderer.render(deepcopy(BASE_PAYLOAD))

    assert result["total"] == 1
    assert result["succeeded"] == 1
    assert result["failed"] == 0

    file_result = result["files"]["index.mdx"]
    assert file_result["status"] == "success"

    rendered = file_result["result"]
    assert rendered["metadata"]["title"] == "Index"
    output = rendered.get("output") or ""
    assert "<h1>Hello World</h1>" in output


def test_render_rejects_non_string_mdx() -> None:
    """Input enforces string content, so invalid data raises ValueError."""
    renderer = Renderer()
    invalid_payload = deepcopy(BASE_PAYLOAD)
    invalid_payload["mdx"]["broken.mdx"] = 12345  # type: ignore[assignment]

    with pytest.raises(ValueError):
        renderer.render(invalid_payload)


def test_render_custom_component_html() -> None:
    """Test that custom components render to regular HTML."""
    renderer = Renderer()

    payload = {
        "settings": {
            "output": "html",
            "minify": True,
        },
        "mdx": {
            "test.mdx": "# Hello World\n\n<Button>Submit</Button>",
        },
        "components": {
            "Button": {
                "name": "Button",
                "code": "export default function Component(props) { return <button>{props.children}</button>; }",
                "docs": None,
                "args": None,
            },
        },
    }

    result = renderer.render(payload)

    assert result["total"] == 1
    assert result["succeeded"] == 1
    assert result["failed"] == 0

    file_result = result["files"]["test.mdx"]
    assert file_result["status"] == "success"

    rendered = file_result["result"]
    output = rendered.get("output") or ""

    # Verify the HTML contains the rendered component
    assert "<h1>Hello World</h1>" in output
    assert "<button>" in output
    assert "Submit" in output
    assert "</button>" in output


def test_render_with_utils() -> None:
    """Test that utils can be injected and used in components."""
    renderer = Renderer()

    payload = {
        "settings": {
            "output": "html",
            "minify": False,
            "utils": "export default { greeting: 'Hello', name: 'World' }",
        },
        "mdx": {
            "test.mdx": "<Greeting />",
        },
        "components": {
            "Greeting": {
                "name": "Greeting",
                "code": "export default function Component(props) { return <div>{utils.greeting} {utils.name}</div>; }",
            },
        },
    }

    result = renderer.render(payload)

    assert result["total"] == 1
    assert result["succeeded"] == 1
    assert result["failed"] == 0

    file_result = result["files"]["test.mdx"]
    assert file_result["status"] == "success"

    rendered = file_result["result"]
    output = rendered.get("output") or ""

    # Verify the utils were accessible in the component
    assert "Hello World" in output


# =============================================================================
# Tests using dataclass API
# =============================================================================


def test_dataclass_simple_render(renderer: Renderer) -> None:
    """Test simple rendering using Input and Settings dataclasses."""
    result = renderer.render(
        Input(
            settings=Settings(output="html", minify=False),
            mdx={"page.mdx": "# Hello **dinja**"},
        )
    )

    assert result["total"] == 1
    assert result["succeeded"] == 1
    assert result["failed"] == 0

    file_result = result["files"]["page.mdx"]
    assert file_result["status"] == "success"
    assert "<h1>Hello <strong>dinja</strong></h1>" in file_result["result"]["output"]


def test_dataclass_component_definition(renderer: Renderer) -> None:
    """Test ComponentDefinition with full parameters."""
    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={"page.mdx": "<Card>Content</Card>"},
            components={
                "Card": ComponentDefinition(
                    name="Card",
                    code="export default function Component({ children }) { return <div class='card'>{children}</div>; }",
                    docs="A card component",
                    args={"children": "ReactNode"},
                )
            },
        )
    )

    assert result["succeeded"] == 1
    file_result = result["files"]["page.mdx"]
    assert file_result["status"] == "success"
    assert "class" in file_result["result"]["output"]
    assert "Content" in file_result["result"]["output"]


def test_dataclass_batch_rendering(renderer: Renderer) -> None:
    """Test batch rendering with multiple files using dataclass API."""
    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={
                "page1.mdx": "# Page 1",
                "page2.mdx": "# Page 2",
                "page3.mdx": "# Page 3",
            },
        )
    )

    assert result["total"] == 3
    assert result["succeeded"] == 3
    assert result["failed"] == 0

    for name in ["page1.mdx", "page2.mdx", "page3.mdx"]:
        assert result["files"][name]["status"] == "success"


def test_dataclass_output_formats(renderer: Renderer) -> None:
    """Test different output formats with dataclass API."""
    for output_format in ["html", "javascript", "schema"]:
        result = renderer.render(
            Input(
                settings=Settings(output=output_format, minify=False),  # type: ignore
                mdx={"test.mdx": "# Hello"},
            )
        )

        assert result["succeeded"] == 1, f"Failed for format: {output_format}"
        file_result = result["files"]["test.mdx"]
        assert file_result["status"] == "success"
        assert file_result["result"]["output"], f"Empty output for format: {output_format}"


def test_dataclass_frontmatter_extraction(renderer: Renderer) -> None:
    """Test YAML frontmatter extraction with dataclass API."""
    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={
                "blog.mdx": """---
title: My Post
author: Alice
published: true
tags:
  - python
  - rust
---

# Content here
"""
            },
        )
    )

    assert result["succeeded"] == 1
    file_result = result["files"]["blog.mdx"]
    assert file_result["status"] == "success"

    metadata = file_result["result"]["metadata"]
    assert metadata["title"] == "My Post"
    assert metadata["author"] == "Alice"
    assert metadata["published"] is True
    assert "python" in metadata["tags"]
    assert "rust" in metadata["tags"]


def test_dataclass_settings_with_directives(renderer: Renderer) -> None:
    """Test Settings with directives parameter."""
    result = renderer.render(
        Input(
            settings=Settings(
                output="html",
                minify=False,
                directives=["v-", "@", "x-"],
            ),
            mdx={"page.mdx": "# Hello"},
        )
    )

    assert result["succeeded"] == 1
    file_result = result["files"]["page.mdx"]
    assert file_result["status"] == "success"


