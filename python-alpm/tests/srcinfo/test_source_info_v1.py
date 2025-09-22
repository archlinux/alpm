"""Tests for SourceInfoV1."""

import tempfile

import pytest
from alpm import alpm_srcinfo


def test_source_info_v1_from_string_valid(valid_srcinfo_content: str) -> None:
    """Test creating SourceInfoV1 from valid string content."""
    srcinfo = alpm_srcinfo.SourceInfoV1(valid_srcinfo_content)
    assert srcinfo is not None


def test_source_info_v1_from_string_invalid() -> None:
    """Test creating SourceInfoV1 from invalid string content raises error."""
    with pytest.raises(alpm_srcinfo.SourceInfoError):
        alpm_srcinfo.SourceInfoV1("some invalid content")


def test_source_info_v1_from_file_valid(valid_srcinfo_content: str) -> None:
    """Test creating SourceInfoV1 from valid file content."""
    with tempfile.NamedTemporaryFile("w+", delete=True) as tmp:
        tmp.write(valid_srcinfo_content)
        tmp.flush()
        srcinfo = alpm_srcinfo.SourceInfoV1.from_file(tmp.name)
        assert srcinfo is not None


def test_source_info_v1_from_file_invalid() -> None:
    """Test creating SourceInfoV1 from invalid file content raises error."""
    with tempfile.NamedTemporaryFile("w+", delete=True) as tmp:
        tmp.write("some invalid content")
        tmp.flush()
        with pytest.raises(alpm_srcinfo.SourceInfoError):
            alpm_srcinfo.SourceInfoV1.from_file(tmp.name)


def test_source_info_v1_from_pkgbuild_file_valid(valid_pkgbuild_content: str) -> None:
    """Test creating SourceInfoV1 from valid PKGBUILD file content."""
    with tempfile.NamedTemporaryFile("w+", delete=True) as tmp:
        tmp.write(valid_pkgbuild_content)
        tmp.flush()
        srcinfo = alpm_srcinfo.SourceInfoV1.from_pkgbuild(tmp.name)
        assert srcinfo is not None


def test_source_info_v1_from_pkgbuild_file_invalid() -> None:
    """Test creating SourceInfoV1 from invalid PKGBUILD file content raises error."""
    with tempfile.NamedTemporaryFile("w+", delete=True) as tmp:
        tmp.write("some invalid content")
        tmp.flush()
        with pytest.raises(alpm_srcinfo.SourceInfoError):
            alpm_srcinfo.SourceInfoV1.from_pkgbuild(tmp.name)
