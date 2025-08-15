"""Tests for environment types: BuildEnvironmentOption, PackageOption, parse_makepkg_option."""

import pytest
from alpm import alpm_types, ALPMError


@pytest.mark.parametrize(
    "option_str",
    [
        "ccache",
        "buildflags",
        "check",
        "color",
        "distcc",
        "makeflags",
        "sign",
    ],
)
@pytest.mark.parametrize("on", [True, False])
def test_build_environment_options(option_str: str, on: bool):
    """Test creating build environment option."""
    flag: str = f"{'!' if not on else ''}{option_str}"
    option = alpm_types.BuildEnvironmentOption(flag)
    assert str(option) == flag
    assert option.name == option_str
    assert option.on == on


def test_build_environment_option_invalid():
    """Test creating an invalid build environment option raises error."""
    with pytest.raises(ALPMError):
        alpm_types.BuildEnvironmentOption("invalid")


@pytest.mark.parametrize(
    "option_str",
    [
        "strip",
        "docs",
        "libtool",
        "staticlibs",
        "emptydirs",
        "zipman",
        "debug",
        "autodeps",
        "lto",
        "purge",
    ],
)
@pytest.mark.parametrize("on", [True, False])
def test_package_option(option_str: str, on: bool):
    """Test creating a package option."""
    flag: str = f"{'!' if not on else ''}{option_str}"
    option = alpm_types.PackageOption(flag)
    assert str(option) == flag
    assert option.name == option_str
    assert option.on == on


def test_parse_makepkg_option_invalid():
    """Test parsing an invalid makepkg option raises error."""
    with pytest.raises(ALPMError):
        alpm_types.parse_makepkg_option("invalid")


def test_build_environment_option_equality():
    """Test build environment option equality."""
    option1 = alpm_types.BuildEnvironmentOption("ccache")
    option2 = alpm_types.BuildEnvironmentOption("ccache")
    assert option1 == option2


def test_package_option_equality():
    """Test package option equality."""
    option1 = alpm_types.PackageOption("strip")
    option2 = alpm_types.PackageOption("strip")
    assert option1 == option2


@pytest.mark.parametrize(
    "option_str, option_type",
    [
        ("ccache", alpm_types.BuildEnvironmentOption),
        ("buildflags", alpm_types.BuildEnvironmentOption),
        ("check", alpm_types.BuildEnvironmentOption),
        ("color", alpm_types.BuildEnvironmentOption),
        ("distcc", alpm_types.BuildEnvironmentOption),
        ("makeflags", alpm_types.BuildEnvironmentOption),
        ("sign", alpm_types.BuildEnvironmentOption),
        ("strip", alpm_types.PackageOption),
        ("docs", alpm_types.PackageOption),
        ("libtool", alpm_types.PackageOption),
        ("staticlibs", alpm_types.PackageOption),
        ("emptydirs", alpm_types.PackageOption),
        ("zipman", alpm_types.PackageOption),
        ("debug", alpm_types.PackageOption),
        ("autodeps", alpm_types.PackageOption),
        ("lto", alpm_types.PackageOption),
        ("purge", alpm_types.PackageOption),
    ],
)
@pytest.mark.parametrize("on", [True, False])
def test_parse_makepkg_option(option_str: str, option_type, on: bool):
    """Test parsing a valid makepkg option."""
    flag: str = f"{'!' if not on else ''}{option_str}"
    option = alpm_types.parse_makepkg_option(flag)
    assert isinstance(option, option_type)
    assert str(option) == flag
    assert option.name == option_str
    assert option.on == on
