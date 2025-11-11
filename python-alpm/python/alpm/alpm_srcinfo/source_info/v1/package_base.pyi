"""Handling of metadata found in the pkgbase section of SRCINFO data."""

from typing import Optional

from alpm.alpm_srcinfo.source_info.v1.package import PackageArchitecture
from alpm.alpm_types import (
    FullVersion,
    Url,
    RelativeFilePath,
    License,
    Architectures,
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
    SkippableCrc32CksumChecksum,
)
from alpm.type_aliases import (
    MakepkgOption,
    OpenPGPIdentifier,
    RelationOrSoname,
    SystemArchitecture,
)

class PackageBaseArchitecture:
    """Architecture specific package base properties for use in PackageBase.

    For each Architecture defined in PackageBase.architectures a
    PackageBaseArchitecture is present in PackageBase.architecture_properties.
    """

    def __init__(self) -> None:
        """Initialize a new PackageBaseArchitecture with default values."""

    def merge_package_properties(self, properties: "PackageArchitecture") -> None:
        """Merge in the architecture specific properties of a package.

        Each existing field of properties overrides the architecture-independent
        pendant on self.

        Args:
            properties (PackageArchitecture): The architecture specific properties of a
                package to merge in.

        """

    @property
    def dependencies(self) -> list["RelationOrSoname"]:
        """The list of run-time dependencies of the package base."""

    @dependencies.setter
    def dependencies(self, dependencies: list["RelationOrSoname"]) -> None: ...
    @property
    def optional_dependencies(self) -> list["OptionalDependency"]:
        """The list of optional dependencies of the package base."""

    @optional_dependencies.setter
    def optional_dependencies(
        self, optional_dependencies: list["OptionalDependency"]
    ) -> None: ...
    @property
    def provides(self) -> list["RelationOrSoname"]:
        """The list of provisions of the package base."""

    @provides.setter
    def provides(self, provides: list["RelationOrSoname"]) -> None: ...
    @property
    def conflicts(self) -> list["PackageRelation"]:
        """The list of conflicts of the package base."""

    @conflicts.setter
    def conflicts(self, conflicts: list["PackageRelation"]) -> None: ...
    @property
    def replaces(self) -> list["PackageRelation"]:
        """The list of replacements of the package base."""

    @replaces.setter
    def replaces(self, replaces: list["PackageRelation"]) -> None: ...
    @property
    def check_dependencies(self) -> list["PackageRelation"]:
        """The list of test dependencies of the package base."""

    @check_dependencies.setter
    def check_dependencies(
        self, check_dependencies: list["PackageRelation"]
    ) -> None: ...
    @property
    def make_dependencies(self) -> list["PackageRelation"]:
        """The list of build dependencies of the package base."""

    @make_dependencies.setter
    def make_dependencies(self, make_dependencies: list["PackageRelation"]) -> None: ...
    @property
    def sources(self) -> list["Source"]:
        """The list of sources of the package base."""

    @sources.setter
    def sources(self, sources: list["Source"]) -> None: ...
    @property
    def b2_checksums(self) -> list["SkippableBlake2b512Checksum"]:
        """The list of Blake2 hash digests for sources of the package base."""

    @b2_checksums.setter
    def b2_checksums(
        self, b2_checksums: list["SkippableBlake2b512Checksum"]
    ) -> None: ...
    @property
    def md5_checksums(self) -> list["SkippableMd5Checksum"]:
        """The list of MD5 hash digests for sources of the package base."""

    @md5_checksums.setter
    def md5_checksums(self, md5_checksums: list["SkippableMd5Checksum"]) -> None: ...
    @property
    def sha1_checksums(self) -> list["SkippableSha1Checksum"]:
        """The list of SHA1 hash digests for sources of the package base."""

    @sha1_checksums.setter
    def sha1_checksums(self, sha1_checksums: list["SkippableSha1Checksum"]) -> None: ...
    @property
    def sha224_checksums(self) -> list["SkippableSha224Checksum"]:
        """The list of SHA224 hash digests for sources of the package base."""

    @sha224_checksums.setter
    def sha224_checksums(
        self, sha224_checksums: list["SkippableSha224Checksum"]
    ) -> None: ...
    @property
    def sha256_checksums(self) -> list["SkippableSha256Checksum"]:
        """The list of SHA256 hash digests for sources of the package base."""

    @sha256_checksums.setter
    def sha256_checksums(
        self, sha256_checksums: list["SkippableSha256Checksum"]
    ) -> None: ...
    @property
    def sha384_checksums(self) -> list["SkippableSha384Checksum"]:
        """The list of SHA384 hash digests for sources of the package base."""

    @sha384_checksums.setter
    def sha384_checksums(
        self, sha384_checksums: list["SkippableSha384Checksum"]
    ) -> None: ...
    @property
    def sha512_checksums(self) -> list["SkippableSha512Checksum"]:
        """The list of SHA512 hash digests for sources of the package base."""

    @sha512_checksums.setter
    def sha512_checksums(
        self, sha512_checksums: list["SkippableSha512Checksum"]
    ) -> None: ...
    @property
    def crc_checksums(self) -> list["SkippableCrc32CksumChecksum"]:
        """The list of CRC-32/CKSUM hash digests for sources of the package base."""

    @crc_checksums.setter
    def crc_checksums(
        self, crc_checksums: list["SkippableCrc32CksumChecksum"]
    ) -> None: ...
    def __eq__(self, other: object) -> bool: ...

class PackageBase:
    """Package base metadata based on the pkgbase section in SRCINFO data.

    All values in this struct act as default values for all Packages in the scope of
    specific SRCINFO data.

    A MergedPackage (a full view on a package's metadata) can be created using
    SourceInfoV1.packages_for_architecture.
    """

    def __init__(self, name: str, version: "FullVersion") -> None:
        """Create a new PackageBase from a name and a FullVersion.

        Uses the name and version and initializes all remaining fields of PackageBase
        with default values.

        Args:
            name (str): The name of the package base.
            version (FullVersion): The version of the package base.

        Raises:
            ALPMError: If the provided name is not valid.

        """

    @property
    def name(self) -> str:
        """The alpm-package-name of the package base."""

    @name.setter
    def name(self, name: str) -> None: ...
    @property
    def description(self) -> Optional[str]:
        """The optional description of the package base."""

    @description.setter
    def description(self, description: Optional[str]) -> None: ...
    @property
    def url(self) -> Optional["Url"]:
        """The optional upstream URL of the package base."""

    @url.setter
    def url(self, url: Optional["Url"]) -> None: ...
    @property
    def changelog(self) -> Optional["RelativeFilePath"]:
        """The optional changelog path of the package base."""

    @changelog.setter
    def changelog(self, changelog: Optional["RelativeFilePath"]) -> None: ...
    @property
    def licenses(self) -> list["License"]:
        """The list of licenses that apply to the package base."""

    @licenses.setter
    def licenses(self, licenses: list["License"]) -> None: ...
    @property
    def install(self) -> Optional["RelativeFilePath"]:
        """Relative path to an alpm-install-scriptlet of the package base."""

    @install.setter
    def install(self, install: Optional["RelativeFilePath"]) -> None: ...
    @property
    def groups(self) -> list[str]:
        """List of alpm-package-groups the package base is part of."""

    @groups.setter
    def groups(self, groups: list[str]) -> None: ...
    @property
    def options(self) -> list["MakepkgOption"]:
        """The list of build tool options used when building."""

    @options.setter
    def options(self, options: list["MakepkgOption"]) -> None: ...
    @property
    def backups(self) -> list["RelativeFilePath"]:
        """Relative paths to files in a package that should be backed up."""

    @backups.setter
    def backups(self, backups: list["RelativeFilePath"]) -> None: ...
    @property
    def version(self) -> "FullVersion":
        """The FullVersion of the package base."""

    @version.setter
    def version(self, version: "FullVersion") -> None: ...
    @property
    def pgp_fingerprints(self) -> list["OpenPGPIdentifier"]:
        """OpenPGPIdentifiers used for the verification of upstream sources."""

    @pgp_fingerprints.setter
    def pgp_fingerprints(self, pgp_fingerprints: list["OpenPGPIdentifier"]) -> None: ...
    @property
    def architectures(self) -> Architectures:
        """Architectures of the package base."""

    @architectures.setter
    def architectures(self, architectures: Architectures) -> None: ...
    @property
    def architecture_properties(
        self,
    ) -> dict[SystemArchitecture, PackageBaseArchitecture]:
        """Architecture specific properties.

        Dict of alpm-architecture specific overrides for package relations of a
        package base.
        """

    @architecture_properties.setter
    def architecture_properties(
        self, architecture_properties: dict[SystemArchitecture, PackageBaseArchitecture]
    ) -> None: ...
    @property
    def dependencies(self) -> list["RelationOrSoname"]:
        """The list of run-time dependencies of the package base."""

    @dependencies.setter
    def dependencies(self, dependencies: list["RelationOrSoname"]) -> None: ...
    @property
    def optional_dependencies(self) -> list["OptionalDependency"]:
        """The list of optional dependencies of the package base."""

    @optional_dependencies.setter
    def optional_dependencies(
        self, optional_dependencies: list["OptionalDependency"]
    ) -> None: ...
    @property
    def provides(self) -> list["RelationOrSoname"]:
        """The list of provisions of the package base."""

    @provides.setter
    def provides(self, provides: list["RelationOrSoname"]) -> None: ...
    @property
    def conflicts(self) -> list["PackageRelation"]:
        """The list of conflicts of the package base."""

    @conflicts.setter
    def conflicts(self, conflicts: list["PackageRelation"]) -> None: ...
    @property
    def replaces(self) -> list["PackageRelation"]:
        """The list of replacements of the package base."""

    @replaces.setter
    def replaces(self, replaces: list["PackageRelation"]) -> None: ...
    @property
    def check_dependencies(self) -> list["PackageRelation"]:
        """The list of test dependencies of the package base."""

    @check_dependencies.setter
    def check_dependencies(
        self, check_dependencies: list["PackageRelation"]
    ) -> None: ...
    @property
    def make_dependencies(self) -> list["PackageRelation"]:
        """The list of build dependencies of the package base."""

    @make_dependencies.setter
    def make_dependencies(self, make_dependencies: list["PackageRelation"]) -> None: ...
    @property
    def sources(self) -> list["Source"]:
        """The list of sources of the package base."""

    @sources.setter
    def sources(self, sources: list["Source"]) -> None: ...
    @property
    def no_extracts(self) -> list[str]:
        """The list of sources of the package base that are not extracted."""

    @no_extracts.setter
    def no_extracts(self, no_extracts: list[str]) -> None: ...
    @property
    def b2_checksums(self) -> list["SkippableBlake2b512Checksum"]:
        """The list of Blake2 hash digests for sources of the package base."""

    @b2_checksums.setter
    def b2_checksums(
        self, b2_checksums: list["SkippableBlake2b512Checksum"]
    ) -> None: ...
    @property
    def md5_checksums(self) -> list["SkippableMd5Checksum"]:
        """The list of MD5 hash digests for sources of the package base."""

    @md5_checksums.setter
    def md5_checksums(self, md5_checksums: list["SkippableMd5Checksum"]) -> None: ...
    @property
    def sha1_checksums(self) -> list["SkippableSha1Checksum"]:
        """The list of SHA1 hash digests for sources of the package base."""

    @sha1_checksums.setter
    def sha1_checksums(self, sha1_checksums: list["SkippableSha1Checksum"]) -> None: ...
    @property
    def sha224_checksums(self) -> list["SkippableSha224Checksum"]:
        """The list of SHA224 hash digests for sources of the package base."""

    @sha224_checksums.setter
    def sha224_checksums(
        self, sha224_checksums: list["SkippableSha224Checksum"]
    ) -> None: ...
    @property
    def sha256_checksums(self) -> list["SkippableSha256Checksum"]:
        """The list of SHA256 hash digests for sources of the package base."""

    @sha256_checksums.setter
    def sha256_checksums(
        self, sha256_checksums: list["SkippableSha256Checksum"]
    ) -> None: ...
    @property
    def sha384_checksums(self) -> list["SkippableSha384Checksum"]:
        """The list of SHA384 hash digests for sources of the package base."""

    @sha384_checksums.setter
    def sha384_checksums(
        self, sha384_checksums: list["SkippableSha384Checksum"]
    ) -> None: ...
    @property
    def sha512_checksums(self) -> list["SkippableSha512Checksum"]:
        """The list of SHA512 hash digests for sources of the package base."""

    @sha512_checksums.setter
    def sha512_checksums(
        self, sha512_checksums: list["SkippableSha512Checksum"]
    ) -> None: ...
    @property
    def crc_checksums(self) -> list["SkippableCrc32CksumChecksum"]:
        """The list of CRC-32/CKSUM hash digests for sources of the package base."""

    @crc_checksums.setter
    def crc_checksums(
        self, crc_checksums: list["SkippableCrc32CksumChecksum"]
    ) -> None: ...
    def __eq__(self, other: object) -> bool: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

__all__ = [
    "PackageBase",
    "PackageBaseArchitecture",
]
