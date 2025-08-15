"""Tests for system alpm_types: Architecture."""

import pytest
from alpm import alpm_types

ARCHITECTURES = [
    alpm_types.Architecture.AARCH64,
    alpm_types.Architecture.ANY,
    alpm_types.Architecture.ARM,
    alpm_types.Architecture.ARMV6H,
    alpm_types.Architecture.ARMV7H,
    alpm_types.Architecture.I386,
    alpm_types.Architecture.I486,
    alpm_types.Architecture.I686,
    alpm_types.Architecture.PENTIUM4,
    alpm_types.Architecture.RISCV32,
    alpm_types.Architecture.RISCV64,
    alpm_types.Architecture.X86_64,
    alpm_types.Architecture.X86_64_V2,
    alpm_types.Architecture.X86_64_V3,
    alpm_types.Architecture.X86_64_V4,
]


@pytest.mark.parametrize("arch", ARCHITECTURES)
def test_architecture_variants(arch: alpm_types.Architecture):
    """Test that all architecture variants are available."""
    assert arch is not None


def test_architecture_equality_variants():
    """Test architecture variants equality/inequality."""
    for i, arch1 in enumerate(ARCHITECTURES):
        for j, arch2 in enumerate(ARCHITECTURES):
            if i != j:
                assert arch1 != arch2
            else:
                assert arch1 == arch2
