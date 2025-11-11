"""Tests for path alpm_types."""

import pytest
from alpm import ALPMError, alpm_types


def test_relative_path_valid() -> None:
    """Test creating a valid relative path."""
    path = alpm_types.RelativeFilePath("path/to/file")
    assert str(path) == "path/to/file"


def test_relative_path_single_file() -> None:
    """Test creating a relative path for a single file."""
    path = alpm_types.RelativeFilePath("file.txt")
    assert str(path) == "file.txt"


def test_relative_path_with_extension() -> None:
    """Test creating a relative path with file extension."""
    path = alpm_types.RelativeFilePath("docs/readme.md")
    assert str(path) == "docs/readme.md"


def test_relative_path_nested_directories() -> None:
    """Test creating a nested directory relative path."""
    path = alpm_types.RelativeFilePath("usr/share/doc/package/changelog")
    assert str(path) == "usr/share/doc/package/changelog"


def test_relative_path_with_parent_reference() -> None:
    """Test that parent directory references are allowed."""
    path = alpm_types.RelativeFilePath("../parent")
    assert str(path) == "../parent"


def test_relative_path_with_multiple_parent_references() -> None:
    """Test multiple parent directory references."""
    path = alpm_types.RelativeFilePath("../../grandparent/file")
    assert str(path) == "../../grandparent/file"


def test_relative_path_invalid_absolute() -> None:
    """Test that absolute paths are invalid."""
    with pytest.raises(ALPMError):
        alpm_types.RelativeFilePath("/absolute/path")


def test_relative_path_with_home_reference() -> None:
    """Test that home directory references are allowed as relative paths."""
    path = alpm_types.RelativeFilePath("~/home/path")
    assert str(path) == "~/home/path"


def test_relative_path_equality() -> None:
    """Test relative path equality."""
    path1 = alpm_types.RelativeFilePath("path/to/file")
    path2 = alpm_types.RelativeFilePath("path/to/file")
    assert path1 == path2


def test_relative_path_inequality() -> None:
    """Test relative path inequality."""
    path1 = alpm_types.RelativeFilePath("path/to/file1")
    path2 = alpm_types.RelativeFilePath("path/to/file2")
    assert path1 != path2


def test_absolute_path_raises_error() -> None:
    """Test RelativeFilePath error handling for invalid path."""
    with pytest.raises(ALPMError):
        alpm_types.RelativeFilePath("/invalid")
