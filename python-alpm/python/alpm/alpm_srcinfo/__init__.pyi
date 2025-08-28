"""A module for parsing and linting of ALPM SRCINFO files."""

from . import error, source_info
from .error import SourceInfoError
from .source_info.v1 import SourceInfoV1
from .source_info.v1.merged import MergedPackage

__all__ = [
    "SourceInfoError",
    "error",
    "source_info",
    "SourceInfoV1",
    "MergedPackage",
]
