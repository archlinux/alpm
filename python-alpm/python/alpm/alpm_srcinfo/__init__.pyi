"""A module for parsing and linting of ALPM SRCINFO files."""

from . import error, source_info, schema
from .error import SourceInfoError
from .schema import SourceInfoSchema
from .source_info import source_info_from_file, source_info_from_str
from .source_info.v1 import SourceInfoV1
from .source_info.v1.merged import MergedPackage

__all__ = [
    "SourceInfoError",
    "error",
    "source_info",
    "SourceInfoV1",
    "MergedPackage",
    "schema",
    "SourceInfoSchema",
    "source_info_from_str",
    "source_info_from_file",
]
