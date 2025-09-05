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
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
    BuildEnvironmentOption,
    PackageOption,
    PackageVersion,
    SonameV1,
    PackageRelation,
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
]
"""A checksum using a supported algorithm"""

OpenPGPIdentifier: TypeAlias = Union[
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
]
"""An OpenPGP key identifier.

The OpenPGPIdentifier type represents a valid OpenPGP identifier, which can be either an OpenPGP Key ID or an OpenPGP v4
fingerprint.
"""

MakepkgOption: TypeAlias = Union[
    BuildEnvironmentOption,
    PackageOption,
]
"""Either PackageOption or BuildEnvironmentOption.

This is necessary for metadata files such as SRCINFO or PKGBUILD package scripts that don't differentiate between
the different types and scopes of options.
"""

VersionOrSoname: TypeAlias = Union[
    PackageVersion,
    str,
]
"""Either a PackageVersion or a string representing a shared object name."""


RelationOrSoname: TypeAlias = Union[
    SonameV1,
    PackageRelation,
]
"""Either a SonameV1 or a PackageRelation."""

SourceInfo: TypeAlias = Union[SourceInfoV1, Any]
"""The representation of SRCINFO data.

Tracks all available versions of the file format.

This union includes Any to allow for future extensions without breaking changes.
"""

__all__ = [
    "Checksum",
    "OpenPGPIdentifier",
    "MakepkgOption",
    "VersionOrSoname",
    "RelationOrSoname",
    "SourceInfo",
]
