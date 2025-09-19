"""Tests that all items in alpm_srcinfo module can be imported."""


def test_shortcut_imports() -> None:
    """Test that all items reexported from submodule can be imported from
    alpm.alpm_srcinfo.
    """
    from alpm.alpm_srcinfo import (  # noqa: F401
        MergedPackage,
        SourceInfoError,
        SourceInfoV1,
    )


def test_imports() -> None:
    """Test that all items can be imported without errors."""
    from alpm.alpm_srcinfo import error, source_info  # noqa: F401
    from alpm.alpm_srcinfo.error import SourceInfoError  # noqa: F401
    from alpm.alpm_srcinfo.source_info import v1  # noqa: F401
    from alpm.alpm_srcinfo.source_info.v1 import (  # noqa: F401
        SourceInfoV1,
        merged,
        package,
        package_base,
    )
    from alpm.alpm_srcinfo.source_info.v1.merged import (  # noqa: F401
        MergedPackage,
        MergedSource,
    )
    from alpm.alpm_srcinfo.source_info.v1.package import (  # noqa: F401
        Override,
        Package,
        PackageArchitecture,
    )
    from alpm.alpm_srcinfo.source_info.v1.package_base import (  # noqa: F401
        PackageBase,
        PackageBaseArchitecture,
    )
