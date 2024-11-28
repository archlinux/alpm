use strum::{Display, EnumString};

use crate::Name;

/// The type of a package
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::PkgType;
///
/// // create PkgType from str
/// assert_eq!(PkgType::from_str("pkg"), Ok(PkgType::Package));
///
/// // format as String
/// assert_eq!("debug", format!("{}", PkgType::Debug));
/// assert_eq!("pkg", format!("{}", PkgType::Package));
/// assert_eq!("src", format!("{}", PkgType::Source));
/// assert_eq!("split", format!("{}", PkgType::Split));
/// ```
#[derive(Copy, Clone, Debug, Display, EnumString, Eq, PartialEq)]
#[non_exhaustive]
pub enum PkgType {
    /// a debug package
    #[strum(to_string = "debug")]
    Debug,
    /// a single (non-split) package
    #[strum(to_string = "pkg")]
    Package,
    /// a source-only package
    #[strum(to_string = "src")]
    Source,
    /// one split package out of a set of several
    #[strum(to_string = "split")]
    Split,
}

/// Description of a package
///
/// This is a type alias for [`String`].
///
/// ## Examples
/// ```
/// use alpm_types::{Error, PkgDesc};
///
/// // Create a PkgDesc
/// let desc: PkgDesc = "A simple package".to_string();
/// ```
pub type PkgDesc = String;

/// Name of the base package information that one or more packages are built from.
///
/// This is a type alias for [`Name`].
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create PkgBase from &str
/// let pkgbase = Name::from_str("test-123@.foo_+")?;
///
/// // format as String
/// let pkgbase = Name::from_str("foo")?;
/// assert_eq!("foo", pkgbase.to_string());
/// # Ok(())
/// # }
/// ```
pub type PkgBase = Name;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("debug", Ok(PkgType::Debug))]
    #[case("pkg", Ok(PkgType::Package))]
    #[case("src", Ok(PkgType::Source))]
    #[case("split", Ok(PkgType::Split))]
    #[case("foo", Err(strum::ParseError::VariantNotFound))]
    fn pkgtype_from_string(
        #[case] from_str: &str,
        #[case] result: Result<PkgType, strum::ParseError>,
    ) {
        assert_eq!(PkgType::from_str(from_str), result);
    }

    #[rstest]
    #[case(PkgType::Debug, "debug")]
    #[case(PkgType::Package, "pkg")]
    #[case(PkgType::Source, "src")]
    #[case(PkgType::Split, "split")]
    fn pkgtype_format_string(#[case] pkgtype: PkgType, #[case] pkgtype_str: &str) {
        assert_eq!(pkgtype_str, format!("{}", pkgtype));
    }
}
