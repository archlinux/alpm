"""Python bindings for alpm-types Rust crate."""

from enum import Enum
from pathlib import Path

from typing_extensions import Union, Optional, TypeAlias

class ALPMError(Exception):
    """The ALPM error type."""

Checksum: TypeAlias = Union[
    Blake2b512Checksum,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
]

class Blake2b512Checksum:
    """A checksum using the Blake2b512 algorithm."""

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Blake2b512Checksum") -> bool: ...
    def __le__(self, other: "Blake2b512Checksum") -> bool: ...
    def __gt__(self, other: "Blake2b512Checksum") -> bool: ...
    def __ge__(self, other: "Blake2b512Checksum") -> bool: ...

class Md5Checksum:
    """A checksum using the Md5 algorithm.

    WARNING: Use of this algorithm is highly discouraged, because it is cryptographically unsafe.
    """

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Md5Checksum") -> bool: ...
    def __le__(self, other: "Md5Checksum") -> bool: ...
    def __gt__(self, other: "Md5Checksum") -> bool: ...
    def __ge__(self, other: "Md5Checksum") -> bool: ...

class Sha1Checksum:
    """A checksum using the Sha1 algorithm.

    WARNING: Use of this algorithm is highly discouraged, because it is cryptographically unsafe.
    """

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Sha1Checksum") -> bool: ...
    def __le__(self, other: "Sha1Checksum") -> bool: ...
    def __gt__(self, other: "Sha1Checksum") -> bool: ...
    def __ge__(self, other: "Sha1Checksum") -> bool: ...

class Sha224Checksum:
    """A checksum using the Sha224 algorithm."""

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Sha224Checksum") -> bool: ...
    def __le__(self, other: "Sha224Checksum") -> bool: ...
    def __gt__(self, other: "Sha224Checksum") -> bool: ...
    def __ge__(self, other: "Sha224Checksum") -> bool: ...

class Sha256Checksum:
    """A checksum using the Sha256 algorithm."""

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Sha256Checksum") -> bool: ...
    def __le__(self, other: "Sha256Checksum") -> bool: ...
    def __gt__(self, other: "Sha256Checksum") -> bool: ...
    def __ge__(self, other: "Sha256Checksum") -> bool: ...

class Sha384Checksum:
    """A checksum using the Sha384 algorithm."""

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Sha384Checksum") -> bool: ...
    def __le__(self, other: "Sha384Checksum") -> bool: ...
    def __gt__(self, other: "Sha384Checksum") -> bool: ...
    def __ge__(self, other: "Sha384Checksum") -> bool: ...

class Sha512Checksum:
    """A checksum using the Sha512 algorithm."""

    def __init__(self, value: str): ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Sha512Checksum") -> bool: ...
    def __le__(self, other: "Sha512Checksum") -> bool: ...
    def __gt__(self, other: "Sha512Checksum") -> bool: ...
    def __ge__(self, other: "Sha512Checksum") -> bool: ...

class BuildEnvironmentOption:
    """An option string used in a build environment.

    The option string is identified by its name and whether it is on (not prefixed with "!") or off
    (prefixed with "!").
    """

    def __init__(self, option: str) -> None:
        """Create a new BuildEnvironmentOption.

        Args:
            option (str): The option string to parse.

        Raises:
            ALPMError: If the input doesn't match any known option.
        """

    @property
    def name(self) -> str:
        """Name of the option."""

    @property
    def on(self) -> bool:
        """True if the option is on."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class PackageOption:
    """An option string used in packaging.

    The option string is identified by its name and whether it is on (not prefixed with "!") or off
    (prefixed with "!").
    """

    def __init__(self, option: str) -> None:
        """Create a new PackageOption.

        Args:
            option (str): The option string to parse.

        Raises:
            ALPMError: If the input doesn't match any known option.
        """

    @property
    def name(self) -> str:
        """Name of the option."""

    @property
    def on(self) -> bool:
        """True if the option is on."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def makepkg_option_from_str(
    option: str,
) -> Union[BuildEnvironmentOption, PackageOption]:
    """Parse a makepkg option string into the appropriate option type.

    Args:
        option (str): The option string to parse.

    Returns:
        BuildEnvironmentOption | PackageOption: A valid option object.

    Raises:
        ALPMError: If the input doesn't match any known option.
    """
    ...

class License:
    """A license expression that can be either a valid SPDX identifier or a non-standard one."""

    def __init__(self, identifier: str) -> None:
        """Create a new License from an SPDX identifier.

        Args:
            license (str): License expression.
        """

    @classmethod
    def from_valid_spdx(cls, identifier: str) -> "License":
        """
        Create a new License instance from a valid SPDX identifier string.

        Args:
            identifier (str): A valid SPDX license identifier.

        Returns:
            license (License): A new License instance.

        Raises:
            ALPMError: If the identifier is not a valid SPDX license identifier.
        """

    @property
    def is_spdx(self) -> bool:
        """True if the license is a valid SPDX identifier."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class OpenPGPKeyId:
    """An OpenPGP Key ID.

    Wraps a string representing a valid OpenPGP Key ID, ensuring that it consists of exactly 16 uppercase hexadecimal
    characters.
    """

    def __init__(self, key_id: str) -> None:
        """Create a new OpenPGP key ID from a string representation.

        Args:
            key_id (str): A string representing the OpenPGP Key ID,
                          which must be exactly 16 uppercase hexadecimal characters.

        Returns:
            OpenPGPKeyId: A new instance of OpenPGPKeyId.

        Raises:
            ALPMError: If the input string is not a valid OpenPGP Key ID.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class OpenPGPv4Fingerprint:
    """An OpenPGP v4 fingerprint.

    Wraps a string representing a valid OpenPGP v4 fingerprint, ensuring that it consists of exactly 40 uppercase
    hexadecimal characters.
    """

    def __init__(self, fingerprint: str) -> None:
        """Create a new OpenPGP v4 fingerprint from a string representation.

        Args:
            fingerprint (str): A string representing the OpenPGP v4 fingerprint,
                               which must be exactly 40 uppercase hexadecimal characters.

        Returns:
            OpenPGPv4Fingerprint: A new instance of OpenPGPv4Fingerprint.

        Raises:
            ALPMError: If the input string is not a valid OpenPGP v4 fingerprint.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def openpgp_identifier_from_str(
    identifier: str,
) -> Union[OpenPGPKeyId, OpenPGPv4Fingerprint]:
    """
    Create a valid OpenPGPKeyId or OpenPGPv4Fingerprint.

    Args:
        identifier (str): OpenPGP identifier string, which can be either a key ID or a v4 fingerprint.

    Returns:
        OpenPGPKeyId | OpenPGPv4Fingerprint: A valid OpenPGP identifier object.

    Raises:
        ALPMError: If the input string isn't a valid OpenPGP key ID or v4 fingerprint.
    """

class RelativePath:
    """A representation of a relative file path.

    Wraps a Path that is guaranteed to represent a relative file path (i.e. it does not end with a '/').
    """

    def __init__(self, path: Union[Path, str]) -> None:
        """Create a new relative path.

        Args:
            path (Path | str): The file path.

        Raises:
            ALPMError: If the provided string is not a valid relative path.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class Architecture(Enum):
    """CPU architecture."""

    AARCH64 = ("aarch64",)
    ANY = ("any",)
    ARM = ("arm",)
    ARMV6H = ("armv6h",)
    ARMV7H = ("armv7h",)
    I386 = ("i386",)
    I486 = ("i486",)
    I686 = ("i686",)
    PENTIUM4 = ("pentium4",)
    RISCV32 = ("riscv32",)
    RISCV64 = ("riscv64",)
    X86_64 = ("x86_64",)
    X86_64_V2 = ("x86_64_v2",)
    X86_64_V3 = ("x86_64_v3",)
    X86_64_V4 = ("x86_64_v4",)

    @classmethod
    def from_str(cls, arch: str) -> "Architecture":
        """Create an Architecture from a string.

        Args:
            arch (str): A string representing CPU architecture.

        Returns:
            Architecture: The corresponding Architecture enum variant.

        Raises:
            ValueError: If the architecture string doesn't match any enum variant.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class Url:
    """Represents a URL.

    It is used to represent the upstream URL of a package.
    This type does not yet enforce a secure connection (e.g. HTTPS).
    """

    def __init__(self, url: str) -> None:
        """Create a new URL from a string representation.

        Args:
            url (str): A string representing URL.

        Raises:
            ALPMError: If the URL is invalid.
        """
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class Epoch:
    """An epoch of a package.

    Epoch is used to indicate the downgrade of a package and is prepended to a version, delimited by a ':' (e.g. '1:' is
    added to '0.10.0-1' to form '1:0.10.0-1' which then orders newer than '1.0.0-1').

    An Epoch wraps an int that is guaranteed to be greater than 0.
    """

    def __init__(self, value: int) -> None:
        """Create a new epoch from a positive integer.

        Args:
            value (int): The epoch value, must be a non-zero positive integer.

        Raises:
            ValueError: If the epoch is not a positive integer.
            OverflowError: If the epoch is greater than the system's pointer size.
        """

    @classmethod
    def from_str(cls, epoch: str) -> "Epoch":
        """Create a new Epoch from a string representation.

        Args:
            epoch (str): The string representation of the epoch.

        Returns:
            Epoch: A new instance of Epoch.

        Raises:
            ALPMError: If the string cannot be parsed as a valid epoch.
        """

    @property
    def value(self) -> int:
        """Epoch value as an integer."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Epoch") -> bool: ...
    def __le__(self, other: "Epoch") -> bool: ...
    def __gt__(self, other: "Epoch") -> bool: ...
    def __ge__(self, other: "Epoch") -> bool: ...

class PackageRelease:
    """The release version of a package.

    Wraps int for its major version and Optional[int] for its minor version.

    Used to indicate the build version of a package.
    """

    def __init__(self, major: int = 0, minor: Optional[int] = None) -> None:
        """Create a new package release.

        Args:
            major (int): The major version of the package release.
            minor (Optional[int]): The minor version of the package release, defaults to None

        Raises:
            OverflowError: If the major or minor version is negative or greater than the system's pointer size.
        """

    @classmethod
    def from_str(cls, version: str) -> "PackageRelease":
        """Create a PackageRelease from a string representation.

        Args:
            version: (str): The string representation of the package release.

        Returns:
            PackageRelease: A new instance of PackageRelease.

        Raises:
            ALPMError: If the string cannot be parsed as a valid package release.
        """

    @property
    def major(self) -> int:
        """Major version of the package release."""

    @property
    def minor(self) -> Optional[int]:
        """Minor version of the package release, if available."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class PackageVersion:
    """A pkgver of a package

    Used to denote the upstream version of a package.

    Wraps a string, which is guaranteed to only contain alphanumeric characters, '_', '+'` or '.', but to not start with
    a '_', a '+' or a '.' character and to be at least one char long.

    NOTE: This implementation of PackageVersion is stricter than that of libalpm/pacman. It does not allow empty strings
    '', or chars that are not in the allowed set, or '.' as the first character.
    """

    def __init__(self, pkgver: str) -> None:
        """Create a new package version from a string representation.

        Args:
            pkgver: (str): The package version string, must be a valid pkgver.

        Raises:
            ALPMError: If the pkgver is not a valid pkgver string.
        """
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "PackageVersion") -> bool: ...
    def __le__(self, other: "PackageVersion") -> bool: ...
    def __gt__(self, other: "PackageVersion") -> bool: ...
    def __ge__(self, other: "PackageVersion") -> bool: ...

class SchemaVersion:
    """The schema version of a type.

    Wraps a semver version. However, for backwards compatibility reasons it is possible to initialize a SchemaVersion
    using a non-semver compatible string, if it can be parsed to a single 64-bit unsigned integer (e.g. '1').
    """

    def __init__(
        self,
        major: int = 0,
        minor: int = 0,
        patch: int = 0,
        pre: str = "",
        build: str = "",
    ) -> None:
        """Create a new schema version.

        Args:
            major (int): The major version number.
            minor (int): The minor version number.
            patch (int): The patch version number.
            pre (str): Optional pre-release identifier on a version string. This comes after '-' in a semver version,
                       like '1.0.0-alpha.1'.
            build (str): Optional build metadata identifier. This comes after '+' in a semver version, as in
                         '0.8.1+zstd.1.5.0'.

        Raises:
            ALPMError: If the pre-release or build metadata cannot be parsed.
            OverflowError: If the major, minor, or patch version is greater than 2^64 - 1 or negative.
        """

    @classmethod
    def from_str(cls, version: str) -> "SchemaVersion":
        """Create a SchemaVersion from a string representation.

        Args:
            version (str): The string representation of the schema version.

        Returns:
            SchemaVersion: A new instance of SchemaVersion.

        Raises:
            ALPMError: If the string cannot be parsed as a valid schema version.
        """

    @property
    def major(self) -> int:
        """Major version number of the schema version."""

    @property
    def minor(self) -> int:
        """Minor version number of the schema version."""

    @property
    def patch(self) -> int:
        """Patch version number of the schema version."""

    @property
    def pre(self) -> str:
        """Pre-release identifier of the schema version."""

    @property
    def build(self) -> str:
        """Build metadata identifier of the schema version."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "SchemaVersion") -> bool: ...
    def __le__(self, other: "SchemaVersion") -> bool: ...
    def __gt__(self, other: "SchemaVersion") -> bool: ...
    def __ge__(self, other: "SchemaVersion") -> bool: ...

class OptionalDependency:
    """An optional dependency for a package.

    This type is used for representing dependencies that are not essential for base functionality of a package, but may
    be necessary to make use of certain features of a package.

    An OptionalDependency consists of a package relation and an optional description separated by a colon (':').
    The package relation component must be a valid PackageRelation.
    If a description is provided it must be at least one character long.
    """

    def __init__(
        self, package_relation: "PackageRelation", description: Optional[str] = None
    ):
        """Create a new optional dependency.

        Args:
            package_relation (PackageRelation): The package relation of the optional dependency.
            description (Optional[str]): An optional description of the dependency.
        """

    @classmethod
    def from_str(cls, s: str):
        """Create a new OptionalDependency from a string representation.

        Args:
            s (str): The string representation of the optional dependency.

        Returns:
            OptionalDependency: A new instance of OptionalDependency.

        Raises:
            ALPMError: If the string cannot be parsed as a valid optional dependency.
        """

    @property
    def name(self) -> str:
        """Name of the optional dependency."""

    @property
    def version_requirement(self) -> Optional["VersionRequirement"]:
        """Version requirement of the optional dependency, if any."""

    @property
    def description(self) -> Optional[str]:
        """Description of the optional dependency, if any."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class PackageRelation:
    """A package relation.

    Describes a relation to a component.
    Package relations may either consist of only a name or of a name and a version_requirement.
    """

    def __init__(
        self, name: str, version_requirement: Optional["VersionRequirement"] = None
    ):
        """Create a new package relation.

        Args:
            name (str): The name of the package.
            version_requirement (Optional[VersionRequirement]): An optional version requirement for the package.

        Raises:
            ALPMError: If the name is invalid.
        """

    @property
    def name(self) -> str:
        """Name of the package."""

    @property
    def version_requirement(self) -> Optional["VersionRequirement"]:
        """Version requirement of the package, if any."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class SonameV1Type(Enum):
    """The form of a SonameV1."""

    BASIC = ("BASIC",)
    UNVERSIONED = ("UNVERSIONED",)
    EXPLICIT = ("EXPLICIT",)

class SonameV1:
    """Representation of soname data of a shared object based on the alpm-sonamev1 specification.

    This type is deprecated and SonameV2 should be preferred instead! Due to the loose nature of the alpm-sonamev1
    specification, the basic form overlaps with the specification of Name and the explicit form overlaps with that of
    PackageRelation.
    """

    def __init__(
        self,
        name: str,
        version_or_soname: Optional["PackageVersion" | str] = None,
        architecture: Optional["ElfArchitectureFormat"] = None,
    ):
        """Create a new SonameV1.

        Depending on input, returns different forms of SonameV1:
        BASIC, if both version_or_soname and architecture are None
        UNVERSIONED, if version_or_soname is a str and architecture is not None
        EXPLICIT, if version_or_soname is a PackageVersion and architecture is not None

        Args:
            name (str): The name of the shared object.
            version_or_soname (Optional[PackageVersion | str]): The package version (for explicit form) or soname (for unversioned form) of the shared object.
            architecture (Optional[ElfArchitectureFormat]): The architecture of the shared object, only for unversioned or explicit forms.

        Raises:
            ALPMError: If the input is invalid.
        """

    @property
    def name(self) -> str:
        """The least specific name of the shared object file."""

    @property
    def soname(self) -> Optional[str]:
        """The value of the shared object's SONAME field in its dynamic section. Available only for unversioned form."""

    @property
    def version(self) -> Optional["PackageVersion"]:
        """The version of the shared object file (as exposed in its _soname_ data). Available only for explicit form."""

    @property
    def architecture(self) -> Optional["ElfArchitectureFormat"]:
        """The ELF architecture format of the shared object file. Not available for basic form."""

    def form(self) -> "SonameV1Type":
        """The form of this SonameV1."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def relation_or_soname_from_str(s: str) -> Union[PackageRelation, SonameV1]:
    """Parse a string into either a PackageRelation or a SonameV1.

    Args:
        s (str): The string representation of PackageRelation or SonameV1.

    Returns:
        PackageRelation | SonameV1: A valid PackageRelation or SonameV1 object.

    Raises:
        ALPMError: If the input string can't be parsed to a valid PackageRelation or SonameV1.
    """

class VersionComparison(Enum):
    """Specifies the comparison function for a VersionRequirement.

    The package version can be required to be:
    - less than ('<')
    - less than or equal to ('<=')
    - equal to ('=')
    - greater than or equal to ('>=')
    - greater than ('>')
    the specified version.
    """

    LESS_OR_EQUAL = ("<=",)
    GREATER_OR_EQUAL = (">=",)
    EQUAL = ("=",)
    LESS = ("<",)
    GREATER = (">",)

    @classmethod
    def from_str(cls, comparison: str) -> "VersionComparison":
        """Parse a version comparison string into a VersionComparison enum variant.

        Args:
            comparison (str): The version comparison string to parse. Must be one of '<', '<=', '=', '>=', '>'.

        Returns:
            VersionComparison: The corresponding VersionComparison enum variant.

        Raises:
            ALPMError: If the input string doesn't match any known comparison.
        """

    def __eq__(self, other: object) -> bool: ...

class VersionRequirement:
    """A version requirement, e.g. for a dependency package.

    It consists of a target version and a VersionComparison. A version requirement of '>=1.5' has a target version of
    '1.5' and a comparison function of VersionComparison.GREATER_OR_EQUAL.
    """

    def __init__(self, comparison: "VersionComparison", version: "Version") -> None:
        """Create a new version requirement.

        Args:
            comparison (VersionComparison): The comparison function.
            version (Version): The version.
        """

    @classmethod
    def from_str(cls, s: str) -> "VersionRequirement":
        """Create a new VersionRequirement from a string representation.

        Args:
            s (str): The string representation of the version requirement.

        Returns:
            VersionRequirement: A new instance of VersionRequirement.

        Raises:
            ALPMError: If the string cannot be parsed as a valid version requirement.
        """

    def is_satisfied_by(self, ver: "Version") -> bool:
        """Returns True if the requirement is satisfied by the given package version.

        Args:
            ver (Version): The version to check.

        Returns:
            bool: True if the version satisfies the requirement, False otherwise.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class ElfArchitectureFormat(Enum):
    """ELF architecture format.

    This enum represents the Class field in the ELF Header.
    """

    BIT_32 = ("32",)
    BIT_64 = ("64",)

    @classmethod
    def from_str(cls, format: str) -> "ElfArchitectureFormat":
        """Parse an ELF architecture format string into an ElfArchitectureFormat enum variant.

        Args:
            format (str): The ELF architecture format string to parse. Must be one of '32' or '64'.

        Returns:
            ElfArchitectureFormat: The corresponding ElfArchitectureFormat enum variant.

        Raises:
            ValueError: If the input string doesn't match any known format.
        """

    def __eq__(self, other: object) -> bool: ...

class FullVersion:
    """A package version with mandatory PackageRelease.

    Tracks an optional Epoch, a PackageVersion and a PackageRelease. This reflects the 'full' and 'full with epoch'
    forms of alpm-package-version.
    """

    def __init__(
        self,
        pkgver: "PackageVersion",
        pkgrel: "PackageRelease",
        epoch: Optional["Epoch"] = None,
    ) -> None:
        """Create a new FullVersion.

        Args:
            pkgver (PackageVersion): The package version.
            pkgrel (PackageRelease): The package release.
            epoch (Optional[Epoch]): The epoch, if any.
        """

    @classmethod
    def from_str(cls, version: str):
        """Create a FullVersion from a string representation.

        Args:
            version (str): The string representation of the full version.

        Returns:
            FullVersion: A new instance of FullVersion.

        Raises:
            ALPMError: If the string cannot be parsed as a valid full version.
        """

    @property
    def pkgver(self) -> "PackageVersion":
        """The package version."""

    @property
    def pkgrel(self) -> "PackageRelease":
        """The package release."""

    @property
    def epoch(self) -> Optional["Epoch"]:
        """The epoch, if any."""

    def vercmp(self, other: "FullVersion") -> int:
        """Compare this FullVersion with another FullVersion.

        Output behavior is based on the behavior of the vercmp tool.

        Args:
            other (FullVersion): The other FullVersion to compare against.

        Returns:
            int: 1 if self is newer than other,
                 0 if they are equal,
                 -1 if self is older than other.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "FullVersion") -> bool: ...
    def __le__(self, other: "FullVersion") -> bool: ...
    def __gt__(self, other: "FullVersion") -> bool: ...
    def __ge__(self, other: "FullVersion") -> bool: ...

class Version:
    """A version of a package.

    A Version generically tracks an optional Epoch, a PackageVersion and an optional PackageRelease.
    """

    def __init__(
        self,
        pkgver: "PackageVersion",
        pkgrel: Optional["PackageRelease"] = None,
        epoch: Optional["Epoch"] = None,
    ) -> None:
        """Create a new Version.

        Args:
            pkgver (PackageVersion): The package version.
            pkgrel (Optional[PackageRelease]): The package release, if any.
            epoch (Optional[Epoch]): The epoch, if any.
        """

    @classmethod
    def from_str(cls, version: str):
        """Create a FullVersion from a string representation.

        Args:
            version (str): The string representation of the full version.

        Returns:
            FullVersion: A new instance of FullVersion.

        Raises:
            ALPMError: If the string cannot be parsed as a valid full version.
        """

    @property
    def pkgver(self) -> "PackageVersion":
        """The package version."""

    @property
    def pkgrel(self) -> Optional["PackageRelease"]:
        """The package release, if any."""

    @property
    def epoch(self) -> Optional["Epoch"]:
        """The epoch, if any."""

    def vercmp(self, other: "Version") -> int:
        """Compare this Version with another Version.

        Output behavior is based on the behavior of the vercmp tool.

        Args:
            other (Version): The other Version to compare against.

        Returns:
            int: 1 if self is newer than other,
                 0 if they are equal,
                 -1 if self is older than other.
        """

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __lt__(self, other: "Version") -> bool: ...
    def __le__(self, other: "Version") -> bool: ...
    def __gt__(self, other: "Version") -> bool: ...
    def __ge__(self, other: "Version") -> bool: ...

__all__ = [
    "ALPMError",
    "Checksum",
    "Blake2b512Checksum",
    "Md5Checksum",
    "Sha1Checksum",
    "Sha224Checksum",
    "Sha256Checksum",
    "Sha384Checksum",
    "Sha512Checksum",
    "BuildEnvironmentOption",
    "PackageOption",
    "makepkg_option_from_str",
    "License",
    "OpenPGPKeyId",
    "OpenPGPv4Fingerprint",
    "openpgp_identifier_from_str",
    "RelativePath",
    "Architecture",
    "Url",
    "Epoch",
    "PackageRelease",
    "PackageVersion",
    "SchemaVersion",
    "OptionalDependency",
    "PackageRelation",
    "SonameV1",
    "SonameV1Type",
    "relation_or_soname_from_str",
    "VersionComparison",
    "VersionRequirement",
    "ElfArchitectureFormat",
    "FullVersion",
    "Version",
]
