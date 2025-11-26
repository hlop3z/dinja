"""
Public Python interface for the dinja MDX renderer.

This shim keeps the top-level `dinja` package pure Python so we can ship
typing markers while the heavy lifting lives in the `_native` extension
module compiled with PyO3.
"""

from __future__ import annotations

from importlib import import_module

_native = import_module("dinja._native")

render = _native.render

__all__ = ["render"]


