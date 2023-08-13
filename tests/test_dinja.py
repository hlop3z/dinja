# Core
import pytest
from jinja2 import Environment, FileSystemLoader

# Module
from dinja import Dinja

# INIT
dinja = Dinja()

# Create a Jinja environment
jinja_env = Environment(
    loader=FileSystemLoader(
        "./tests/templates"
    )  # Load templates from the current directory
)


# Define Tools
@dinja.filter  # dinja.filter(is_safe=True)
def my_filter(value):
    return "Filtered: " + value.lower()


@dinja.tag  # dinja.tag(mode="value", is_safe=True)
def simple_tag():
    return "Simple Tag"


@dinja.tag(mode="value")
def my_value_tag(value):
    return "<h1>" + value + "</h1>"


@dinja.tag(mode="content")
def my_content_tag(content):
    return content


# Load Tools
dinja.load(jinja_env)


def test_simple_tag():
    template = jinja_env.get_template("simple_tag.html")
    results = template.render()
    assert results.strip() == "Simple Tag"


def test_value_tag():
    template = jinja_env.get_template("value_tag.html")
    results = template.render()
    assert results.strip() == "<h1>some_value</h1>"


def test_content_tag():
    template = jinja_env.get_template("content_tag.html")
    results = template.render()
    assert results.strip() == "<div>Custom Content</div>"


def test_my_filter():
    template = jinja_env.get_template("my_filter.html")
    results = template.render()
    assert results.strip() == "<div>Filtered: some-value</div>"
