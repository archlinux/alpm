"""Type aliases for various submodules of the alpm package."""

from typing import TypeAlias, Union, Any

from alpm.alpm_types import (
    Blake2b512Checksum,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
    Crc32CksumChecksum,
    SkippableBlake2b512Checksum,
    SkippableMd5Checksum,
    SkippableSha1Checksum,
    SkippableSha224Checksum,
    SkippableSha256Checksum,
    SkippableSha384Checksum,
    SkippableSha512Checksum,
    SkippableCrc32CksumChecksum,
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
    BuildEnvironmentOption,
    PackageOption,
    PackageVersion,
    SonameV1,
    SonameV2,
    PackageRelation,
    BzrInfo,
    FossilInfo,
    GitInfo,
    HgInfo,
    SvnInfo,
    KnownArchitecture,
    UnknownArchitecture,
)

from alpm.alpm_srcinfo import SourceInfoV1

Checksum: TypeAlias = Union[
    Blake2b512Checksum,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
    Crc32CksumChecksum,
]
"""A checksum using a supported algorithm"""

SkippableChecksum: TypeAlias = Union[
    SkippableBlake2b512Checksum,
    SkippableMd5Checksum,
    SkippableSha1Checksum,
    SkippableSha224Checksum,
    SkippableSha256Checksum,
    SkippableSha384Checksum,
    SkippableSha512Checksum,
    SkippableCrc32CksumChecksum,
]
""" A skippable checksum using a supported algorithm.

Strings representing checksums are used to verify the integrity of files.
If the "SKIP" keyword is found, the integrity check is skipped.
"""

OpenPGPIdentifier: TypeAlias = Union[
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
]
"""An OpenPGP key identifier.

The OpenPGPIdentifier type represents a valid OpenPGP identifier, which can be either
an OpenPGP Key ID or an OpenPGP v4 fingerprint.
"""

MakepkgOption: TypeAlias = Union[
    BuildEnvironmentOption,
    PackageOption,
]
"""Either PackageOption or BuildEnvironmentOption.

This is necessary for metadata files such as SRCINFO or PKGBUILD package scripts that 
don't differentiate between the different types and scopes of options.
"""

VersionOrSoname: TypeAlias = Union[
    PackageVersion,
    str,
]
"""Either a PackageVersion or a string representing a shared object name."""

Soname: TypeAlias = Union[
    SonameV1,
    SonameV2,
]
"""Either a SonameV1 or SonameV2 record."""

RelationOrSoname: TypeAlias = Union[
    Soname,
    PackageRelation,
]
"""Either a SonameV1, SonameV2, or a PackageRelation."""

SystemArchitecture: TypeAlias = Union[KnownArchitecture, UnknownArchitecture]
"""A specific system architecture.

Either KnownArchitecture or UnknownArchitecture.
"""

SourceInfo: TypeAlias = Union[SourceInfoV1, Any]
"""The representation of SRCINFO data.

Tracks all available versions of the file format.

This union includes Any to allow for future extensions without breaking changes.
"""

VcsInfo: TypeAlias = Union[
    BzrInfo,
    FossilInfo,
    GitInfo,
    HgInfo,
    SvnInfo,
]
"""Information on Version Control Systems (VCS) using a URL.

Several different VCS systems can be used in the context of a SourceUrl.
Each system supports addressing different types of objects and may optionally require 
signature verification for those objects.
"""

__all__ = [
    "Checksum",
    "SkippableChecksum",
    "OpenPGPIdentifier",
    "MakepkgOption",
    "Soname",
    "VersionOrSoname",
    "RelationOrSoname",
    "SystemArchitecture",
    "SourceInfo",
    "VcsInfo",
]
