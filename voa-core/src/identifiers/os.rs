use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use crate::{error::Error, identifiers};

/// The Os identifier is used to uniquely identify an Operating System (OS), it relies on data
/// provided by [`os-release`].
///
/// [`os-release`]: https://man.archlinux.org/man/os-release.5.en
///
/// # Format
///
/// An Os identifier consists of up to five parts.
/// Each part of the identifier can consist of the characters "0–9", "a–z", ".", "_" and "-".
///
/// In the filesystem, the parts are concatenated into one path using `:` (colon) symbols
/// (e.g. `debian:12:server:company-x:25.01`).
///
/// Trailing colons must be omitted for all parts that are unset
/// (e.g. `arch` instead of `arch::::`).
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#os>
#[derive(Clone, Debug, PartialEq)]
pub struct Os {
    id: String,
    version_id: Option<String>,
    variant_id: Option<String>,
    image_id: Option<String>,
    image_version: Option<String>,
}

impl Os {
    /// Creates a new operating system identifier.
    ///
    /// Returns `Error` if any parameter contains illegal characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use voa_core::identifiers::Os;
    ///
    /// # fn main() -> Result<(), voa_core::Error> {
    /// // Arch Linux is a rolling release distribution.
    /// Os::new("arch".into(), None, None, None, None);
    ///
    /// // This Debian system is a special purpose image-based OS.
    /// Os::new(
    ///     "debian".into(),
    ///     Some("12".into()),
    ///     Some("workstation".into()),
    ///     Some("cashier-system".into()),
    ///     Some("1.0.0".into()),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        id: String,
        version_id: Option<String>,
        variant_id: Option<String>,
        image_id: Option<String>,
        image_version: Option<String>,
    ) -> Result<Self, Error> {
        // Check validity of inputs as identifier parts
        identifiers::check_identifier_part(&id)?;

        if let Some(version_id) = &version_id {
            identifiers::check_identifier_part(version_id)?;
        }

        if let Some(variant_id) = &variant_id {
            identifiers::check_identifier_part(variant_id)?;
        }

        if let Some(image_id) = &image_id {
            identifiers::check_identifier_part(image_id)?;
        }

        if let Some(image_version) = &image_version {
            identifiers::check_identifier_part(image_version)?;
        }

        Ok(Self {
            id,
            version_id,
            variant_id,
            image_id,
            image_version,
        })
    }

    /// A [`String`] representation of this Os specifier.
    ///
    /// All parts are joined with `:`, trailing colons are omitted.
    /// Parts that are unset are represented as empty strings.
    fn os_to_string(&self) -> String {
        let os = format!(
            "{}:{}:{}:{}:{}",
            &self.id,
            self.version_id.as_deref().unwrap_or(""),
            self.variant_id.as_deref().unwrap_or(""),
            self.image_id.as_deref().unwrap_or(""),
            self.image_version.as_deref().unwrap_or(""),
        );

        os.trim_end_matches(':').into()
    }

    /// A [`PathBuf`] representation of this Os specifier.
    ///
    /// All parts are joined with `:`, trailing colons are omitted.
    /// Parts that are unset are represented as empty strings.
    pub(crate) fn path_segment(&self) -> PathBuf {
        self.os_to_string().into()
    }
}

impl Display for Os {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.os_to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::identifiers::Os;

    #[test]
    fn context_display() {
        let os1 = Os::new("arch".into(), None, None, None, None).unwrap();

        assert_eq!(format!("{os1}"), "arch");

        let os2 = Os::new(
            "debian".into(),
            Some("12".into()),
            Some("workstation".into()),
            Some("cashier-system".into()),
            Some("1.0.0".into()),
        )
        .unwrap();

        assert_eq!(
            format!("{os2}"),
            "debian:12:workstation:cashier-system:1.0.0"
        );

        let os3 = Os::new(
            "debian".into(),
            Some("12".into()),
            Some("workstation".into()),
            None,
            None,
        )
        .unwrap();

        assert_eq!(format!("{os3}"), "debian:12:workstation");
    }
}
