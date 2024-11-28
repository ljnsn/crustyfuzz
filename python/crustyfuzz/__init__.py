# ruff: noqa: F405

import sys

from .crustyfuzz import *  # noqa: F403

sys.modules["crustyfuzz.fuzz"] = crustyfuzz.fuzz
sys.modules["crustyfuzz.process"] = crustyfuzz.process

__doc__ = crustyfuzz.__doc__
if hasattr(crustyfuzz, "__all__"):
    __all__ = crustyfuzz.__all__
