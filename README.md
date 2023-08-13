# **Dinja**: `Jinja` Tool-Kit

`Dinja` is a singleton class that provides decorators to create custom **`filters`** and **`tags`** for Jinja templates. It also allows you to load these filters and tags into a Jinja environment.

## Install

```sh
pip install dinja Jinja2
```

## Usage

- **`Load`** custom filters and tags into a Jinja environment using `Dinja.load`.
- Create custom **`filters`** using the `@Dinja.filter` decorator.
- Create custom **`tags`** using the `@Dinja.tag` decorator.

### Method: `load`

Load custom filters and tags into a Jinja environment.

**Parameters:**

- `jinja_env` (jinja2.Environment): The Jinja environment to load into.

## Example

```python
from jinja2 import Environment, FileSystemLoader
from dinja import Dinja

dinja = Dinja()
jinja_env = Environment(loader=FileSystemLoader("./templates"))

dinja.load(jinja_env)
```

## Decorator: `filter`

Decorator to create custom filters for Jinja templates.

**Parameters:**

- `method` (function): The filter function.
- `is_safe` (bool): Indicate whether the tag result is safe for HTML rendering (default: `False`).

**Usage:**

```python
from dinja import Dinja

dinja = Dinja()

@dinja.filter # dinja.filter(is_safe=True)
def my_filter(value):
    return "Filtered: " + value.lower()
```

```html
<div>{{ "Some-Value" | my_filter }}</div>
```

**Returns:**

- **`Filter`**: The wrapped filter function.

## Decorator: `tag`

Decorator to create custom tags for Jinja templates.

**Parameters:**

- `method` (function): The tag function.
- `mode` (str): Jinja tag's current mode, options: {`simple`, `value`, `content`} (default: `simple`).
- `is_safe` (bool): Indicate whether the tag result is safe for HTML rendering (default: `False`).

**Usage:**

```python
from dinja import Dinja

dinja = Dinja()

@dinja.tag # dinja.tag(mode="value", is_safe=True)
def simple_tag():
    return "Simple Tag"

@dinja.tag(mode="value")
def my_value_tag(value):
    return '<h1>' + value + '</h1>'

@dinja.tag(mode="content")
def my_content_tag(content):
    return content
```

```html
<!-- @Example: { simple_tag }  -->
{% simple_tag %}

<!-- @Example: { my_value_tag }  -->
{% my_value_tag "some_value" %}

<!-- @Example: { my_content_tag }  -->
{% my_content_tag %}

<div>Custom Content</div>

{% endmy_content_tag %}
```

**Returns:**

- **`Extension`**: The custom Jinja2 extension class.
