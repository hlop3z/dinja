"""Basic example demonstrating all output formats in Dinja.

This example shows how to use the different output formats:
- schema: Extract component names
- html: Render to HTML
- javascript: Transform to JavaScript
- json: Get JSON tree structure
"""

from dinja import Renderer


def demo_schema_extraction():
    """Demonstrate schema output - extracts component names."""
    print("=" * 60)
    print("SCHEMA OUTPUT - Component Name Extraction")
    print("=" * 60)

    mdx_content = """---
title: Component Discovery Demo
---

# {context('title')}

Welcome! Here are some components:

<Alert type="info">This is an alert</Alert>

<Card title="Features">
  <Button primary>Get Started</Button>
  <Button>Learn More</Button>
</Card>

<Footer>© 2024</Footer>
"""

    payload = {
        "settings": {"output": "schema", "minify": False},
        "mdx": {"example.mdx": mdx_content},
    }

    renderer = Renderer()
    result = renderer.render(payload)
    entry = result["files"]["example.mdx"]

    if entry["status"] == "success":
        rendered = entry["result"]
        metadata = rendered.get("metadata", {})
        components = rendered.get("output")

        print(f"\nMetadata:")
        print(f"  Title: {metadata.get('title')}")
        print(f"\nDiscovered Components: {components}")
        print(f"  Count: {len(eval(components))}")
    else:
        print("error:", entry.get("error"))


def demo_html_output():
    """Demonstrate HTML output."""
    print("\n" + "=" * 60)
    print("HTML OUTPUT")
    print("=" * 60)

    payload = {
        "settings": {"output": "html", "minify": True},
        "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
    }

    renderer = Renderer()
    result = renderer.render(payload)
    entry = result["files"]["example.mdx"]

    if entry["status"] == "success":
        rendered = entry["result"]
        print(f"\nHTML: {rendered.get('output')}")
    else:
        print("error:", entry.get("error"))


if __name__ == "__main__":
    demo_schema_extraction()
    demo_html_output()
