"""Tests for source_info_from_file and source_info_from_str functions."""

import tempfile
from pathlib import Path

import pytest
from alpm.alpm_srcinfo import (
    SourceInfoError,
    SourceInfoSchema,
    SourceInfoV1,
    source_info_from_file,
    source_info_from_str,
)
from alpm.alpm_types import Epoch, FullVersion, PackageRelease, PackageVersion


@pytest.fixture
def full_version() -> FullVersion:
    return FullVersion(PackageVersion("0.1.0"), PackageRelease(1), Epoch(1))


def test_from_str_valid(full_version: FullVersion, valid_srcinfo_content: str) -> None:
    """Test parsing valid SRCINFO from string."""
    result = source_info_from_str(valid_srcinfo_content)
    assert type(result) is SourceInfoV1
    assert result.base.version == full_version


def test_from_str_with_schema(
    full_version: FullVersion, valid_srcinfo_content: str
) -> None:
    """Test parsing SRCINFO from string with explicit schema."""
    schema = SourceInfoSchema("1")
    result = source_info_from_str(valid_srcinfo_content, schema)
    assert type(result) is SourceInfoV1
    assert result.base.version == full_version


def test_from_str_with_none_schema(
    full_version: FullVersion, valid_srcinfo_content: str
) -> None:
    """Test parsing SRCINFO from string with None schema (auto-detection)."""
    result = source_info_from_str(valid_srcinfo_content, None)
    assert type(result) is SourceInfoV1
    assert result.base.version == full_version


@pytest.mark.parametrize(
    "content",
    [
        "not valid",
        "",
        """
pkgbase = example
pkgdesc = Test package
    """,
    ],
)
def test_from_str_invalid(content: str) -> None:
    """Test parsing invalid SRCINFO from string raises error."""
    with pytest.raises(SourceInfoError):
        source_info_from_str(content)


@pytest.mark.parametrize("schema", [None, SourceInfoSchema("1")])
def test_from_file_valid(
    full_version: FullVersion,
    valid_srcinfo_content: str,
    schema: SourceInfoSchema | None,
) -> None:
    """Test parsing SRCINFO from file with explicit schema."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write(valid_srcinfo_content)
        tmp.flush()

        result = source_info_from_file(tmp.name, schema)
        assert type(result) is SourceInfoV1
        assert result.base.version == full_version


def test_from_file_path_object(
    full_version: FullVersion, valid_srcinfo_content: str
) -> None:
    """Test parsing SRCINFO from file using Path object."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write(valid_srcinfo_content)
        tmp.flush()

        result = source_info_from_file(Path(tmp.name))
        assert type(result) is SourceInfoV1
        assert result.base.version == full_version


@pytest.mark.parametrize(
    "content",
    [
        "not valid",
        "",
        """
pkgbase = example
pkgdesc = Test package
    """,
    ],
)
def test_from_file_invalid(content: str) -> None:
    """Test parsing invalid SRCINFO from file raises error."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".SRCINFO", delete=True) as tmp:
        tmp.write(content)
        tmp.flush()

        with pytest.raises(SourceInfoError):
            source_info_from_file(tmp.name)


def test_from_file_nonexistent() -> None:
    """Test parsing from nonexistent file raises error."""
    with pytest.raises(SourceInfoError):
        source_info_from_file("/nonexistent/file.SRCINFO")
