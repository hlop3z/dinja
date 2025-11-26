"""Python integration tests for the dinja bindings."""

from __future__ import annotations

from copy import deepcopy

import pytest

from dinja import render


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
    result = render(deepcopy(BASE_PAYLOAD))

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
    """NamedMdxBatchInput enforces string content, so invalid data raises ValueError."""
    invalid_payload = deepcopy(BASE_PAYLOAD)
    invalid_payload["mdx"]["broken.mdx"] = 12345  # type: ignore[assignment]

    with pytest.raises(ValueError):
        render(invalid_payload)

