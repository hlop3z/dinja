import functools
from typing import Any, Callable, Dict, Optional

from jinja2 import Environment, nodes
from jinja2.ext import Extension
from markupsafe import Markup


def parse_render_no_content_tag(self, parser):
    lineno = next(parser.stream).lineno
    args = []
    return nodes.CallBlock(
        self.call_method("_render_no_content_tag", args), [], [], []
    ).set_lineno(lineno)


def parse_render_content_tag(self, parser, name):
    lineno = next(parser.stream).lineno
    body = parser.parse_statements([f"name:end{name}"], drop_needle=True)
    return nodes.CallBlock(
        self.call_method("_render_custom_template_tag", []), [], [], body
    ).set_lineno(lineno)


def parse_render_value_tag(self, parser):
    lineno = next(parser.stream).lineno
    args = [parser.parse_expression()]
    return nodes.CallBlock(
        self.call_method("_render_value_tag", args), [], [], []
    ).set_lineno(lineno)


class Dinja:
    """
    Dinja: Jinja (Filters & Tags).

    A singleton class that provides decorators to create custom filters and tags
    for Jinja templates. Also allows you to load these filters and tags into a
    Jinja environment.

    Usage:
    - Create custom filters using the `@Dinja.filter` decorator.
    - Create custom tags using the `@Dinja.tag` decorator.
    - Load custom filters and tags into a Jinja environment using `Dinja.load`.

    Example:
    dinja = Dinja()
    env = jinja2.Environment()
    dinja.load(env)
    """

    _instance = None
    filters: Dict[str, Callable] = {}
    tags: Dict[str, Extension] = {}

    def __new__(cls) -> "Dinja":
        """
        Ensure a singleton instance of Dinja is created.
        """
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.initialized = False
        return cls._instance

    def initialize(self, value: Any) -> None:
        """
        Initialize the Dinja instance with a value.

        Parameters:
        - value (Any): Value to be initialized with.
        """
        if not self.initialized:
            self.value = value
            self.initialized = True

    @classmethod
    def filter(cls, method: Optional[Callable] = None, **params: Any) -> Callable:
        """
        Decorator to create `custom-filters` for Jinja templates.

        Parameters (`Optional`):
        - `is_safe` (bool): Indicate whether the tag result is safe for HTML rendering (default: `False`).

        Usage:
        >>> @Dinja.filter(is_safe=True)
        >>> def my_filter(value):
        >>>     return "Filtered: " + value

        Returns:
        - Filter: The custom Jinja2 filter.
        """
        if method is None:
            return functools.partial(cls.filter, **params)

        @functools.wraps(method)
        def the_wrapper(*args: Any, **kwargs: Any) -> Any:
            is_safe = params.get("is_safe", False)
            result = method(*args, **kwargs)
            return Markup(result) if is_safe else result

        cls.filters[method.__name__] = the_wrapper
        return the_wrapper

    @classmethod
    def tag(cls, method: Optional[Callable] = None, **params: Any) -> Extension:
        """
        Decorator to create `custom-tags` for Jinja templates.

        Parameters (`Optional`):
        - `mode` (str): Jinja tag's current mode, options: {`simple`, `value`, `content`} (default: `simple`).
        - `is_safe` (bool): Indicate whether the tag result is safe for HTML rendering (default: `False`).

        Usage:
        >>> @Dinja.tag(is_safe=True)
        >>> def my_tag(content, caller):
        >>>     return '<div>' + content + '</div>'

        Returns:
        - Extension: The custom Jinja2 extension class representing the tag.

        Examples:
            The following example demonstrates how to use the `@Dinja.tag` decorator to create a custom tag.

            >>> @Dinja.tag(is_safe=True)
            >>> def my_tag(content, caller):
            >>>     return '<div>' + content + '</div>'
        """
        if method is None:
            return functools.partial(cls.tag, **params)

        ext_mode = params.get("mode", "simple")
        is_safe = params.get("is_safe", False)

        @functools.wraps(method)
        def render_no_content_tag_wrapper(
            self, caller: Any, *args: Any, **kwargs: Any
        ) -> Any:
            result = method(*args, **kwargs)
            return Markup(result) if is_safe else result

        @functools.wraps(method)
        def render_value_tag_wrapper(
            self, content: str, caller: Any, *args: Any, **kwargs: Any
        ) -> Any:
            result = method(content, *args, **kwargs)
            return Markup(result) if is_safe else result

        @functools.wraps(method)
        def render_content_tag_wrapper(
            self, caller: Any, *args: Any, **kwargs: Any
        ) -> Any:
            content = caller()
            result = method(content, *args, **kwargs)
            return Markup(result) if is_safe else result

        # Build Extension
        ext_name = method.__name__
        ext_config = {
            "tags": {ext_name},
        }

        match ext_mode:
            case "simple":
                ext_config["_render_no_content_tag"] = render_no_content_tag_wrapper
                ext_config["parse"] = parse_render_no_content_tag
            case "value":
                ext_config["_render_value_tag"] = render_value_tag_wrapper
                ext_config["parse"] = parse_render_value_tag
            case "content":
                ext_config["_render_custom_template_tag"] = render_content_tag_wrapper
                ext_config["parse"] = lambda self, parser: parse_render_content_tag(
                    self, parser, ext_name
                )

        extension = type(
            method.__name__.title().replace("_", ""),
            (Extension,),
            ext_config,
        )

        cls.tags[method.__name__] = extension
        return extension

    @classmethod
    def load(cls, jinja_env: Environment) -> None:
        """
        Load custom filters and tags into a Jinja environment.

        Parameters:
        - jinja_env (jinja2.Environment): The Jinja environment to load into.
        """
        for key, apply_filter in cls.filters.items():
            jinja_env.filters[key] = apply_filter
        for tag in cls.tags.values():
            jinja_env.add_extension(tag)
