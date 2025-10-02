"""Provides fully resolved package metadata derived from SRCINFO data."""

from typing import Optional, Union
from pathlib import Path

from alpm.alpm_srcinfo.source_info.v1.package import Package
from alpm.alpm_srcinfo.source_info.v1.package_base import PackageBase
from alpm.type_aliases import MakepkgOption, OpenPGPIdentifier, RelationOrSoname
from alpm.alpm_types import (
    Url,
    License,
    Architecture,
    FullVersion,
    PackageRelation,
    OptionalDependency,
    Source,
    SkippableBlake2b512Checksum,
    SkippableMd5Checksum,
    SkippableSha1Checksum,
    SkippableSha224Checksum,
    SkippableSha256Checksum,
    SkippableSha384Checksum,
    SkippableSha512Checksum,
)

class MergedPackage:
    """Fully resolved metadata of a single package based on SRCINFO data.

    This struct incorporates all PackageBase properties and the Package specific
    overrides in an architecture-specific representation of a package. It can be
    created using SourceInfoV1.packages_for_architecture.
    """

    def __init__(
        self,
        architecture: "Architecture",
        base: "PackageBase",
        package_or_name: Union[Package | str],
    ):
        """Create architecture-specific metadata representation of a package.

        Based on the provided parameters can either create a fully resolved or a basic
        (incomplete) MergedPackage.

        Args:
            architecture (Architecture): Defines the architecture for which to create
                the representation.
            base (PackageBase): The package base which provides the initial data.
            package_or_name (Union[Package, str]): Either the Package from which to
                derive the metadata for a fully resolved MergedPackage, or a name of
                the package for a basic, incomplete representation of a package.

        Raises:
            ALPMError: If the provided name is not valid.

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
        """Licenses that apply to the package."""

    @property
    def architecture(self) -> "Architecture":
        """Alpm-architecture for the package."""

    @property
    def changelog(self) -> Optional[Path]:
        """Path to a changelog file for the package."""

    @property
    def install(self) -> Optional[Path]:
        """Path to an alpm-install-scriptlet for the package."""

    @property
    def groups(self) -> list[str]:
        """Alpm-package-groups the package is part of."""

    @property
    def options(self) -> list["MakepkgOption"]:
        """Build tool options used when building the package."""

    @property
    def backups(self) -> list[Path]:
        """Paths to files in the package that should be backed up."""

    @property
    def version(self) -> "FullVersion":
        """The full version of the package."""

    @property
    def pgp_fingerprints(self) -> list["OpenPGPIdentifier"]:
        """OpenPGPIdentifiers used for the verification of upstream sources."""

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

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class MergedSource:
    """A merged representation of source related information.

    SRCINFO provides this info as separate lists. This struct resolves that list
    representation and provides a convenient aggregated representation for a single
    source.
    """

    @property
    def source(self) -> "Source":
        """The source."""

    @property
    def b2_checksum(self) -> Optional[SkippableBlake2b512Checksum]:
        """The optional Blake2 hash digest of source."""

    @property
    def md5_checksum(self) -> Optional[SkippableMd5Checksum]:
        """The optional MD-5 hash digest of source."""

    @property
    def sha1_checksum(self) -> Optional[SkippableSha1Checksum]:
        """The optional SHA-1 hash digest of source."""

    @property
    def sha224_checksum(self) -> Optional[SkippableSha224Checksum]:
        """The optional SHA-224 hash digest of source."""

    @property
    def sha256_checksum(self) -> Optional[SkippableSha256Checksum]:
        """The optional SHA-256 hash digest of source."""

    @property
    def sha384_checksum(self) -> Optional[SkippableSha384Checksum]:
        """The optional SHA-384 hash digest of source."""

    @property
    def sha512_checksum(self) -> Optional[SkippableSha512Checksum]:
        """The optional SHA-512 hash digest of source."""

__all__ = [
    "MergedPackage",
    "MergedSource",
]
