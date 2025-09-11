"""Handling of metadata found in the pkgbase section of SRCINFO data."""

from typing import Optional

from alpm.alpm_srcinfo.source_info.v1.package import PackageArchitecture
from alpm.alpm_types import (
    FullVersion,
    Url,
    RelativePath,
    License,
    Architecture,
    OptionalDependency,
    PackageRelation,
    Source,
    SkippableBlake2b512Checksum,
    SkippableMd5Checksum,
    SkippableSha1Checksum,
    SkippableSha224Checksum,
    SkippableSha256Checksum,
    SkippableSha384Checksum,
    SkippableSha512Checksum,
)
from alpm.type_aliases import MakepkgOption, OpenPGPIdentifier, RelationOrSoname

class PackageBase:
    """Package base metadata based on the pkgbase section in SRCINFO data.

    All values in this struct act as default values for all Packages in the scope of specific SRCINFO data.

    A MergedPackage (a full view on a package's metadata) can be created using SourceInfoV1.packages_for_architecture.
    """

    def __init__(self, name: str, version: "FullVersion"):
        """Create a new PackageBase from a name and a FullVersion.

        Uses the name and version and initializes all remaining fields of PackageBase with default values.

        Args:
            name (str): The name of the package base.
            version (FullVersion): The version of the package base.

        Raises:
            ALPMError: If the provided name is not valid.
        """

    @property
    def name(self) -> str:
        """The alpm-package-name of the package base."""

    @property
    def description(self) -> Optional[str]:
        """The optional description of the package base."""

    @property
    def url(self) -> Optional["Url"]:
        """The optional upstream URL of the package base."""

    @property
    def changelog(self) -> Optional["RelativePath"]:
        """The optional changelog path of the package base."""

    @property
    def licenses(self) -> list["License"]:
        """The list of licenses that apply to the package base."""

    @property
    def install(self) -> Optional["RelativePath"]:
        """The optional relative path to an alpm-install-scriptlet of the package base."""

    @property
    def groups(self) -> list[str]:
        """The optional list of alpm-package-groups the package base is part of."""

    @property
    def options(self) -> list["MakepkgOption"]:
        """The list of build tool options used when building."""

    @property
    def backups(self) -> list["RelativePath"]:
        """The list of relative paths to files in a package that should be backed up."""

    @property
    def version(self) -> "FullVersion":
        """The FullVersion of the package base."""

    @property
    def pgp_fingerprints(self) -> list["OpenPGPIdentifier"]:
        """The list of OpenPGP fingerprints of OpenPGP certificates used for the verification of upstream sources."""

    @property
    def architectures(self) -> list["Architecture"]:
        """Architectures and architecture specific properties"""

    @property
    def architecture_properties(self) -> dict[Architecture, PackageBaseArchitecture]:
        """Dict of alpm-architecture specific overrides for package relations of a package base."""

    @property
    def dependencies(self) -> list["RelationOrSoname"]:
        """The list of run-time dependencies of the package base."""

    @property
    def optional_dependencies(self) -> list["OptionalDependency"]:
        """The list of optional dependencies of the package base."""

    @property
    def provides(self) -> list["RelationOrSoname"]:
        """The list of provisions of the package base."""

    @property
    def conflicts(self) -> list["PackageRelation"]:
        """The list of conflicts of the package base."""

    @property
    def replaces(self) -> list["PackageRelation"]:
        """The list of replacements of the package base."""

    @property
    def check_dependencies(self) -> list["PackageRelation"]:
        """The list of test dependencies of the package base."""

    @property
    def make_dependencies(self) -> list["PackageRelation"]:
        """The list of build dependencies of the package base."""

    @property
    def sources(self) -> list["Source"]:
        """The list of sources of the package base."""

    @property
    def no_extracts(self) -> list[str]:
        """The list of sources of the package base that are not extracted."""

    @property
    def b2_checksums(self) -> list["SkippableBlake2b512Checksum"]:
        """The list of Blake2 hash digests for sources of the package base."""

    @property
    def md5_checksums(self) -> list["SkippableMd5Checksum"]:
        """The list of MD5 hash digests for sources of the package base."""

    @property
    def sha1_checksums(self) -> list["SkippableSha1Checksum"]:
        """The list of SHA1 hash digests for sources of the package base."""

    @property
    def sha224_checksums(self) -> list["SkippableSha224Checksum"]:
        """The list of SHA224 hash digests for sources of the package base."""

    @property
    def sha256_checksums(self) -> list["SkippableSha256Checksum"]:
        """The list of SHA256 hash digests for sources of the package base."""

    @property
    def sha384_checksums(self) -> list["SkippableSha384Checksum"]:
        """The list of SHA384 hash digests for sources of the package base."""

    @property
    def sha512_checksums(self) -> list["SkippableSha512Checksum"]:
        """The list of SHA512 hash digests for sources of the package base."""

    def __eq__(self, other: object) -> bool: ...

class PackageBaseArchitecture:
    """Architecture specific package base properties for use in PackageBase.

    For each Architecture defined in PackageBase.architectures a
    PackageBaseArchitecture is present in PackageBase.architecture_properties.
    """

    def merge_package_properties(self, properties: "PackageArchitecture") -> None:
        """Merges in the architecture specific properties of a package.

        Each existing field of properties overrides the architecture-independent pendant on self.
        """

    @property
    def dependencies(self) -> list["RelationOrSoname"]:
        """The list of run-time dependencies of the package base."""

    @property
    def optional_dependencies(self) -> list["OptionalDependency"]:
        """The list of optional dependencies of the package base."""

    @property
    def provides(self) -> list["RelationOrSoname"]:
        """The list of provisions of the package base."""

    @property
    def conflicts(self) -> list["PackageRelation"]:
        """The list of conflicts of the package base."""

    @property
    def replaces(self) -> list["PackageRelation"]:
        """The list of replacements of the package base."""

    @property
    def check_dependencies(self) -> list["PackageRelation"]:
        """The list of test dependencies of the package base."""

    @property
    def make_dependencies(self) -> list["PackageRelation"]:
        """The list of build dependencies of the package base."""

    @property
    def sources(self) -> list["Source"]:
        """The list of sources of the package base."""

    @property
    def b2_checksums(self) -> list["SkippableBlake2b512Checksum"]:
        """The list of Blake2 hash digests for sources of the package base."""

    @property
    def md5_checksums(self) -> list["SkippableMd5Checksum"]:
        """The list of MD5 hash digests for sources of the package base."""

    @property
    def sha1_checksums(self) -> list["SkippableSha1Checksum"]:
        """The list of SHA1 hash digests for sources of the package base."""

    @property
    def sha224_checksums(self) -> list["SkippableSha224Checksum"]:
        """The list of SHA224 hash digests for sources of the package base."""

    @property
    def sha256_checksums(self) -> list["SkippableSha256Checksum"]:
        """The list of SHA256 hash digests for sources of the package base."""

    @property
    def sha384_checksums(self) -> list["SkippableSha384Checksum"]:
        """The list of SHA384 hash digests for sources of the package base."""

    @property
    def sha512_checksums(self) -> list["SkippableSha512Checksum"]:
        """The list of SHA512 hash digests for sources of the package base."""

    def __eq__(self, other: object) -> bool: ...

__all__ = [
    "PackageBase",
    "PackageBaseArchitecture",
]
