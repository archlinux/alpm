use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_parsers::traits::{AlpmParser, ParserUntil};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    ascii::Caseless,
    combinator::{alt, cut_err, eof, not, repeat},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::one_of,
};

use crate::Error;

/// Specific CPU architecture
///
/// Can be either a known variant or an unknown architecture represented as
/// a case-insensitive string, that:
///
/// - consists only of ASCII alphanumeric characters and underscores
/// - is not "any"
///
/// Members of the [`SystemArchitecture`] enum can be created from `&str`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{SystemArchitecture, UnknownArchitecture};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create SystemArchitecture from str
/// assert_eq!(
///     SystemArchitecture::from_str("aarch64"),
///     Ok(SystemArchitecture::Aarch64)
/// );
///
/// // Format as String
/// assert_eq!("x86_64", format!("{}", SystemArchitecture::X86_64));
/// # Ok(())
/// # }
/// ```
#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    VariantNames,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SystemArchitecture {
    /// ARMv8 64-bit
    Aarch64,
    /// ARM
    Arm,
    /// ARMv6 hard-float
    Armv6h,
    /// ARMv7 hard-float
    Armv7h,
    /// Intel 386
    I386,
    /// Intel 486
    I486,
    /// Intel 686
    I686,
    /// Intel Pentium 4
    Pentium4,
    /// RISC-V 32-bit
    Riscv32,
    /// RISC-V 64-bit
    Riscv64,
    /// Intel x86_64
    X86_64,
    /// Intel x86_64 version 2
    #[strum(to_string = "x86_64_v2")]
    X86_64V2,
    /// Intel x86_64 version 3
    #[strum(to_string = "x86_64_v3")]
    X86_64V3,
    /// Intel x86_64 version 4
    #[strum(to_string = "x86_64_v4")]
    X86_64V4,
    /// Unknown architecture
    #[strum(transparent)]
    #[serde(untagged)]
    Unknown(UnknownArchitecture),
}

impl AlpmParser for SystemArchitecture {
    /// Recognizes a [`SystemArchitecture`] in an input string.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not begin with a valid `SystemArchitecture`.
    fn parser(input: &mut &str) -> ModalResult<SystemArchitecture> {
        // Make sure we don't have an `any`.
        cut_err(not((Caseless("any"), eof)))
            .context(StrContext::Label(
                "system architecture. 'any' has a special meaning and is not allowed here.",
            ))
            .parse_next(input)?;

        let alphanum = |c: char| c.is_ascii_alphanumeric();
        let special_chars = ['_'];

        // We consume as many valid characters as we can until we hit an unknown char or `eof`.
        // E.g.
        // `asdfasdf_x86_64_omega:test` -> `:test`
        let architecture: String = cut_err(repeat(1.., one_of((alphanum, special_chars))))
            .context(StrContext::Label("character in system architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a string containing only ASCII alphanumeric characters and underscores.",
            )))
            .parse_next(input)?;

        // We now take that valid architecture and check it against all known static variants in our
        // SystemArchitecture enum.
        // If none of those match, return it as an SystemArchitecture::Unknown.
        let architecture = match architecture.as_str() {
            // Handle all static variants
            "aarch64" => SystemArchitecture::Aarch64,
            "armv6h" => SystemArchitecture::Armv6h,
            "armv7h" => SystemArchitecture::Armv7h,
            "arm" => SystemArchitecture::Arm,
            "i386" => SystemArchitecture::I386,
            "i486" => SystemArchitecture::I486,
            "i686" => SystemArchitecture::I686,
            "pentium4" => SystemArchitecture::Pentium4,
            "riscv32" => SystemArchitecture::Riscv32,
            "riscv64" => SystemArchitecture::Riscv64,
            "x86_64_v2" => SystemArchitecture::X86_64V2,
            "x86_64_v3" => SystemArchitecture::X86_64V3,
            "x86_64_v4" => SystemArchitecture::X86_64V4,
            "x86_64" => SystemArchitecture::X86_64,
            // Generic fallback handler.
            other => SystemArchitecture::Unknown(UnknownArchitecture::new(other)),
        };

        Ok(architecture)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<&'a str, O, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>>,
    {
        parser
            .context(StrContext::Label("character in system architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a string containing only ASCII alphanumeric characters and underscores.",
            )))
    }
}

impl FromStr for SystemArchitecture {
    type Err = Error;

    /// Creates a [`SystemArchitecture`] from a string slice.
    ///
    /// Delegates to [`SystemArchitecture::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`SystemArchitecture::parser`] fails.
    fn from_str(s: &str) -> Result<SystemArchitecture, Self::Err> {
        Ok(Self::parser_until_eof.parse(s)?)
    }
}

/// An unknown architecture that is a valid [alpm-architecture].
///
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UnknownArchitecture(String);

impl UnknownArchitecture {
    /// Create a new `UnknownArchitecture` by name.
    ///
    /// This is not publicly exposed, to ensure that this type can only be created by using
    /// [`SystemArchitecture`]'s constructors.
    ///
    /// That way, we can uphold the invariant that `UnknownArchitecture` will never contain a valid
    /// and known `SystemArchitecture` variant, as any values **must** pass through
    /// `SystemArchitecture`'s parser.
    pub(crate) fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl From<UnknownArchitecture> for SystemArchitecture {
    /// Converts an [`UnknownArchitecture`] into a [`SystemArchitecture`].
    fn from(value: UnknownArchitecture) -> Self {
        SystemArchitecture::Unknown(value)
    }
}

impl From<UnknownArchitecture> for Architecture {
    /// Converts an [`UnknownArchitecture`] into an [`Architecture`].
    fn from(value: UnknownArchitecture) -> Self {
        Architecture::Some(value.into())
    }
}

impl Display for UnknownArchitecture {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

impl AsRef<str> for UnknownArchitecture {
    fn as_ref(&self) -> &str {
        self.inner()
    }
}

/// A valid [alpm-architecture], either "any" or a specific [`SystemArchitecture`].
///
/// Members of the [`Architecture`] enum can be created from `&str`.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Architecture, SystemArchitecture, UnknownArchitecture};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create Architecture from str
/// assert_eq!(
///     Architecture::from_str("aarch64"),
///     Ok(SystemArchitecture::Aarch64.into())
/// );
/// assert_eq!(Architecture::from_str("any"), Ok(Architecture::Any));
///
/// // format as String
/// assert_eq!("any", format!("{}", Architecture::Any));
/// assert_eq!(
///     "x86_64",
///     format!("{}", Architecture::Some(SystemArchitecture::X86_64))
/// );
/// assert_eq!(
///     "custom_arch",
///     format!("{}", Architecture::from_str("custom_arch")?)
/// );
/// # Ok(())
/// # }
/// ```
///
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    VariantNames,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    /// Any architecture
    Any,
    /// Specific architecture
    #[strum(transparent)]
    #[serde(untagged)]
    Some(SystemArchitecture),
}

impl AlpmParser for Architecture {
    /// Recognizes an [`Architecture`] in an input string.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not begin with a valid Architecture.
    fn parser(input: &mut &str) -> ModalResult<Architecture> {
        alt((
            Caseless("any").value(Architecture::Any),
            SystemArchitecture::parser.map(Architecture::Some),
        ))
        .context(StrContext::Label("alpm-architecture"))
        .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<&'a str, O, ErrMode<ContextError>>
    where
        P: Parser<&'a str, O, ErrMode<ContextError>>,
    {
        parser
            .context(StrContext::Label("character in architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a string containing only ASCII alphanumeric characters and underscores.",
            )))
    }
}

impl FromStr for Architecture {
    type Err = Error;

    /// Creates an [`Architecture`] from a string slice.
    ///
    /// Delegates to [`Architecture::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Architecture::parser`] fails.
    fn from_str(s: &str) -> Result<Architecture, Self::Err> {
        Ok(Self::parser_until_eof.parse(s)?)
    }
}

impl From<SystemArchitecture> for Architecture {
    /// Converts a [`SystemArchitecture`] into an [`Architecture`].
    fn from(value: SystemArchitecture) -> Self {
        Architecture::Some(value)
    }
}

/// Represents multiple valid [alpm-architecture]s.
///
/// Can be either "any" or multiple specific [`SystemArchitecture`]s.
///
/// [`Architectures`] enum can be created from a vector of [`Architecture`]s using a [`TryFrom`]
/// implementation.
///
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
#[derive(
    Clone,
    Debug,
    Deserialize,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    VariantNames,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Architectures {
    /// Any architecture
    Any,
    /// Specific architectures
    #[strum(transparent)]
    #[serde(untagged)]
    Some(Vec<SystemArchitecture>),
}

impl Architectures {
    /// Returns the number of entries in the architectures list.
    pub fn len(&self) -> usize {
        match self {
            Architectures::Any => 1,
            Architectures::Some(archs) => archs.len(),
        }
    }

    /// Returns `true` if there are no entries in the architectures list.
    pub fn is_empty(&self) -> bool {
        match self {
            Architectures::Any => false,
            Architectures::Some(archs) => archs.is_empty(),
        }
    }
}

impl Display for Architectures {
    /// Formats the [`Architectures`] as a comma-separated string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architectures::Any => {
                write!(f, "any")
            }
            Architectures::Some(archs) => {
                write!(
                    f,
                    "{}",
                    archs
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

impl From<Architecture> for Architectures {
    /// Converts a single [`Architecture`] into an [`Architectures`].
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::Any => Architectures::Any,
            Architecture::Some(arch) => Architectures::Some(vec![arch]),
        }
    }
}

impl From<&Architectures> for Vec<Architecture> {
    /// Converts an [`Architectures`] into a vector of [`Architecture`]s.
    fn from(value: &Architectures) -> Self {
        match value {
            Architectures::Any => vec![Architecture::Any],
            Architectures::Some(archs) => {
                archs.clone().into_iter().map(Architecture::Some).collect()
            }
        }
    }
}

impl IntoIterator for &Architectures {
    type Item = Architecture;
    type IntoIter = std::vec::IntoIter<Architecture>;
    /// Creates an iterator over [`Architecture`]s.
    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<Architecture> = self.into();
        vec.into_iter()
    }
}

impl TryFrom<Vec<&Architecture>> for Architectures {
    type Error = Error;

    /// Tries to convert a vector of [`Architecture`] into an [`Architectures`].
    ///
    /// # Errors
    ///
    /// The conversion fails if the input vector contains [`Architecture::Any`] along with other
    /// architectures.
    fn try_from(value: Vec<&Architecture>) -> Result<Self, Self::Error> {
        if value.contains(&&Architecture::Any) {
            if value.len() > 1 {
                Err(Error::InvalidArchitectures {
                    architectures: value.iter().map(|&v| v.clone()).collect(),
                    context: "'any' cannot be used in combination with other architectures.",
                })
            } else {
                Ok(Architectures::Any)
            }
        } else {
            let archs: Vec<SystemArchitecture> = value
                .into_iter()
                .map(|arch| {
                    if let Architecture::Some(specific) = arch {
                        specific.clone()
                    } else {
                        // This case is already handled above
                        unreachable!()
                    }
                })
                .collect();
            Ok(Architectures::Some(archs))
        }
    }
}

impl TryFrom<Vec<Architecture>> for Architectures {
    type Error = Error;

    /// Tries to convert a vector of [`Architecture`] into an [`Architectures`].
    ///
    /// Delegates to the [`TryFrom`] implementation for `Vec<&Architecture>`.
    fn try_from(value: Vec<Architecture>) -> Result<Self, Self::Error> {
        value.iter().collect::<Vec<&Architecture>>().try_into()
    }
}

/// ELF architecture format.
///
/// This enum represents the _Class_ field in the [_ELF Header_].
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::ElfArchitectureFormat;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create ElfArchitectureFormat from str
/// assert_eq!(
///     ElfArchitectureFormat::from_str("32"),
///     Ok(ElfArchitectureFormat::Bit32)
/// );
/// assert_eq!(
///     ElfArchitectureFormat::from_str("64"),
///     Ok(ElfArchitectureFormat::Bit64)
/// );
///
/// // format as String
/// assert_eq!("32", format!("{}", ElfArchitectureFormat::Bit32));
/// assert_eq!("64", format!("{}", ElfArchitectureFormat::Bit64));
/// # Ok(())
/// # }
/// ```
///
/// [_ELF Header_]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#ELF_header
#[derive(
    Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
#[strum(serialize_all = "lowercase")]
pub enum ElfArchitectureFormat {
    /// 32-bit
    #[strum(to_string = "32")]
    Bit32 = 32,
    /// 64-bit
    #[strum(to_string = "64")]
    Bit64 = 64,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use insta::assert_snapshot;
    use rstest::rstest;
    use strum::ParseError;

    use super::*;
    use crate::configure_insta;

    #[rstest]
    #[case("aarch64", SystemArchitecture::Aarch64)]
    #[case("f_oo", UnknownArchitecture::new("f_oo").into())]
    fn system_architecture_from_string(#[case] s: &str, #[case] arch: SystemArchitecture) {
        assert_eq!(SystemArchitecture::from_str(s), Ok(arch));
    }

    #[rstest]
    #[case("f oo")]
    #[case("any")]
    fn invalid_system_architecture_from_string(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = SystemArchitecture::from_str(input) else {
            panic!("'{input}' erroneously parsed as a SystemArchitecture")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }

    #[rstest]
    #[case(SystemArchitecture::Aarch64, "aarch64")]
    #[case(SystemArchitecture::from_str("f_o_o").unwrap(), "f_o_o")]
    fn system_architecture_format_string(#[case] arch: SystemArchitecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{arch}"));
    }

    #[rstest]
    #[case("any", Architecture::Any)]
    #[case("aarch64", SystemArchitecture::Aarch64.into())]
    #[case("arm", SystemArchitecture::Arm.into())]
    #[case("armv6h", SystemArchitecture::Armv6h.into())]
    #[case("armv7h", SystemArchitecture::Armv7h.into())]
    #[case("i386", SystemArchitecture::I386.into())]
    #[case("i486", SystemArchitecture::I486.into())]
    #[case("i686", SystemArchitecture::I686.into())]
    #[case("pentium4", SystemArchitecture::Pentium4.into())]
    #[case("riscv32", SystemArchitecture::Riscv32.into())]
    #[case("riscv64", SystemArchitecture::Riscv64.into())]
    #[case("x86_64", SystemArchitecture::X86_64.into())]
    #[case("x86_64_v2", SystemArchitecture::X86_64V2.into())]
    #[case("x86_64_v3", SystemArchitecture::X86_64V3.into())]
    #[case("x86_64_v4", SystemArchitecture::X86_64V4.into())]
    #[case("foo", UnknownArchitecture::new("foo").into())]
    #[case("f_oo", UnknownArchitecture::new("f_oo").into())]
    fn architecture_from_string(#[case] input: &str, #[case] arch: Architecture) {
        assert_eq!(Architecture::from_str(input), Ok(arch));
    }

    #[rstest]
    #[case("f oo")]
    fn invalid_architecture_from_string(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = Architecture::from_str(input) else {
            panic!("'{input}' erroneously parsed as a Architecture")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }

    #[rstest]
    #[case(Architecture::Any, "any")]
    #[case(SystemArchitecture::Aarch64.into(), "aarch64")]
    #[case(Architecture::from_str("foo").unwrap(), "foo")]
    fn architecture_format_string(#[case] arch: Architecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{arch}"));
    }

    #[rstest]
    #[case(vec![Architecture::Any], Ok(Architectures::Any))]
    #[case(
        vec![SystemArchitecture::Aarch64.into()],
        Ok(Architectures::Some(vec![SystemArchitecture::Aarch64]))
    )]
    #[case(
        vec![SystemArchitecture::Arm.into(), SystemArchitecture::I386.into()],
        Ok(Architectures::Some(vec![SystemArchitecture::Arm, SystemArchitecture::I386]))
    )]
    // Duplicates are allowed (discouraged by linter)
    #[case(
        vec![SystemArchitecture::Arm.into(), SystemArchitecture::Arm.into()],
        Ok(Architectures::Some(vec![SystemArchitecture::Arm, SystemArchitecture::Arm]))
    )]
    #[case(
        vec![Architecture::Any, SystemArchitecture::I386.into()],
        Err(Error::InvalidArchitectures {
            architectures: vec![Architecture::Any, SystemArchitecture::I386.into()],
            context: "'any' cannot be used in combination with other architectures.",
        })
    )]
    #[case(vec![Architecture::Any, Architecture::Any], Err(Error::InvalidArchitectures {
        architectures: vec![Architecture::Any, Architecture::Any],
        context: "'any' cannot be used in combination with other architectures.",
    }))]
    #[case(vec![], Ok(Architectures::Some(vec![])))]
    fn architectures_from_vec(
        #[case] archs: Vec<Architecture>,
        #[case] expected: Result<Architectures, Error>,
    ) {
        assert_eq!(archs.try_into(), expected);
    }

    #[rstest]
    #[case(Architectures::Any, "any")]
    #[case(Architectures::Some(vec![SystemArchitecture::Aarch64]), "aarch64")]
    #[case(Architectures::Some(vec![SystemArchitecture::Arm, SystemArchitecture::I386]), "arm, i386")]
    #[case(Architectures::Some(vec![]), "")]
    fn architectures_format_display(#[case] archs: Architectures, #[case] archs_str: &str) {
        assert_eq!(archs_str, format!("{archs}"));
    }

    #[rstest]
    #[case("32", Ok(ElfArchitectureFormat::Bit32))]
    #[case("64", Ok(ElfArchitectureFormat::Bit64))]
    #[case("foo", Err(ParseError::VariantNotFound))]
    fn elf_architecture_format_from_string(
        #[case] s: &str,
        #[case] arch: Result<ElfArchitectureFormat, ParseError>,
    ) {
        assert_eq!(ElfArchitectureFormat::from_str(s), arch);
    }

    #[rstest]
    #[case(ElfArchitectureFormat::Bit32, "32")]
    #[case(ElfArchitectureFormat::Bit64, "64")]
    fn elf_architecture_format_display(
        #[case] arch: ElfArchitectureFormat,
        #[case] arch_str: &str,
    ) {
        assert_eq!(arch_str, format!("{arch}"));
    }
}
