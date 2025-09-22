"""Data representations and integrations for reading of SRCINFO data."""

from pathlib import Path
from typing import Optional, TYPE_CHECKING, Union

from . import v1

if TYPE_CHECKING:
    from alpm.alpm_srcinfo.schema import SourceInfoSchema
    from alpm.type_aliases import SourceInfo

def source_info_from_str(
    s: str, schema: Optional["SourceInfoSchema"] = None
) -> "SourceInfo":
    """Create a SourceInfo object from a string.

    Optionally validated using a SourceInfoSchema.

    If schema is None, attempts to detect the SourceInfoSchema from s.

    Args:
        s (str): The srcinfo string to parse.
        schema (Optional[SourceInfoSchema]): The schema to validate against.

    Returns:
        SourceInfo: The parsed SourceInfo object.

    Raises:
        SourceInfoError: If the string is not a valid SRCINFO.

    """

def source_info_from_file(
    path: Union[str, Path], schema: Optional["SourceInfoSchema"] = None
) -> "SourceInfo":
    """Create a SourceInfo object from a file.

    Optionally validated using a SourceInfoSchema.

    If schema is None, attempts to detect the SourceInfoSchema from the file contents.

    Args:
        path (Union[str, Path]): The path to the file containing SRCINFO data.
        schema (Optional[SourceInfoSchema]): The schema to validate against.

    Returns:
        SourceInfo: The parsed SourceInfo object.

    Raises:
        SourceInfoError: If the file cannot be read or is not a valid SRCINFO.

    """

__all__ = ["v1", "source_info_from_str", "source_info_from_file"]
