"""README usage example for the dinja Python bindings.

This script mirrors the payload accepted by the `/render` HTTP handler in
`core/src/handlers.rs`. Run it after installing the bindings locally (see
the README for `uv run maturin develop` instructions).
"""

from __future__ import annotations

from pprint import pprint
from typing import Any, Dict

from dinja import Renderer

# The payload matches the Input structure from the Rust core.
PAYLOAD: Dict[str, Any] = {
    "settings": {
        "output": "schema",  # "html", "javascript", or "schema"
        "minify": True,
    },
    "mdx": {
        "home": "---\ntitle: Home\n---\n# Welcome\nThis is the **home page**",
        "about": (
            "---\ntitle: About\nauthor: Alice\n---\n"
            "## About Us\nSome details {2+3} equals five"
        ),
        "contact": (
            "---\ntitle: Contact\ndescription: Contact us\n---\n"
            "<Hero title={context('title')} description={context('description')} />"
        ),
    },
    # Provide custom component definitions if needed
    "components": {
        "Hero": {
            "name": "Hero",
            "code": "export default function Component(props) { return <div class='hero'><h1>{props.title}</h1><p>{props.description}</p></div>; }",
        },
        "Feature": {
            "name": "Feature",
            "code": "export default function Component(props) { return <div class='feature'>{props.children}</div>; }",
        },
    },
}


def main() -> None:
    renderer = Renderer()
    outcome = renderer.render(PAYLOAD)
    print(f"Processed {outcome['total']} file(s)")
    print(f"Succeeded: {outcome['succeeded']}  Failed: {outcome['failed']}")

    for filename, entry in outcome["files"].items():
        status = entry["status"]
        print(f"\n{filename}: {status}")

        if status == "success":
            rendered = entry.get("result") or {}
            metadata = rendered.get("metadata")
            if metadata:
                print("metadata:")
                pprint(metadata)

            output = rendered.get("output") or ""
            if output:
                preview = output[:160] + ("…" if len(output) > 160 else "")
                print(f"output preview: {preview}")
        else:
            print(f"error: {entry.get('error', 'unknown error')}")


if __name__ == "__main__":
    main()
