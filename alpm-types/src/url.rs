use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::Error;

/// Represents a URL.
///
/// It is used to represent the upstream URL of a package.
/// This type does not yet enforce a secure connection (e.g. HTTPS).
///
/// The `Url` type wraps the [`url::Url`] type.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Url;
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // Create Url from &str
/// let url = Url::from_str("https://example.com/download")?;
/// assert_eq!(url.as_str(), "https://example.com/download");
///
/// // Format as String
/// assert_eq!(format!("{url}"), "https://example.com/download");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url(url::Url);

impl Url {
    /// Creates a new `Url` instance.
    pub fn new(url: url::Url) -> Result<Self, Error> {
        Ok(Self(url))
    }

    /// Returns a reference to the inner `url::Url` as a `&str`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the `Url` and returns the inner `url::Url`.
    pub fn into_inner(self) -> url::Url {
        self.0
    }

    /// Returns a reference to the inner `url::Url`.
    pub fn inner(&self) -> &url::Url {
        &self.0
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Url {
    type Err = Error;

    /// Creates a new `Url` instance from a string slice.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::Url;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let url = Url::from_str("https://archlinux.org/")?;
    /// assert_eq!(url.as_str(), "https://archlinux.org/");
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = url::Url::parse(s).map_err(Error::InvalidUrl)?;
        Self::new(url)
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("https://example.com/", Ok("https://example.com/"))]
    #[case(
        "https://example.com/path?query=1",
        Ok("https://example.com/path?query=1")
    )]
    #[case("ftp://example.com/", Ok("ftp://example.com/"))]
    #[case("not-a-url", Err(url::ParseError::RelativeUrlWithoutBase.into()))]
    fn test_url_parsing(#[case] input: &str, #[case] expected: Result<&str, Error>) {
        let result = input.parse::<Url>();
        assert_eq!(
            result.as_ref().map(|v| v.to_string()),
            expected.as_ref().map(|v| v.to_string())
        );

        if let Ok(url) = result {
            assert_eq!(url.as_str(), input);
        }
    }
}
