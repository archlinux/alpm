"""Schemas for SRCINFO data."""

from pathlib import Path
from typing import Union, TYPE_CHECKING

if TYPE_CHECKING:
    from alpm.alpm_types import SchemaVersion

class SourceInfoSchema:
    """SRCINFO schema.

    The schema of a SRCINFO refers to the minimum required sections and keywords, as
    well as the complete set of available keywords in a specific version.
    """

    def __init__(self, version: Union[str, "SchemaVersion"]) -> None:
        """Create a SourceInfoSchema from SchemaVersion or its string representation.

        Args:
            version (Union[str, SchemaVersion]): either a SchemaVersion or a string
                representation of SchemaVersion.

        Raises:
            ALPMError: if the string representation of the version is invalid.
            SourceInfoError: if there is no corresponding schema for the provided major
                version.

        """

    @staticmethod
    def derive_from_str(srcinfo: str) -> "SourceInfoSchema":
        """Derive the schema from a string containing SRCINFO data.

        Args:
            srcinfo (str): The srcinfo string to derive the schema from.

        Returns:
            SourceInfoSchema: The derived schema.

        Raises:
            SourceInfoError: if the srcinfo string cannot be parsed.

        """

    @staticmethod
    def derive_from_file(path: Union[str, Path]) -> "SourceInfoSchema":
        """Derive the schema from a file containing SRCINFO data.

        Args:
            path (Union[str, Path]): The path to the file containing SRCINFO data.

        Returns:
            SourceInfoSchema: The derived schema.

        Raises:
            SourceInfoError: if the file cannot be read or parsed.

        """

    @property
    def version(self) -> "SchemaVersion":
        """The schema version."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

__all__ = [
    """SourceInfoSchema""",
]
