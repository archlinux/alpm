"""Tests for path alpm_types: RelativePath."""

import pytest
from alpm import alpm_types, ALPMError


def test_relative_path_valid():
    """Test creating a valid relative path."""
    path = alpm_types.RelativePath("path/to/file")
    assert str(path) == "path/to/file"


def test_relative_path_single_file():
    """Test creating a relative path for a single file."""
    path = alpm_types.RelativePath("file.txt")
    assert str(path) == "file.txt"


def test_relative_path_with_extension():
    """Test creating a relative path with file extension."""
    path = alpm_types.RelativePath("docs/readme.md")
    assert str(path) == "docs/readme.md"


def test_relative_path_nested_directories():
    """Test creating a nested directory relative path."""
    path = alpm_types.RelativePath("usr/share/doc/package/changelog")
    assert str(path) == "usr/share/doc/package/changelog"


def test_relative_path_with_parent_reference():
    """Test that parent directory references are allowed."""
    path = alpm_types.RelativePath("../parent")
    assert str(path) == "../parent"


def test_relative_path_with_multiple_parent_references():
    """Test multiple parent directory references."""
    path = alpm_types.RelativePath("../../grandparent/file")
    assert str(path) == "../../grandparent/file"


def test_relative_path_invalid_absolute():
    """Test that absolute paths are invalid."""
    with pytest.raises(ALPMError):
        alpm_types.RelativePath("/absolute/path")


def test_relative_path_with_home_reference():
    """Test that home directory references are allowed as relative paths."""
    path = alpm_types.RelativePath("~/home/path")
    assert str(path) == "~/home/path"


def test_relative_path_equality():
    """Test relative path equality."""
    path1 = alpm_types.RelativePath("path/to/file")
    path2 = alpm_types.RelativePath("path/to/file")
    assert path1 == path2


def test_relative_path_inequality():
    """Test relative path inequality."""
    path1 = alpm_types.RelativePath("path/to/file1")
    path2 = alpm_types.RelativePath("path/to/file2")
    assert path1 != path2


def test_absolute_path_raises_error():
    """Test RelativePath error handling for invalid path."""
    with pytest.raises(ALPMError):
        alpm_types.RelativePath("/invalid")
