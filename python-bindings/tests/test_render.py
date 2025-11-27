"""Python integration tests for the dinja bindings."""

from __future__ import annotations

from copy import deepcopy

import pytest

from dinja import Renderer


BASE_PAYLOAD = {
    "settings": {
        "output": "html",
        "minify": True,
        "engine": "base",
        "components": [],
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
            "engine": "custom",
            "components": [],
        },
        "mdx": {
            "test.mdx": "# Hello World\n\n<Button>Submit</Button>",
        },
        "components": {
            "Button": {
                "name": "Button",
                "code": "function Component(props) { return engine.h('button', null, props.children); }",
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

