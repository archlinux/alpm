"""Tests for system-related types."""

import pytest
from alpm import ALPMError, alpm_types


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
    assert arch.is_any is True
    assert arch.system_arch is None


def test_architecture_from_known_architecture() -> None:
    """Test creating Architecture from KnownArchitecture enum."""
    arch = alpm_types.Architecture(alpm_types.KnownArchitecture.X86_64)
    assert arch is not None
    assert str(arch) == "x86_64"
    assert arch.is_any is False


@pytest.mark.parametrize(
    ("arch_str", "known_arch"),
    [
        ("x86_64", alpm_types.KnownArchitecture.X86_64),
        ("aarch64", alpm_types.KnownArchitecture.AARCH64),
        ("arm", alpm_types.KnownArchitecture.ARM),
        ("armv6h", alpm_types.KnownArchitecture.ARMV6H),
        ("armv7h", alpm_types.KnownArchitecture.ARMV7H),
        ("i386", alpm_types.KnownArchitecture.I386),
        ("i486", alpm_types.KnownArchitecture.I486),
        ("i686", alpm_types.KnownArchitecture.I686),
        ("pentium4", alpm_types.KnownArchitecture.PENTIUM4),
        ("riscv32", alpm_types.KnownArchitecture.RISCV32),
        ("riscv64", alpm_types.KnownArchitecture.RISCV64),
        ("x86_64_v2", alpm_types.KnownArchitecture.X86_64_V2),
        ("x86_64_v3", alpm_types.KnownArchitecture.X86_64_V3),
        ("x86_64_v4", alpm_types.KnownArchitecture.X86_64_V4),
    ],
)
def test_architecture_string_equals_enum(
    arch_str: str, known_arch: alpm_types.KnownArchitecture
) -> None:
    """Test that string representation of KnownArchitecture equals enum version."""
    arch_from_str = alpm_types.Architecture(arch_str)
    arch_from_enum = alpm_types.Architecture(known_arch)
    assert arch_from_str == arch_from_enum
    assert str(arch_from_str) == str(arch_from_enum)
    assert arch_from_str.system_arch == arch_from_enum.system_arch
    assert arch_from_str.is_any == arch_from_enum.is_any


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
    with pytest.raises(ALPMError):
        alpm_types.Architecture(invalid_arch)


def test_architecture_is_any_true() -> None:
    """Test Architecture.is_any returns True for 'any' architecture."""
    arch = alpm_types.Architecture("any")
    assert arch.is_any is True
    assert arch.system_arch is None


def test_architecture_is_any_false_known() -> None:
    """Test Architecture.is_any returns False for known architecture."""
    arch = alpm_types.Architecture(alpm_types.KnownArchitecture.AARCH64)
    assert arch.is_any is False
    assert arch.system_arch is not None
    assert isinstance(arch.system_arch, alpm_types.KnownArchitecture)
    assert arch.system_arch == alpm_types.KnownArchitecture.AARCH64


def test_architecture_is_any_false_unknown() -> None:
    """Test Architecture.is_any returns False for unknown architecture."""
    arch = alpm_types.Architecture("custom_arch")
    assert arch.is_any is False
    assert arch.system_arch is not None


def test_architecture_system_arch_known() -> None:
    """Test Architecture.system_arch returns KnownArchitecture for known arch."""
    arch = alpm_types.Architecture(alpm_types.KnownArchitecture.X86_64)
    assert arch.system_arch is not None
    assert isinstance(arch.system_arch, alpm_types.KnownArchitecture)
    assert arch.system_arch == alpm_types.KnownArchitecture.X86_64


def test_architecture_system_arch_unknown() -> None:
    """Test Architecture.system_arch returns UnknownArchitecture for unknown arch."""
    arch = alpm_types.Architecture("custom_arch")
    assert arch.system_arch is not None
    assert isinstance(arch.system_arch, alpm_types.UnknownArchitecture)
    assert arch.system_arch.value == "custom_arch"
    assert str(arch.system_arch) == "custom_arch"


def test_unknown_architecture_value() -> None:
    """Test UnknownArchitecture.value property."""
    arch = alpm_types.Architecture("my_custom_arch")
    unknown = arch.system_arch
    assert isinstance(unknown, alpm_types.UnknownArchitecture)
    assert unknown.value == "my_custom_arch"


def test_unknown_architecture_str() -> None:
    """Test UnknownArchitecture string representation."""
    arch = alpm_types.Architecture("another_arch")
    unknown = arch.system_arch
    assert isinstance(unknown, alpm_types.UnknownArchitecture)
    assert str(unknown) == "another_arch"


def test_unknown_architecture_repr() -> None:
    """Test UnknownArchitecture repr."""
    arch = alpm_types.Architecture("test_arch")
    unknown = arch.system_arch
    assert isinstance(unknown, alpm_types.UnknownArchitecture)
    assert repr(unknown) is not None


def test_unknown_architecture_eq_ord_hash() -> None:
    """Test presence of eq, ord and hash methods on UnknownArchitecture."""
    arch = alpm_types.Architecture("foo_barch")
    unknown = arch.system_arch
    assert hasattr(unknown, "__eq__")
    assert hasattr(unknown, "__lt__")
    assert unknown.__hash__ is not None


def test_architecture_eq_ord_hash() -> None:
    """Test presence of eq, ord and hash methods on Architecture."""
    arch = alpm_types.Architecture("x86_64")
    assert hasattr(arch, "__eq__")
    assert hasattr(arch, "__lt__")
    assert arch.__hash__ is not None


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
    with pytest.raises(ALPMError):
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
    with pytest.raises(ALPMError):
        alpm_types.Architectures([invalid_arch])


def test_architectures_iteration() -> None:
    """Test iterating over Architectures."""
    archs = alpm_types.Architectures(["x86_64", "aarch64"])
    items = 0
    for arch in archs:
        items += 1
        assert isinstance(arch, alpm_types.Architecture)
    assert items == 2


def test_architectures_eq_ord_hash() -> None:
    """Test presence of eq, ord and hash methods on Architectures."""
    archs = alpm_types.Architectures(["x86_64", "aarch64"])
    assert hasattr(archs, "__eq__")
    assert hasattr(archs, "__lt__")
    assert archs.__hash__ is not None


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


def test_known_architecture_eq_ord_hash() -> None:
    """Test presence of eq, ord and hash methods on KnownArchitecture."""
    arch = alpm_types.KnownArchitecture.X86_64
    assert hasattr(arch, "__eq__")
    assert hasattr(arch, "__lt__")
    assert arch.__hash__ is not None
