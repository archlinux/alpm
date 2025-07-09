use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use crate::{error::Error, identifiers};

/// The name of a technology backend.
///
/// Technology-specific backends implement the logic for each supported verification technology
/// in VOA.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#technology>
#[derive(Clone, Debug, strum::Display, PartialEq)]
pub enum Technology {
    /// The [OpenPGP] technology.
    ///
    /// [OpenPGP]: https://www.openpgp.org/
    #[strum(to_string = "openpgp")]
    OpenPGP,

    /// The [SSH] technology.
    ///
    /// [SSH]: https://www.openssh.com/
    #[strum(to_string = "ssh")]
    SSH,

    /// Defines a custom [`Technology`] name.
    #[strum(to_string = "{0}")]
    Custom(CustomTechnology),
}

impl Technology {
    pub(crate) fn path_segment(&self) -> PathBuf {
        format!("{self}").into()
    }
}

/// A `CustomTechnology` defines a technology name that is not covered by the variants defined in
/// [Technology].
#[derive(Clone, Debug, PartialEq)]
pub struct CustomTechnology {
    technology: String,
}

impl CustomTechnology {
    /// Creates a new `CustomTechnology` instance.
    ///
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(value: String) -> Result<Self, Error> {
        identifiers::check_identifier_part(&value)?;

        Ok(Self { technology: value })
    }
}

impl Display for CustomTechnology {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.technology)
    }
}

impl AsRef<str> for CustomTechnology {
    fn as_ref(&self) -> &str {
        self.technology.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::identifiers::{CustomTechnology, Technology};

    #[test]
    fn technology_display() {
        assert_eq!(format!("{}", Technology::OpenPGP), "openpgp");
        assert_eq!(
            format!(
                "{}",
                Technology::Custom(CustomTechnology::new("foo".into()).unwrap())
            ),
            "foo"
        );
    }
}
