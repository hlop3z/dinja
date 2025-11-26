from dinja import Renderer


if __name__ == "__main__":
    payload = {
        "settings": {"output": "schema", "minify": True, "engine": "base"},
        "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
    }

    renderer = Renderer()
    result = renderer.render(payload)
    entry = result["files"]["example.mdx"]

    if entry["status"] == "success":
        rendered = entry["result"]
        metadata = rendered.get("metadata", {})
        print("title:", metadata.get("title"))
        print("html:", rendered.get("output"))
    else:
        print("error:", entry.get("error"))
