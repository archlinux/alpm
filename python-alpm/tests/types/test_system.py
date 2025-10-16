"""Tests for system-related types."""

import pytest
from alpm import alpm_types


@pytest.mark.parametrize(
    "format_str",
    [
        "32",
        "64",
    ],
)
def test_elf_architecture_format_from_str_valid(format_str: str) -> None:
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
def test_elf_architecture_format_from_str_invalid(invalid_format: str) -> None:
    """Test creating ElfArchitectureFormat from invalid string raises error."""
    with pytest.raises(ValueError):
        alpm_types.ElfArchitectureFormat.from_str(invalid_format)


def test_elf_architecture_format_equality() -> None:
    """Test ElfArchitectureFormat equality."""
    arch1 = alpm_types.ElfArchitectureFormat.BIT_64
    arch2 = alpm_types.ElfArchitectureFormat.BIT_64
    assert arch1 == arch2


def test_elf_architecture_format_inequality() -> None:
    """Test ElfArchitectureFormat inequality."""
    arch1 = alpm_types.ElfArchitectureFormat.BIT_32
    arch2 = alpm_types.ElfArchitectureFormat.BIT_64
    assert arch1 != arch2


def test_elf_architecture_format_enum_values() -> None:
    """Test ElfArchitectureFormat enum values."""
    assert alpm_types.ElfArchitectureFormat.BIT_32 is not None
    assert alpm_types.ElfArchitectureFormat.BIT_64 is not None


@pytest.mark.parametrize(
    "arch_str",
    [
        "any",
        "x86_64",
        "aarch64",
        "custom_arch",
        "my_arch_123",
    ],
)
def test_architecture_from_str_valid(arch_str: str) -> None:
    """Test creating Architecture from valid string."""
    arch = alpm_types.Architecture(arch_str)
    assert arch is not None
    assert str(arch) == arch_str


def test_architecture_default() -> None:
    """Test creating Architecture with default value."""
    arch = alpm_types.Architecture()
    assert arch is not None
    assert str(arch) == "any"
    assert arch.arch_type == alpm_types.ArchitectureType.ANY


def test_architecture_from_known_architecture() -> None:
    """Test creating Architecture from KnownArchitecture enum."""
    arch = alpm_types.Architecture(alpm_types.KnownArchitecture.X86_64)
    assert arch is not None
    assert str(arch) == "x86_64"
    assert arch.arch_type == alpm_types.ArchitectureType.KNOWN


@pytest.mark.parametrize(
    "invalid_arch",
    [
        "",
        " ",
        "invalid arch",
        "arch-with-dash",
        "arch.with.dot",
    ],
)
def test_architecture_from_str_invalid(invalid_arch: str) -> None:
    """Test creating Architecture from invalid string raises error."""
    with pytest.raises(alpm_types.ALPMError):
        alpm_types.Architecture(invalid_arch)


def test_architecture_type_any() -> None:
    """Test Architecture with ANY type."""
    arch = alpm_types.Architecture("any")
    assert arch.arch_type == alpm_types.ArchitectureType.ANY
    assert arch.system_arch is None


def test_architecture_type_known() -> None:
    """Test Architecture with KNOWN type."""
    arch = alpm_types.Architecture(alpm_types.KnownArchitecture.AARCH64)
    assert arch.arch_type == alpm_types.ArchitectureType.KNOWN
    assert arch.system_arch == alpm_types.KnownArchitecture.AARCH64


def test_architecture_type_unknown() -> None:
    """Test Architecture with UNKNOWN type."""
    arch = alpm_types.Architecture("custom_arch")
    assert arch.arch_type == alpm_types.ArchitectureType.UNKNOWN
    assert arch.system_arch == "custom_arch"


def test_architecture_equality() -> None:
    """Test Architecture equality."""
    arch1 = alpm_types.Architecture("x86_64")
    arch2 = alpm_types.Architecture("x86_64")
    assert arch1 == arch2


def test_architecture_inequality() -> None:
    """Test Architecture inequality."""
    arch1 = alpm_types.Architecture("x86_64")
    arch2 = alpm_types.Architecture("aarch64")
    assert arch1 != arch2


def test_architecture_ordering() -> None:
    """Test Architecture ordering."""
    arch1 = alpm_types.Architecture("aarch64")
    arch2 = alpm_types.Architecture("x86_64")
    assert arch1 < arch2
    assert arch1 <= arch2
    assert arch2 > arch1
    assert arch2 >= arch1


def test_architecture_hash() -> None:
    """Test Architecture can be hashed."""
    arch = alpm_types.Architecture("x86_64")
    assert hash(arch) is not None
    # Test that equal architectures have same hash
    arch2 = alpm_types.Architecture("x86_64")
    assert hash(arch) == hash(arch2)


def test_architecture_repr() -> None:
    """Test Architecture repr."""
    arch = alpm_types.Architecture("x86_64")
    assert repr(arch) is not None
    assert "x86_64" in repr(arch)


def test_architectures_default() -> None:
    """Test creating Architectures with default value."""
    archs = alpm_types.Architectures()
    assert archs is not None
    assert archs.is_any is True
    assert len(archs) == 1


def test_architectures_from_any() -> None:
    """Test creating Architectures with 'any'."""
    archs = alpm_types.Architectures(["any"])
    assert archs.is_any is True
    assert len(archs) == 1


def test_architectures_from_str_list() -> None:
    """Test creating Architectures from string list."""
    archs = alpm_types.Architectures(["x86_64", "aarch64"])
    assert archs is not None
    assert archs.is_any is False
    assert len(archs) == 2


def test_architectures_from_known_list() -> None:
    """Test creating Architectures from KnownArchitecture list."""
    archs = alpm_types.Architectures(
        [alpm_types.KnownArchitecture.X86_64, alpm_types.KnownArchitecture.AARCH64]
    )
    assert archs is not None
    assert archs.is_any is False
    assert len(archs) == 2


def test_architectures_from_mixed_list() -> None:
    """Test creating Architectures from mixed list."""
    archs = alpm_types.Architectures(
        [alpm_types.KnownArchitecture.X86_64, "custom_arch"]
    )
    assert archs is not None
    assert archs.is_any is False
    assert len(archs) == 2


def test_architectures_any_with_others_raises() -> None:
    """Test creating Architectures with 'any' and other architectures raises error."""
    with pytest.raises(alpm_types.ALPMError):
        alpm_types.Architectures(["any", "x86_64"])


@pytest.mark.parametrize(
    "invalid_arch",
    [
        "",
        " ",
        "invalid arch",
        "arch-with-dash",
    ],
)
def test_architectures_invalid_arch_raises(invalid_arch: str) -> None:
    """Test creating Architectures with invalid architecture raises error."""
    with pytest.raises(alpm_types.ALPMError):
        alpm_types.Architectures([invalid_arch])


def test_architectures_iteration() -> None:
    """Test iterating over Architectures."""
    archs = alpm_types.Architectures(["x86_64", "aarch64"])
    arch_list = archs.__iter__()
    assert len(arch_list) == 2
    assert all(isinstance(arch, alpm_types.Architecture) for arch in arch_list)


def test_architectures_equality() -> None:
    """Test Architectures equality."""
    archs1 = alpm_types.Architectures(["x86_64", "aarch64"])
    archs2 = alpm_types.Architectures(["x86_64", "aarch64"])
    assert archs1 == archs2


def test_architectures_inequality() -> None:
    """Test Architectures inequality."""
    archs1 = alpm_types.Architectures(["x86_64"])
    archs2 = alpm_types.Architectures(["aarch64"])
    assert archs1 != archs2


def test_architectures_hash() -> None:
    """Test Architectures can be hashed."""
    archs = alpm_types.Architectures(["x86_64", "aarch64"])
    assert hash(archs) is not None


def test_architectures_repr() -> None:
    """Test Architectures repr."""
    archs = alpm_types.Architectures(["x86_64"])
    assert repr(archs) is not None


def test_architectures_str() -> None:
    """Test Architectures str."""
    archs = alpm_types.Architectures(["x86_64"])
    assert str(archs) is not None


@pytest.mark.parametrize(
    "known_arch",
    [
        alpm_types.KnownArchitecture.AARCH64,
        alpm_types.KnownArchitecture.ARM,
        alpm_types.KnownArchitecture.ARMV6H,
        alpm_types.KnownArchitecture.ARMV7H,
        alpm_types.KnownArchitecture.I386,
        alpm_types.KnownArchitecture.I486,
        alpm_types.KnownArchitecture.I686,
        alpm_types.KnownArchitecture.PENTIUM4,
        alpm_types.KnownArchitecture.RISCV32,
        alpm_types.KnownArchitecture.RISCV64,
        alpm_types.KnownArchitecture.X86_64,
        alpm_types.KnownArchitecture.X86_64_V2,
        alpm_types.KnownArchitecture.X86_64_V3,
        alpm_types.KnownArchitecture.X86_64_V4,
    ],
)
def test_known_architecture_values(known_arch: alpm_types.KnownArchitecture) -> None:
    """Test KnownArchitecture enum values are accessible."""
    assert known_arch is not None
    assert str(known_arch) is not None


def test_known_architecture_equality() -> None:
    """Test KnownArchitecture equality."""
    arch1 = alpm_types.KnownArchitecture.X86_64
    arch2 = alpm_types.KnownArchitecture.X86_64
    assert arch1 == arch2


def test_known_architecture_inequality() -> None:
    """Test KnownArchitecture inequality."""
    arch1 = alpm_types.KnownArchitecture.X86_64
    arch2 = alpm_types.KnownArchitecture.AARCH64
    assert arch1 != arch2


def test_known_architecture_hash() -> None:
    """Test KnownArchitecture can be hashed."""
    arch = alpm_types.KnownArchitecture.X86_64
    assert hash(arch) is not None


def test_architecture_type_values() -> None:
    """Test ArchitectureType enum values."""
    assert alpm_types.ArchitectureType.ANY is not None
    assert alpm_types.ArchitectureType.KNOWN is not None
    assert alpm_types.ArchitectureType.UNKNOWN is not None


def test_architecture_type_equality() -> None:
    """Test ArchitectureType equality."""
    type1 = alpm_types.ArchitectureType.ANY
    type2 = alpm_types.ArchitectureType.ANY
    assert type1 == type2


def test_architecture_type_inequality() -> None:
    """Test ArchitectureType inequality."""
    type1 = alpm_types.ArchitectureType.ANY
    type2 = alpm_types.ArchitectureType.KNOWN
    assert type1 != type2


def test_architecture_type_hash() -> None:
    """Test ArchitectureType can be hashed."""
    arch_type = alpm_types.ArchitectureType.ANY
    assert hash(arch_type) is not None
