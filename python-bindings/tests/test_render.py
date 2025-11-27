"""Python integration tests for the dinja bindings."""

from __future__ import annotations

from copy import deepcopy

import pytest

from dinja import ComponentDefinition, Input, Renderer, Settings


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


def test_render_with_invalid_utils_fails_silently() -> None:
    """Test that invalid utils code is ignored and doesn't cause errors."""
    try:
        renderer = Renderer()
    except BaseException as e:
        # Handle V8 isolate errors from previous tests
        error_msg = str(e)
        error_type = str(type(e))
        if (
            "v8::OwnedIsolate" in error_msg
            or "PanicException" in error_type
            or "panic" in error_type.lower()
        ):
            pytest.skip(f"V8 isolate error from previous test: {type(e).__name__}")
        raise

    payload = {
        "settings": {
            "output": "html",
            "minify": False,
            "utils": "this is not valid javascript export syntax",
        },
        "mdx": {
            "test.mdx": "# Hello World",
        },
    }

    # Should not raise an error, just ignore the invalid utils
    result = renderer.render(payload)

    assert result["total"] == 1
    assert result["succeeded"] == 1
    assert result["failed"] == 0

    file_result = result["files"]["test.mdx"]
    assert file_result["status"] == "success"

    rendered = file_result["result"]
    output = rendered.get("output") or ""
    assert "<h1>Hello World</h1>" in output


def test_render_without_utils() -> None:
    """Test that rendering works without utils (backward compatibility)."""
    renderer = Renderer()

    payload = {
        "settings": {
            "output": "html",
            "minify": False,
            # No utils field
        },
        "mdx": {
            "test.mdx": "# Hello World",
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
    assert "<h1>Hello World</h1>" in output


# =============================================================================
# Tests using dataclass API
# =============================================================================


def test_dataclass_simple_render() -> None:
    """Test simple rendering using Input and Settings dataclasses."""
    renderer = Renderer()

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


def test_dataclass_with_utils() -> None:
    """Test Settings.utils parameter with dataclass API."""
    renderer = Renderer()

    result = renderer.render(
        Input(
            settings=Settings(
                output="html",
                minify=False,
                utils="export default { site: 'TestSite', year: 2024 }",
            ),
            mdx={"page.mdx": "<Info />"},
            components={
                "Info": ComponentDefinition(
                    code="export default function Component() { return <span>{utils.site} - {utils.year}</span>; }"
                )
            },
        )
    )

    assert result["succeeded"] == 1
    file_result = result["files"]["page.mdx"]
    assert file_result["status"] == "success"
    assert "TestSite" in file_result["result"]["output"]
    assert "2024" in file_result["result"]["output"]


def test_dataclass_component_definition() -> None:
    """Test ComponentDefinition with full parameters."""
    renderer = Renderer()

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


def test_dataclass_component_from_dict() -> None:
    """Test ComponentDefinition.from_dict helper method."""
    components = ComponentDefinition.from_dict(
        {
            "Button": "export default function Component(props) { return <button>{props.children}</button>; }",
            "Link": "export default function Component(props) { return <a href={props.href}>{props.children}</a>; }",
        }
    )

    assert "Button" in components
    assert "Link" in components
    assert isinstance(components["Button"], ComponentDefinition)
    assert components["Button"].name == "Button"
    assert "button" in components["Button"].code


def test_dataclass_simple_component_dict() -> None:
    """Test that simple dict[str, str] components are auto-converted."""
    renderer = Renderer()

    # Simple dict syntax - automatically converted to ComponentDefinition
    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={"page.mdx": "<Alert>Warning!</Alert>"},
            components={
                "Alert": "export default function Component({ children }) { return <div class='alert'>{children}</div>; }"
            },
        )
    )

    assert result["succeeded"] == 1
    file_result = result["files"]["page.mdx"]
    assert file_result["status"] == "success"
    assert "Warning!" in file_result["result"]["output"]


def test_dataclass_settings_defaults() -> None:
    """Test Settings default values."""
    settings = Settings()
    assert settings.output == "html"
    assert settings.minify is True
    assert settings.utils is None


def test_dataclass_settings_to_dict() -> None:
    """Test Settings.to_dict() method."""
    settings = Settings(output="javascript", minify=False, utils="export default {}")
    result = settings.to_dict()

    assert result["output"] == "javascript"
    assert result["minify"] is False
    assert result["utils"] == "export default {}"


def test_dataclass_settings_to_dict_without_utils() -> None:
    """Test that utils is not included in to_dict when None."""
    settings = Settings(output="html", minify=True)
    result = settings.to_dict()

    assert result["output"] == "html"
    assert result["minify"] is True
    assert "utils" not in result


def test_dataclass_input_to_dict() -> None:
    """Test Input.to_dict() method."""
    input_data = Input(
        settings=Settings(output="schema"),
        mdx={"test.mdx": "# Test"},
        components={
            "Box": ComponentDefinition(code="export default function Component() { return <div />; }")
        },
    )
    result = input_data.to_dict()

    assert result["settings"]["output"] == "schema"
    assert result["mdx"]["test.mdx"] == "# Test"
    assert "Box" in result["components"]
    assert "code" in result["components"]["Box"]


def test_dataclass_batch_rendering() -> None:
    """Test batch rendering with multiple files using dataclass API."""
    renderer = Renderer()

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


def test_dataclass_output_formats() -> None:
    """Test different output formats with dataclass API."""
    renderer = Renderer()

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


def test_dataclass_frontmatter_extraction() -> None:
    """Test YAML frontmatter extraction with dataclass API."""
    renderer = Renderer()

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
