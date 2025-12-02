//! Data representations and integrations for reading of SRCINFO data.
pub mod parser;
pub mod v1;

use std::{fs::File, path::Path, str::FromStr};

use alpm_common::MetadataFile;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{Error, SourceInfoSchema, SourceInfoV1};

/// The representation of SRCINFO data.
///
/// Tracks all available versions of the file format.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourceInfo {
    /// The [SRCINFO] file format.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    V1(SourceInfoV1),
}

impl MetadataFile<SourceInfoSchema> for SourceInfo {
    type Err = Error;

    /// Creates a [`SourceInfo`] from `file`, optionally validated using a [`SourceInfoSchema`].
    ///
    /// Opens the `file` and defers to [`SourceInfo::from_reader_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_file`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with SRCINFO data
    /// let srcinfo_file = tempfile::NamedTempFile::new()?;
    /// let (file, srcinfo_data) = {
    ///     let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///     let mut output = File::create(&srcinfo_file)?;
    ///     write!(output, "{}", srcinfo_data)?;
    ///     (srcinfo_file, srcinfo_data)
    /// };
    ///
    /// let srcinfo = SourceInfo::from_file_with_schema(
    ///     file.path().to_path_buf(),
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - no variant of [`SourceInfo`] can be constructed from the contents of `file`,
    /// - or `schema` is [`Some`] and the [`SourceInfoSchema`] does not match the contents of
    ///   `file`.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<SourceInfoSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| Error::IoPath {
                path: file.to_path_buf(),
                context: t!("error-io-path-opening-file"),
                source,
            })?,
            schema,
        )
    }

    /// Creates a [`SourceInfo`] from a `reader`, optionally validated using a
    /// [`SourceInfoSchema`].
    ///
    /// Reads the `reader` to string and defers to [`SourceInfo::from_str_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_reader`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let srcinfo_file = tempfile::NamedTempFile::new()?;
    /// // Prepare a reader with SRCINFO data
    /// let (reader, srcinfo_data) = {
    ///     let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///     let mut output = File::create(&srcinfo_file)?;
    ///     write!(output, "{}", srcinfo_data)?;
    ///     (File::open(&srcinfo_file.path())?, srcinfo_data)
    /// };
    ///
    /// let srcinfo = SourceInfo::from_reader_with_schema(
    ///     reader,
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - no variant of [`SourceInfo`] can be constructed from the contents of the `reader`,
    /// - or `schema` is [`Some`] and the [`SourceInfoSchema`] does not match the contents of the
    ///   `reader`.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<SourceInfoSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-read-srcinfo-data"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`SourceInfo`] from string slice, optionally validated using a
    /// [`SourceInfoSchema`].
    ///
    /// If `schema` is [`None`] attempts to detect the [`SourceInfoSchema`] from `s`.
    /// Attempts to create a [`SourceInfo`] variant that corresponds to the [`SourceInfoSchema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///
    /// let srcinfo = SourceInfo::from_str_with_schema(
    ///     srcinfo_data,
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `schema` is [`Some`] and the specified variant of [`SourceInfo`] cannot be constructed
    ///   from `s`,
    /// - `schema` is [`None`] and
    ///   - a [`SourceInfoSchema`] cannot be derived from `s`,
    ///   - or the detected variant of [`SourceInfo`] cannot be constructed from `s`.
    fn from_str_with_schema(s: &str, schema: Option<SourceInfoSchema>) -> Result<Self, Error> {
        // NOTE: This does not use `SourceInfoSchema::derive_from_str`,
        // to not run the parser twice.
        // In the future, this should run `SourceInfoContent` parser directly
        // and delegate to `from_raw` instead of `from_string`.

        let schema = match schema {
            Some(schema) => schema,
            None => SourceInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
        };

        match schema {
            SourceInfoSchema::V1(_) => Ok(SourceInfo::V1(SourceInfoV1::from_string(s)?)),
        }
    }
}

impl FromStr for SourceInfo {
    type Err = Error;

    /// Creates a [`SourceInfo`] from string slice `s`.
    ///
    /// Calls [`SourceInfo::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`SourceInfoSchema`] cannot be derived from `s`,
    /// - or the detected variant of [`SourceInfo`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}
