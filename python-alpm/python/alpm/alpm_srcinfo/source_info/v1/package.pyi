"""Handling of metadata found in a `pkgname` section of SRCINFO data."""

class Package:
    """Package metadata based on a pkgname section in SRCINFO data.

    This class only contains package specific overrides.
    Only in combination with PackageBase data a full view on a package's metadata is possible.
    """

class PackageArchitecture:
    """Architecture specific package properties for use in Package.

    For each Architecture defined in Package.architectures, a PackageArchitecture is present in
    Package.architecture_properties.
    """

__all__ = [
    "Package",
    "PackageArchitecture",
]
