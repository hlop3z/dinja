"""Example: Rendering HTML with custom components via HTTP server.

This example demonstrates how to use the HTTP server to render MDX files
with custom components to HTML output.

Make sure the server is running first:
    ./run.sh
    # or
    cd core && cargo run --features http
"""

from __future__ import annotations

import json
import sys

try:
    import requests
except ImportError:
    print("Error: 'requests' library is required")
    print("Install it with: uv add requests")
    sys.exit(1)


def main() -> None:
    """Demonstrate custom component rendering to HTML via HTTP server."""
    print("Custom Components HTML Rendering Example (HTTP Client)")
    print("=" * 60)
    print()

    # Server URL
    server_url = "http://127.0.0.1:8080"
    render_endpoint = f"{server_url}/render"

    print(f"Server endpoint: {render_endpoint}")
    print()

    # Define custom components (using TSX/JSX syntax)
    components = {
        "Button": {
            "name": "Button",
            "code": "function Component(props) { return <button class={props.class || 'btn'}>{props.children}</button>; }",
            "docs": "A button component",
            "args": None,
        },
        "Card": {
            "name": "Card",
            "code": "function Component(props) { return <div class='card'><h3>{props.title}</h3><div class='card-content'>{props.children}</div></div>; }",
            "docs": "A card component with title and content",
            "args": None,
        },
        "Greeting": {
            "name": "Greeting",
            "code": "function Component(props) { return <div>Hello, <strong>{props.name}</strong>!</div>; }",
            "docs": "A greeting component",
            "args": None,
        },
    }

    # MDX content using custom components
    # Note: Using explicit HTML tags to avoid markdown creating nested p tags
    mdx_content = """---
title: Custom Components Demo
author: Dinja
---

# {context('title')}

Welcome to the custom components example!

<Greeting name="World" />

## Interactive Components

<Card title="Button Example">
<p>Click the button below to see custom components in action:</p>
<Button class="btn-primary">Submit</Button>
</Card>

<Card title="Another Card">
<p>This is another card component demonstrating reusability.</p>
<Button>Click Me</Button>
</Card>
"""

    # Prepare the render request
    payload = {
        "settings": {
            "output": "html",
            "minify": False,  # Set to False for readable output
            "engine": "custom",
            "components": [],
        },
        "mdx": {
            "demo.mdx": mdx_content,
        },
        "components": components,
    }

    print("Sending render request to server...")
    print("-" * 60)

    try:
        # Make HTTP POST request to the server
        response = requests.post(
            render_endpoint,
            json=payload,
            headers={"Content-Type": "application/json"},
            timeout=30,
        )

        # Check HTTP status
        if response.status_code == 200:
            result = response.json()
        elif response.status_code == 207:  # Multi-Status
            result = response.json()
            print("⚠ Warning: Some files failed to render (207 Multi-Status)")
        elif response.status_code == 400:
            error_data = response.json()
            print(f"✗ Bad Request: {error_data.get('error', 'Unknown error')}")
            sys.exit(1)
        elif response.status_code == 403:
            error_data = response.json()
            print(f"✗ Forbidden: {error_data.get('error', 'Unknown error')}")
            sys.exit(1)
        elif response.status_code == 500:
            try:
                error_data = response.json()
                error_msg = error_data.get("error", "Unknown error")
                print(f"✗ Internal Server Error: {error_msg}")
                # Print full error details if available
                if "error_chain" in error_data:
                    print("\nFull error details:")
                    print(error_data["error_chain"])
            except json.JSONDecodeError:
                print("✗ Internal Server Error (non-JSON response):")
                print(f"Status: {response.status_code}")
                print(f"Response: {response.text[:500]}")
            sys.exit(1)
        else:
            print(f"✗ Unexpected status code: {response.status_code}")
            print(f"Response headers: {dict(response.headers)}")
            print(f"Response text: {response.text[:500]}")
            try:
                error_data = response.json()
                print(f"Response JSON: {json.dumps(error_data, indent=2)}")
            except json.JSONDecodeError:
                pass
            sys.exit(1)

        # Check results
        print(f"Total files: {result['total']}")
        print(f"Succeeded: {result['succeeded']}")
        print(f"Failed: {result['failed']}")
        print()

        if result["failed"] > 0:
            print("Errors:")
            for error in result.get("errors", []):
                print(f"  - {error['file']}: {error['message']}")
            print()

        # Display rendered output
        file_result = result["files"]["demo.mdx"]

        if file_result["status"] == "success":
            rendered = file_result["result"]
            metadata = rendered.get("metadata", {})
            output = rendered.get("output", "")

            print("✓ Rendering successful!")
            print()
            print("Metadata:")
            print(f"  Title: {metadata.get('title', 'N/A')}")
            print(f"  Author: {metadata.get('author', 'N/A')}")
            print()

            print("Rendered HTML:")
            print("-" * 60)
            print(output)
            print("-" * 60)
            print()

            # Verify key HTML elements
            print("Verification:")
            checks = [
                ("<h1>", "Heading 1"),
                ("<button", "Button component"),
                ("<div", "Card component"),
                ("Hello,", "Greeting component"),
                ("<strong>World</strong>", "Nested content"),
            ]

            for check, description in checks:
                if check in output:
                    print(f"  ✓ {description}: Found")
                else:
                    print(f"  ✗ {description}: Not found")

        else:
            print(f"✗ Rendering failed: {file_result.get('error', 'Unknown error')}")

    except requests.exceptions.ConnectionError:
        print(f"✗ Connection error: Could not connect to server at {render_endpoint}")
        print("\nMake sure the server is running:")
        print("  ./run.sh")
        print("  # or")
        print("  cd core && cargo run --features http")
        sys.exit(1)
    except requests.exceptions.Timeout as e:
        print(f"✗ Request timeout: {e}")
        print("The server took too long to respond. Try again.")
        sys.exit(1)
    except requests.exceptions.RequestException as e:
        print(f"✗ HTTP request error: {e}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"✗ Failed to parse JSON response: {e}")
        print(f"Response text: {response.text[:200]}")
        sys.exit(1)
    except Exception as e:
        print(f"✗ Unexpected error: {e}")
        import traceback

        traceback.print_exc()

    print()
    print("=" * 60)
    print("Example completed!")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"Unexpected error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
