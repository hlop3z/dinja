"""Example usage of the dinja MDX rendering library.

This example demonstrates how to use the Renderer class to render MDX content
to HTML, JavaScript, or schema format.
"""

import json
from dinja import Renderer


def main():
    """Main example function."""
    # Create a renderer instance (engine loads once)
    renderer = Renderer()
    
    # Example 1: Simple MDX rendering
    print("Example 1: Simple MDX rendering")
    print("=" * 50)

    input_data = {
        "settings": {
            "output": "html",
            "minify": True,
        },
        "mdx": {
            "page1.mdx": "# Hello World\n\nThis is a simple MDX file.",
            "page2.mdx": "## Introduction\n\nThis is another page with **bold** text.",
        },
    }

    try:
        result = renderer.render(input_data)

        print(f"Total files processed: {result['total']}")
        print(f"Succeeded: {result['succeeded']}")
        print(f"Failed: {result['failed']}")
        print(f"\nErrors: {len(result.get('errors', []))}")

        if result["errors"]:
            for error in result["errors"]:
                print(f"  - {error['file']}: {error['message']}")

        print("\nRendered files:")
        for filename, outcome in result["files"].items():
            print(f"\n{filename}:")
            print(f"  Status: {outcome['status']}")
            if outcome["status"] == "success":
                if "result" in outcome and outcome["result"]:
                    rendered = outcome["result"]
                    print(
                        f"  Metadata: {json.dumps(rendered.get('metadata', {}), indent=4)}"
                    )
                    if "output" in rendered:
                        output_preview = (
                            rendered["output"][:100] + "..."
                            if len(rendered["output"]) > 100
                            else rendered["output"]
                        )
                        print(f"  Output preview: {output_preview}")
            else:
                print(f"  Error: {outcome.get('error', 'Unknown error')}")

    except ValueError as e:
        print(f"Validation error: {e}")
    except RuntimeError as e:
        print(f"Runtime error: {e}")

    # Example 2: Rendering with custom components
    print("\n\nExample 2: Rendering with custom components")
    print("=" * 50)

    input_data_custom = {
        "settings": {
            "output": "html",
            "minify": True,
        },
        "mdx": {
            "custom-page.mdx": "# Custom Component\n\n<MyComponent prop1='value' />",
        },
        "components": {
            "MyComponent": {
                "name": "MyComponent",
                "code": "export default function Component(props) { return <div>Custom: {props.prop1}</div>; }",
            },
        },
    }

    try:
        result = renderer.render(input_data_custom)
        print(
            f"Custom component rendering: {result['succeeded']} succeeded, {result['failed']} failed"
        )

        if result["files"]:
            filename = list(result["files"].keys())[0]
            outcome = result["files"][filename]
            if outcome["status"] == "success" and "result" in outcome:
                print(f"Successfully rendered {filename}")

    except Exception as e:
        print(f"Error: {e}")

    # Example 3: Different output formats
    print("\n\nExample 3: Different output formats")
    print("=" * 50)

    formats = ["html", "javascript", "schema"]

    for output_format in formats:
        input_data_format = {
            "settings": {
                "output": output_format,
                "minify": False,
            },
            "mdx": {
                f"example-{output_format}.mdx": "# Example\n\nThis is an example.",
            },
        }

        try:
            result = renderer.render(input_data_format)
            if result["succeeded"] > 0:
                filename = list(result["files"].keys())[0]
                outcome = result["files"][filename]
                if outcome["status"] == "success" and "result" in outcome:
                    output = outcome["result"].get("output", "")
                    preview = output[:80] + "..." if len(output) > 80 else output
                    print(f"{output_format.upper()}: {preview}")
        except Exception as e:
            print(f"{output_format.upper()}: Error - {e}")


if __name__ == "__main__":
    print("Dinja MDX Rendering Example")
    print("=" * 50)
    print()

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
