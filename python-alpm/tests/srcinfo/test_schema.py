"""Tests for SourceInfoSchema."""

import tempfile
from pathlib import Path

import pytest
from alpm.alpm_srcinfo import SourceInfoError, SourceInfoSchema
from alpm.alpm_types import SchemaVersion


def test_creation() -> None:
    """Test creating SourceInfoSchema with different version formats."""
    schema = SourceInfoSchema("1")
    assert str(schema.version) == "1.0.0"

    schema_full = SourceInfoSchema("1.0.0")
    assert str(schema_full.version) == "1.0.0"

    schema_from_version = SourceInfoSchema(SchemaVersion(1))
    assert str(schema_from_version.version) == "1.0.0"


def test_derive_from_str(valid_srcinfo_content: str) -> None:
    """Test deriving SourceInfoSchema from SRCINFO string content."""
    schema = SourceInfoSchema.derive_from_str(valid_srcinfo_content)
    assert str(schema.version) == "1.0.0"


def test_derive_from_str_invalid() -> None:
    """Test deriving SourceInfoSchema from invalid SRCINFO string raises error."""
    with pytest.raises(SourceInfoError):
        SourceInfoSchema.derive_from_str("not a valid srcinfo")


def test_derive_from_file(valid_srcinfo_content: str) -> None:
    """Test deriving SourceInfoSchema from SRCINFO file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write(valid_srcinfo_content)
        tmp.flush()

        schema = SourceInfoSchema.derive_from_file(Path(tmp.name))
        assert str(schema.version) == "1.0.0"


def test_derive_from_file_string_path(valid_srcinfo_content: str) -> None:
    """Test deriving SourceInfoSchema from SRCINFO file using string path."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write(valid_srcinfo_content)
        tmp.flush()

        schema = SourceInfoSchema.derive_from_file(tmp.name)
        assert str(schema.version) == "1.0.0"


def test_derive_from_file_invalid() -> None:
    """Test deriving SourceInfoSchema from invalid file raises error."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write("not a valid srcinfo")
        tmp.flush()

        with pytest.raises(SourceInfoError):
            SourceInfoSchema.derive_from_file(tmp.name)


def test_derive_from_nonexistent_file() -> None:
    """Test deriving SourceInfoSchema from nonexistent file raises error."""
    with pytest.raises(SourceInfoError):
        SourceInfoSchema.derive_from_file("/nonexistent/file.SRCINFO")


def test_invalid_version() -> None:
    """Test that invalid schema versions raise appropriate errors."""
    from alpm.alpm_types import ALPMError

    with pytest.raises(ALPMError):
        # not a valid SchemaVersion
        SourceInfoSchema("invalid")

    with pytest.raises(SourceInfoError):
        # valid SchemaVersion but unsupported
        SourceInfoSchema("999")


def test_str_repr() -> None:
    """Test string representation of SourceInfoSchema."""
    schema = SourceInfoSchema("1")

    assert "1" == str(schema)
    assert "SourceInfoSchema(1)" == repr(schema)


def test_version_consistency(valid_srcinfo_content: str) -> None:
    """Test that schema versions are consistent across different creation methods."""
    explicit_schema = SourceInfoSchema("1")

    derived_schema = SourceInfoSchema.derive_from_str(valid_srcinfo_content)

    assert explicit_schema == derived_schema


def test_equality() -> None:
    """Test that SourceInfoSchema instances with same version are equal."""
    schema1 = SourceInfoSchema("1")
    schema2 = SourceInfoSchema("1.0.0")

    assert schema1 == schema2


def test_derive_from_empty_file() -> None:
    """Test deriving SourceInfoSchema from empty file raises error."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write("")
        tmp.flush()

        with pytest.raises(SourceInfoError):
            SourceInfoSchema.derive_from_file(tmp.name)
