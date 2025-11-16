"""Tests for relation functionality."""

import pytest
from alpm import ALPMError, alpm_types


def test_package_relation_init_name_only() -> None:
    """Test creating PackageRelation with name only."""
    relation = alpm_types.PackageRelation("libfoo")
    assert relation.name == "libfoo"
    assert relation.version_requirement is None


def test_package_relation_init_with_version_requirement() -> None:
    """Test creating PackageRelation with version requirement."""
    version_req = alpm_types.VersionRequirement.from_str(">=1.0.0")
    relation = alpm_types.PackageRelation("libfoo", version_req)
    assert relation.name == "libfoo"
    assert relation.version_requirement is not None


@pytest.mark.parametrize(
    "name",
    [
        "libfoo",
        "lib32-foo",
        "lib.foo",
        "lib_foo",
        "lib-foo-123",
    ],
)
def test_package_relation_valid_names(name: str) -> None:
    """Test creating PackageRelation with various valid names."""
    relation = alpm_types.PackageRelation(name)
    assert relation.name == name


@pytest.mark.parametrize(
    "invalid_name",
    [
        "",
        " ",
        "lib foo",  # space in name
        "-libfoo",  # starts with dash
        ".libfoo",  # starts with dot
        "libfoo/",  # contains slash
    ],
)
def test_package_relation_invalid_names(invalid_name: str) -> None:
    """Test creating PackageRelation with invalid names raises error."""
    with pytest.raises(ALPMError):
        alpm_types.PackageRelation(invalid_name)


def test_package_relation_equality() -> None:
    """Test PackageRelation equality."""
    relation1 = alpm_types.PackageRelation("libfoo")
    relation2 = alpm_types.PackageRelation("libfoo")
    assert relation1 == relation2


def test_package_relation_inequality() -> None:
    """Test PackageRelation inequality."""
    relation1 = alpm_types.PackageRelation("libfoo")
    relation2 = alpm_types.PackageRelation("libbar")
    assert relation1 != relation2


def test_package_relation_string_representation() -> None:
    """Test PackageRelation string representation."""
    relation = alpm_types.PackageRelation("libfoo")
    str_repr = str(relation)
    assert "libfoo" in str_repr


def test_package_relation_repr() -> None:
    """Test PackageRelation repr."""
    relation = alpm_types.PackageRelation("libfoo")
    repr_str = repr(relation)
    assert "PackageRelation" in repr_str
    assert "libfoo" in repr_str


@pytest.mark.parametrize(
    "comparison",
    [
        ">=1.0.0",
        "<=2.0.0",
        "=1.5.0",
        ">0.5.0",
        "<3.0.0",
    ],
)
def test_package_relation_with_various_version_requirements(comparison: str) -> None:
    """Test PackageRelation with various version requirements."""
    version_req = alpm_types.VersionRequirement.from_str(comparison)
    relation = alpm_types.PackageRelation("libfoo", version_req)
    assert relation.name == "libfoo"
    assert relation.version_requirement is not None


def test_optional_dependency_init_basic() -> None:
    """Test creating OptionalDependency with basic PackageRelation."""
    package_relation = alpm_types.PackageRelation("libfoo")
    opt_dep = alpm_types.OptionalDependency(package_relation)
    assert opt_dep.name == "libfoo"
    assert opt_dep.description is None
    assert opt_dep.version_requirement is None


def test_optional_dependency_init_with_description() -> None:
    """Test creating OptionalDependency with description."""
    package_relation = alpm_types.PackageRelation("libfoo")
    opt_dep = alpm_types.OptionalDependency(package_relation, "for foo support")
    assert opt_dep.name == "libfoo"
    assert opt_dep.description == "for foo support"


def test_optional_dependency_init_with_version_requirement() -> None:
    """Test creating OptionalDependency with version requirement."""
    version_req = alpm_types.VersionRequirement.from_str(">=1.0.0")
    package_relation = alpm_types.PackageRelation("libfoo", version_req)
    opt_dep = alpm_types.OptionalDependency(package_relation, "for advanced features")
    assert opt_dep.name == "libfoo"
    assert opt_dep.version_requirement is not None
    assert opt_dep.description == "for advanced features"


@pytest.mark.parametrize(
    "dep_str, expected_name, has_description",
    [
        ("libfoo", "libfoo", False),
        ("libfoo: for foo support", "libfoo", True),
        ("libbar>=1.0: for bar integration", "libbar", True),
        ("lib-foohttp: HTTP library", "lib-foohttp", True),
    ],
)
def test_optional_dependency_from_str_valid(
    dep_str: str, expected_name: str, has_description: bool
) -> None:
    """Test creating OptionalDependency from valid string."""
    opt_dep = alpm_types.OptionalDependency.from_str(dep_str)
    assert opt_dep.name == expected_name
    if has_description:
        assert opt_dep.description is not None
        assert len(opt_dep.description) > 0
    else:
        assert opt_dep.description is None


@pytest.mark.parametrize(
    "invalid_dep",
    [
        "",
        ":",  # empty name and description
        "libfoo:",  # empty description
        ":description",  # empty name
        "lib foo",  # invalid package name
    ],
)
def test_optional_dependency_from_str_invalid(invalid_dep: str) -> None:
    """Test creating OptionalDependency from invalid string raises error."""
    with pytest.raises(ALPMError):
        alpm_types.OptionalDependency.from_str(invalid_dep)


def test_optional_dependency_equality() -> None:
    """Test OptionalDependency equality."""
    package_relation1 = alpm_types.PackageRelation("libfoo")
    package_relation2 = alpm_types.PackageRelation("libfoo")
    opt_dep1 = alpm_types.OptionalDependency(package_relation1, "desc")
    opt_dep2 = alpm_types.OptionalDependency(package_relation2, "desc")
    assert opt_dep1 == opt_dep2


def test_optional_dependency_string_representation() -> None:
    """Test OptionalDependency string representation."""
    package_relation = alpm_types.PackageRelation("libfoo")
    opt_dep = alpm_types.OptionalDependency(package_relation, "for foo support")
    str_repr = str(opt_dep)
    assert "libfoo" in str_repr


@pytest.mark.parametrize(
    "input_str",
    [
        "libfoo",
        "libbar>=1.0.0",
        "lib-foo<=2.0.0",
        "lib-bar=11.1.0",
    ],
)
def test_relation_or_soname_from_str_package_relations(input_str: str) -> None:
    """Test parsing strings that should return PackageRelation."""
    result = alpm_types.relation_or_soname_from_str(input_str)
    assert isinstance(result, alpm_types.PackageRelation)


@pytest.mark.parametrize(
    "input_str",
    [
        "libfoo.so",
    ],
)
def test_relation_or_soname_from_str_sonames(input_str: str) -> None:
    """Test parsing strings that should return SonameV1."""
    result = alpm_types.relation_or_soname_from_str(input_str)
    assert isinstance(result, alpm_types.SonameV1)


@pytest.mark.parametrize(
    "input_str",
    [
        "lib:libfoo.so",
        "lib:libfoo.so.1",
    ],
)
def test_relation_or_soname_from_str_soname_v2(input_str: str) -> None:
    """Test parsing strings that should return SonameV2."""
    result = alpm_types.relation_or_soname_from_str(input_str)
    assert isinstance(result, alpm_types.SonameV2)


@pytest.mark.parametrize(
    "invalid_str",
    [
        "",
        " ",
        "lib foo",  # space in name
        ">>invalid",  # invalid comparison
    ],
)
def test_relation_or_soname_from_str_invalid(invalid_str: str) -> None:
    """Test parsing invalid strings raises error."""
    with pytest.raises(ALPMError):
        alpm_types.relation_or_soname_from_str(invalid_str)


def test_relation_or_soname_from_str_complex_package_relation() -> None:
    """Test parsing complex package relation."""
    result = alpm_types.relation_or_soname_from_str("python-setuptools>=40.0.0")
    assert isinstance(result, alpm_types.PackageRelation)


@pytest.mark.parametrize("comparison_op", [">=", "<=", "=", ">", "<"])
def test_relation_or_soname_from_str_all_comparisons(comparison_op: str) -> None:
    """Test parsing package relations with all comparison operators."""
    input_str = f"libtest{comparison_op}1.0.0"
    result = alpm_types.relation_or_soname_from_str(input_str)
    assert isinstance(result, alpm_types.PackageRelation)
