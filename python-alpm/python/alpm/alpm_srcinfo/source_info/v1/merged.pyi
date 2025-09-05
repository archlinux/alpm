"""Provides fully resolved package metadata derived from SRCINFO data."""

from typing import Optional

from alpm.type_aliases import MakepkgOption, OpenPGPIdentifier, RelationOrSoname
from alpm.alpm_types import (
    Url,
    License,
    Architecture,
    RelativePath,
    FullVersion,
    PackageRelation,
    OptionalDependency,
)

class MergedPackage:
    """Fully resolved metadata of a single package based on SRCINFO data.

    This struct incorporates all PackageBase properties and the Package specific overrides in an architecture-specific
    representation of a package. It can be created using SourceInfoV1.packages_for_architecture.
    """

    @property
    def name(self) -> str:
        """The alpm-package-name for the package."""

    @property
    def description(self) -> Optional[str]:
        """The description for the package."""

    @property
    def url(self) -> Optional["Url"]:
        """The upstream URL for the package."""

    @property
    def licenses(self) -> list["License"]:
        """The list of licenses that apply to the package."""

    @property
    def architecture(self) -> "Architecture":
        """The alpm-architecture for the package."""

    @property
    def changelog(self) -> Optional["RelativePath"]:
        """The optional relative path to a changelog file for the package."""

    @property
    def install(self) -> Optional["RelativePath"]:
        """The optional relative path to an alpm-install-scriptlet for the package."""

    @property
    def groups(self) -> list[str]:
        """The list of alpm-package-groups the package is part of."""

    @property
    def options(self) -> list["MakepkgOption"]:
        """The list of build tool options used when building the package."""

    @property
    def backups(self) -> list["RelativePath"]:
        """The list of relative paths to files in the package that should be backed up."""

    @property
    def version(self) -> "FullVersion":
        """The full version of the package."""

    @property
    def pgp_fingerprints(self) -> list["OpenPGPIdentifier"]:
        """The list of OpenPGP fingerprints of OpenPGP certificates used for the verification of upstream sources."""

    @property
    def dependencies(self) -> list["RelationOrSoname"]:
        """The list of run-time dependencies."""

    @property
    def optional_dependencies(self) -> list["OptionalDependency"]:
        """The list of optional dependencies."""

    @property
    def provides(self) -> list["RelationOrSoname"]:
        """The list of provisions."""

    @property
    def conflicts(self) -> list["PackageRelation"]:
        """The list of conflicts."""

    @property
    def replaces(self) -> list["PackageRelation"]:
        """The list of replacements."""

    @property
    def check_dependencies(self) -> list["PackageRelation"]:
        """The list of test dependencies."""

    @property
    def make_dependencies(self) -> list["PackageRelation"]:
        """The list of build dependencies."""

    @property
    def sources(self) -> list["MergedSource"]:
        """The list of sources for the package."""

    @property
    def no_extracts(self) -> list[str]:
        """The list of sources for the package that are not extracted."""

class MergedSource: ...

__all__ = [
    "MergedPackage",
    "MergedSource",
]
