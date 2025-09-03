# TODO: <https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/204>
from typing import Union

from typing_extensions import TypeAlias

from .alpm import alpm_types as _mod

Checksum: TypeAlias = Union[
    _mod.Blake2b512Checksum,
    _mod.Md5Checksum,
    _mod.Sha1Checksum,
    _mod.Sha224Checksum,
    _mod.Sha256Checksum,
    _mod.Sha384Checksum,
    _mod.Sha512Checksum,
]

__all__ = [*_mod.__all__, "Checksum"]
for name in getattr(_mod, "__all__"):
    globals()[name] = getattr(_mod, name)
