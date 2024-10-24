# ruff: noqa: F405

import sys

from .crustyfuzz import *  # noqa: F403

sys.modules["crustyfuzz.distance"] = crustyfuzz.distance

__doc__ = crustyfuzz.__doc__
if hasattr(crustyfuzz, "__all__"):
    __all__ = crustyfuzz.__all__
