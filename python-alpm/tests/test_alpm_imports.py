def test_imports():
    """Test that all items can be imported without errors."""
    import alpm  # noqa: F401
    from alpm import alpm_types, alpm_srcinfo, ALPMError  # noqa: F401
