import dinja


if __name__ == "__main__":
    payload = {
        "settings": {"output": "schema", "minify": True, "engine": "base"},
        "mdx": {"example.mdx": "---\ntitle: Demo\n---\n# Hello **dinja**"},
    }

    result = dinja.render(payload)
    entry = result["files"]["example.mdx"]

    if entry["status"] == "success":
        rendered = entry["result"]
        metadata = rendered.get("metadata", {})
        print("title:", metadata.get("title"))
        print("html:", rendered.get("output"))
    else:
        print("error:", entry.get("error"))
