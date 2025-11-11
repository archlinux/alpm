"""Tests for Package and PackageArchitecture classes."""

import pytest
from alpm.alpm_srcinfo.source_info.v1.package import (
    Override,
    Package,
    PackageArchitecture,
)
from alpm.alpm_types import (
    ALPMError,
    Architectures,
    KnownArchitecture,
    License,
    OptionalDependency,
    PackageRelation,
    RelativeFilePath,
    SonameV1,
    Url,
    Version,
    VersionComparison,
    VersionRequirement,
    makepkg_option_from_str,
    relation_or_soname_from_str,
)
from alpm.type_aliases import RelationOrSoname, SystemArchitecture


def test_package_init_valid_name() -> None:
    """Test Package initialization with valid name."""
    package = Package("test-package")
    assert package.name == "test-package"


@pytest.mark.parametrize(
    "invalid_name",
    [
        "",
        "invalid name",
        "-invalid",
    ],
)
def test_package_init_invalid_name(invalid_name: str) -> None:
    """Test Package initialization with invalid names raises ALPMError."""
    with pytest.raises(ALPMError):
        Package(invalid_name)


def test_package_name_getter_setter() -> None:
    """Test Package name property getter and setter."""
    package = Package("original-name")
    assert package.name == "original-name"

    package.name = "new-name"
    assert package.name == "new-name"


def test_package_name_setter_invalid() -> None:
    """Test Package name setter with invalid name raises ALPMError."""
    package = Package("valid-name")
    with pytest.raises(ALPMError):
        package.name = "invalid name"


def test_package_description_getter_setter() -> None:
    """Test Package description property getter and setter."""
    package = Package("test-package")

    assert package.description is None

    # Override
    override_desc = Override("Test description")
    package.description = override_desc
    assert package.description.value == "Test description"

    # No override
    package.description = None
    assert package.description is None

    # Clear
    package.description = Override(None)
    assert package.description.value is None


def test_package_url_getter_setter() -> None:
    """Test Package url property getter and setter."""
    package = Package("test-package")

    assert package.url is None

    # Override
    url = Url("https://example.com")
    override_url = Override(url)
    package.url = override_url
    assert package.url.value == url

    # No override
    package.url = None
    assert package.url is None

    # Clear
    package.url = Override(None)
    assert package.url.value is None


def test_package_changelog_getter_setter() -> None:
    """Test Package changelog property getter and setter."""
    package = Package("test-package")

    assert package.changelog is None

    # Override
    changelog = RelativeFilePath("CHANGELOG.md")
    override_changelog = Override(changelog)
    package.changelog = override_changelog
    assert package.changelog.value == changelog

    # No override
    package.changelog = None
    assert package.changelog is None

    # Clear
    package.changelog = Override(None)
    assert package.changelog.value is None


def test_package_licenses_getter_setter() -> None:
    """Test Package licenses property getter and setter."""
    package = Package("test-package")

    assert package.licenses is None

    # Override
    licenses = [License("MIT"), License("Apache-2.0")]
    override_licenses = Override(licenses)
    package.licenses = override_licenses
    assert package.licenses.value == licenses

    # No override
    package.licenses = None
    assert package.licenses is None

    # Clear
    package.licenses = Override(None)
    assert package.licenses.value is None


def test_package_install_getter_setter() -> None:
    """Test Package install property getter and setter."""
    package = Package("test-package")

    assert package.install is None

    # Override
    install = RelativeFilePath("install.sh")
    override_install = Override(install)
    package.install = override_install
    assert package.install.value == install

    # No override
    package.install = None
    assert package.install is None

    # Clear
    package.install = Override(None)
    assert package.install.value is None


def test_package_groups_getter_setter() -> None:
    """Test Package groups property getter and setter."""
    package = Package("test-package")

    assert package.groups is None

    # Override
    groups = ["base", "devel"]
    override_groups = Override(groups)
    package.groups = override_groups
    assert package.groups.value == groups

    # No override
    package.groups = None
    assert package.groups is None

    # Clear
    package.groups = Override(None)
    assert package.groups.value is None


def test_package_options_getter_setter() -> None:
    """Test Package options property getter and setter."""
    package = Package("test-package")

    assert package.options is None

    # Override
    options = [makepkg_option_from_str("!strip"), makepkg_option_from_str("!docs")]
    override_options = Override(options)
    package.options = override_options
    assert package.options.value == options

    # No override
    package.options = None
    assert package.options is None

    # Clear
    package.options = Override(None)
    assert package.options.value is None


def test_package_backups_getter_setter() -> None:
    """Test Package backups property getter and setter."""
    package = Package("test-package")

    assert package.backups is None

    # Override
    backups = [RelativeFilePath("etc/config.conf"), RelativeFilePath("etc/other.conf")]
    override_backups = Override(backups)
    package.backups = override_backups
    assert package.backups.value == backups

    # No override
    package.backups = None
    assert package.backups is None

    # Clear
    package.backups = Override(None)
    assert package.backups.value is None


def test_package_architectures_getter_setter() -> None:
    """Test Package architectures property getter and setter."""
    package = Package("test-package")

    assert package.architectures is None

    # Override
    architectures = Architectures([KnownArchitecture.X86_64, KnownArchitecture.AARCH64])
    package.architectures = architectures
    assert package.architectures == architectures

    # No override
    package.architectures = None
    assert package.architectures is None


def test_package_architecture_properties_getter_setter() -> None:
    """Test Package architecture_properties property getter and setter."""
    package = Package("test-package")

    assert package.architecture_properties == {}

    arch_props: dict[SystemArchitecture, PackageArchitecture] = {
        KnownArchitecture.X86_64: PackageArchitecture(),
        KnownArchitecture.AARCH64: PackageArchitecture(),
    }
    package.architecture_properties = arch_props
    assert len(package.architecture_properties) == 2


def test_package_dependencies_getter_setter() -> None:
    """Test Package dependencies property getter and setter."""
    package = Package("test-package")

    assert package.dependencies is None

    # Override
    dependencies: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(
                VersionComparison.GREATER_OR_EQUAL, Version.from_str("1.0.0")
            ),
        ),
    ]
    package.dependencies = Override(dependencies)
    assert package.dependencies.value == dependencies

    # No override
    package.dependencies = None
    assert package.dependencies is None

    # Clear
    package.dependencies = Override(None)
    assert package.dependencies.value is None


def test_package_optional_dependencies_getter_setter() -> None:
    """Test Package optional_dependencies property getter and setter."""
    package = Package("test-package")

    assert package.optional_dependencies is None

    # Override
    opt_deps = [OptionalDependency(PackageRelation("foo"), "for extra foofyness")]
    override_opt_deps = Override(opt_deps)
    package.optional_dependencies = override_opt_deps
    assert package.optional_dependencies.value == opt_deps

    # No override
    package.optional_dependencies = None
    assert package.optional_dependencies is None

    # Clear
    package.optional_dependencies = Override(None)
    assert package.optional_dependencies.value is None


def test_package_provides_getter_setter() -> None:
    """Test Package provides property getter and setter."""
    package = Package("test-package")

    assert package.provides is None

    # Override
    provides: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(VersionComparison.EQUAL, Version.from_str("1.0.0")),
        ),
    ]
    package.provides = Override(provides)
    assert package.provides.value == provides

    # No override
    package.provides = None
    assert package.provides is None

    # Clear
    package.provides = Override(None)
    assert package.provides.value is None


def test_package_conflicts_getter_setter() -> None:
    """Test Package conflicts property getter and setter."""
    package = Package("test-package")

    assert package.conflicts is None

    # Override
    conflicts = [
        PackageRelation(
            "conflict",
            VersionRequirement(VersionComparison.LESS, Version.from_str("2.0.0")),
        )
    ]
    package.conflicts = Override(conflicts)
    assert package.conflicts.value == conflicts

    # No override
    package.conflicts = None
    assert package.conflicts is None

    # Clear
    package.conflicts = Override(None)
    assert package.conflicts.value is None


def test_package_replaces_getter_setter() -> None:
    """Test Package replaces property getter and setter."""
    package = Package("test-package")

    assert package.replaces is None

    # Override
    replaces = [
        PackageRelation(
            "replace",
            VersionRequirement(VersionComparison.LESS, Version.from_str("2.0.0")),
        )
    ]
    package.replaces = Override(replaces)
    assert package.replaces.value == replaces

    # No override
    package.replaces = None
    assert package.replaces is None

    # Clear
    package.replaces = Override(None)
    assert package.replaces.value is None


def test_package_architecture_dependencies_getter_setter() -> None:
    """Test PackageArchitecture dependencies property getter and setter."""
    arch_props = PackageArchitecture()

    assert arch_props.dependencies is None

    # Override
    dependencies: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(
                VersionComparison.GREATER_OR_EQUAL, Version.from_str("2.0.0")
            ),
        ),
    ]
    arch_props.dependencies = Override(dependencies)
    assert arch_props.dependencies.value == dependencies

    # No override
    arch_props.dependencies = None
    assert arch_props.dependencies is None

    # Clear
    arch_props.dependencies = Override(None)
    assert arch_props.dependencies.value is None


def test_package_architecture_optional_dependencies_getter_setter() -> None:
    """Test PackageArchitecture optional_dependencies property getter and setter."""
    arch_props = PackageArchitecture()

    assert arch_props.optional_dependencies is None

    # Override
    opt_deps = [OptionalDependency(PackageRelation("foo"), "for extra foofyness")]
    override_opt_deps = Override(opt_deps)
    arch_props.optional_dependencies = override_opt_deps
    assert arch_props.optional_dependencies.value == opt_deps

    # No override
    arch_props.optional_dependencies = None
    assert arch_props.optional_dependencies is None

    # Clear
    arch_props.optional_dependencies = Override(None)
    assert arch_props.optional_dependencies.value is None


def test_package_architecture_provides_getter_setter() -> None:
    """Test PackageArchitecture provides property getter and setter."""
    arch_props = PackageArchitecture()

    assert arch_props.provides is None

    # Override
    provides: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(VersionComparison.EQUAL, Version.from_str("1.0.0")),
        ),
    ]
    arch_props.provides = Override(provides)
    assert arch_props.provides.value == provides

    # No override
    arch_props.provides = None
    assert arch_props.provides is None

    # Clear
    arch_props.provides = Override(None)
    assert arch_props.provides.value is None


def test_package_architecture_conflicts_getter_setter() -> None:
    """Test PackageArchitecture conflicts property getter and setter."""
    arch_props = PackageArchitecture()

    assert arch_props.conflicts is None

    # Override
    conflicts = [
        PackageRelation(
            "conflict",
            VersionRequirement(VersionComparison.LESS, Version.from_str("2.0.0")),
        )
    ]
    arch_props.conflicts = Override(conflicts)
    assert arch_props.conflicts.value == conflicts

    # No override
    arch_props.conflicts = None
    assert arch_props.conflicts is None

    # Clear
    arch_props.conflicts = Override(None)
    assert arch_props.conflicts.value is None


def test_package_architecture_replaces_getter_setter() -> None:
    """Test PackageArchitecture replaces property getter and setter."""
    arch_props = PackageArchitecture()

    assert arch_props.replaces is None

    # Override
    replaces = [
        PackageRelation(
            "replace",
            VersionRequirement(VersionComparison.LESS, Version.from_str("2.0.0")),
        )
    ]
    arch_props.replaces = Override(replaces)
    assert arch_props.replaces.value == replaces

    # No override
    arch_props.replaces = None
    assert arch_props.replaces is None

    # Clear
    arch_props.replaces = Override(None)
    assert arch_props.replaces.value is None


def test_override_init_with_value() -> None:
    """Test Override initialization with a value."""
    override = Override("test value")
    assert override.value == "test value"


def test_override_init_with_none() -> None:
    """Test Override initialization with None (clear field)."""
    override: Override[str] = Override(None)
    assert override.value is None


def test_override_value_property() -> None:
    """Test Override value property."""
    override: Override[str] = Override("initial value")
    assert override.value == "initial value"


@pytest.mark.parametrize(
    "test_value",
    [
        "string value",
        ["list", "value"],
        None,
    ],
)
def test_override_with_valid_types(test_value: str | list[str] | None) -> None:
    """Test Override with valid value types."""
    override: Override[str | list[str]] = Override(test_value)
    assert override.value == test_value


def test_override_with_url() -> None:
    """Test Override with Url type."""
    url = Url("https://example.com")
    override: Override[Url] = Override(url)
    assert override.value == url


def test_override_with_relative_path() -> None:
    """Test Override with RelativeFilePath type."""
    path = RelativeFilePath("path/to/file")
    override: Override[RelativeFilePath] = Override(path)
    assert override.value == path


def test_override_clear_vs_none() -> None:
    """Test difference between Override(None) (clear) and None (no override)."""
    package = Package("test-package")

    package.description = None
    assert package.description is None

    package.description = Override(None)
    assert package.description is not None
    assert package.description.value is None


def test_package_equality() -> None:
    """Test Package equality comparison."""
    package1 = Package("test-package")
    package2 = Package("test-package")
    package3 = Package("different-package")

    assert package1 == package2
    assert package1 != package3
    assert package1 != "test-package"  # type: ignore[comparison-overlap]


@pytest.mark.parametrize(
    "valid_name",
    [
        "simple-name",
        "package-with-numbers123",
        "a",
        "very-long-package-name-with-many-hyphens",
        "name123",
        "123name",
    ],
)
def test_package_init_valid_names(valid_name: str) -> None:
    """Test Package initialization with various valid name patterns."""
    package = Package(valid_name)
    assert package.name == valid_name


def test_package_architecture_equality() -> None:
    """Test PackageArchitecture equality comparison."""
    arch1 = PackageArchitecture()
    arch2 = PackageArchitecture()

    assert arch1 == arch2

    arch1.dependencies = Override([relation_or_soname_from_str("test-dep")])
    arch2.dependencies = Override([relation_or_soname_from_str("test-dep")])
    assert arch1 == arch2

    arch2.dependencies = Override([relation_or_soname_from_str("other-dep")])
    assert arch1 != arch2

    assert arch1 != "not an architecture"  # type: ignore[comparison-overlap]
