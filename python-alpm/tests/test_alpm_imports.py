"""Tests that all items in alpm module can be imported."""


def test_imports() -> None:
    """Test that all items can be imported without errors."""
    import alpm  # noqa: F401
    from alpm import ALPMError, alpm_srcinfo, alpm_types, type_aliases  # noqa: F401


def test_type_aliases_imports() -> None:
    """Test that all type aliases can be imported without errors."""
    from alpm.type_aliases import (  # noqa: F401
        Checksum,
        MakepkgOption,
        OpenPGPIdentifier,
        RelationOrSoname,
        SkippableChecksum,
        Soname,
        SourceInfo,
        SystemArchitecture,
        VcsInfo,
        VersionOrSoname,
    )
