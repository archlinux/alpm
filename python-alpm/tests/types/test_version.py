"""Tests for version-related alpm_types."""

import pytest
from alpm import ALPMError, alpm_types


# PackageVersion tests
def test_package_version_valid() -> None:
    """Test creating a valid package version."""
    version = alpm_types.PackageVersion("1.2.3")
    assert str(version) == "1.2.3"
    assert repr(version) == "PackageVersion('1.2.3')"


def test_package_version_with_alphanumeric() -> None:
    """Test creating a package version with alphanumeric characters."""
    version = alpm_types.PackageVersion("1.2.3a")
    assert str(version) == "1.2.3a"


def test_package_version_with_underscore() -> None:
    """Test creating a package version with underscores."""
    version = alpm_types.PackageVersion("1.2.3_beta")
    assert str(version) == "1.2.3_beta"


def test_package_version_with_plus() -> None:
    """Test creating a package version with plus signs."""
    version = alpm_types.PackageVersion("1.2.3+git")
    assert str(version) == "1.2.3+git"


def test_package_version_invalid_empty() -> None:
    """Test creating an empty package version raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageVersion("")


def test_package_version_equality() -> None:
    """Test package version equality."""
    version1 = alpm_types.PackageVersion("1.2.3")
    version2 = alpm_types.PackageVersion("1.2.3")
    assert version1 == version2


def test_package_version_ordering() -> None:
    """Test package version ordering."""
    version1 = alpm_types.PackageVersion("1.2.3")
    version2 = alpm_types.PackageVersion("1.2.4")
    assert version1 < version2


def test_package_version_frozen() -> None:
    """Test that PackageVersion is frozen (immutable)."""
    version = alpm_types.PackageVersion("1.2.3")
    with pytest.raises(AttributeError):
        version.new_attr = "test"  # type: ignore[attr-defined]


# SchemaVersion tests
def test_schema_version_valid() -> None:
    """Test creating a valid schema version."""
    version = alpm_types.SchemaVersion(1, 0)
    assert version is not None


def test_schema_version_with_patch() -> None:
    """Test creating a schema version with patch version."""
    version = alpm_types.SchemaVersion(1, 2, 3)
    assert version.major == 1
    assert version.minor == 2
    assert version.patch == 3


def test_schema_version_with_pre_release() -> None:
    """Test creating a schema version with pre-release."""
    version = alpm_types.SchemaVersion(1, pre="alpha.1")
    assert version.pre == "alpha.1"


def test_schema_version_with_build_metadata() -> None:
    """Test creating a schema version with build metadata."""
    version = alpm_types.SchemaVersion(1, build="build.123")
    assert version.build == "build.123"


def test_schema_version_from_str() -> None:
    """Test creating schema version from string."""
    version = alpm_types.SchemaVersion.from_str("1.2.3")
    assert version.major == 1
    assert version.minor == 2
    assert version.patch == 3


# Epoch tests
def test_epoch_valid() -> None:
    """Test creating a valid epoch."""
    epoch = alpm_types.Epoch(1)
    assert epoch.value == 1


def test_epoch_zero_invalid() -> None:
    """Test that epoch 0 is invalid."""
    with pytest.raises(ValueError):
        alpm_types.Epoch(0)


def test_epoch_from_str() -> None:
    """Test creating epoch from string."""
    epoch = alpm_types.Epoch.from_str("42")
    assert epoch.value == 42


def test_epoch_str_representation() -> None:
    """Test epoch string representation."""
    epoch = alpm_types.Epoch(5)
    assert str(epoch) == "5"
    assert repr(epoch) == "Epoch(5)"


# PackageRelease tests
def test_package_release_valid() -> None:
    """Test creating a valid package release."""
    release = alpm_types.PackageRelease(1)
    assert release.major == 1
    assert release.minor is None
    assert str(release) == "1"


def test_package_release_with_minor() -> None:
    """Test creating a package release with minor version."""
    release = alpm_types.PackageRelease(1, 1)
    assert release.major == 1
    assert release.minor == 1
    assert str(release) == "1.1"


def test_package_release_from_str() -> None:
    """Test creating package release from string."""
    release = alpm_types.PackageRelease.from_str("2.5")
    assert release.major == 2
    assert release.minor == 5


def test_package_release_repr() -> None:
    """Test package release representation."""
    release1 = alpm_types.PackageRelease(1)
    release2 = alpm_types.PackageRelease(1, 2)

    assert repr(release1) == "PackageRelease(major=1)"
    assert repr(release2) == "PackageRelease(major=1, minor=2)"


# VersionComparison tests
@pytest.mark.parametrize(
    "comparison_str, variant",
    [
        ("<", alpm_types.VersionComparison.LESS),
        ("<=", alpm_types.VersionComparison.LESS_OR_EQUAL),
        ("=", alpm_types.VersionComparison.EQUAL),
        (">=", alpm_types.VersionComparison.GREATER_OR_EQUAL),
        (">", alpm_types.VersionComparison.GREATER),
    ],
)
def test_version_comparison_from_str_valid(
    comparison_str: str, variant: alpm_types.VersionComparison
) -> None:
    """Test creating VersionComparison from valid string."""
    comparison = alpm_types.VersionComparison.from_str(comparison_str)
    assert comparison == variant


@pytest.mark.parametrize(
    "invalid_comparison",
    [
        "invalid",
        "<<",
        "==",
        ">>",
        "",
        "!=",
        "~",
    ],
)
def test_version_comparison_from_str_invalid(invalid_comparison: str) -> None:
    """Test creating VersionComparison from invalid string raises error."""
    with pytest.raises(ValueError):
        alpm_types.VersionComparison.from_str(invalid_comparison)


def test_version_comparison_equality() -> None:
    """Test VersionComparison equality."""
    comp1 = alpm_types.VersionComparison.EQUAL
    comp2 = alpm_types.VersionComparison.EQUAL
    assert comp1 == comp2


def test_version_comparison_inequality() -> None:
    """Test VersionComparison inequality."""
    comp1 = alpm_types.VersionComparison.LESS
    comp2 = alpm_types.VersionComparison.GREATER
    assert comp1 != comp2


# FullVersion tests
def test_full_version_from_str_valid() -> None:
    """Test creating FullVersion from valid string."""
    full_version = alpm_types.FullVersion.from_str("1.2.3-1")
    assert str(full_version.pkgver) == "1.2.3"
    assert str(full_version.pkgrel) == "1"
    assert full_version.epoch is None


def test_full_version_from_str_with_epoch() -> None:
    """Test creating FullVersion from string with epoch."""
    full_version = alpm_types.FullVersion.from_str("2:1.2.3-1")
    assert str(full_version.pkgver) == "1.2.3"
    assert str(full_version.pkgrel) == "1"
    assert str(full_version.epoch) == "2"


@pytest.mark.parametrize(
    "version_str, expected_pkgver, expected_pkgrel, expected_epoch",
    [
        ("1.2.3-1", "1.2.3", "1", None),
        ("2:1.2.3-1", "1.2.3", "1", "2"),
        ("0.5.0-2", "0.5.0", "2", None),
        ("1:2.0.0-3", "2.0.0", "3", "1"),
    ],
)
def test_full_version_from_str_parametrized(
    version_str: str,
    expected_pkgver: str,
    expected_pkgrel: str,
    expected_epoch: str | None,
) -> None:
    """Test creating FullVersion from various valid string formats."""
    full_version = alpm_types.FullVersion.from_str(version_str)
    assert str(full_version.pkgver) == expected_pkgver
    assert str(full_version.pkgrel) == expected_pkgrel
    if expected_epoch:
        assert str(full_version.epoch) == expected_epoch
    else:
        assert full_version.epoch is None


@pytest.mark.parametrize(
    "invalid_version",
    [
        "",
        "1.2.3",  # missing pkgrel
        "-1",  # missing pkgver
        "1.2.3-",  # empty pkgrel
        ":1.2.3-1",  # empty epoch
        "a:1.2.3-1",  # invalid epoch
        "1.2.3-a",  # invalid pkgrel
    ],
)
def test_full_version_from_str_invalid(invalid_version: str) -> None:
    """Test creating FullVersion from invalid string raises error."""
    with pytest.raises(ALPMError):
        alpm_types.FullVersion.from_str(invalid_version)


def test_full_version_vercmp_equal() -> None:
    """Test FullVersion vercmp with equal versions."""
    v1 = alpm_types.FullVersion.from_str("1.2.3-1")
    v2 = alpm_types.FullVersion.from_str("1.2.3-1")
    assert v1.vercmp(v2) == 0


def test_full_version_vercmp_newer() -> None:
    """Test FullVersion vercmp with newer version."""
    v1 = alpm_types.FullVersion.from_str("1.2.4-1")
    v2 = alpm_types.FullVersion.from_str("1.2.3-1")
    assert v1.vercmp(v2) == 1


def test_full_version_vercmp_older() -> None:
    """Test FullVersion vercmp with older version."""
    v1 = alpm_types.FullVersion.from_str("1.2.2-1")
    v2 = alpm_types.FullVersion.from_str("1.2.3-1")
    assert v1.vercmp(v2) == -1


def test_full_version_vercmp_with_epoch() -> None:
    """Test FullVersion vercmp with epoch."""
    v1 = alpm_types.FullVersion.from_str("2:1.0.0-1")
    v2 = alpm_types.FullVersion.from_str("1:2.0.0-1")
    assert v1.vercmp(v2) == 1  # higher epoch wins


def test_full_version_comparison_operators() -> None:
    """Test FullVersion comparison operators."""
    v1 = alpm_types.FullVersion.from_str("1.2.3-1")
    v2 = alpm_types.FullVersion.from_str("1.2.4-1")
    v3 = alpm_types.FullVersion.from_str("1.2.3-1")

    assert v1 < v2
    assert v1 <= v2
    assert v2 > v1
    assert v2 >= v1
    assert v1 == v3
    assert v1 <= v3
    assert v1 >= v3


def test_full_version_string_representation() -> None:
    """Test FullVersion string representation."""
    full_version = alpm_types.FullVersion.from_str("1.2.3-1")
    str_repr = str(full_version)
    assert "1.2.3" in str_repr
    assert "1" in str_repr


def test_full_version_repr() -> None:
    """Test FullVersion repr."""
    full_version = alpm_types.FullVersion.from_str("1.2.3-1")
    repr_str = repr(full_version)
    assert "FullVersion" in repr_str


# Version tests
def test_version_init_basic() -> None:
    """Test creating Version with basic components."""
    pkgver = alpm_types.PackageVersion("1.2.3")
    version = alpm_types.Version(pkgver)
    assert version.pkgver == pkgver
    assert version.pkgrel is None
    assert version.epoch is None


@pytest.mark.parametrize(
    "version_str, expected_has_pkgrel, expected_has_epoch",
    [
        ("1.2.3", False, False),
        ("1.2.3-1", True, False),
        ("2:1.2.3", False, True),
        ("2:1.2.3-1", True, True),
    ],
)
def test_version_from_str_valid(
    version_str: str, expected_has_pkgrel: bool, expected_has_epoch: bool
) -> None:
    """Test creating Version from valid string."""
    version = alpm_types.Version.from_str(version_str)
    assert version.pkgver is not None
    if expected_has_pkgrel:
        assert version.pkgrel is not None
    else:
        assert version.pkgrel is None
    if expected_has_epoch:
        assert version.epoch is not None
    else:
        assert version.epoch is None


@pytest.mark.parametrize(
    "invalid_version",
    [
        "",
        "-1",  # missing pkgver
        ":1.2.3",  # empty epoch
        "a:1.2.3",  # invalid epoch
        "1.2.3-a",  # invalid pkgrel
    ],
)
def test_version_from_str_invalid(invalid_version: str) -> None:
    """Test creating Version from invalid string raises error."""
    with pytest.raises(ALPMError):
        alpm_types.Version.from_str(invalid_version)


def test_version_vercmp_equal() -> None:
    """Test Version vercmp with equal versions."""
    v1 = alpm_types.Version.from_str("1.2.3")
    v2 = alpm_types.Version.from_str("1.2.3")
    assert v1.vercmp(v2) == 0


def test_version_vercmp_newer() -> None:
    """Test Version vercmp with newer version."""
    v1 = alpm_types.Version.from_str("1.2.4")
    v2 = alpm_types.Version.from_str("1.2.3")
    assert v1.vercmp(v2) == 1


def test_version_vercmp_older() -> None:
    """Test Version vercmp with older version."""
    v1 = alpm_types.Version.from_str("1.2.2")
    v2 = alpm_types.Version.from_str("1.2.3")
    assert v1.vercmp(v2) == -1


def test_version_vercmp_with_pkgrel() -> None:
    """Test Version vercmp with package release."""
    v1 = alpm_types.Version.from_str("1.2.3-2")
    v2 = alpm_types.Version.from_str("1.2.3-1")
    assert v1.vercmp(v2) == 1


def test_version_string_representation() -> None:
    """Test Version string representation."""
    version = alpm_types.Version.from_str("1.2.3")
    str_repr = str(version)
    assert "1.2.3" in str_repr


def test_version_repr() -> None:
    """Test Version repr."""
    version = alpm_types.Version.from_str("1.2.3")
    repr_str = repr(version)
    assert "Version" in repr_str


def test_epoch_error_handling() -> None:
    """Test that Epoch raises ValueError for zero and meaningful error messages."""
    with pytest.raises(ValueError) as exc_info:
        alpm_types.Epoch(0)
    assert "positive integer" in str(exc_info.value)


def test_schema_version_error_handling() -> None:
    """Test SchemaVersion error handling for invalid pre-release and build metadata."""
    # Test invalid pre-release identifier
    with pytest.raises(ALPMError):
        alpm_types.SchemaVersion(pre="invalid..pre")

    # Test invalid build metadata
    with pytest.raises(ALPMError):
        alpm_types.SchemaVersion(build="invalid..build")
