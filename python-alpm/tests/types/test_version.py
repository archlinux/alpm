"""Tests for version-related alpm_types: PackageVersion, SchemaVersion, Epoch, PackageRelease."""

import pytest
from alpm import alpm_types, ALPMError


# PackageVersion tests
def test_package_version_valid():
    """Test creating a valid package version."""
    version = alpm_types.PackageVersion("1.2.3")
    assert str(version) == "1.2.3"
    assert repr(version) == "PackageVersion('1.2.3')"


def test_package_version_with_alphanumeric():
    """Test creating a package version with alphanumeric characters."""
    version = alpm_types.PackageVersion("1.2.3a")
    assert str(version) == "1.2.3a"


def test_package_version_with_underscore():
    """Test creating a package version with underscores."""
    version = alpm_types.PackageVersion("1.2.3_beta")
    assert str(version) == "1.2.3_beta"


def test_package_version_with_plus():
    """Test creating a package version with plus signs."""
    version = alpm_types.PackageVersion("1.2.3+git")
    assert str(version) == "1.2.3+git"


def test_package_version_invalid_empty():
    """Test creating an empty package version raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("")


def test_package_version_invalid_starts_with_dot():
    """Test creating a package version starting with dot raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion(".1.2.3")


def test_package_version_invalid_starts_with_underscore():
    """Test creating a package version starting with underscore raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("_1.2.3")


def test_package_version_invalid_starts_with_plus():
    """Test creating a package version starting with plus raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("+1.2.3")


def test_package_version_equality():
    """Test package version equality."""
    version1 = alpm_types.PackageVersion("1.2.3")
    version2 = alpm_types.PackageVersion("1.2.3")
    assert version1 == version2


def test_package_version_ordering():
    """Test package version ordering."""
    version1 = alpm_types.PackageVersion("1.2.3")
    version2 = alpm_types.PackageVersion("1.2.4")
    assert version1 < version2


def test_package_version_frozen():
    """Test that PackageVersion is frozen (immutable)."""
    version = alpm_types.PackageVersion("1.2.3")
    with pytest.raises(AttributeError):
        version.new_attr = "test"


# SchemaVersion tests
def test_schema_version_valid():
    """Test creating a valid schema version."""
    version = alpm_types.SchemaVersion(1, 0)
    assert version is not None


def test_schema_version_with_patch():
    """Test creating a schema version with patch version."""
    version = alpm_types.SchemaVersion(1, 2, 3)
    assert version.major == 1
    assert version.minor == 2
    assert version.patch == 3


def test_schema_version_with_pre_release():
    """Test creating a schema version with pre-release."""
    version = alpm_types.SchemaVersion(1, pre="alpha.1")
    assert version.pre == "alpha.1"


def test_schema_version_with_build_metadata():
    """Test creating a schema version with build metadata."""
    version = alpm_types.SchemaVersion(1, build="build.123")
    assert version.build == "build.123"


def test_schema_version_from_str():
    """Test creating schema version from string."""
    version = alpm_types.SchemaVersion.from_str("1.2.3")
    assert version.major == 1
    assert version.minor == 2
    assert version.patch == 3


# Epoch tests
def test_epoch_valid():
    """Test creating a valid epoch."""
    epoch = alpm_types.Epoch(1)
    assert epoch.value == 1


def test_epoch_zero_invalid():
    """Test that epoch 0 is invalid."""
    with pytest.raises(ValueError):
        alpm_types.Epoch(0)


def test_epoch_from_str():
    """Test creating epoch from string."""
    epoch = alpm_types.Epoch.from_str("42")
    assert epoch.value == 42


def test_epoch_str_representation():
    """Test epoch string representation."""
    epoch = alpm_types.Epoch(5)
    assert str(epoch) == "5"
    assert repr(epoch) == "Epoch(5)"


# PackageRelease tests
def test_package_release_valid():
    """Test creating a valid package release."""
    release = alpm_types.PackageRelease(1)
    assert release.major == 1
    assert release.minor is None
    assert str(release) == "1"


def test_package_release_with_minor():
    """Test creating a package release with minor version."""
    release = alpm_types.PackageRelease(1, 1)
    assert release.major == 1
    assert release.minor == 1
    assert str(release) == "1.1"


def test_package_release_from_str():
    """Test creating package release from string."""
    release = alpm_types.PackageRelease.from_str("2.5")
    assert release.major == 2
    assert release.minor == 5


def test_package_release_repr():
    """Test package release representation."""
    release1 = alpm_types.PackageRelease(1)
    release2 = alpm_types.PackageRelease(1, 2)

    assert repr(release1) == "PackageRelease(major=1)"
    assert repr(release2) == "PackageRelease(major=1, minor=2)"


def test_package_version_error_handling():
    """Test that PackageVersion raises ALPMError"""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("")

    with pytest.raises(ALPMError):
        alpm_types.PackageVersion(".invalid")

    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("_invalid")


def test_epoch_error_handling():
    """Test that Epoch raises ValueError for zero and meaningful error messages."""
    with pytest.raises(ValueError) as exc_info:
        alpm_types.Epoch(0)
    assert "positive integer" in str(exc_info.value)


def test_schema_version_error_handling():
    """Test SchemaVersion error handling for invalid pre-release and build metadata."""
    # Test invalid pre-release identifier
    with pytest.raises(ALPMError):
        alpm_types.SchemaVersion(pre="invalid..pre")

    # Test invalid build metadata
    with pytest.raises(ALPMError):
        alpm_types.SchemaVersion(build="invalid..build")
