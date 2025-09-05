"""Type aliases for various submodules of the alpm package."""

from typing import TypeAlias, Union

from alpm.alpm_types import (
    Blake2b512Checksum,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
)

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

__all__ = [
    "Checksum",
]
