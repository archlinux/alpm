use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    combinator::cut_err,
    error::{StrContext, StrContextValue},
    token::rest,
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
/// use alpm_types::SystemArchitecture;
///
/// // create SystemArchitecture from str
/// assert_eq!(
///     SystemArchitecture::from_str("aarch64"),
///     Ok(SystemArchitecture::Aarch64)
/// );
///
/// // format as String
/// assert_eq!("x86_64", format!("{}", SystemArchitecture::X86_64));
/// assert_eq!(
///     "custom_arch",
///     format!("{}", SystemArchitecture::Unknown("custom_arch".to_string()))
/// );
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
    Unknown(String),
}

impl SystemArchitecture {
    /// Recognizes a [`SystemArchitecture`] in an input string.
    ///
    /// Consumes all input and returns an error if the string doesn't match any architecture.
    pub fn parser(input: &mut &str) -> ModalResult<SystemArchitecture> {
        // we forbid "any" as it is handled by Architecture::Any
        cut_err(rest.try_map(SystemArchitecture::from_str))
            .context(StrContext::Label("specific system architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a specific system architecture consisting of ASCII alphanumeric characters and underscores, cannot be 'any'",
            )))
            .parse_next(input)
    }
}

impl FromStr for SystemArchitecture {
    type Err = Error;

    /// Creates a [`SystemArchitecture`] from a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the input string:
    ///
    /// - is empty.
    /// - contains invalid characters (non-ASCII alphanumeric and non-underscore).
    /// - is "any" (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::ValueDoesNotMatchRestrictions {
                restrictions: vec!["length greater than 0".to_string()],
            });
        } else if let Some(invalid_char) =
            s.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '_')
        {
            return Err(Error::ValueContainsInvalidChars { invalid_char });
        } else if s.eq_ignore_ascii_case("any") {
            return Err(Error::AnyAsSystemArchitecture);
        }
        match s.to_lowercase().as_str() {
            "aarch64" => Ok(SystemArchitecture::Aarch64),
            "arm" => Ok(SystemArchitecture::Arm),
            "armv6h" => Ok(SystemArchitecture::Armv6h),
            "armv7h" => Ok(SystemArchitecture::Armv7h),
            "i386" => Ok(SystemArchitecture::I386),
            "i486" => Ok(SystemArchitecture::I486),
            "i686" => Ok(SystemArchitecture::I686),
            "pentium4" => Ok(SystemArchitecture::Pentium4),
            "riscv32" => Ok(SystemArchitecture::Riscv32),
            "riscv64" => Ok(SystemArchitecture::Riscv64),
            "x86_64" => Ok(SystemArchitecture::X86_64),
            "x86_64_v2" => Ok(SystemArchitecture::X86_64V2),
            "x86_64_v3" => Ok(SystemArchitecture::X86_64V3),
            "x86_64_v4" => Ok(SystemArchitecture::X86_64V4),
            other => Ok(SystemArchitecture::Unknown(other.to_string())),
        }
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
/// use alpm_types::{Architecture, SystemArchitecture};
///
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
///     format!(
///         "{}",
///         Architecture::Some(SystemArchitecture::Unknown("custom_arch".to_string()))
///     )
/// );
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

impl Architecture {
    /// Recognizes an [`Architecture`] in an input string.
    ///
    /// Consumes all input and returns an error if the string doesn't match any architecture.
    pub fn parser(input: &mut &str) -> ModalResult<Architecture> {
        cut_err(rest.try_map(Architecture::from_str))
            .context(StrContext::Label("architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "either 'any' or a specific system architecture that consists of ASCII alphanumeric characters and underscores",
            )))
            .parse_next(input)
    }
}

impl FromStr for Architecture {
    type Err = Error;

    /// Creates an [`Architecture`] from a string slice.
    ///
    /// Parses "any" (case-insensitive) as [`Architecture::Any`], otherwise delegates to
    /// [`SystemArchitecture::from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if the input string:
    ///
    /// - is empty
    /// - contains invalid characters (non-ASCII alphanumeric and non-underscore)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("any") {
            Ok(Architecture::Any)
        } else {
            SystemArchitecture::from_str(s).map(Architecture::Some)
        }
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
    /// Returns the number of entries when converted into iterator.
    pub fn len(&self) -> usize {
        match self {
            Architectures::Any => 1,
            Architectures::Some(archs) => archs.len(),
        }
    }

    /// Returns `true` if [`IntoIterator::into_iter`] results in an empty iterator.
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

    use rstest::rstest;
    use strum::ParseError;
    use winnow::error::ContextError;

    use super::*;

    #[rstest]
    #[case(SystemArchitecture::Aarch64.into(), Architecture::Some(SystemArchitecture::Aarch64))]
    #[case(SystemArchitecture::Arm.into(), Architecture::Some(SystemArchitecture::Arm))]
    #[case(SystemArchitecture::Armv6h.into(), Architecture::Some(SystemArchitecture::Armv6h))]
    #[case(SystemArchitecture::Armv7h.into(), Architecture::Some(SystemArchitecture::Armv7h))]
    #[case(SystemArchitecture::I386.into(), Architecture::Some(SystemArchitecture::I386))]
    #[case(SystemArchitecture::I486.into(), Architecture::Some(SystemArchitecture::I486))]
    #[case(SystemArchitecture::I686.into(), Architecture::Some(SystemArchitecture::I686))]
    #[case(SystemArchitecture::Pentium4.into(), Architecture::Some(SystemArchitecture::Pentium4))]
    #[case(SystemArchitecture::Riscv32.into(), Architecture::Some(SystemArchitecture::Riscv32))]
    #[case(SystemArchitecture::Riscv64.into(), Architecture::Some(SystemArchitecture::Riscv64))]
    #[case(SystemArchitecture::X86_64.into(), Architecture::Some(SystemArchitecture::X86_64))]
    #[case(SystemArchitecture::X86_64V2.into(), Architecture::Some(SystemArchitecture::X86_64V2))]
    #[case(SystemArchitecture::X86_64V3.into(), Architecture::Some(SystemArchitecture::X86_64V3))]
    #[case(SystemArchitecture::X86_64V4.into(), Architecture::Some(SystemArchitecture::X86_64V4))]
    #[case(SystemArchitecture::Unknown("foo".to_string()).into(), Architecture::Some(SystemArchitecture::Unknown("foo".to_string())))]
    #[case(SystemArchitecture::Unknown("f_oo".to_string()).into(), Architecture::Some(SystemArchitecture::Unknown("f_oo".to_string())))]
    fn system_architecture_into_architecture(
        #[case] left: Architecture,
        #[case] right: Architecture,
    ) {
        assert_eq!(left, right);
    }

    #[rstest]
    #[case("aarch64", Ok(SystemArchitecture::Aarch64))]
    #[case("arm", Ok(SystemArchitecture::Arm))]
    #[case("armv6h", Ok(SystemArchitecture::Armv6h))]
    #[case("armv7h", Ok(SystemArchitecture::Armv7h))]
    #[case("i386", Ok(SystemArchitecture::I386))]
    #[case("i486", Ok(SystemArchitecture::I486))]
    #[case("i686", Ok(SystemArchitecture::I686))]
    #[case("pentium4", Ok(SystemArchitecture::Pentium4))]
    #[case("riscv32", Ok(SystemArchitecture::Riscv32))]
    #[case("riscv64", Ok(SystemArchitecture::Riscv64))]
    #[case("x86_64", Ok(SystemArchitecture::X86_64))]
    #[case("x86_64_v2", Ok(SystemArchitecture::X86_64V2))]
    #[case("x86_64_v3", Ok(SystemArchitecture::X86_64V3))]
    #[case("x86_64_v4", Ok(SystemArchitecture::X86_64V4))]
    #[case("foo", Ok(SystemArchitecture::Unknown("foo".to_string())))]
    #[case("f_oo", Ok(SystemArchitecture::Unknown("f_oo".to_string())))]
    #[case("f oo", Err(Error::ValueContainsInvalidChars { invalid_char: ' ' }))]
    #[case("any", Err(Error::AnyAsSystemArchitecture))]
    fn system_architecture_from_string(
        #[case] s: &str,
        #[case] arch: Result<SystemArchitecture, Error>,
    ) {
        assert_eq!(SystemArchitecture::from_str(s), arch);
    }

    #[rstest]
    #[case("aarch64", Ok(SystemArchitecture::Aarch64))]
    #[case("arm", Ok(SystemArchitecture::Arm))]
    #[case("armv6h", Ok(SystemArchitecture::Armv6h))]
    #[case("armv7h", Ok(SystemArchitecture::Armv7h))]
    #[case("i386", Ok(SystemArchitecture::I386))]
    #[case("i486", Ok(SystemArchitecture::I486))]
    #[case("i686", Ok(SystemArchitecture::I686))]
    #[case("pentium4", Ok(SystemArchitecture::Pentium4))]
    #[case("riscv32", Ok(SystemArchitecture::Riscv32))]
    #[case("riscv64", Ok(SystemArchitecture::Riscv64))]
    #[case("x86_64", Ok(SystemArchitecture::X86_64))]
    #[case("x86_64_v2", Ok(SystemArchitecture::X86_64V2))]
    #[case("x86_64_v3", Ok(SystemArchitecture::X86_64V3))]
    #[case("x86_64_v4", Ok(SystemArchitecture::X86_64V4))]
    #[case("foo", Ok(SystemArchitecture::Unknown("foo".to_string())))]
    #[case("_f_oo", Ok(SystemArchitecture::Unknown("_f_oo".to_string())))]
    fn system_architecture_parser(
        #[case] s: &str,
        #[case] arch: Result<SystemArchitecture, winnow::error::ParseError<&str, ContextError>>,
    ) {
        assert_eq!(SystemArchitecture::parser.parse(s), arch);
    }

    #[rstest]
    #[case(SystemArchitecture::Aarch64, "aarch64")]
    #[case(SystemArchitecture::Arm, "arm")]
    #[case(SystemArchitecture::Armv6h, "armv6h")]
    #[case(SystemArchitecture::Armv7h, "armv7h")]
    #[case(SystemArchitecture::I386, "i386")]
    #[case(SystemArchitecture::I486, "i486")]
    #[case(SystemArchitecture::I686, "i686")]
    #[case(SystemArchitecture::Pentium4, "pentium4")]
    #[case(SystemArchitecture::Riscv32, "riscv32")]
    #[case(SystemArchitecture::Riscv64, "riscv64")]
    #[case(SystemArchitecture::X86_64, "x86_64")]
    #[case(SystemArchitecture::X86_64V2, "x86_64_v2")]
    #[case(SystemArchitecture::X86_64V3, "x86_64_v3")]
    #[case(SystemArchitecture::X86_64V4, "x86_64_v4")]
    #[case(SystemArchitecture::Unknown("f_o_o".to_string()), "f_o_o")]
    fn system_architecture_format_string(#[case] arch: SystemArchitecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{arch}"));
    }

    #[rstest]
    #[case("any", Ok(Architecture::Any))]
    #[case("aarch64", Ok(SystemArchitecture::Aarch64.into()))]
    #[case("arm", Ok(SystemArchitecture::Arm.into()))]
    #[case("armv6h", Ok(SystemArchitecture::Armv6h.into()))]
    #[case("armv7h", Ok(SystemArchitecture::Armv7h.into()))]
    #[case("i386", Ok(SystemArchitecture::I386.into()))]
    #[case("i486", Ok(SystemArchitecture::I486.into()))]
    #[case("i686", Ok(SystemArchitecture::I686.into()))]
    #[case("pentium4", Ok(SystemArchitecture::Pentium4.into()))]
    #[case("riscv32", Ok(SystemArchitecture::Riscv32.into()))]
    #[case("riscv64", Ok(SystemArchitecture::Riscv64.into()))]
    #[case("x86_64", Ok(SystemArchitecture::X86_64.into()))]
    #[case("x86_64_v2", Ok(SystemArchitecture::X86_64V2.into()))]
    #[case("x86_64_v3", Ok(SystemArchitecture::X86_64V3.into()))]
    #[case("x86_64_v4", Ok(SystemArchitecture::X86_64V4.into()))]
    #[case("foo", Ok(SystemArchitecture::Unknown("foo".to_string()).into()))]
    #[case("f_oo", Ok(SystemArchitecture::Unknown("f_oo".to_string()).into()))]
    #[case("f oo", Err(Error::ValueContainsInvalidChars { invalid_char: ' ' }))]
    fn architecture_from_string(#[case] s: &str, #[case] arch: Result<Architecture, Error>) {
        assert_eq!(Architecture::from_str(s), arch);
    }

    #[rstest]
    #[case("any", Ok(Architecture::Any))]
    #[case("aarch64", Ok(SystemArchitecture::Aarch64.into()))]
    #[case("arm", Ok(SystemArchitecture::Arm.into()))]
    #[case("armv6h", Ok(SystemArchitecture::Armv6h.into()))]
    #[case("armv7h", Ok(SystemArchitecture::Armv7h.into()))]
    #[case("i386", Ok(SystemArchitecture::I386.into()))]
    #[case("i486", Ok(SystemArchitecture::I486.into()))]
    #[case("i686", Ok(SystemArchitecture::I686.into()))]
    #[case("pentium4", Ok(SystemArchitecture::Pentium4.into()))]
    #[case("riscv32", Ok(SystemArchitecture::Riscv32.into()))]
    #[case("riscv64", Ok(SystemArchitecture::Riscv64.into()))]
    #[case("x86_64", Ok(SystemArchitecture::X86_64.into()))]
    #[case("x86_64_v2", Ok(SystemArchitecture::X86_64V2.into()))]
    #[case("x86_64_v3", Ok(SystemArchitecture::X86_64V3.into()))]
    #[case("x86_64_v4", Ok(SystemArchitecture::X86_64V4.into()))]
    #[case("foo", Ok(SystemArchitecture::Unknown("foo".to_string()).into()))]
    #[case("_f_oo", Ok(SystemArchitecture::Unknown("_f_oo".to_string()).into()))]
    fn architecture_parser(
        #[case] s: &str,
        #[case] arch: Result<Architecture, winnow::error::ParseError<&str, ContextError>>,
    ) {
        assert_eq!(Architecture::parser.parse(s), arch);
    }

    #[rstest]
    #[case(Architecture::Any, "any")]
    #[case(SystemArchitecture::Aarch64.into(), "aarch64")]
    #[case(SystemArchitecture::Arm.into(), "arm")]
    #[case(SystemArchitecture::Armv6h.into(), "armv6h")]
    #[case(SystemArchitecture::Armv7h.into(), "armv7h")]
    #[case(SystemArchitecture::I386.into(), "i386")]
    #[case(SystemArchitecture::I486.into(), "i486")]
    #[case(SystemArchitecture::I686.into(), "i686")]
    #[case(SystemArchitecture::Pentium4.into(), "pentium4")]
    #[case(SystemArchitecture::Riscv32.into(), "riscv32")]
    #[case(SystemArchitecture::Riscv64.into(), "riscv64")]
    #[case(SystemArchitecture::X86_64.into(), "x86_64")]
    #[case(SystemArchitecture::X86_64V2.into(), "x86_64_v2")]
    #[case(SystemArchitecture::X86_64V3.into(), "x86_64_v3")]
    #[case(SystemArchitecture::X86_64V4.into(), "x86_64_v4")]
    #[case(SystemArchitecture::Unknown("foo".to_string()).into(), "foo")]
    fn architecture_format_string(#[case] arch: Architecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{arch}"));
    }

    #[rstest]
    #[case(vec![Architecture::Any], Ok(Architectures::Any))]
    #[case(vec![SystemArchitecture::Aarch64.into()], Ok(Architectures::Some(vec![SystemArchitecture::Aarch64])))]
    #[case(vec![SystemArchitecture::Arm.into(), SystemArchitecture::I386.into()], Ok(Architectures::Some(vec![SystemArchitecture::Arm, SystemArchitecture::I386])))]
    // Duplicates are allowed (discouraged by linter)
    #[case(vec![SystemArchitecture::Arm.into(), SystemArchitecture::Arm.into()], Ok(Architectures::Some(vec![SystemArchitecture::Arm, SystemArchitecture::Arm])))]
    #[case(vec![Architecture::Any, SystemArchitecture::I386.into()], Err(Error::InvalidArchitectures {
        architectures: vec![Architecture::Any, SystemArchitecture::I386.into()],
        context: "'any' cannot be used in combination with other architectures.",
    }))]
    #[case(vec![Architecture::Any, Architecture::Any], Err(Error::InvalidArchitectures {
        architectures: vec![Architecture::Any, Architecture::Any],
        context: "'any' cannot be used in combination with other architectures.",
    }))]
    #[case(vec![], Ok(Architectures::Some(vec![])))]
    fn architecutes_from_vec(
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
