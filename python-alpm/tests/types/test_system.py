"""Tests for system-related types: ElfArchitectureFormat."""

import pytest
from alpm import alpm_types


@pytest.mark.parametrize(
    "format_str",
    [
        "32",
        "64",
    ],
)
def test_elf_architecture_format_from_str_valid(format_str):
    """Test creating ElfArchitectureFormat from valid string."""
    arch_format = alpm_types.ElfArchitectureFormat.from_str(format_str)
    assert arch_format is not None


@pytest.mark.parametrize(
    "invalid_format",
    [
        "",
        " ",
        "16",
        "invalid",
    ],
)
def test_elf_architecture_format_from_str_invalid(invalid_format):
    """Test creating ElfArchitectureFormat from invalid string raises error."""
    with pytest.raises(ValueError):
        alpm_types.ElfArchitectureFormat.from_str(invalid_format)


def test_elf_architecture_format_equality():
    """Test ElfArchitectureFormat equality."""
    arch1 = alpm_types.ElfArchitectureFormat.BIT_64
    arch2 = alpm_types.ElfArchitectureFormat.BIT_64
    assert arch1 == arch2


def test_elf_architecture_format_inequality():
    """Test ElfArchitectureFormat inequality."""
    arch1 = alpm_types.ElfArchitectureFormat.BIT_32
    arch2 = alpm_types.ElfArchitectureFormat.BIT_64
    assert arch1 != arch2


def test_elf_architecture_format_enum_values():
    """Test ElfArchitectureFormat enum values."""
    assert alpm_types.ElfArchitectureFormat.BIT_32 is not None
    assert alpm_types.ElfArchitectureFormat.BIT_64 is not None
