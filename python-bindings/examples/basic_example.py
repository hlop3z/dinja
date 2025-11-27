"""Example usage of the dinja Python bindings with utils.

This example demonstrates how to use the `utils` setting to inject
global JavaScript utilities via `export default { ... }`.
"""

from __future__ import annotations

from pprint import pprint
from typing import Any, Dict

from dinja import Renderer

# The payload matches the Input structure from the Rust core.
PAYLOAD: Dict[str, Any] = {
    "settings": {
        "output": "html",  # "html", "javascript", "schema", or "json"
        "minify": True,
        # Utils must use `export default` to return a single object
        "utils": """export default {
            formatDate: (date) => new Date(date).toLocaleDateString(),
            uppercase: (str) => str.toUpperCase(),
            siteName: "My Awesome Site"
        }""",
    },
    "mdx": {
        "home": """---
title: Home
---
# Welcome
This is the **{utils.siteName} | Home Page**""",
        "about": """---
title: About
author: Alice
---
## About Us
Some details {2+3} equals five""",
        "contact": """---
title: Contact
description: Contact us
---
<Hero title="Contact" description="Contact us" />""",
    },
    # Provide custom component definitions if needed
    "components": {
        "Hero": {
            "name": "Hero",
            "code": """export default function Component(props) {
    return <div class="hero"><h1>{props.title}</h1><p>{props.description}</p></div>;
}""",
        },
        "Feature": {
            "name": "Feature",
            "code": """export default function Component(props) {
    return <div class="feature">{props.children}</div>;
}""",
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
