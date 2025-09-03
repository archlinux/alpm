def test_shortcut_imports():
    """Test that all items reexported from submodules can be imported from alpm.alpm_srcinfo."""
    from alpm.alpm_srcinfo import SourceInfoError, SourceInfoV1, MergedPackage  # noqa: F401


def test_imports():
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
    from alpm.alpm_srcinfo.source_info.v1.merged import MergedPackage, MergedSource  # noqa: F401
    from alpm.alpm_srcinfo.source_info.v1.package import Package  # noqa: F401
    from alpm.alpm_srcinfo.source_info.v1.package_base import PackageBase  # noqa: F401
