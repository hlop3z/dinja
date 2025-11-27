"""Example usage of the dinja Python bindings with the dataclass API.

This example demonstrates how to use the type-safe dataclass API with
Settings, Input, and ComponentDefinition classes.
"""

from __future__ import annotations

from pprint import pprint

from dinja import ComponentDefinition, Input, Renderer, Settings


def main() -> None:
    """Main example function demonstrating the dataclass API."""
    print("Dinja Python Bindings - Dataclass API Example")
    print("=" * 60)
    print()

    # Create a renderer instance
    renderer = Renderer()

    # Example 1: Simple MDX rendering with Settings
    print("Example 1: Simple MDX rendering")
    print("-" * 60)

    result = renderer.render(
        Input(
            settings=Settings(output="html", minify=False),
            mdx={"hello.mdx": "# Hello **dinja**\n\nThis is rendered from Python!"},
        )
    )

    print(f"Status: {result['succeeded']}/{result['total']} succeeded")
    file_result = result["files"]["hello.mdx"]
    if file_result["status"] == "success":
        print(f"Output: {file_result['result']['output']}")
    print()

    # Example 2: Using utils for global JavaScript utilities
    print("Example 2: Global utils")
    print("-" * 60)

    result = renderer.render(
        Input(
            settings=Settings(
                output="html",
                minify=False,
                utils="export default { siteName: 'My Site', version: '1.0.0' }",
            ),
            mdx={"page.mdx": "<SiteInfo />"},
            components={
                "SiteInfo": ComponentDefinition(
                    code="""export default function Component() {
    return <p>Welcome to {utils.siteName} v{utils.version}</p>;
}"""
                )
            },
        )
    )

    file_result = result["files"]["page.mdx"]
    if file_result["status"] == "success":
        print(f"Output: {file_result['result']['output']}")
    print()

    # Example 3: Custom components with simple dict syntax
    print("Example 3: Custom components (simple dict syntax)")
    print("-" * 60)

    # You can use a simple dict of name -> code strings
    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={"buttons.mdx": "<Button>Click me</Button>\n<Button>Submit</Button>"},
            # Simple dict syntax - automatically converted to ComponentDefinition
            components={
                "Button": """export default function Component({ children }) {
    return <button class="btn">{children}</button>;
}"""
            },
        )
    )

    file_result = result["files"]["buttons.mdx"]
    if file_result["status"] == "success":
        print(f"Output: {file_result['result']['output']}")
    print()

    # Example 4: YAML frontmatter extraction
    print("Example 4: YAML frontmatter")
    print("-" * 60)

    result = renderer.render(
        Input(
            settings=Settings(output="html"),
            mdx={
                "blog.mdx": """---
title: My Blog Post
author: Alice
date: 2024-01-15
tags:
  - python
  - rust
  - mdx
---

# {frontmatter.title}

Written by **{frontmatter.author}**
"""
            },
        )
    )

    file_result = result["files"]["blog.mdx"]
    if file_result["status"] == "success":
        print("Metadata:")
        pprint(file_result["result"]["metadata"])
        print(f"\nOutput preview: {file_result['result']['output'][:100]}...")
    print()

    # Example 5: Multiple output formats
    print("Example 5: Different output formats")
    print("-" * 60)

    for output_format in ["html", "javascript", "schema"]:
        result = renderer.render(
            Input(
                settings=Settings(output=output_format, minify=False),  # type: ignore
                mdx={"test.mdx": "# Hello\n\nWorld"},
            )
        )
        file_result = result["files"]["test.mdx"]
        if file_result["status"] == "success":
            output = file_result["result"]["output"]
            preview = output[:60] + "..." if len(output) > 60 else output
            print(f"{output_format.upper():12} {preview}")

    print()
    print("=" * 60)
    print("All examples completed successfully!")


if __name__ == "__main__":
    main()
