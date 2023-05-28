// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use std::string::ToString;

use crate::regex_once;
use crate::Error;

/// A single 'md5sum' attribute
///
/// Md5Sum consists of 32 characters `[a-f0-9]`.
///
/// ## Examples
/// ```
/// use alpm_types::{Md5Sum, Error};
/// use std::str::FromStr;
///
/// // create Md5Sum from &str
/// assert_eq!(
///     Md5Sum::from_str("5eb63bbbe01eeed093cb22bb8f5acdc3"),
///     Ok(Md5Sum::new("5eb63bbbe01eeed093cb22bb8f5acdc3".to_string()).unwrap())
/// );
/// assert_eq!(
///     Md5Sum::from_str("foobar"),
///     Err(Error::InvalidMd5Sum("foobar".to_string()))
/// );
///
/// // format as &str
/// assert_eq!(
///   "5eb63bbbe01eeed093cb22bb8f5acdc3",
///   format!("{}", Md5Sum::new("5eb63bbbe01eeed093cb22bb8f5acdc3".to_string()).unwrap()),
/// );
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Md5Sum(String);

impl Md5Sum {
    /// Create a new Md5Sum in a Result
    ///
    /// If the supplied string is valid on the basis of the allowed characters
    /// then an Md5Sum is returned as a Result, otherwise an InvalidMd5Sum Error
    /// is returned.
    pub fn new(md5sum: String) -> Result<Md5Sum, Error> {
        if regex_once!(r"^[a-f0-9]{32}$").is_match(md5sum.as_str()) {
            Ok(Md5Sum(md5sum))
        } else {
            Err(Error::InvalidMd5Sum(md5sum))
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for Md5Sum {
    type Err = Error;
    /// Create a Md5Sum from a string
    fn from_str(input: &str) -> Result<Md5Sum, Self::Err> {
        Md5Sum::new(input.to_string())
    }
}

impl Display for Md5Sum {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_md5sum_from_string(md5sum_str in r"[a-f0-9]{32}") {
            let md5sum = Md5Sum::from_str(&md5sum_str).unwrap();
            prop_assert_eq!(md5sum_str, format!("{}", md5sum));
        }

        #[test]
        fn invalid_md5sum_from_string_bigger_size(md5sum_str in r"[a-f0-9]{64}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }

        #[test]
        fn invalid_md5sum_from_string_smaller_size(md5sum_str in r"[a-f0-9]{16}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }

        #[test]
        fn invalid_md5sum_from_string_wrong_chars(md5sum_str in r"[e-z0-9]{32}") {
            let error = Md5Sum::from_str(&md5sum_str).unwrap_err();
            assert!(format!("{}", error).ends_with(&md5sum_str));
        }
    }
}
