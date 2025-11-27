# Components

Dinja supports custom components in your MDX files when using the `custom` engine.

## Defining Components

Components are defined as JavaScript functions that return JSX:

```python
components = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }",
    "Card": "function Component(props) { return <div class='card'>{props.children}</div>; }",
}
```

## Using Components in MDX

```mdx
# My Page

<Card>
  <Button>Click me</Button>
</Card>
```

## Python Example

```python
from dinja import Renderer, Input, Settings

components = {
    "Button": "function Component(props) { return <button>{props.children}</button>; }",
    "Card": "function Component(props) { return <div class='card'>{props.children}</div>; }",
}

mdx_content = """
# My Page

<Card>
  <Button>Click me</Button>
</Card>
"""

renderer = Renderer()
result = renderer.render(
    Input(
        mdx={"page.mdx": mdx_content},
        settings=Settings(),
        components=components,
    )
)
```

## Component Props

Components receive props as a JavaScript object:

```python
components = {
    "Greeting": """
        function Component(props) {
            return <h1>Hello, {props.name}!</h1>;
        }
    """,
}
```

```mdx
<Greeting name="World" />
```

## Best Practices

1. **Keep Components Simple** – Components should be pure functions when possible
2. **Use Props** – Pass data through props rather than accessing global state
3. **Reuse Components** – Define components once and reuse across multiple MDX files
4. **Type Safety** – Use TypeScript-style prop definitions when possible

