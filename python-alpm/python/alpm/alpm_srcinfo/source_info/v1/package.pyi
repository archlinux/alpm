"""Handling of metadata found in a `pkgname` section of SRCINFO data."""

from typing import TypeVar, Generic, Optional, TypeAlias, Union

from alpm.alpm_types import (
    Url,
    RelativeFilePath,
    License,
    OptionalDependency,
    PackageRelation,
    Architectures,
)
from alpm.type_aliases import RelationOrSoname, MakepkgOption, SystemArchitecture

Overridable: TypeAlias = Union[
    str,
    list[str],
    Url,
    list[MakepkgOption],
    RelativeFilePath,
    list[RelativeFilePath],
    list[License],
    list[RelationOrSoname],
    list[OptionalDependency],
    list[PackageRelation],
]

T = TypeVar("T", bound=Optional[Overridable])

class Override(Generic[T]):
    """An override for respective defaults in PackageBase.

    Used as Override[T] | None:
        - **None** - no override, use the default from PackageBase.

          Equivalent to Override::No in Rust alpm-srcinfo crate.

        - **Override(None)** - clear the field.

          Equivalent to Override::Clear in Rust alpm-srcinfo crate.

        - **Override(value)** - override the field with value.

          Equivalent to Override::Yes { value } in Rust alpm-srcinfo crate.
    """

    def __init__(self, value: Optional[T]):
        """Create a new Override with the given value.

        If value is None, the override clears the field.

        Args:
            value (Optional[T]): The value of the override, or None to clear the field.

        """

    @property
    def value(self) -> Optional[T]:
        """The value of the override."""

    def __repr__(self) -> str: ...

class Package:
    """Package metadata based on a pkgname section in SRCINFO data.

    This class only contains package specific overrides.
    Only in combination with PackageBase data a full view on a package's metadata is
    possible.
    """

    __hash__ = None  # type: ignore

    def __init__(self, name: str):
        """Initialize a new Package with the given name.

        Args:
            name (str): The name of the package. Must be a valid alpm-package-name.

        Raises:
            ALPMError: If the name is not a valid alpm-package-name.

        """

    @property
    def name(self) -> str:
        """The alpm-package-name of the package."""

    @name.setter
    def name(self, name: str) -> None: ...
    @property
    def description(self) -> Optional[Override[str]]:
        """Override of the package's description."""

    @description.setter
    def description(self, description: Optional[Override[str]]) -> None: ...
    @property
    def url(self) -> Optional[Override[Url]]:
        """Override of the package's upstream URL."""

    @url.setter
    def url(self, url: Optional[Override[Url]]) -> None: ...
    @property
    def changelog(self) -> Optional[Override[RelativeFilePath]]:
        """Override of the package's path to a changelog file."""

    @changelog.setter
    def changelog(self, changelog: Optional[Override[RelativeFilePath]]) -> None: ...
    @property
    def licenses(self) -> Optional[Override[list[License]]]:
        """Override of licenses that apply to the package."""

    @licenses.setter
    def licenses(self, licenses: Optional[Override[list[License]]]) -> None: ...
    @property
    def install(self) -> Optional[Override[RelativeFilePath]]:
        """Override of the package's install script path."""

    @install.setter
    def install(self, install: Optional[Override[RelativeFilePath]]) -> None: ...
    @property
    def groups(self) -> Optional[Override[list[str]]]:
        """Override of alpm-package-groups the package is part of."""

    @groups.setter
    def groups(self, groups: Optional[Override[list[str]]]) -> None: ...
    @property
    def options(self) -> Optional[Override[list[MakepkgOption]]]:
        """Override of build tool options used when building the package."""

    @options.setter
    def options(self, options: Optional[Override[list[MakepkgOption]]]) -> None: ...
    @property
    def backups(self) -> Optional[Override[list[RelativeFilePath]]]:
        """Override of paths to files in the package that should be backed up."""

    @backups.setter
    def backups(self, backups: Optional[Override[list[RelativeFilePath]]]) -> None: ...
    @property
    def architectures(self) -> Optional[Architectures]:
        """The architectures that are supported by this package."""

    @architectures.setter
    def architectures(self, architectures: Optional[Architectures]) -> None: ...
    @property
    def architecture_properties(
        self,
    ) -> dict[SystemArchitecture, "PackageArchitecture"]:
        """Architecture specific overrides for the package.

        The keys of the dictionary are the architectures for which overrides are
        specified. The values are PackageArchitecture instances containing the
        overrides.

        This field is only relevant if Package.architectures is set.
        """

    @architecture_properties.setter
    def architecture_properties(
        self, architecture_properties: dict[SystemArchitecture, "PackageArchitecture"]
    ) -> None: ...
    @property
    def dependencies(self) -> Optional[Override[list[RelationOrSoname]]]:
        """The (potentially overridden) list of run-time dependencies of the package."""

    @dependencies.setter
    def dependencies(
        self, dependencies: Optional[Override[list[RelationOrSoname]]]
    ) -> None: ...
    @property
    def optional_dependencies(self) -> Optional[Override[list[OptionalDependency]]]:
        """The (potentially overridden) list of optional dependencies of the package."""

    @optional_dependencies.setter
    def optional_dependencies(
        self, optional_dependencies: Optional[Override[list[OptionalDependency]]]
    ) -> None: ...
    @property
    def provides(self) -> Optional[Override[list[RelationOrSoname]]]:
        """The (potentially overridden) list of provisions of the package."""

    @provides.setter
    def provides(
        self, provides: Optional[Override[list[RelationOrSoname]]]
    ) -> None: ...
    @property
    def conflicts(self) -> Optional[Override[list[PackageRelation]]]:
        """The (potentially overridden) list of conflicts of the package."""

    @conflicts.setter
    def conflicts(
        self, conflicts: Optional[Override[list[PackageRelation]]]
    ) -> None: ...
    @property
    def replaces(self) -> Optional[Override[list[PackageRelation]]]:
        """The (potentially overridden) list of replacements of the package."""

    @replaces.setter
    def replaces(self, replaces: Optional[Override[list[PackageRelation]]]) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class PackageArchitecture:
    """Architecture specific package properties for use in Package.

    For each Architecture defined in Package.architectures, a PackageArchitecture is
    present in Package.architecture_properties.
    """

    __hash__ = None  # type: ignore

    def __init__(self) -> None:
        """Initialize an empty PackageArchitecture with all overrides set to None."""

    @property
    def dependencies(self) -> Optional[Override[list[RelationOrSoname]]]:
        """The (potentially overridden) list of run-time dependencies of the package."""

    @dependencies.setter
    def dependencies(
        self, dependencies: Optional[Override[list[RelationOrSoname]]]
    ) -> None: ...
    @property
    def optional_dependencies(self) -> Optional[Override[list[OptionalDependency]]]:
        """The (potentially overridden) list of optional dependencies of the package."""

    @optional_dependencies.setter
    def optional_dependencies(
        self, optional_dependencies: Optional[Override[list[OptionalDependency]]]
    ) -> None: ...
    @property
    def provides(self) -> Optional[Override[list[RelationOrSoname]]]:
        """The (potentially overridden) list of provisions of the package."""

    @provides.setter
    def provides(
        self, provides: Optional[Override[list[RelationOrSoname]]]
    ) -> None: ...
    @property
    def conflicts(self) -> Optional[Override[list[PackageRelation]]]:
        """The (potentially overridden) list of conflicts of the package."""

    @conflicts.setter
    def conflicts(
        self, conflicts: Optional[Override[list[PackageRelation]]]
    ) -> None: ...
    @property
    def replaces(self) -> Optional[Override[list[PackageRelation]]]:
        """The (potentially overridden) list of replacements of the package."""

    @replaces.setter
    def replaces(self, replaces: Optional[Override[list[PackageRelation]]]) -> None: ...

__all__ = [
    "Override",
    "Package",
    "PackageArchitecture",
]
