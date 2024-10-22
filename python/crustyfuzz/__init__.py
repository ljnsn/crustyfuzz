"""Rusty string matching library."""

from .crustyfuzz import *  # noqa: F403

# ruff: noqa: F405

__doc__ = crustyfuzz.__doc__
if hasattr(crustyfuzz, "__all__"):
    __all__ = crustyfuzz.__all__
