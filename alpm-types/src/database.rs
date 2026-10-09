//! Components for various types of databases.
//!
//! This module encompasses types for the use in [alpm-db] and [alpm-repo-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
//! [alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html

use std::{
    fmt::{Display, Formatter},
    fs::symlink_metadata,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_parsers::prelude::*;
use fluent_i18n::t;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winnow::{
    Parser,
    combinator::{fail, peek, repeat_till},
    error::ErrMode,
    stream::Stream,
    token::any,
};

use crate::{FullVersion, Name};

/// The error that may occur when handling [alpm-db] or [alpm-repo-db] databases.
///
/// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
/// [alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [alpm-db] entry is not a directory but a symlink.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-is-not-a-directory", { "path" => .path }))]
    IsNotADirectory {
        /// The path of the entry, that is not a directory.
        path: PathBuf,
    },

    /// An [alpm-db] entry has no filename.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-has-no-filename", { "path" => .path }))]
    HasNoFilename {
        /// The path, that has no filename.
        path: PathBuf,
    },

    /// An [alpm-db] entry contains non-UTF-8 characters.
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    #[error("{msg}", msg = t!("error-database-entry-has-no-filename", { "path" => .path }))]
    ContainsNonUtf8Chars {
        /// The path, that has no filename.
        path: PathBuf,
    },
}

/// The name of a directory that stores the metadata of an installed package.
///
/// This combines an [alpm-package-name] and a **full** [alpm-package-version].
///
/// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
/// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-version.7.html
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct EntryName {
    /// The package name.
    ///
    /// See
    /// [alpm-package-name](https://alpm.archlinux.page/specifications/alpm-package-name.7.html).
    pub name: Name,

    /// The full package version.
    ///
    /// See
    /// [alpm-package-version](https://alpm.archlinux.page/specifications/alpm-package-version.7.html).
    pub version: FullVersion,
}

impl EntryName {
    /// Creates a new [`EntryName`].
    pub fn new(name: Name, version: FullVersion) -> Self {
        Self { name, version }
    }

    /// Returns a reference to the [`Name`].
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns a reference to the [`FullVersion`].
    pub fn version(&self) -> &FullVersion {
        &self.version
    }

    /// Returns the entry name as [`PathBuf`].
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }
}

impl Display for EntryName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

impl FromStr for EntryName {
    type Err = crate::Error;

    /// Parses a [`EntryName`] from a string slice.
    ///
    /// Delegates to [`EntryName::parser_until`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`EntryName::parser_until`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
    }
}

impl ParserUntil for EntryName {
    /// Recognizes an [`EntryName`] in a string slice before a `delimiter`.
    ///
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - the [`Name`] component can not be recognized,
    /// - or the [`FullVersion`] component can not be recognized,
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_parsers::prelude::*;
    /// use alpm_types::EntryName;
    /// use winnow::Parser;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let entry_name = "example-package-1:1.0.0-1";
    /// assert_eq!(
    ///     entry_name,
    ///     EntryName::parser_until_eof
    ///         .parse(Input::new(entry_name))?
    ///         .to_string()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    // TODO(cleanup): Investigate the arithmetic_side_effects
    #[expect(clippy::arithmetic_side_effects)]
    fn parser_until<'a, P>(delimiter: P) -> impl Parser<Input<'a>, Self, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, &'a str, ErrMode<ParseStack<'a>>>,
    {
        // Define the actual parser closure.
        // The delimiter is moved into the closure and borrowed via `by_ref()` on each call.
        let mut delimiter_parser = delimiter;
        let parser = move |input: &mut Input<'a>| -> PResult<'a, Self> {
            // Detect the amount of dashes in input and subsequently in the Name component.
            //
            // Note: This is a necessary step because dashes are used as delimiters between the
            // components of the entry name and the Name component (an alpm-package-name) can
            // contain dashes, too.
            // We know that the minimum amount of dashes in a valid entry name of an alpm-db is
            // two (one dash between the Name and FullVersion component and one dash in the
            // FullVersion component). We rely on this fact to determine the amount of
            // dashes in the Name component and thereby the cut-off point between the
            // Name and the FullVersion component.
            let checkpoint = input.checkpoint();
            let dashes: usize =
                repeat_till::<_, _, (), _, _, _, _>(0.., any, peek(delimiter_parser.by_ref()))
                    .take()
                    .map(|s| {
                        s.chars().fold(0, |acc, char| {
                            if char == '-' {
                                return acc + 1;
                            }
                            acc
                        })
                    })
                    .parse_next(input)?;
            input.reset(&checkpoint);

            if dashes < 2 {
                return fail
                    .label("alpm-db entry name")
                    .description(concat!(
                        "Expected a package name, followed by an alpm-package-version (full or full with epoch).",
                        "\nAll components must be delimited with a dash ('-')."
                    ))
                    .parse_next(input);
            }

            // The (zero or more) dashes in the Name component.
            let dashes_till_version = dashes.saturating_sub(1);

            // Advance the parser to the dash just behind the Name component, based on the amount of
            // dashes in the Name, e.g.:
            // "example-package-1:1.0.0-1" -> "-1:1.0.0-1"
            let name =
                Name::parse_name_followed_by_version(dashes_till_version).parse_next(input)?;

            // Consume leading dash in front of FullVersion, e.g.:
            // "-1:1.0.0-1" -> "1:1.0.0-1"
            "-".parse_next(input)?;

            // Parse the FullVersion component.
            // Advance the parser to beyond the FullVersion component (which contains one dash),
            // e.g.: "1:1.0.0-1" -> ""
            let version: FullVersion =
                FullVersion::parser_until(delimiter_parser.by_ref()).parse_next(input)?;

            Ok(Self { name, version })
        };

        parser.layer("alpm-package filename")
    }
}

impl TryFrom<&Path> for EntryName {
    type Error = crate::Error;

    /// Creates an [`EntryName`] from the file name of a [`Path`].
    ///
    /// # Note
    ///
    /// The last component of the path is used as the entry name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the metadata of the directory cannot be read,
    /// - the path is not a directory,
    /// - the path does not have a file name,
    /// - the file name cannot be converted to a string,
    /// - or the file name cannot be parsed as a valid entry name (see [`FromStr`] implementation).
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let metadata = symlink_metadata(value).map_err(|source| crate::Error::IoPath {
            path: value.to_path_buf(),
            context: t!("error-io-path-reading-metadata-of-directory"),
            source,
        })?;

        if !metadata.file_type().is_dir() {
            return Err(Error::IsNotADirectory {
                path: value.to_path_buf(),
            }
            .into());
        }

        let Some(file_name) = value.file_name() else {
            return Err(Error::HasNoFilename {
                path: value.to_path_buf(),
            }
            .into());
        };

        let Some(file_name_str) = file_name.to_str() else {
            return Err(Error::ContainsNonUtf8Chars {
                path: value.to_path_buf(),
            }
            .into());
        };

        Self::from_str(file_name_str)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::create_dir_all, os::unix::fs::symlink, str::FromStr};

    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    #[rstest]
    #[case::no_dash_name_full_version("example-1.0.0-1", "example".parse()?, "1.0.0-1".parse()?)]
    #[case::no_dash_name_full_version_with_epoch("example-1:1.0.0-1", "example".parse()?, "1:1.0.0-1".parse()?)]
    #[case::multi_dash_name_full_version("example-foo-1.0.0-1", "example-foo".parse()?, "1.0.0-1".parse()?)]
    #[case::multi_dash_name_full_version_with_epoch("example-foo-1:1.0.0-1", "example-foo".parse()?, "1:1.0.0-1".parse()?)]
    fn entry_name_from_str_succeeds(
        #[case] input: &str,
        #[case] name: Name,
        #[case] version: FullVersion,
    ) -> TestResult {
        let entry_name = EntryName::from_str(input)?;
        assert_eq!(entry_name.name(), &name);
        assert_eq!(entry_name.version(), &version);

        Ok(())
    }

    /// Ensures, that [`EntryName::try_from`] succeeds for valid directories.
    #[test]
    fn entry_name_try_from_path_succeeds() -> TestResult {
        let tempdir = tempdir()?;
        let entry_name_dir = tempdir.path().join("example-1.0.0-1");
        create_dir_all(&entry_name_dir)?;

        let entry_name = EntryName::try_from(entry_name_dir.as_path())?;

        assert_eq!(
            entry_name,
            EntryName::new("example".parse()?, "1.0.0-1".parse()?)
        );

        Ok(())
    }

    /// Ensures, that [`EntryName::try_from`] fails on non-existent directories.
    #[test]
    fn entry_name_try_from_path_fails_on_nonexistent_path() -> TestResult {
        let tempdir = tempdir()?;
        let entry_name_dir = tempdir.path().join("example-1.0.0-1");

        let Err(crate::Error::IoPath { .. }) = EntryName::try_from(entry_name_dir.as_path()) else {
            panic!("Expected to fail with Error::IoPath");
        };

        Ok(())
    }

    /// Ensures, that [`EntryName::try_from`] fails on the path being a symlink.
    #[test]
    fn entry_name_try_from_path_fails_on_symlink() -> TestResult {
        let tempdir = tempdir()?;
        let target_dir = tempdir.path().join("malicious");
        let entry_name_dir = tempdir.path().join("example-1.0.0-1");
        create_dir_all(&target_dir)?;
        symlink(&target_dir, &entry_name_dir)?;

        let Err(crate::Error::Database(Error::IsNotADirectory { .. })) =
            EntryName::try_from(entry_name_dir.as_path())
        else {
            panic!("Expected to fail with Error::Database(Error::IsNotADirectory)");
        };

        Ok(())
    }

    /// Ensures, that [`EntryName::try_from`] fails on the path terminating with `..`.
    #[test]
    fn entry_name_try_from_path_fails_on_no_filename() -> TestResult {
        let tempdir = tempdir()?;
        let entry_name_dir = tempdir.path().join("example-1.0.0-1").join("..");
        create_dir_all(&entry_name_dir)?;

        let Err(crate::Error::Database(Error::HasNoFilename { .. })) =
            EntryName::try_from(entry_name_dir.as_path())
        else {
            panic!("Expected to fail with Error::Database(Error::HasNoFilename)");
        };

        Ok(())
    }
}
