"""Handling of metadata found in the pkgbase section of SRCINFO data."""

class PackageBase:
    """Package base metadata based on the pkgbase section in SRCINFO data.

    All values in this struct act as default values for all Packages in the scope of specific SRCINFO data.

    A MergedPackage (a full view on a package's metadata) can be created using SourceInfoV1.packages_for_architecture.
    """

__all__ = [
    "PackageBase",
]
