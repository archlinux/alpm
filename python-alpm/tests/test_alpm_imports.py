def test_imports():
    """Test that all items can be imported without errors."""
    import alpm  # noqa: F401
    from alpm import alpm_types, alpm_srcinfo, type_aliases, ALPMError  # noqa: F401


def test_type_aliases_imports():
    """Test that all type aliases can be imported without errors."""
    from alpm.type_aliases import (  # noqa: F401
        Checksum,
        SkippableChecksum,
        OpenPGPIdentifier,
        MakepkgOption,
        VersionOrSoname,
        RelationOrSoname,
        SourceInfo,
        VcsInfo,
    )
