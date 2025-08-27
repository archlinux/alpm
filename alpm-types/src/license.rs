use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use spdx::Expression;

use crate::Error;

/// Represents a license expression that can be either a valid SPDX identifier
/// or a non-standard one.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::License;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create License from a valid SPDX identifier
/// let license = License::from_str("MIT")?;
/// assert!(license.is_spdx());
/// assert_eq!(license.to_string(), "MIT");
///
/// // Create License from an invalid/non-SPDX identifier
/// let license = License::from_str("My-Custom-License")?;
/// assert!(!license.is_spdx());
/// assert_eq!(license.to_string(), "My-Custom-License");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum License {
    /// A valid SPDX license expression
    ///
    /// This variant is boxed to avoid large allocations
    Spdx(Box<spdx::Expression>),
    /// A non-standard license identifier
    Unknown(String),
}

impl Serialize for License {
    /// Custom serde serialization as Spdx doesn't provide a serde [`Serialize`] implementation.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for License {
    /// Custom serde serialization as Spdx doesn't provide a serde [`Deserialize`] implementation.
    /// This implements deserialization from a string type.
    ///
    /// Attempt to parse the given input as an [spdx::Expression] and to return a [License::Spdx].
    /// If that fails, treat it as a [License::Unknown] that contains the original string.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Ok(expr) = spdx::Expression::from_str(&s) {
            return Ok(License::Spdx(Box::new(expr)));
        }

        Ok(License::Unknown(s))
    }
}

impl License {
    /// Creates a new license
    ///
    /// This function accepts both SPDX and non-standard identifiers
    /// and it is the same as as calling [`License::from_str`]
    pub fn new(license: String) -> Result<Self, Error> {
        Self::from_valid_spdx(license.clone()).or(Ok(Self::Unknown(license)))
    }

    /// Creates a new license from a valid SPDX identifier
    ///
    /// ## Examples
    ///
    /// ```
    /// use alpm_types::{Error, License};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let license = License::from_valid_spdx("Apache-2.0".to_string())?;
    /// assert!(license.is_spdx());
    /// assert_eq!(license.to_string(), "Apache-2.0");
    ///
    /// assert!(License::from_valid_spdx("GPL-0.0".to_string()).is_err());
    /// assert!(License::from_valid_spdx("Custom-License".to_string()).is_err());
    ///
    /// assert_eq!(
    ///     License::from_valid_spdx("GPL-2.0".to_string()),
    ///     Err(Error::DeprecatedLicense("GPL-2.0".to_string()))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// This function uses [strict parsing] which means:
    ///
    /// 1. Only license identifiers in the SPDX license list, or Document/LicenseRef, are allowed.
    ///    The license identifiers are also case-sensitive.
    /// 2. `WITH`, `AND`, and `OR`, case-insensitive, are the only valid operators.
    /// 3. Deprecated licenses are not allowed and will return an error
    ///    ([`Error::DeprecatedLicense`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the given input cannot be parsed or is a deprecated license.
    ///
    /// [strict parsing]: https://docs.rs/spdx/latest/spdx/lexer/struct.ParseMode.html#associatedconstant.STRICT
    pub fn from_valid_spdx(identifier: String) -> Result<Self, Error> {
        let expression = match Expression::parse(&identifier) {
            Ok(expr) => expr,
            Err(e) => {
                if e.reason == spdx::error::Reason::DeprecatedLicenseId {
                    return Err(Error::DeprecatedLicense(identifier));
                } else {
                    return Err(Error::InvalidLicense(e));
                }
            }
        };

        Ok(Self::Spdx(Box::new(expression)))
    }

    /// Returns `true` if the license is a valid SPDX identifier
    pub fn is_spdx(&self) -> bool {
        matches!(self, License::Spdx(_))
    }
}

impl FromStr for License {
    type Err = Error;

    /// Creates a new `License` instance from a string slice.
    ///
    /// If the input is a valid SPDX license expression,
    /// it will be marked as such; otherwise, it will be treated as
    /// a non-standard license identifier.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::License;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let license = License::from_str("Apache-2.0")?;
    /// assert!(license.is_spdx());
    /// assert_eq!(license.to_string(), "Apache-2.0");
    ///
    /// let license = License::from_str("NonStandard-License")?;
    /// assert!(!license.is_spdx());
    /// assert_eq!(license.to_string(), "NonStandard-License");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the given input is a deprecated SPDX license.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Display for License {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            License::Spdx(expr) => write!(f, "{expr}"),
            License::Unknown(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("MIT", License::Spdx(Box::new(Expression::parse("MIT").unwrap())))]
    #[case("Apache-2.0", License::Spdx(Box::new(Expression::parse("Apache-2.0").unwrap())))]
    #[case("Apache-2.0+", License::Spdx(Box::new(Expression::parse("Apache-2.0+").unwrap())))]
    #[case(
        "Apache-2.0 WITH LLVM-exception",
        License::Spdx(Box::new(Expression::parse("Apache-2.0 WITH LLVM-exception").unwrap()))
    )]
    #[case("GPL-3.0-or-later", License::Spdx(Box::new(Expression::parse("GPL-3.0-or-later").unwrap())))]
    #[case("HPND-Fenneberg-Livingston", License::Spdx(Box::new(Expression::parse("HPND-Fenneberg-Livingston").unwrap())))]
    #[case(
        "NonStandard-License",
        License::Unknown(String::from("NonStandard-License"))
    )]
    fn test_parse_license(
        #[case] input: &str,
        #[case] expected: License,
    ) -> testresult::TestResult<()> {
        let license = input.parse::<License>()?;
        assert_eq!(license, expected);
        assert_eq!(license.to_string(), input.to_string());
        Ok(())
    }

    #[rstest]
    #[case("Apache-2.0 WITH",
        Err(spdx::ParseError {
            original: String::from("Apache-2.0 WITH"),
            span: 15..15,
            reason: spdx::error::Reason::Unexpected(&["<addition>"])
        }.into())
    )]
    #[case("Custom-License",
        Err(spdx::ParseError {
            original: String::from("Custom-License"),
            span: 0..14,
            reason: spdx::error::Reason::UnknownTerm
        }.into())
    )]
    fn test_invalid_spdx(#[case] input: &str, #[case] expected: Result<License, Error>) {
        let result = License::from_valid_spdx(input.to_string());
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("BSD-2-Clause-FreeBSD")]
    #[case("BSD-2-Clause-NetBSD")]
    #[case("bzip2-1.0.5")]
    #[case("GPL-2.0")]
    fn test_deprecated_spdx(#[case] input: &str) {
        let result = License::from_valid_spdx(input.to_string());
        assert_eq!(result, Err(Error::DeprecatedLicense(input.to_string())));
    }

    #[rstest]
    #[case("MIT", true)]
    #[case("Custom-License", false)]
    fn test_license_kind(#[case] input: &str, #[case] is_spdx: bool) -> testresult::TestResult<()> {
        let spdx_license = License::from_str(input)?;
        assert_eq!(spdx_license.is_spdx(), is_spdx);

        Ok(())
    }
}
