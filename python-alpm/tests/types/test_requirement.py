"""Tests for version requirement functionality."""

import pytest
from alpm import ALPMError, alpm_types


def test_version_requirement_init() -> None:
    """Test creating VersionRequirement with constructor."""
    pkgver = alpm_types.PackageVersion("1.2.3")
    version = alpm_types.Version(pkgver)
    comparison = alpm_types.VersionComparison.GREATER_OR_EQUAL
    req = alpm_types.VersionRequirement(comparison, version)
    assert req is not None


@pytest.mark.parametrize(
    "req_str",
    [
        ">=1.2.3",
        "<=2.0.0",
        "=1.0",
        ">0.5",
        "<3.0",
    ],
)
def test_version_requirement_from_str_valid(req_str: str) -> None:
    """Test creating VersionRequirement from valid string."""
    req = alpm_types.VersionRequirement.from_str(req_str)
    assert req is not None


@pytest.mark.parametrize(
    "invalid_req",
    [
        "invalid",
        "1.2.3",  # no comparison
        ">>1.2.3",  # invalid comparison
        ">=",  # no version
        "",
        "~=1.2.3",  # unsupported comparison
    ],
)
def test_version_requirement_from_str_invalid(invalid_req: str) -> None:
    """Test creating VersionRequirement from invalid string raises error."""
    with pytest.raises(ALPMError):
        alpm_types.VersionRequirement.from_str(invalid_req)


def test_version_requirement_with_complex_version() -> None:
    """Test creating VersionRequirement with complex version strings."""
    req = alpm_types.VersionRequirement.from_str(">=1.2.3+git20240101")
    assert req is not None


@pytest.mark.parametrize(
    "comparison_type",
    [
        alpm_types.VersionComparison.LESS,
        alpm_types.VersionComparison.LESS_OR_EQUAL,
        alpm_types.VersionComparison.EQUAL,
        alpm_types.VersionComparison.GREATER_OR_EQUAL,
        alpm_types.VersionComparison.GREATER,
    ],
)
def test_version_requirement_all_comparisons(
    comparison_type: alpm_types.VersionComparison,
) -> None:
    """Test creating VersionRequirement with all comparison types."""
    pkgver = alpm_types.PackageVersion("1.0.0")
    version = alpm_types.Version(pkgver)
    req = alpm_types.VersionRequirement(comparison_type, version)
    assert req is not None


def test_version_requirement_with_epoch() -> None:
    """Test creating VersionRequirement with epoch in version."""
    req = alpm_types.VersionRequirement.from_str(">=2:1.0.0")
    assert req is not None


def test_version_requirement_with_pkgrel() -> None:
    """Test creating VersionRequirement with package release in version."""
    req = alpm_types.VersionRequirement.from_str(">=1.0.0-2")
    assert req is not None


def test_version_requirement_edge_cases() -> None:
    """Test VersionRequirement with edge case version strings."""
    # Test with pre-release versions
    req1 = alpm_types.VersionRequirement.from_str(">=1.0.0rc1")
    assert req1 is not None

    # Test with build metadata
    req2 = alpm_types.VersionRequirement.from_str(">=1.0.0+build123")
    assert req2 is not None
