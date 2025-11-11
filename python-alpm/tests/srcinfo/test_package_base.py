"""Tests for PackageBase and PackageBaseArchitecture classes."""

import pytest
from alpm.alpm_srcinfo.source_info.v1.package import PackageArchitecture
from alpm.alpm_srcinfo.source_info.v1.package_base import (
    PackageBase,
    PackageBaseArchitecture,
)
from alpm.alpm_types import (
    ALPMError,
    Architectures,
    FullVersion,
    KnownArchitecture,
    License,
    OptionalDependency,
    PackageRelation,
    RelativeFilePath,
    SkippableBlake2b512Checksum,
    SkippableCrc32CksumChecksum,
    SkippableMd5Checksum,
    SkippableSha1Checksum,
    SkippableSha224Checksum,
    SkippableSha256Checksum,
    SkippableSha384Checksum,
    SkippableSha512Checksum,
    SonameV1,
    Source,
    Url,
    Version,
    VersionComparison,
    VersionRequirement,
    makepkg_option_from_str,
    openpgp_identifier_from_str,
)
from alpm.type_aliases import RelationOrSoname, SkippableChecksum, SystemArchitecture


def test_package_base_init_valid() -> None:
    """Test PackageBase initialization with valid parameters."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.name == "test-package"
    assert package_base.version == version


@pytest.mark.parametrize(
    "invalid_name",
    [
        "",
        "invalid name with spaces",
        "-invalid",
    ],
)
def test_package_base_init_invalid_name(invalid_name: str) -> None:
    """Test PackageBase initialization with invalid names raises ALPMError."""
    version = FullVersion.from_str("1.0.0-1")
    with pytest.raises(ALPMError):
        PackageBase(invalid_name, version)


def test_package_base_name_getter_setter() -> None:
    """Test PackageBase name property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("original-name", version)
    assert package_base.name == "original-name"

    package_base.name = "new-name"
    assert package_base.name == "new-name"


def test_package_base_name_setter_invalid() -> None:
    """Test PackageBase name setter with invalid name raises ALPMError."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("valid-name", version)
    with pytest.raises(ALPMError):
        package_base.name = "invalid name"


def test_package_base_version_getter_setter() -> None:
    """Test PackageBase version property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)
    assert package_base.version == version

    new_version = FullVersion.from_str("2.0.0-1")
    package_base.version = new_version
    assert package_base.version == new_version


def test_package_base_description_getter_setter() -> None:
    """Test PackageBase description property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.description is None

    package_base.description = "Test package description"
    assert package_base.description == "Test package description"

    package_base.description = None
    assert package_base.description is None


def test_package_base_url_getter_setter() -> None:
    """Test PackageBase url property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.url is None

    url = Url("https://example.com")
    package_base.url = url
    assert package_base.url == url


def test_package_base_changelog_getter_setter() -> None:
    """Test PackageBase changelog property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.changelog is None

    changelog = RelativeFilePath("CHANGELOG.md")
    package_base.changelog = changelog
    assert package_base.changelog == changelog


def test_package_base_licenses_getter_setter() -> None:
    """Test PackageBase licenses property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.licenses == []

    licenses = [License("MIT"), License("Apache-2.0")]
    package_base.licenses = licenses
    assert package_base.licenses == licenses


def test_package_base_install_getter_setter() -> None:
    """Test PackageBase install property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.install is None

    install = RelativeFilePath("install.sh")
    package_base.install = install
    assert package_base.install == install


def test_package_base_groups_getter_setter() -> None:
    """Test PackageBase groups property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.groups == []

    groups = ["base", "devel"]
    package_base.groups = groups
    assert package_base.groups == groups


def test_package_base_options_getter_setter() -> None:
    """Test PackageBase options property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.options == []

    options = [makepkg_option_from_str("!strip"), makepkg_option_from_str("!docs")]
    package_base.options = options
    assert package_base.options == options


def test_package_base_backups_getter_setter() -> None:
    """Test PackageBase backups property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.backups == []

    backups = [RelativeFilePath("etc/config.conf"), RelativeFilePath("etc/other.conf")]
    package_base.backups = backups
    assert package_base.backups == backups


def test_package_base_pgp_fingerprints_getter_setter() -> None:
    """Test PackageBase pgp_fingerprints property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.pgp_fingerprints == []

    fingerprints = [
        openpgp_identifier_from_str("ABCD1234ABCD1234"),  # 16 hex chars for key ID
        openpgp_identifier_from_str(
            "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234"
        ),  # 40 hex chars for v4 fingerprint
    ]
    package_base.pgp_fingerprints = fingerprints
    assert package_base.pgp_fingerprints == fingerprints


def test_package_base_architectures_getter_setter() -> None:
    """Test PackageBase architectures property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert len(package_base.architectures) == 0

    architectures = Architectures([KnownArchitecture.X86_64, KnownArchitecture.AARCH64])
    package_base.architectures = architectures
    assert package_base.architectures == architectures


def test_package_base_architecture_properties_getter_setter() -> None:
    """Test PackageBase architecture_properties property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.architecture_properties == {}

    arch_props: dict[SystemArchitecture, PackageBaseArchitecture] = {
        KnownArchitecture.X86_64: PackageBaseArchitecture(),
        KnownArchitecture.AARCH64: PackageBaseArchitecture(),
    }
    package_base.architecture_properties = arch_props
    assert len(package_base.architecture_properties) == 2


def test_package_base_dependencies_getter_setter() -> None:
    """Test PackageBase dependencies property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    # Initially empty list
    assert package_base.dependencies == []

    dependencies: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(
                VersionComparison.GREATER_OR_EQUAL, Version.from_str("1.0.0")
            ),
        ),
    ]
    package_base.dependencies = dependencies
    assert package_base.dependencies == dependencies


def test_package_base_optional_dependencies_getter_setter() -> None:
    """Test PackageBase optional_dependencies property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.optional_dependencies == []

    opt_deps = [OptionalDependency(PackageRelation("foo"), "for extra foofyness")]
    package_base.optional_dependencies = opt_deps
    assert package_base.optional_dependencies == opt_deps


def test_package_base_provides_getter_setter() -> None:
    """Test PackageBase provides property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.provides == []

    provides: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(VersionComparison.EQUAL, Version.from_str("1.0.0")),
        ),
    ]
    package_base.provides = provides
    assert package_base.provides == provides


def test_package_base_conflicts_getter_setter() -> None:
    """Test PackageBase conflicts property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.conflicts == []

    conflicts = [PackageRelation("conflicting-package")]
    package_base.conflicts = conflicts
    assert package_base.conflicts == conflicts


def test_package_base_replaces_getter_setter() -> None:
    """Test PackageBase replaces property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.replaces == []

    replaces = [PackageRelation("old-package")]
    package_base.replaces = replaces
    assert package_base.replaces == replaces


def test_package_base_check_dependencies_getter_setter() -> None:
    """Test PackageBase check_dependencies property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.check_dependencies == []

    check_deps = [PackageRelation("check-tool")]
    package_base.check_dependencies = check_deps
    assert package_base.check_dependencies == check_deps


def test_package_base_make_dependencies_getter_setter() -> None:
    """Test PackageBase make_dependencies property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.make_dependencies == []

    make_deps = [PackageRelation("build-tool")]
    package_base.make_dependencies = make_deps
    assert package_base.make_dependencies == make_deps


def test_package_base_sources_getter_setter() -> None:
    """Test PackageBase sources property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.sources == []

    sources = [Source("https://example.com/source.tar.gz")]
    package_base.sources = sources
    assert package_base.sources == sources


def test_package_base_no_extracts_getter_setter() -> None:
    """Test PackageBase no_extracts property getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert package_base.no_extracts == []

    no_extracts = ["foo.tar.gz", "bar.tar.gz"]
    package_base.no_extracts = no_extracts
    assert package_base.no_extracts == no_extracts


@pytest.mark.parametrize(
    "checksum_attr,checksum_class,valid_hash",
    [
        (
            "b2_checksums",
            SkippableBlake2b512Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "md5_checksums",
            SkippableMd5Checksum,
            "abcdef1234567890abcdef1234567890",
        ),
        (
            "sha1_checksums",
            SkippableSha1Checksum,
            "abcdef1234567890abcdef1234567890abcdef12",
        ),
        (
            "sha224_checksums",
            SkippableSha224Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        ),
        (
            "sha256_checksums",
            SkippableSha256Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "sha384_checksums",
            SkippableSha384Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "sha512_checksums",
            SkippableSha512Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "crc_checksums",
            SkippableCrc32CksumChecksum,
            "4294967295",
        ),
    ],
)
def test_package_base_checksums_getter_setter(
    checksum_attr: str, checksum_class: type[SkippableChecksum], valid_hash: str
) -> None:
    """Test PackageBase checksum properties getter and setter."""
    version = FullVersion.from_str("1.0.0-1")
    package_base = PackageBase("test-package", version)

    assert getattr(package_base, checksum_attr) == []

    checksums = [checksum_class("SKIP"), checksum_class(valid_hash)]
    setattr(package_base, checksum_attr, checksums)

    retrieved_checksums = getattr(package_base, checksum_attr)
    assert len(retrieved_checksums) == 2
    assert str(retrieved_checksums[0]) == "SKIP"
    assert str(retrieved_checksums[1]) == valid_hash


def test_package_base_equality() -> None:
    """Test PackageBase equality comparison."""
    version = FullVersion.from_str("1.0.0-1")
    package_base1 = PackageBase("test-package", version)
    package_base2 = PackageBase("test-package", version)
    package_base3 = PackageBase("different-package", version)

    assert package_base1 == package_base2
    assert package_base1 != package_base3
    assert package_base1 != "not a package base"


def test_package_base_architecture_merge_package_properties() -> None:
    """Test PackageBaseArchitecture merge_package_properties method."""
    arch_props = PackageBaseArchitecture()
    package_arch = PackageArchitecture()
    arch_props.merge_package_properties(package_arch)


def test_package_base_architecture_dependencies_getter_setter() -> None:
    """Test PackageBaseArchitecture dependencies property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.dependencies == []

    dependencies: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(
                VersionComparison.GREATER_OR_EQUAL, Version.from_str("2.0.0")
            ),
        ),
    ]
    arch_props.dependencies = dependencies
    assert arch_props.dependencies == dependencies


def test_package_base_architecture_optional_dependencies_getter_setter() -> None:
    """Test PackageBaseArchitecture optional_dependencies property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.optional_dependencies == []

    opt_deps = [OptionalDependency(PackageRelation("foo"), "for extra foofyness")]
    arch_props.optional_dependencies = opt_deps
    assert arch_props.optional_dependencies == opt_deps


def test_package_base_architecture_provides_getter_setter() -> None:
    """Test PackageBaseArchitecture provides property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.provides == []

    provides: list[RelationOrSoname] = [
        SonameV1("foo.so"),
        PackageRelation(
            "bar",
            VersionRequirement(VersionComparison.EQUAL, Version.from_str("1.0.0")),
        ),
    ]
    arch_props.provides = provides
    assert arch_props.provides == provides


def test_package_base_architecture_conflicts_getter_setter() -> None:
    """Test PackageBaseArchitecture conflicts property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.conflicts == []

    conflicts = [PackageRelation("arch-conflicting-package")]
    arch_props.conflicts = conflicts
    assert arch_props.conflicts == conflicts


def test_package_base_architecture_replaces_getter_setter() -> None:
    """Test PackageBaseArchitecture replaces property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.replaces == []

    replaces = [PackageRelation("arch-old-package")]
    arch_props.replaces = replaces
    assert arch_props.replaces == replaces


def test_package_base_architecture_check_dependencies_getter_setter() -> None:
    """Test PackageBaseArchitecture check_dependencies property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.check_dependencies == []

    check_deps = [PackageRelation("arch-check-tool")]
    arch_props.check_dependencies = check_deps
    assert arch_props.check_dependencies == check_deps


def test_package_base_architecture_make_dependencies_getter_setter() -> None:
    """Test PackageBaseArchitecture make_dependencies property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.make_dependencies == []

    make_deps = [PackageRelation("arch-build-tool")]
    arch_props.make_dependencies = make_deps
    assert arch_props.make_dependencies == make_deps


def test_package_base_architecture_sources_getter_setter() -> None:
    """Test PackageBaseArchitecture sources property getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert arch_props.sources == []

    sources = [Source("https://example.com/arch-source.tar.gz")]
    arch_props.sources = sources
    assert arch_props.sources == sources


@pytest.mark.parametrize(
    "checksum_attr,checksum_class,valid_hash",
    [
        (
            "b2_checksums",
            SkippableBlake2b512Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "md5_checksums",
            SkippableMd5Checksum,
            "abcdef1234567890abcdef1234567890",
        ),
        (
            "sha1_checksums",
            SkippableSha1Checksum,
            "abcdef1234567890abcdef1234567890abcdef12",
        ),
        (
            "sha224_checksums",
            SkippableSha224Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
        ),
        (
            "sha256_checksums",
            SkippableSha256Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "sha384_checksums",
            SkippableSha384Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "sha512_checksums",
            SkippableSha512Checksum,
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        ),
        (
            "crc_checksums",
            SkippableCrc32CksumChecksum,
            "4294967295",
        ),
    ],
)
def test_package_base_architecture_checksums_getter_setter(
    checksum_attr: str, checksum_class: type[SkippableChecksum], valid_hash: str
) -> None:
    """Test PackageBaseArchitecture checksum properties getter and setter."""
    arch_props = PackageBaseArchitecture()

    assert getattr(arch_props, checksum_attr) == []

    checksums = [checksum_class("SKIP"), checksum_class(valid_hash)]
    setattr(arch_props, checksum_attr, checksums)

    retrieved_checksums = getattr(arch_props, checksum_attr)
    assert len(retrieved_checksums) == 2
    assert str(retrieved_checksums[0]) == "SKIP"
    assert str(retrieved_checksums[1]) == valid_hash


def test_package_base_architecture_equality() -> None:
    """Test PackageBaseArchitecture equality comparison."""
    arch1 = PackageBaseArchitecture()
    arch2 = PackageBaseArchitecture()

    assert arch1 == arch2

    arch1.dependencies = [SonameV1("test-dep.so")]
    arch2.dependencies = [SonameV1("test-dep.so")]
    assert arch1 == arch2

    arch2.dependencies = [SonameV1("different-dep.so")]
    assert arch1 != arch2

    assert arch1 != "not an architecture"
