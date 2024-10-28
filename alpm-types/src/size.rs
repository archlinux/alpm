use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use crate::Error;

/// Compressed size of a file (in bytes)
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{CompressedSize, Error};
///
/// // create CompressedSize from &str
/// assert_eq!(CompressedSize::from_str("1"), Ok(CompressedSize(1)));
/// assert_eq!(
///     CompressedSize::from_str("-1"),
///     Err(Error::InvalidCompressedSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", CompressedSize(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedSize(pub u64);

impl FromStr for CompressedSize {
    type Err = Error;

    /// Create a CompressedSize from a string
    fn from_str(input: &str) -> Result<CompressedSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(compressedsize) => Ok(CompressedSize(compressedsize)),
            _ => Err(Error::InvalidCompressedSize(input.to_string())),
        }
    }
}

impl Display for CompressedSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

/// Installed size of a package (in bytes)
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, InstalledSize};
///
/// // create InstalledSize from &str
/// assert_eq!(InstalledSize::from_str("1"), Ok(InstalledSize(1)));
/// assert_eq!(
///     InstalledSize::from_str("-1"),
///     Err(Error::InvalidInstalledSize(String::from("-1")))
/// );
///
/// // format as String
/// assert_eq!("1", format!("{}", InstalledSize(1)));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstalledSize(pub u64);

impl FromStr for InstalledSize {
    type Err = Error;
    /// Create a InstalledSize from a string
    fn from_str(input: &str) -> Result<InstalledSize, Self::Err> {
        match input.parse::<u64>() {
            Ok(size) => Ok(InstalledSize(size)),
            _ => Err(Error::InvalidInstalledSize(input.to_string())),
        }
    }
}

impl Display for InstalledSize {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1", Ok(CompressedSize(1)))]
    #[case("-1", Err(Error::InvalidCompressedSize(String::from("-1"))))]
    fn compressedsize_from_string(
        #[case] from_str: &str,
        #[case] result: Result<CompressedSize, Error>,
    ) {
        assert_eq!(CompressedSize::from_str(from_str), result);
    }

    #[rstest]
    fn compressedsize_format_string() {
        assert_eq!("1", format!("{}", CompressedSize(1)));
    }

    #[rstest]
    #[case("1", Ok(InstalledSize(1)))]
    #[case("-1", Err(Error::InvalidInstalledSize(String::from("-1"))))]
    fn installedsize_from_string(
        #[case] from_str: &str,
        #[case] result: Result<InstalledSize, Error>,
    ) {
        assert_eq!(InstalledSize::from_str(from_str), result);
    }

    #[rstest]
    fn installedsize_format_string() {
        assert_eq!("1", format!("{}", InstalledSize(1)));
    }
}
