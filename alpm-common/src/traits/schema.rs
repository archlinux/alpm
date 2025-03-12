//! Traits for schemas.

#[cfg(doc)]
use std::str::FromStr;
use std::{io::Read, path::Path};

use alpm_types::SchemaVersion;

/// A trait for file format schemas and their versioning.
///
/// File formats are expected to either expose the schema version directly, or at least make it
/// possible to derive the version from them.
pub trait FileFormatSchema {
    /// The Error type to use.
    type Err;

    /// Returns the reference to an inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion;

    /// Derives [`Self`] from a `file`.
    ///
    /// # Note
    ///
    /// This function is meant for implementers to _derive_ [`Self`] based on the properties of the
    /// `file` contents (e.g. a context-specific version identifier, etc.).
    /// Instead of creating [`Self`] from considering all of `file`, this function is only used to
    /// introspect `file` and to retrieve required information to _derive_ which variant of
    /// [`Self`] to create or whether to fail.
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Self::Err>
    where
        Self: Sized;

    /// Derives [`Self`] from a [`Read`] implementer.
    ///
    /// # Note
    ///
    /// This function is meant for implementers to _derive_ [`Self`] based on the properties of the
    /// `reader` contents (e.g. a context-specific version identifier, etc.).
    /// Instead of creating [`Self`] from considering all of `reader`, this function is only used to
    /// introspect `reader` and to retrieve required information to _derive_ which variant of
    /// [`Self`] to create or whether to fail.
    fn derive_from_reader(reader: impl Read) -> Result<Self, Self::Err>
    where
        Self: Sized;

    /// Derives [`Self`] from a string slice `s`.
    ///
    /// # Note
    ///
    /// This function is meant for implementers to _derive_ [`Self`] based on the
    /// properties of `s` (e.g. a context-specific version identifier, etc.).
    /// Instead of creating [`Self`] from considering all of `s`, this function is only used to
    /// introspect `s` and to retrieve required information to _derive_ which variant of [`Self`] to
    /// create or whether to fail.
    ///
    /// For _creation_ from the entire string slice the [`FromStr`] trait should be implemented
    /// instead.
    fn derive_from_str(s: &str) -> Result<Self, Self::Err>
    where
        Self: Sized;
}
