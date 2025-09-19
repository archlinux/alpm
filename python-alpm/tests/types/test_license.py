"""Tests for license alpm_types."""

import pytest
from alpm import ALPMError, alpm_types


@pytest.mark.parametrize("license_str", ["MIT", "Apache-2.0", "Apache-2.0 OR MIT"])
def test_license_valid(license_str: str) -> None:
    """Test creating a valid license."""
    license_obj = alpm_types.License(license_str)
    assert str(license_obj) == license_str
    assert license_obj.is_spdx


@pytest.mark.parametrize("license_str", ["MIT", "Apache-2.0", "Apache-2.0 OR MIT"])
def test_spdx_license_valid(license_str: str) -> None:
    """Test creating a valid spdx license."""
    license_obj = alpm_types.License.from_valid_spdx(license_str)
    assert str(license_obj) == license_str
    assert license_obj.is_spdx


def test_spdx_license_invalid() -> None:
    """Test creating an invalid SPDX license raises error."""
    with pytest.raises(ALPMError):
        alpm_types.License.from_valid_spdx("Invalid-License")


def test_license_custom() -> None:
    """Test creating custom license."""
    license_obj = alpm_types.License("custom")
    assert str(license_obj) == "custom"
    assert not license_obj.is_spdx


def test_license_equality() -> None:
    """Test license equality."""
    license1 = alpm_types.License("MIT")
    license2 = alpm_types.License("MIT")
    assert license1 == license2


def test_license_inequality() -> None:
    """Test license inequality."""
    license1 = alpm_types.License("MIT")
    license2 = alpm_types.License("GPL-3.0")
    assert license1 != license2
