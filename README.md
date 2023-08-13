# Dinja: Jinja Tool-Kit

`Dinja` is a singleton class that provides decorators to create custom **`filters`** and **`tags`** for Jinja templates. It also allows you to load these filters and tags into a Jinja environment.

## Install

```sh
pip install dinja
```

## Usage

- Create custom **`filters`** using the `@Dinja.filter` decorator.
- Create custom **`tags`** using the `@Dinja.tag` decorator.
- Load custom filters and tags into a Jinja environment using `Dinja.load`.

## Method: `filter`

Decorator to create custom filters for Jinja templates.

**Parameters:**

- `method` (function): The filter function.
- `params` (dict): Optional arguments for the filter.

**Usage:**

```python
from dinja import Dinja

dinja = Dinja()

@dinja.filter(is_safe=True)
def my_filter(value):
    return "Filtered: " + value

@dinja.filter
def my_other_filter(value):
    return "Other Filtered: " + value
```

**Returns:**

- function: The wrapped filter function.

## Method: `tag`

Decorator to create custom tags for Jinja templates.

**Parameters:**

- `method` (function): The tag function.
- `params` (dict): Optional arguments for the tag.

**Usage:**

```python
from dinja import Dinja

dinja = Dinja()

@dinja.tag(is_safe=True)
def my_tag(content, caller):
    return '<h1>' + content + '</h1>'

@dinja.tag
def my_other_tag(content, caller):
    return '<div>' + content + '</div>'
```

**Returns:**

- Extension: The custom Jinja2 extension class.

### Method: `load`

Load custom filters and tags into a Jinja environment.

**Parameters:**

- `jinja_env` (jinja2.Environment): The Jinja environment to load into.

## Example

```python
from jinja2 import Environment

dinja = Dinja()
env = Environment()
dinja.load(env)
```
