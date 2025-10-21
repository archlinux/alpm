"""Contains the second parsing and linting pass."""

from pathlib import Path
from typing import Union

from . import merged, package, package_base
from .package import Package
from .package_base import PackageBase
from .merged import MergedPackage
from alpm.alpm_types import Architecture

class SourceInfoV1:
    """The representation of SRCINFO data.

    Provides access to a PackageBase which tracks all data in a pkgbase section and a
    list of Package instances that provide the accumulated data of all pkgname sections.

    This is the entry point for parsing SRCINFO files. Once created,
    packages_for_architecture method can be used to create usable MergedPackages.
    """

    __hash__ = None  # type: ignore

    def __init__(self, content: str):
        """Create SourceInfoV1 from a string representation.

        Args:
            content (str): The content of a SRCINFO as a string.

        Raises:
            SourceInfoError: If the content is not a valid SRCINFO representation.

        """

    @staticmethod
    def from_file(path: Union[Path, str]) -> "SourceInfoV1":
        """Read the file at the specified path and convert it into a SourceInfoV1.

        Args:
            path (Path | str): The path to the SRCINFO file.

        Returns:
            SourceInfoV1: The SourceInfoV1 instance created from the file content.

        Raises:
            SourceInfoError: If the file content is not a valid SRCINFO representation.

        """

    @staticmethod
    def from_pkgbuild(path: Union[Path, str]) -> "SourceInfoV1":
        """Create a SourceInfoV1 from a PKGBUILD file.

        Args:
            path (Path | str): The path to the PKGBUILD file.

        Returns:
            SourceInfoV1: The SourceInfoV1 instance created from the PKGBUILD file.

        Raises:
            SourceInfoError: If the PKGBUILD file cannot be parsed or is invalid.

        """

    @property
    def base(self) -> "PackageBase":
        """The information of the pkgbase section."""

    @property
    def packages(self) -> list["Package"]:
        """The information of the pkgname sections."""

    def as_srcinfo(self) -> str:
        """Get a string representation in valid SRCINFO format.

        Returns:
            str: The string representation of the SourceInfoV1 instance.

        """

    def packages_for_architecture(
        self, architecture: "Architecture"
    ) -> list["MergedPackage"]:
        """Get a list of all packages for architecture.

        Args:
            architecture (Architecture): The architecture to get packages for.

        Returns:
            list[MergedPackage]: A list of all packages for the given architecture.

        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

__all__ = [
    "SourceInfoV1",
    "merged",
    "package",
    "package_base",
]
