//! Traits for metadata files.

#[cfg(doc)]
use std::str::FromStr;
use std::{
    io::{Read, stdin},
    path::Path,
};

use crate::FileFormatSchema;

/// A trait for metadata files.
///
/// Metadata files are expected to adhere to a [`FileFormatSchema`] that is encoded in them
/// (purposefully or not).
/// This trait provides a set of functions to allow the easy creation of objects representing
/// metadata files from a diverse set of inputs.
/// Some functions allow the optional creation of the metadata file objects based a provided
/// [`FileFormatSchema`].
pub trait MetadataFile<T>
where
    T: FileFormatSchema,
{
    /// The Error type to use.
    type Err;

    /// Creates [`Self`] from file.
    ///
    /// # Note
    ///
    /// Implementations of this function are expected to automatically detect a [`FileFormatSchema`]
    /// that the resulting [`Self`] is based on.
    ///
    /// The blanket implementation calls [`Self::from_file_with_schema`] with [`None`] as `schema`.
    fn from_file(file: impl AsRef<Path>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Self::from_file_with_schema(file, None)
    }

    /// Creates [`Self`] from `file`, optionally validated by a `schema`.
    ///
    /// If a [`FileFormatSchema`] is provided, [`Self`] must be validated using it.
    fn from_file_with_schema(file: impl AsRef<Path>, schema: Option<T>) -> Result<Self, Self::Err>
    where
        Self: Sized;

    /// Creates [`Self`] from [`stdin`].
    ///
    /// # Note
    ///
    /// Implementations of this function are expected to automatically detect a [`FileFormatSchema`]
    /// that the resulting [`Self`] is based on.
    ///
    /// The blanket implementation calls [`Self::from_stdin_with_schema`] with [`None`] as `schema`.
    fn from_stdin() -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Self::from_stdin_with_schema(None)
    }

    /// Creates [`Self`] from [`stdin`], optionally validated by a `schema`.
    ///
    /// If a [`FileFormatSchema`] is provided, [`Self`] must be validated using it.
    ///
    /// The blanket implementation calls [`Self::from_reader_with_schema`] by passing in [`stdin`]
    /// and the provided `schema`.
    fn from_stdin_with_schema(schema: Option<T>) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Self::from_reader_with_schema(stdin(), schema)
    }

    /// Creates [`Self`] from a [`Read`] implementer.
    ///
    /// # Note
    ///
    /// Implementations of this function are expected to automatically detect a [`FileFormatSchema`]
    /// that the resulting [`Self`] is based on.
    ///
    /// The blanket implementation calls [`Self::from_reader_with_schema`] with [`None`] as
    /// `schema`.
    fn from_reader(reader: impl Read) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Self::from_reader_with_schema(reader, None)
    }

    /// Creates [`Self`] from a [`Read`] implementer, optionally validated by a `schema`.
    ///
    /// If a [`FileFormatSchema`] is provided, [`Self`] must be validated using it.
    fn from_reader_with_schema(reader: impl Read, schema: Option<T>) -> Result<Self, Self::Err>
    where
        Self: Sized;

    /// Creates [`Self`] from a string slice, optionally validated by a `schema`.
    ///
    /// If a [`FileFormatSchema`] is provided, [`Self`] must be validated using it.
    ///
    /// # Note
    ///
    /// When also implementing [`FromStr`], it is advisable to redirect its implementation to call
    /// [`Self::from_str_with_schema`] with [`None`] as `schema`.
    fn from_str_with_schema(s: &str, schema: Option<T>) -> Result<Self, Self::Err>
    where
        Self: Sized;
}
