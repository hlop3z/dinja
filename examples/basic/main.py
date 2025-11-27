from dinja import Renderer, Input, Settings

# Create a renderer instance (engine loads once)
renderer = Renderer()
result = renderer.render(
    Input(
        mdx={
            "page.mdx": """
---
title: Welcome
author: Alice
---
# {context('title')}

By {context('author')}
"""
        },
        settings=Settings(output="html"),
    )
)

# print(result)
