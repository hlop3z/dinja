# Usage

1. Install the Python bindings (uses the same payload as the `/render` HTTP handler):

   ```bash
   cd python-bindings
   uv run maturin develop
   ```

2. Run the working README example:

   ```bash
   uv run python examples/readme_example.py
   ```

3. Or call the API directly:

   ```python
   from dinja import render

   PAYLOAD = {
       "settings": {
           "output": "schema",  # "html", "javascript", or "schema"
           "engine": "base",  # "base" or "custom"
           "minify": True,
           "components": ["Hero", "Feature"],
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
       # Supply component definitions when using the "custom" engine.
       "components": None,
   }

   result: dict = render(PAYLOAD)
   print(result)
   ```
