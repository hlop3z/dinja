import functools

from jinja2 import nodes
from jinja2.ext import Extension
from markupsafe import Markup


def jinja_tag_parse(self, parser):
    lineno = next(parser.stream).lineno
    args = [parser.parse_expression()]
    return nodes.CallBlock(
        self.call_method("_render_custom_template_tag", args), [], [], []
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
    filters = {}
    tags = {}

    def __new__(cls):
        """
        Ensure a singleton instance of Dinja is created.
        """
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.initialized = False
        return cls._instance

    def initialize(self, value):
        """
        Initialize the Dinja instance with a value.

        Parameters:
        - value (Any): Value to be initialized with.
        """
        if not self.initialized:
            self.value = value
            self.initialized = True

    @classmethod
    def filter(cls, method=None, **params):
        """
        Decorator to create custom filters for Jinja templates.

        Parameters:
        - method (function): The filter function.
        - params (dict): Optional arguments for the filter.

        Usage:
        @Dinja.filter(is_safe=True)
        def my_filter(value):
            return "Filtered: " + value

        Returns:
        - function: The wrapped filter function.
        """

        # Optional Arguments
        if method is None:
            return functools.partial(cls.filter, **params)

        # The Wrapper
        @functools.wraps(method)
        def the_wrapper(*args, **kwargs):
            is_safe = params.get("is_safe", False)
            result = method(*args, **kwargs)
            return Markup(result) if is_safe else result

        # Return @Decorator
        cls.filters[method.__name__] = the_wrapper
        return the_wrapper

    @classmethod
    def tag(cls, method=None, **params):
        """
        Decorator to create custom tags for Jinja templates.

        Parameters:
        - method (function): The tag function.
        - params (dict): Optional arguments for the tag.

        Usage:
        @Dinja.tag(is_safe=True)
        def my_tag(content, caller):
            return '<div>' + content + '</div>'

        Returns:
        - Extension: The custom Jinja2 extension class.
        """

        # Optional Arguments
        if method is None:
            return functools.partial(cls.tag, **params)

        # The Wrapper
        @functools.wraps(method)
        def the_wrapper(self, content, caller, *args, **kwargs):
            is_safe = params.get("is_safe", False)
            result = method(content, *args, **kwargs)
            return Markup(result) if is_safe else result

        extension = type(
            method.__name__.title().replace("_", ""),
            (Extension,),
            {
                "parse": jinja_tag_parse,
                "tags": {method.__name__},
                "_render_custom_template_tag": the_wrapper,
            },
        )

        # Return @Decorator
        cls.tags[method.__name__] = extension
        return extension

    @classmethod
    def load(cls, jinja_env):
        """
        Load custom filters and tags into a Jinja environment.

        Parameters:
        - jinja_env (jinja2.Environment): The Jinja environment to load into.
        """
        # Filters
        for key, apply_filter in cls.filters.items():
            jinja_env.filters[key] = apply_filter
        # Tags
        for tag in cls.tags.values():
            jinja_env.add_extension(tag)
