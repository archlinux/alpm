use std::path::PathBuf;

use crate::{error::Error, identifiers};

/// A context within a [`crate::identifiers::purpose::Purpose`] for more fine-grained verifier
/// assignments.
///
/// An example for context is the name of a specific software repository when certificates are
/// used in the context of the packages purpose (e.g. "core").
///
/// If no specific context is required, the context `Default` must be used.
///
/// See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#context>
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Context {
    /// The default context.
    #[default]
    Default,

    /// Defines a custom [`Context`] for verifiers within an [`crate::identifiers::os::Os`] and
    /// [`crate::identifiers::purpose::Purpose`].
    Custom(CustomContext),
}

impl Context {
    pub(crate) fn path_segment(&self) -> PathBuf {
        match self {
            Self::Default => "default".into(),
            Self::Custom(custom) => custom.as_ref().into(),
        }
    }
}

/// A `CustomContext` encodes a value for a [Context] that is not [Context::Default]
#[derive(Clone, Debug, PartialEq)]
pub struct CustomContext {
    context: String,
}

impl CustomContext {
    /// Creates a new `CustomContext` instance.
    ///
    /// Returns `Error` if `value` contains illegal characters.
    pub fn new(value: String) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::IllegalIdentifier);
        }

        identifiers::check_identifier_part(&value)?;

        Ok(Self { context: value })
    }
}

impl AsRef<str> for CustomContext {
    fn as_ref(&self) -> &str {
        self.context.as_ref()
    }
}
