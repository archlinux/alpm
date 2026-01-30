//! Schema definition for the [alpm-repo-desc] file format.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{
    fmt::{Display, Formatter},
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::FileFormatSchema;
use alpm_types::{SchemaVersion, semver_version::Version};
use fluent_i18n::t;

use crate::Error;

/// An enum describing all valid [alpm-repo-desc] schemas.
///
/// Each variant corresponds to a specific revision of the
/// specification.
///
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepoDescSchema {
    /// Schema for the [alpm-repo-descv1] file format.
    ///
    /// [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
    V1(SchemaVersion),
    /// Schema for the [alpm-repo-descv2] file format.
    ///
    /// [alpm-repo-descv2]: https://alpm.archlinux.page/specifications/alpm-repo-descv2.5.html
    V2(SchemaVersion),
}

impl FileFormatSchema for RepoDescSchema {
    type Err = Error;

    /// Returns a reference to the inner [`SchemaVersion`].
    fn inner(&self) -> &SchemaVersion {
        match self {
            RepoDescSchema::V1(v) => v,
            RepoDescSchema::V2(v) => v,
        }
    }

    /// Derives a [`RepoDescSchema`] from an [alpm-repo-desc] file on disk.
    ///
    /// Opens the `file` and defers to [`RepoDescSchema::derive_from_reader`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - or deriving a [`RepoDescSchema`] from its contents fails.
    ///
    /// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
    fn derive_from_file(file: impl AsRef<Path>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let file = file.as_ref();
        Self::derive_from_reader(File::open(file).map_err(|source| Error::IoPath {
            path: PathBuf::from(file),
            context: t!("error-io-path-schema-file"),
            source,
        })?)
    }

    /// Derives a [`RepoDescSchema`] from [alpm-repo-desc] data in a reader.
    ///
    /// Reads the `reader` to a string and defers to [`RepoDescSchema::derive_from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - reading from `reader` fails,
    /// - or deriving a [`RepoDescSchema`] from its contents fails.
    ///
    /// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
    fn derive_from_reader(reader: impl std::io::Read) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut buf = String::new();
        let mut reader = reader;
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoRead {
                context: t!("error-io-read-schema-data"),
                source,
            })?;
        Self::derive_from_str(&buf)
    }

    /// Derives a [`RepoDescSchema`] from a string slice containing [alpm-repo-desc] data.
    ///
    /// The parser uses a simple heuristic:
    ///
    /// - v1 → `%MD5SUM%` section present
    /// - v2 → no `%MD5SUM%` section present
    ///
    /// This approach avoids relying on explicit version metadata, as the package repository desc
    /// format itself is not self-describing.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::FileFormatSchema;
    /// use alpm_repo_db::desc::RepoDescSchema;
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> Result<(), alpm_repo_db::Error> {
    /// let v1_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %MD5SUM%
    /// d3b07384d113edec49eaa6238ad5ff00
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %PGPSIG%
    /// iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///
    /// assert_eq!(
    ///     RepoDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
    ///     RepoDescSchema::derive_from_str(v1_data)?
    /// );
    ///
    /// let v2_data = r#"%FILENAME%
    /// example-meta-1.0.0-1-any.pkg.tar.zst
    ///
    /// %NAME%
    /// example-meta
    ///
    /// %BASE%
    /// example-meta
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %DESC%
    /// An example meta package
    ///
    /// %CSIZE%
    /// 4634
    ///
    /// %ISIZE%
    /// 0
    ///
    /// %SHA256SUM%
    /// b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %LICENSE%
    /// GPL-3.0-or-later
    ///
    /// %ARCH%
    /// any
    ///
    /// %BUILDDATE%
    /// 1729181726
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// "#;
    ///
    /// assert_eq!(
    ///     RepoDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0))),
    ///     RepoDescSchema::derive_from_str(v2_data)?
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error only if internal conversion or string handling fails.
    ///
    /// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
    fn derive_from_str(s: &str) -> Result<RepoDescSchema, Error> {
        // Instead of an explicit "format" key, we use a heuristic:
        // presence of `%MD5SUM%` implies version 1.
        if s.contains("%MD5SUM%") {
            Ok(RepoDescSchema::V1(SchemaVersion::new(Version::new(
                1, 0, 0,
            ))))
        } else {
            Ok(RepoDescSchema::V2(SchemaVersion::new(Version::new(
                2, 0, 0,
            ))))
        }
    }
}

impl Default for RepoDescSchema {
    /// Returns the default schema variant ([`RepoDescSchema::V2`]).
    fn default() -> Self {
        Self::V2(SchemaVersion::new(Version::new(2, 0, 0)))
    }
}

impl FromStr for RepoDescSchema {
    type Err = Error;

    /// Parses a [`RepoDescSchema`] from a version string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input string is not a valid version,
    /// - or the version does not correspond to a known schema variant.
    fn from_str(s: &str) -> Result<RepoDescSchema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version) => Self::try_from(version),
            Err(_) => Err(Error::UnsupportedSchemaVersion(s.to_string())),
        }
    }
}

impl TryFrom<SchemaVersion> for RepoDescSchema {
    type Error = Error;

    /// Converts a [`SchemaVersion`] into a corresponding [`RepoDescSchema`].
    ///
    /// # Errors
    ///
    /// Returns an error if the major version of [`SchemaVersion`] does not
    /// correspond to a known [`RepoDescSchema`] variant.
    fn try_from(value: SchemaVersion) -> Result<Self, Self::Error> {
        match value.inner().major {
            1 => Ok(RepoDescSchema::V1(value)),
            2 => Ok(RepoDescSchema::V2(value)),
            _ => Err(Error::UnsupportedSchemaVersion(value.to_string())),
        }
    }
}

impl Display for RepoDescSchema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                RepoDescSchema::V1(version) | RepoDescSchema::V2(version) => version.inner().major,
            }
        )
    }
}
