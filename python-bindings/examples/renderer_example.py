"""Example demonstrating the Renderer class for reusable rendering.

This example shows how to use the Renderer class to avoid v8 isolate issues
when rendering with different output modes. The Renderer class maintains a
single service instance that reuses the renderer pool, preventing issues with
rapid successive renders with different modes.
"""

from __future__ import annotations

from dinja import Renderer


def main() -> None:
    """Demonstrate Renderer class usage."""
    print("Renderer Class Example")
    print("=" * 50)
    print()

    # Create a renderer instance (engine loads once)
    print("Creating Renderer instance...")
    renderer = Renderer()
    print("Renderer created successfully!")
    print()

    # Example 1: Render with HTML output
    print("Example 1: Rendering with HTML output")
    print("-" * 50)
    payload_html = {
        "settings": {"output": "html", "minify": True},
        "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
    }

    result_html = renderer.render(payload_html)
    entry_html = result_html["files"]["example.mdx"]

    if entry_html["status"] == "success":
        rendered = entry_html["result"]
        metadata = rendered.get("metadata", {})
        print(f"Title: {metadata.get('title')}")
        output_preview = rendered.get("output", "")[:100]
        print(f"Output preview: {output_preview}...")
    else:
        print(f"Error: {entry_html.get('error')}")
    print()

    # Example 2: Render with schema output (same instance, different mode)
    print("Example 2: Rendering with schema output (same instance)")
    print("-" * 50)
    payload_schema = {
        "settings": {"output": "schema", "minify": True},
        "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
    }

    result_schema = renderer.render(payload_schema)
    entry_schema = result_schema["files"]["example.mdx"]

    if entry_schema["status"] == "success":
        rendered = entry_schema["result"]
        metadata = rendered.get("metadata", {})
        print(f"Title: {metadata.get('title')}")
        output_preview = rendered.get("output", "")[:100]
        print(f"Output preview: {output_preview}...")
    else:
        print(f"Error: {entry_schema.get('error')}")
    print()

    # Example 3: Multiple renders with different modes (demonstrates fix)
    print("Example 3: Rapid successive renders with different modes")
    print("-" * 50)
    print("This demonstrates that the Renderer class prevents v8 isolate issues")
    print("when switching between different output modes on the same instance.")
    print()

    formats = ["html", "javascript", "schema"]
    for output_format in formats:
        payload = {
            "settings": {
                "output": output_format,
                "minify": False,
            },
            "mdx": {
                f"example-{output_format}.mdx": f"# Example {output_format.upper()}\n\nThis is an example.",
            },
        }

        try:
            result = renderer.render(payload)
            if result["succeeded"] > 0:
                filename = list(result["files"].keys())[0]
                outcome = result["files"][filename]
                if outcome["status"] == "success":
                    output = outcome["result"].get("output", "")
                    preview = output[:60] + "..." if len(output) > 60 else output
                    print(f"✓ {output_format.upper()}: {preview}")
                else:
                    print(f"✗ {output_format.upper()}: {outcome.get('error', 'Unknown error')}")
            else:
                print(f"✗ {output_format.upper()}: No files succeeded")
        except Exception as e:
            print(f"✗ {output_format.upper()}: Error - {e}")

    print()
    print("=" * 50)
    print("All renders completed successfully using the same Renderer instance!")
    print("This avoids v8 isolate issues that can occur with rapid mode switching.")


if __name__ == "__main__":
    try:
        main()
    except ImportError as e:
        print(f"Import error: {e}")
        print("\nMake sure dinja is installed:")
        print("  cd python-bindings")
        print("  maturin develop")
    except Exception as e:
        print(f"Unexpected error: {e}")
        import traceback

        traceback.print_exc()

