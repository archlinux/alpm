/// Compressed size of a file (in bytes)
///
/// This is a type alias for [`u64`].
///
/// ## Examples
/// ```
/// use std::{num::IntErrorKind, str::FromStr};
///
/// use alpm_types::{CompressedSize, Error};
///
/// assert_eq!(CompressedSize::from_str("1"), Ok(1));
/// assert!(CompressedSize::from_str("-1").is_err());
/// ```
pub type CompressedSize = u64;

/// Installed size of a package (in bytes)
///
/// This is a type alias for [`u64`].
///
/// ## Examples
/// ```
/// use std::{num::IntErrorKind, str::FromStr};
///
/// use alpm_types::{Error, InstalledSize};
///
/// // create InstalledSize from &str
/// assert_eq!(InstalledSize::from_str("1"), Ok(1));
/// assert!(InstalledSize::from_str("-1").is_err());
/// ```
pub type InstalledSize = u64;
