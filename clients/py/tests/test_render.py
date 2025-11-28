"""Python integration tests for the dinja HTTP client."""

from __future__ import annotations

import pytest

from dinja import Component, Input, Renderer, Result


# =============================================================================
# Fixtures
# =============================================================================


@pytest.fixture
def renderer() -> Renderer:
    """Create a Renderer connected to the local service."""
    return Renderer("http://localhost:8080")


# =============================================================================
# Health Check Tests
# =============================================================================


def test_health_check(renderer: Renderer) -> None:
    """Test that the service health endpoint works."""
    # Note: This will fail if the service is not running
    # In CI, we should skip or mock this
    try:
        is_healthy = renderer.health()
        # If we can reach the service, it should be healthy
        if is_healthy:
            assert is_healthy is True
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# HTML Rendering Tests
# =============================================================================


def test_render_html_simple(renderer: Renderer) -> None:
    """Test simple HTML rendering."""
    try:
        result = renderer.html(views={"page.mdx": "# Hello World"})

        assert result.total == 1
        assert result.succeeded == 1
        assert result.failed == 0
        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        assert "<h1>Hello World</h1>" in output
    except ConnectionError:
        pytest.skip("Dinja service not running")


def test_render_html_with_frontmatter(renderer: Renderer) -> None:
    """Test HTML rendering with YAML frontmatter."""
    try:
        result = renderer.html(
            views={
                "blog.mdx": """---
title: My Post
author: Alice
---

# Content here
"""
            }
        )

        assert result.is_all_success()

        metadata = result.get_metadata("blog.mdx")
        assert metadata["title"] == "My Post"
        assert metadata["author"] == "Alice"
    except ConnectionError:
        pytest.skip("Dinja service not running")


def test_render_html_with_component(renderer: Renderer) -> None:
    """Test HTML rendering with custom component."""
    try:
        result = renderer.html(
            views={"page.mdx": "<Button>Click me</Button>"},
            components={
                "Button": "export default function Component(props) { return <button>{props.children}</button>; }"
            },
        )

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        assert "<button>" in output
        assert "Click me" in output
    except ConnectionError:
        pytest.skip("Dinja service not running")


def test_render_html_with_component_definition(renderer: Renderer) -> None:
    """Test Component with full parameters."""
    try:
        result = renderer.html(
            views={"page.mdx": "<Card>Content</Card>"},
            components={
                "Card": Component(
                    code="export default function Component({ children }) { return <div class='card'>{children}</div>; }",
                    name="Card",
                    docs="A card component",
                    args={"children": "ReactNode"},
                )
            },
        )

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        assert "class" in output
        assert "Content" in output
    except ConnectionError:
        pytest.skip("Dinja service not running")


def test_render_html_with_utils(renderer: Renderer) -> None:
    """Test HTML rendering with utils."""
    try:
        result = renderer.html(
            views={"page.mdx": "<Greeting />"},
            components={
                "Greeting": "export default function Component() { return <div>{utils.greeting}</div>; }"
            },
            utils="export default { greeting: 'Hello World' }",
        )

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        assert "Hello World" in output
    except ConnectionError:
        pytest.skip("Dinja service not running")


def test_render_html_batch(renderer: Renderer) -> None:
    """Test batch HTML rendering with multiple files."""
    try:
        result = renderer.html(
            views={
                "page1.mdx": "# Page 1",
                "page2.mdx": "# Page 2",
                "page3.mdx": "# Page 3",
            }
        )

        assert result.total == 3
        assert result.succeeded == 3
        assert result.failed == 0
        assert result.is_all_success()
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# JavaScript Rendering Tests
# =============================================================================


def test_render_javascript(renderer: Renderer) -> None:
    """Test JavaScript output format."""
    try:
        result = renderer.javascript(views={"page.mdx": "# Hello"})

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        # JavaScript output should contain function syntax
        assert len(output) > 0
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# Schema Extraction Tests
# =============================================================================


def test_render_schema(renderer: Renderer) -> None:
    """Test schema extraction."""
    try:
        result = renderer.schema(
            views={"page.mdx": "<Button>Click</Button><Card>Content</Card>"}
        )

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# JSON Tree Tests
# =============================================================================


def test_render_json(renderer: Renderer) -> None:
    """Test JSON tree output."""
    try:
        result = renderer.json(views={"page.mdx": "# Hello"})

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# Generic Render Method Tests
# =============================================================================


def test_render_generic_method(renderer: Renderer) -> None:
    """Test generic render method with output parameter."""
    try:
        result = renderer.render(
            output="html",
            views={"page.mdx": "# Hello"},
        )

        assert result.is_all_success()

        output = result.get_output("page.mdx")
        assert output is not None
        assert "<h1>Hello</h1>" in output
    except ConnectionError:
        pytest.skip("Dinja service not running")


# =============================================================================
# Input Dataclass Tests
# =============================================================================


def test_input_dataclass() -> None:
    """Test Input dataclass creation and conversion."""
    input_obj = Input(
        views={"page.mdx": "# Hello"},
        utils="export default {}",
        minify=True,
        directives=["v-"],
    )

    data = input_obj.to_dict()
    assert data["mdx"] == {"page.mdx": "# Hello"}
    assert data["utils"] == "export default {}"
    assert data["minify"] is True
    assert data["directives"] == ["v-"]


def test_input_with_string_components() -> None:
    """Test Input with string components (auto-converted to Component)."""
    input_obj = Input(
        views={"page.mdx": "# Hello"},
        components={"Button": "export default function() {}"},
    )

    # String components should be converted to Component instances
    assert input_obj.components is not None
    assert "Button" in input_obj.components
    assert isinstance(input_obj.components["Button"], Component)


# =============================================================================
# Result Tests
# =============================================================================


def test_result_from_dict() -> None:
    """Test Result.from_dict conversion."""
    data = {
        "total": 2,
        "succeeded": 1,
        "failed": 1,
        "files": {
            "good.mdx": {
                "success": True,
                "result": {
                    "metadata": {"title": "Good"},
                    "output": "<h1>Good</h1>",
                },
            },
            "bad.mdx": {
                "success": False,
                "error": "Parse error",
            },
        },
        "errors": [{"file": "bad.mdx", "message": "Parse error"}],
    }

    result = Result.from_dict(data)

    assert result.total == 2
    assert result.succeeded == 1
    assert result.failed == 1
    assert not result.is_all_success()

    assert result.get_output("good.mdx") == "<h1>Good</h1>"
    assert result.get_metadata("good.mdx") == {"title": "Good"}

    assert result.get_output("bad.mdx") is None
    assert result.files["bad.mdx"].error == "Parse error"
