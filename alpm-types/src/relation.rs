use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Serialize;
use strum::IntoEnumIterator;
use winnow::{
    ModalResult,
    Parser,
    ascii::digit1,
    combinator::{alt, cut_err, eof, fail, peek, repeat, repeat_till},
    error::{StrContext, StrContextValue},
    stream::Stream,
    token::{any, rest, take_while},
};

use crate::{
    ElfArchitectureFormat,
    Error,
    Name,
    PackageVersion,
    SharedObjectName,
    VersionComparison,
    VersionRequirement,
};

/// Provides either a [`PackageVersion`] or a [`SharedObjectName`].
///
/// This enum is used when creating [`SonameV1`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionOrSoname {
    /// A version for a [`SonameV1`].
    Version(PackageVersion),

    /// A soname for a [`SonameV1`].
    Soname(SharedObjectName),
}

impl FromStr for VersionOrSoname {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl VersionOrSoname {
    /// Recognizes a [`PackageVersion`] or [`SharedObjectName`] in a string slice.
    ///
    /// First attempts to recognize a [`SharedObjectName`] and if that fails, falls back to
    /// recognizing a [`PackageVersion`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // In the following, we're doing our own `alt` implementation.
        // The reason for this is that we build our type parsers so that they throw errors
        // if they encounter unexpected input instead of backtracking.
        let checkpoint = input.checkpoint();
        let soname_result = SharedObjectName::parser.parse_next(input);
        if soname_result.is_ok() {
            let soname = soname_result?;
            return Ok(VersionOrSoname::Soname(soname));
        }

        input.reset(&checkpoint);
        let version_result = rest.try_map(PackageVersion::from_str).parse_next(input);
        if version_result.is_ok() {
            let version = version_result?;
            return Ok(VersionOrSoname::Version(version));
        }

        cut_err(fail)
            .context(StrContext::Expected(StrContextValue::Description(
                "version or shared object name",
            )))
            .parse_next(input)
    }
}

impl Display for VersionOrSoname {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionOrSoname::Version(version) => write!(f, "{version}"),
            VersionOrSoname::Soname(soname) => write!(f, "{soname}"),
        }
    }
}

/// Representation of [soname] data of a shared object based on the [alpm-sonamev1] specification.
///
/// Soname data may be used as [alpm-package-relation] of type _provision_ and _run-time
/// dependency_.
/// This type distinguishes between three forms: _basic_, _unversioned_ and _explicit_.
///
/// - [`SonameV1::Basic`] is used when only the `name` of a _shared object_ file is used. This form
///   can be used in files that may contain static data about package sources (e.g. [PKGBUILD] or
///   [SRCINFO] files).
/// - [`SonameV1::Unversioned`] is used when the `name` of a _shared object_ file, its _soname_
///   (which does _not_ expose a specific version) and its `architecture` (derived from the [ELF]
///   class of the file) are used. This form can be used in files that may contain dynamic data
///   derived from a specific package build environment (i.e. [PKGINFO]). It is discouraged to use
///   this form in files that contain static data about package sources (e.g. [PKGBUILD] or
///   [SRCINFO] files).
/// - [`SonameV1::Explicit`] is used when the `name` of a _shared object_ file, the `version` from
///   its _soname_ and its `architecture` (derived from the [ELF] class of the file) are used. This
///   form can be used in files that may contain dynamic data derived from a specific package build
///   environment (i.e. [PKGINFO]). It is discouraged to use this form in files that contain static
///   data about package sources (e.g. [PKGBUILD] or [SRCINFO] files).
///
/// # Warning
///
/// This type is **deprecated** and `SonameV2` should be preferred instead!
/// Due to the loose nature of the [alpm-sonamev1] specification, the _basic_ form overlaps with the
/// specification of [`Name`] and the _explicit_ form overlaps with that of [`PackageRelation`].
///
/// # Examples
///
/// ```
/// use alpm_types::{ElfArchitectureFormat, SonameV1};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// let basic_soname = SonameV1::Basic("example.so".parse()?);
/// let unversioned_soname = SonameV1::Unversioned {
///     name: "example.so".parse()?,
///     soname: "example.so".parse()?,
///     architecture: ElfArchitectureFormat::Bit64,
/// };
/// let explicit_soname = SonameV1::Explicit {
///     name: "example.so".parse()?,
///     version: "1.0.0".parse()?,
///     architecture: ElfArchitectureFormat::Bit64,
/// };
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [alpm-sonamev1]: https://alpm.archlinux.page/specifications/alpm-sonamev1.7.html
/// [ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
/// [soname]: https://en.wikipedia.org/wiki/Soname
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SonameV1 {
    /// Basic representation of a _shared object_ file.
    ///
    /// Tracks the `name` of a _shared object_ file.
    /// This form is used when referring to _shared object_ files without their soname data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::SonameV1;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let soname = SonameV1::from_str("example.so")?;
    /// assert_eq!(soname, SonameV1::Basic("example.so".parse()?));
    /// # Ok(())
    /// # }
    /// ```
    Basic(SharedObjectName),

    /// Unversioned representation of an ELF file's soname data.
    ///
    /// Tracks the `name` of a _shared object_ file, its _soname_ instead of a version and its
    /// `architecture`. This form is used if the _soname data_ of a _shared object_ does not
    /// expose a version.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{ElfArchitectureFormat, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let soname = SonameV1::from_str("example.so=example.so-64")?;
    /// assert_eq!(
    ///     soname,
    ///     SonameV1::Unversioned {
    ///         name: "example.so".parse()?,
    ///         soname: "example.so".parse()?,
    ///         architecture: ElfArchitectureFormat::Bit64,
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    Unversioned {
        /// The least specific name of the shared object file.
        name: SharedObjectName,
        /// The value of the shared object's _SONAME_ field in its _dynamic section_.
        soname: SharedObjectName,
        /// The ELF architecture format of the shared object file.
        architecture: ElfArchitectureFormat,
    },

    /// Explicit representation of an ELF file's soname data.
    ///
    /// Tracks the `name` of a _shared object_ file, the `version` of its _soname_ and its
    /// `architecture`. This form is used if the _soname data_ of a _shared object_ exposes a
    /// specific version.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{ElfArchitectureFormat, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let soname = SonameV1::from_str("example.so=1.0.0-64")?;
    /// assert_eq!(
    ///    soname,
    ///    SonameV1::Explicit {
    ///         name: "example.so".parse()?,
    ///         version: "1.0.0".parse()?,
    ///         architecture: ElfArchitectureFormat::Bit64,
    ///     }
    /// );
    /// # Ok(())
    /// # }
    Explicit {
        /// The least specific name of the shared object file.
        name: SharedObjectName,
        /// The version of the shared object file (as exposed in its _soname_ data).
        version: PackageVersion,
        /// The ELF architecture format of the shared object file.
        architecture: ElfArchitectureFormat,
    },
}

impl SonameV1 {
    /// Creates a new [`SonameV1`].
    ///
    /// Depending on input, this function returns different variants of [`SonameV1`]:
    ///
    /// - [`SonameV1::Basic`], if both `version_or_soname` and `architecture` are [`None`]
    /// - [`SonameV1::Unversioned`], if `version_or_soname` is [`VersionOrSoname::Soname`] and
    ///   `architecture` is [`Some`]
    /// - [`SonameV1::Explicit`], if `version_or_soname` is [`VersionOrSoname::Version`] and
    ///   `architecture` is [`Some`]
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::{ElfArchitectureFormat, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let basic_soname = SonameV1::new("example.so".parse()?, None, None)?;
    /// assert_eq!(basic_soname, SonameV1::Basic("example.so".parse()?));
    ///
    /// let unversioned_soname = SonameV1::new(
    ///     "example.so".parse()?,
    ///     Some("example.so".parse()?),
    ///     Some(ElfArchitectureFormat::Bit64),
    /// )?;
    /// assert_eq!(
    ///     unversioned_soname,
    ///     SonameV1::Unversioned {
    ///         name: "example.so".parse()?,
    ///         soname: "example.so".parse()?,
    ///         architecture: "64".parse()?
    ///     }
    /// );
    ///
    /// let explicit_soname = SonameV1::new(
    ///     "example.so".parse()?,
    ///     Some("1.0.0".parse()?),
    ///     Some(ElfArchitectureFormat::Bit64),
    /// )?;
    /// assert_eq!(
    ///     explicit_soname,
    ///     SonameV1::Explicit {
    ///         name: "example.so".parse()?,
    ///         version: "1.0.0".parse()?,
    ///         architecture: "64".parse()?
    ///     }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        name: SharedObjectName,
        version_or_soname: Option<VersionOrSoname>,
        architecture: Option<ElfArchitectureFormat>,
    ) -> Result<Self, Error> {
        match (version_or_soname, architecture) {
            (None, None) => Ok(Self::Basic(name)),
            (Some(VersionOrSoname::Version(version)), Some(architecture)) => Ok(Self::Explicit {
                name,
                version,
                architecture,
            }),
            (Some(VersionOrSoname::Soname(soname)), Some(architecture)) => Ok(Self::Unversioned {
                name,
                soname,
                architecture,
            }),
            (None, Some(_)) => Err(Error::InvalidSonameV1(
                "SonameV1 needs a version when specifying architecture",
            )),
            (Some(_), None) => Err(Error::InvalidSonameV1(
                "SonameV1 needs an architecture when specifying version",
            )),
        }
    }

    /// Parses a [`SonameV1`] from a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Parse the shared object name.
        let name = Self::parse_shared_object_name(input)?;

        // Parse the version delimiter `=`.
        //
        // If it doesn't exist, it is the basic form.
        if Self::parse_version_delimiter(input).is_err() {
            return Ok(SonameV1::Basic(name));
        }

        // Take all input until we hit the delimiter and architecture.
        let (raw_version_or_soname, _): (String, _) =
            cut_err(repeat_till(1.., any, peek(("-", digit1, eof))))
                .context(StrContext::Expected(StrContextValue::Description(
                    "a version or shared object name, followed by an ELF architecture format",
                )))
                .parse_next(input)?;

        // Two cases are possible here:
        //
        // 1. Unversioned: `name=soname-architecture`
        // 2. Explicit: `name=version-architecture`
        let version_or_soname =
            VersionOrSoname::parser.parse_next(&mut raw_version_or_soname.as_str())?;

        // Parse the `-` delimiter
        Self::parse_architecture_delimiter(input)?;

        // Parse the architecture
        let architecture = Self::parse_architecture(input)?;

        match version_or_soname {
            VersionOrSoname::Version(version) => Ok(SonameV1::Explicit {
                name,
                version,
                architecture,
            }),
            VersionOrSoname::Soname(soname) => Ok(SonameV1::Unversioned {
                name,
                soname,
                architecture,
            }),
        }
    }

    /// Parses the shared object name until the version delimiter `=`.
    fn parse_shared_object_name(input: &mut &str) -> ModalResult<SharedObjectName> {
        repeat_till(1.., any, peek(alt(("=", eof))))
            .try_map(|(name, _): (String, &str)| SharedObjectName::from_str(&name))
            .context(StrContext::Label("shared object name"))
            .parse_next(input)
    }

    /// Parses the version delimiter `=`.
    ///
    /// This function discards the result for only checking if the version delimiter is present.
    fn parse_version_delimiter(input: &mut &str) -> ModalResult<()> {
        cut_err("=")
            .context(StrContext::Label("version delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "version delimiter `=`",
            )))
            .parse_next(input)
            .map(|_| ())
    }

    /// Parses the architecture delimiter `-`.
    fn parse_architecture_delimiter(input: &mut &str) -> ModalResult<()> {
        cut_err("-")
            .context(StrContext::Label("architecture delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "architecture delimiter `-`",
            )))
            .parse_next(input)
            .map(|_| ())
    }

    /// Parses the architecture.
    fn parse_architecture(input: &mut &str) -> ModalResult<ElfArchitectureFormat> {
        cut_err(take_while(1.., |c: char| c.is_ascii_digit()))
            .try_map(ElfArchitectureFormat::from_str)
            .context(StrContext::Label("architecture"))
            .parse_next(input)
    }
}

impl FromStr for SonameV1 {
    type Err = Error;
    /// Parses a [`SonameV1`] from a string slice.
    ///
    /// The string slice must be in the format `name[=version-architecture]`.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`SonameV1`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{ElfArchitectureFormat, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     SonameV1::from_str("example.so=1.0.0-64")?,
    ///     SonameV1::Explicit {
    ///         name: "example.so".parse()?,
    ///         version: "1.0.0".parse()?,
    ///         architecture: ElfArchitectureFormat::Bit64,
    ///     },
    /// );
    /// assert_eq!(
    ///     SonameV1::from_str("example.so=example.so-64")?,
    ///     SonameV1::Unversioned {
    ///         name: "example.so".parse()?,
    ///         soname: "example.so".parse()?,
    ///         architecture: ElfArchitectureFormat::Bit64,
    ///     },
    /// );
    /// assert_eq!(
    ///     SonameV1::from_str("example.so")?,
    ///     SonameV1::Basic("example.so".parse()?),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for SonameV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(name) => write!(f, "{name}"),
            Self::Unversioned {
                name,
                soname,
                architecture,
            } => write!(f, "{name}={soname}-{architecture}"),
            Self::Explicit {
                name,
                version,
                architecture,
            } => write!(f, "{name}={version}-{architecture}"),
        }
    }
}

/// A prefix associated with a library lookup directory.
///
/// Library lookup directories are used when detecting shared object files on a system.
/// Each such lookup directory can be assigned to a _prefix_, which allows identifying them in other
/// contexts. E.g. `lib` may serve as _prefix_ for the lookup directory `/usr/lib`.
///
/// This is a type alias for [`Name`].
pub type SharedLibraryPrefix = Name;

/// The value of a shared object's _soname_.
///
/// This data may be present in the _SONAME_ or _NEEDED_ fields of a shared object's _dynamic
/// section_.
///
/// The _soname_ data may contain only a shared object name (e.g. `libexample.so`) or a shared
/// object name, that also encodes version information (e.g. `libexample.so.1`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Soname {
    /// The name part of a shared object's _soname_.
    pub name: SharedObjectName,
    /// The optional version part of a shared object's _soname_.
    pub version: Option<PackageVersion>,
}

impl Soname {
    /// Creates a new [`Soname`].
    pub fn new(name: SharedObjectName, version: Option<PackageVersion>) -> Self {
        Self { name, version }
    }

    /// Recognizes a [`Soname`] in a string slice.
    ///
    /// The passed data can be in the following formats:
    ///
    /// - `<name>.so`: A shared object name without a version. (e.g. `libexample.so`)
    /// - `<name>.so.<version>`: A shared object name with a version. (e.g. `libexample.so.1`)
    ///     - The version must be a valid [`PackageVersion`].
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let name = cut_err(
            (
                // Parse the name of the shared object until eof or the `.so` is hit.
                repeat_till::<_, _, String, _, _, _, _>(1.., any, peek(alt((".so", eof)))),
                // Parse at least one or more `.so` suffix(es).
                cut_err(repeat::<_, _, String, _, _>(1.., ".so"))
                    .context(StrContext::Label("suffix"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "shared object name suffix '.so'",
                    ))),
            )
                // Take both parts and map them onto a SharedObjectName
                .take()
                .try_map(Name::from_str)
                .map(SharedObjectName),
        )
        .context(StrContext::Label("shared object name"))
        .parse_next(input)?;

        // Parse the version delimiter.
        let delimiter = cut_err(alt((".", eof)))
            .context(StrContext::Label("version delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "version delimiter `.`",
            )))
            .parse_next(input)?;

        // If a `.` is found, map the rest of the string to a version.
        // Otherwise, we hit the `eof` and there's no version.
        let version = match delimiter {
            "" => None,
            "." => Some(rest.try_map(PackageVersion::from_str).parse_next(input)?),
            _ => unreachable!(),
        };

        Ok(Self { name, version })
    }
}

impl Display for Soname {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            Some(version) => write!(f, "{name}.{version}", name = self.name),
            None => write!(f, "{name}", name = self.name),
        }
    }
}

impl FromStr for Soname {
    type Err = Error;

    /// Recognizes a [`Soname`] in a string slice.
    ///
    /// The string slice must be in the format of `<name>.so` or `<name>.so.<version>`.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`Soname`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::Soname;
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     Soname::from_str("libexample.so.1")?,
    ///     Soname::new("libexample.so".parse()?, Some("1".parse()?)),
    /// );
    /// assert_eq!(
    ///     Soname::from_str("libexample.so")?,
    ///     Soname::new("libexample.so".parse()?, None),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

/// Representation of [soname] data of a shared object based on the [alpm-sonamev2] specification.
///
/// Soname data may be used as [alpm-package-relation] of type _provision_ or _run-time dependency_
/// in [`PackageInfoV1`] and [`PackageInfoV2`]. The data consists of the arbitrarily
/// defined `prefix`, which denotes the use name of a specific library directory, and the `soname`,
/// which refers to the value of either the _SONAME_ or a _NEEDED_ field in the _dynamic section_ of
/// an [ELF] file.
///
/// # Examples
///
/// This example assumpes that `lib` is used as the `prefix` for the library directory `/usr/lib`
/// and the following files are contained in it:
///
/// ```bash
/// /usr/lib/libexample.so -> libexample.so.1
/// /usr/lib/libexample.so.1 -> libexample.so.1.0.0
/// /usr/lib/libexample.so.1.0.0
/// ```
///
/// The above file `/usr/lib/libexample.so.1.0.0` represents an [ELF] file, that exposes
/// `libexample.so.1` as value of the _SONAME_ field in its _dynamic section_. This data can be
/// represented as follows, using [`SonameV2`]:
///
/// ```rust
/// use alpm_types::{Soname, SonameV2};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// let soname_data = SonameV2 {
///     prefix: "lib".parse()?,
///     soname: Soname {
///         name: "libexample.so".parse()?,
///         version: Some("1".parse()?),
///     },
/// };
/// assert_eq!(soname_data.to_string(), "lib:libexample.so.1");
/// # Ok(())
/// # }
/// ```
///
/// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
/// [soname]: https://en.wikipedia.org/wiki/Soname
/// [`PackageInfoV1`]: https://docs.rs/alpm_pkginfo/latest/alpm_pkginfo/struct.PackageInfoV1.html
/// [`PackageInfoV2`]: https://docs.rs/alpm_pkginfo/latest/alpm_pkginfo/struct.PackageInfoV2.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SonameV2 {
    /// The directory prefix of the shared object file.
    pub prefix: SharedLibraryPrefix,
    /// The _soname_ of a shared object file.
    pub soname: Soname,
}

impl SonameV2 {
    /// Creates a new [`SonameV2`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::SonameV2;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// SonameV2::new("lib".parse()?, "libexample.so.1".parse()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(prefix: SharedLibraryPrefix, soname: Soname) -> Self {
        Self { prefix, soname }
    }

    /// Recognizes a [`SonameV2`] in a string slice.
    ///
    /// The passed data must be in the format `<prefix>:<soname>`. (e.g. `lib:libexample.so.1`)
    ///
    /// See [`Soname::parser`] for details on the format of `<soname>`.
    ///
    /// # Errors
    ///
    /// Returns an error if no [`SonameV2`] can be created from `input`.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Parse everything from the start to the first `:` and parse as `SharedLibraryPrefix`.
        let prefix = cut_err(
            repeat_till(1.., any, peek(alt((":", eof))))
                .try_map(|(name, _): (String, &str)| SharedLibraryPrefix::from_str(&name)),
        )
        .context(StrContext::Label("prefix for a shared object lookup path"))
        .parse_next(input)?;

        cut_err(":")
            .context(StrContext::Label("shared library prefix delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "shared library prefix `:`",
            )))
            .parse_next(input)?;

        let soname = Soname::parser.parse_next(input)?;

        Ok(Self { prefix, soname })
    }
}

impl FromStr for SonameV2 {
    type Err = Error;

    /// Parses a [`SonameV2`] from a string slice.
    ///
    /// The string slice must be in the format `<prefix>:<soname>`.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`SonameV2`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{Soname, SonameV2};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     SonameV2::from_str("lib:libexample.so.1")?,
    ///     SonameV2::new(
    ///         "lib".parse()?,
    ///         Soname::new("libexample.so".parse()?, Some("1".parse()?))
    ///     ),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for SonameV2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{prefix}:{soname}",
            prefix = self.prefix,
            soname = self.soname
        )
    }
}

/// A package relation
///
/// Describes a relation to a component.
/// Package relations may either consist of only a [`Name`] *or* of a [`Name`] and a
/// [`VersionRequirement`].
///
/// ## Note
///
/// A [`PackageRelation`] covers all **alpm-package-relations** *except* optional
/// dependencies, as those behave differently.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PackageRelation {
    /// The name of the package
    pub name: Name,
    /// The version requirement for the package
    pub version_requirement: Option<VersionRequirement>,
}

impl PackageRelation {
    /// Creates a new [`PackageRelation`]
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::{PackageRelation, VersionComparison, VersionRequirement};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// PackageRelation::new(
    ///     "example".parse()?,
    ///     Some(VersionRequirement {
    ///         comparison: VersionComparison::Less,
    ///         version: "1.0.0".parse()?,
    ///     }),
    /// );
    ///
    /// PackageRelation::new("example".parse()?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: Name, version_requirement: Option<VersionRequirement>) -> Self {
        Self {
            name,
            version_requirement,
        }
    }
}

impl Display for PackageRelation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(version_requirement) = self.version_requirement.as_ref() {
            write!(f, "{}{}", self.name, version_requirement)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl FromStr for PackageRelation {
    type Err = Error;
    /// Parses a [`PackageRelation`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`PackageRelation`] can not be parsed from input.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{PackageRelation, VersionComparison, VersionRequirement};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     PackageRelation::from_str("example<1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Less,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example<=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::LessOrEqual,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Equal,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example>1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::Greater,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example>=1.0.0")?,
    ///     PackageRelation::new(
    ///         "example".parse()?,
    ///         Some(VersionRequirement {
    ///             comparison: VersionComparison::GreaterOrEqual,
    ///             version: "1.0.0".parse()?
    ///         })
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     PackageRelation::from_str("example")?,
    ///     PackageRelation::new("example".parse()?, None),
    /// );
    ///
    /// assert!(PackageRelation::from_str("example<").is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: The string splitting relies on the specific ordering of the VersionComparison
        // variants (which orders two-letter comparators over one-letter ones)!
        for comparison in VersionComparison::iter() {
            if let Some((name, version)) = s.split_once(comparison.as_ref()) {
                return Ok(Self {
                    name: Name::new(name)?,
                    version_requirement: Some(VersionRequirement {
                        comparison,
                        version: version.parse()?,
                    }),
                });
            }
        }

        Ok(Self {
            name: Name::new(s)?,
            version_requirement: None,
        })
    }
}

/// An optional dependency for a package.
///
/// This type is used for representing dependencies that are not essential for base functionality
/// of a package, but may be necessary to make use of certain features of a package.
///
/// An [`OptionalDependency`] consists of a package relation and an optional description separated
/// by a colon (`:`).
///
/// - The package relation component must be a valid [`PackageRelation`].
/// - If a description is provided it must be at least one character long.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Name, OptionalDependency};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create OptionalDependency from &str
/// let opt_depend = OptionalDependency::from_str("example: this is an example dependency")?;
///
/// // Get the name
/// assert_eq!("example", opt_depend.name().as_ref());
///
/// // Get the description
/// assert_eq!(
///     Some("this is an example dependency"),
///     opt_depend.description().as_deref()
/// );
///
/// // Format as String
/// assert_eq!(
///     "example: this is an example dependency",
///     format!("{opt_depend}")
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OptionalDependency {
    package_relation: PackageRelation,
    description: Option<String>,
}

impl OptionalDependency {
    /// Create a new OptionalDependency in a Result
    pub fn new(
        package_relation: PackageRelation,
        description: Option<String>,
    ) -> OptionalDependency {
        OptionalDependency {
            package_relation,
            description,
        }
    }

    /// Return the name of the optional dependency
    pub fn name(&self) -> &Name {
        &self.package_relation.name
    }

    /// Return the version requirement of the optional dependency
    pub fn version_requirement(&self) -> &Option<VersionRequirement> {
        &self.package_relation.version_requirement
    }

    /// Return the description for the optional dependency, if it exists
    pub fn description(&self) -> &Option<String> {
        &self.description
    }
}

impl FromStr for OptionalDependency {
    type Err = Error;

    /// Create an OptionalDependency from a string slice
    fn from_str(s: &str) -> Result<OptionalDependency, Self::Err> {
        if let Some((name, description)) = s.split_once(":") {
            let description = description.trim_start();
            let relation = PackageRelation::from_str(name)?;
            Ok(Self::new(
                relation,
                (!description.is_empty()).then_some(description.to_string()),
            ))
        } else {
            Ok(Self::new(PackageRelation::new(Name::new(s)?, None), None))
        }
    }
}

impl Display for OptionalDependency {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.description {
            Some(ref description) => write!(fmt, "{}: {}", self.name(), description),
            None => write!(fmt, "{}", self.name()),
        }
    }
}

/// Group of a package
///
/// Represents an arbitrary collection of packages that share a common
/// characteristic or functionality.
///
/// While group names can be any valid UTF-8 string, it is recommended to follow
/// the format of [`Name`] (`[a-z\d\-._@+]` but must not start with `[-.]`)
/// to ensure consistency and ease of use.
///
/// This is a type alias for [`String`].
///
/// ## Examples
/// ```
/// use alpm_types::Group;
///
/// // Create a Group
/// let group: Group = "package-group".to_string();
/// ```
pub type Group = String;

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest, test_runner::Config as ProptestConfig};
    use rstest::rstest;

    use super::*;

    const COMPARATOR_REGEX: &str = r"(<|<=|=|>=|>)";
    /// NOTE: [`Epoch`][alpm_types::Epoch] is implicitly constrained by [`std::usize::MAX`].
    /// However, it's unrealistic to ever reach that many forced downgrades for a package, hence
    /// we don't test that fully
    const EPOCH_REGEX: &str = r"[1-9]{1}[0-9]{0,10}";
    const NAME_REGEX: &str = r"[a-z0-9_@+]+[a-z0-9\-._@+]*";
    const PKGREL_REGEX: &str = r"[1-9]+[0-9]*(|[.]{1}[1-9]{1}[0-9]*)";
    const PKGVER_REGEX: &str = r"([[:alnum:]][[:alnum:]_+.]*)";
    const DESCRIPTION_REGEX: &str = r"[[:alnum:]][[:alnum:] _+.,-]*";

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]


        #[test]
        fn valid_package_relation_from_str(s in format!("{NAME_REGEX}(|{COMPARATOR_REGEX}(|{EPOCH_REGEX}:){PKGVER_REGEX}(|-{PKGREL_REGEX}))").as_str()) {
            println!("s: {s}");
            let name = PackageRelation::from_str(&s).unwrap();
            prop_assert_eq!(s, format!("{}", name));
        }
    }

    proptest! {
        #[test]
        fn opt_depend_from_str(s in format!("{NAME_REGEX}(: {DESCRIPTION_REGEX})?").as_str()) {
            println!("s: {s}");
            let opt_depend = OptionalDependency::from_str(&s).unwrap();
            let formatted = format!("{}", opt_depend);
            prop_assert_eq!(s.trim_end(), formatted.trim_end(), "Formatted output doesn't match input");
        }
    }

    #[rstest]
    #[case(
        "example: this is an example dependency",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("example").unwrap(),
                version_requirement: None,
            },
            description: Some("this is an example dependency".to_string()),
        }),
    )]
    #[case(
        "dep_name",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        }),
    )]
    #[case(
        "dep_name: ",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name").unwrap(),
                version_requirement: None,
            },
            description: None,
        }),
    )]
    #[case(
        "dep_name_with_special_chars-123: description with !@#$%^&*",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("dep_name_with_special_chars-123").unwrap(),
                version_requirement: None,
            },
            description: Some("description with !@#$%^&*".to_string()),
        }),
    )]
    #[case(
        "#invalid-name: this is an example dependency",
        Err(Error::RegexDoesNotMatch {
            value: "#invalid-name".to_string(),
            regex_type: "pkgname".to_string(),
            regex: crate::name::NAME_REGEX.to_string(),
        }),
    )]
    #[case(
        ": no_name_colon",
        Err(Error::RegexDoesNotMatch {
            value: "".to_string(),
            regex_type: "pkgname".to_string(),
            regex: crate::name::NAME_REGEX.to_string(),
        }),
    )]
    // versioned optional dependencies
    #[case(
        "elfutils=0.192: for translations",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("elfutils").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::Equal,
                    version: "0.192".parse().unwrap(),
                }),
            },
            description: Some("for translations".to_string()),
        }),
    )]
    #[case(
        "python>=3: For Python bindings",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("python").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "3".parse().unwrap(),
                }),
            },
            description: Some("For Python bindings".to_string()),
        }),
    )]
    #[case(
        "java-environment>=17: required by extension-wiki-publisher and extension-nlpsolver",
        Ok(OptionalDependency {
            package_relation: PackageRelation {
                name: Name::new("java-environment").unwrap(),
                version_requirement: Some(VersionRequirement {
                    comparison: VersionComparison::GreaterOrEqual,
                    version: "17".parse().unwrap(),
                }),
            },
            description: Some("required by extension-wiki-publisher and extension-nlpsolver".to_string()),
        }),
    )]
    fn opt_depend_from_string(
        #[case] input: &str,
        #[case] expected_result: Result<OptionalDependency, Error>,
    ) {
        let opt_depend_result = OptionalDependency::from_str(input);
        assert_eq!(expected_result, opt_depend_result);
    }

    #[rstest]
    #[case("example.so", SonameV1::Basic("example.so".parse().unwrap()))]
    #[case("example.so=1.0.0-64", SonameV1::Explicit {
        name: "example.so".parse().unwrap(),
        version: "1.0.0".parse().unwrap(),
        architecture: ElfArchitectureFormat::Bit64,
    })]
    fn sonamev1_from_string(
        #[case] input: &str,
        #[case] expected_result: SonameV1,
    ) -> testresult::TestResult<()> {
        let soname = SonameV1::from_str(input)?;
        assert_eq!(expected_result, soname);
        assert_eq!(input, soname.to_string());
        Ok(())
    }

    #[rstest]
    #[case(
        "libwlroots-0.18.so=libwlroots-0.18.so-64",
        SonameV1::Unversioned {
            name: "libwlroots-0.18.so".parse().unwrap(),
            soname: "libwlroots-0.18.so".parse().unwrap(),
            architecture: ElfArchitectureFormat::Bit64,
        },
    )]
    #[case(
        "libexample.so=otherlibexample.so-64",
        SonameV1::Unversioned {
            name: "libexample.so".parse().unwrap(),
            soname: "otherlibexample.so".parse().unwrap(),
            architecture: ElfArchitectureFormat::Bit64,
        },
    )]
    fn sonamev1_from_string_without_version(
        #[case] input: &str,
        #[case] expected_result: SonameV1,
    ) -> testresult::TestResult<()> {
        let soname = SonameV1::from_str(input)?;
        assert_eq!(expected_result, soname);
        assert_eq!(input, soname.to_string());
        Ok(())
    }

    #[rstest]
    #[case("noso", "invalid shared object name")]
    #[case("invalidversion.so=1*2-64", "expected version or shared object name")]
    #[case(
        "nodelimiter.so=1.64",
        "expected a version or shared object name, followed by an ELF architecture format"
    )]
    #[case(
        "noarchitecture.so=1-",
        "expected a version or shared object name, followed by an ELF architecture format"
    )]
    #[case("invalidarchitecture.so=1-82", "invalid architecture")]
    #[case("invalidsoname.so~1.64", "unexpected trailing content")]
    fn invalid_sonamev1_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SonameV1::from_str(input);
        assert!(result.is_err(), "Expected SonameV1 parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }

    #[rstest]
    #[case(
        "otherlibexample.so",
        VersionOrSoname::Soname(
            SharedObjectName::new("otherlibexample.so").unwrap())
    )]
    #[case(
        "1.0.0",
        VersionOrSoname::Version(
            PackageVersion::from_str("1.0.0").unwrap())
    )]
    fn version_or_soname_from_string(
        #[case] input: &str,
        #[case] expected_result: VersionOrSoname,
    ) -> testresult::TestResult<()> {
        let version = VersionOrSoname::from_str(input)?;
        assert_eq!(expected_result, version);
        assert_eq!(input, version.to_string());
        Ok(())
    }

    #[rstest]
    #[case(
        "lib:libexample.so",
        SonameV2 {
            prefix: "lib".parse().unwrap(),
            soname: Soname {
                name: "libexample.so".parse().unwrap(),
                version: None,
            },
        },
    )]
    #[case(
        "usr:libexample.so.1",
        SonameV2 {
            prefix: "usr".parse().unwrap(),
            soname: Soname {
                name: "libexample.so".parse().unwrap(),
                version: "1".parse().ok(),
            },
        },
    )]
    #[case(
        "lib:libexample.so.1.2.3",
        SonameV2 {
            prefix: "lib".parse().unwrap(),
            soname: Soname {
                name: "libexample.so".parse().unwrap(),
                version: "1.2.3".parse().ok(),
            },
        },
    )]
    #[case(
        "lib:libexample.so.so.420",
        SonameV2 {
            prefix: "lib".parse().unwrap(),
            soname: Soname {
                name: "libexample.so.so".parse().unwrap(),
                version: "420".parse().ok(),
            },
        },
    )]
    #[case(
        "lib:libexample.so.test",
        SonameV2 {
            prefix: "lib".parse().unwrap(),
            soname: Soname {
                name: "libexample.so".parse().unwrap(),
                version: "test".parse().ok(),
            },
        },
    )]
    fn sonamev2_from_string(
        #[case] input: &str,
        #[case] expected_result: SonameV2,
    ) -> testresult::TestResult<()> {
        let soname = SonameV2::from_str(input)?;
        assert_eq!(expected_result, soname);
        assert_eq!(input, soname.to_string());
        Ok(())
    }

    #[rstest]
    #[case("libexample.so.1", "invalid shared library prefix delimiter")]
    #[case("lib:libexample.so-abc", "invalid version delimiter")]
    #[case(
        "lib:libexample.so.10-10",
        "Value '10-10' does not match the 'pkgver' regex"
    )]
    #[case(
        "lib:libexample.so.1.0.0-64",
        "Value '1.0.0-64' does not match the 'pkgver' regex"
    )]
    fn invalid_sonamev2_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SonameV2::from_str(input);
        assert!(result.is_err(), "Expected SonameV2 parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
