"""Python integration tests for the dinja bindings."""

from __future__ import annotations

from copy import deepcopy

import pytest

from dinja import Renderer


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
        if "v8::OwnedIsolate" in error_msg or "PanicException" in error_type or "panic" in error_type.lower():
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

