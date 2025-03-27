//! Data related to package file contents.

use std::str::FromStr;

/// The name of an [alpm-install-scriptlet] file in an [alpm-package].
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
pub const INSTALL_SCRIPTLET_FILE_NAME: &str = ".INSTALL";

/// The name of a required metadata file in an [alpm-package].
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(
    strum::AsRefStr,
    Clone,
    Copy,
    Debug,
    serde::Deserialize,
    strum::Display,
    Eq,
    strum::IntoStaticStr,
    PartialEq,
    serde::Serialize,
)]
#[serde(try_from = "String", into = "String")]
pub enum MetadataFileName {
    /// The [BUILDINFO] file.
    ///
    /// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
    #[strum(to_string = ".BUILDINFO")]
    BuildInfo,

    /// The [ALPM-MTREE] file.
    ///
    /// [ALPM-MTREE]: ahttps://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
    #[strum(to_string = ".MTREE")]
    Mtree,

    /// The [PKGINFO] file.
    ///
    /// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
    #[strum(to_string = ".PKGINFO")]
    PackageInfo,
}

impl FromStr for MetadataFileName {
    type Err = crate::Error;

    /// Creates a [`MetadataFileName`] from string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `s` does not equal the string representation of a [`MetadataFileName`]
    /// variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::MetadataFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     MetadataFileName::BuildInfo,
    ///     MetadataFileName::from_str(".BUILDINFO")?
    /// );
    /// assert_eq!(
    ///     MetadataFileName::Mtree,
    ///     MetadataFileName::from_str(".MTREE")?
    /// );
    /// assert_eq!(
    ///     MetadataFileName::PackageInfo,
    ///     MetadataFileName::from_str(".PKGINFO")?
    /// );
    /// assert!(MetadataFileName::from_str(".WRONG").is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ".BUILDINFO" => Self::BuildInfo,
            ".MTREE" => Self::Mtree,
            ".PKGINFO" => Self::PackageInfo,
            _ => {
                return Err(crate::PackageError::InvalidMetadataFilename {
                    name: s.to_string(),
                }
                .into());
            }
        })
    }
}

impl From<MetadataFileName> for String {
    /// Creates a [`String`] from [`MetadataFileName`].
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::MetadataFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(MetadataFileName::BuildInfo.to_string(), ".BUILDINFO");
    /// assert_eq!(MetadataFileName::Mtree.to_string(), ".MTREE");
    /// assert_eq!(MetadataFileName::PackageInfo.to_string(), ".PKGINFO");
    /// # Ok(())
    /// # }
    /// ```
    fn from(value: MetadataFileName) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for MetadataFileName {
    type Error = crate::Error;

    /// Creates a [`MetadataFileName`] from [`String`].
    ///
    /// # Errors
    ///
    /// Returns an error if `s` does not equal the string representation of a [`MetadataFileName`]
    /// variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::MetadataFileName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert_eq!(
    ///     MetadataFileName::BuildInfo,
    ///     MetadataFileName::try_from(".BUILDINFO".to_string())?
    /// );
    /// assert_eq!(
    ///     MetadataFileName::Mtree,
    ///     MetadataFileName::try_from(".MTREE".to_string())?
    /// );
    /// assert_eq!(
    ///     MetadataFileName::PackageInfo,
    ///     MetadataFileName::try_from(".PKGINFO".to_string())?
    /// );
    /// assert!(MetadataFileName::try_from(".WRONG".to_string()).is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}
