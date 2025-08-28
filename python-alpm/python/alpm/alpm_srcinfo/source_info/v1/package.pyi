"""Handling of metadata found in a `pkgname` section of SRCINFO data."""

class Package:
    """Package metadata based on a pkgname section in SRCINFO data.

    This class only contains package specific overrides.
    Only in combination with PackageBase data a full view on a package's metadata is possible.
    """

__all__ = [
    "Package",
]
