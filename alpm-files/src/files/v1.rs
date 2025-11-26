//! The representation of [alpm-files] files (version 1).
//!
//! [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html

use std::{collections::HashSet, fmt::Display, path::PathBuf, str::FromStr};

use alpm_common::relative_files;
use fluent_i18n::t;
use winnow::{
    ModalResult,
    Parser,
    ascii::{line_ending, multispace0, space1, till_line_ending},
    combinator::{alt, cut_err, eof, fail, not, opt, repeat, terminated},
    error::{StrContext, StrContextValue},
};

use crate::files::{Error, FilesStyle, FilesStyleToString};

/// The raw data section in [alpm-files] data.
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Debug)]
pub(crate) struct FilesSection(Vec<PathBuf>);

impl FilesSection {
    /// The section keyword ("%FILES%").
    pub(crate) const SECTION_KEYWORD: &str = "%FILES%";

    /// Recognizes a [`PathBuf`] in a single line.
    ///
    /// # Note
    ///
    /// This parser only consumes till the end of a line and attempts to parse a [`PathBuf`] from
    /// it. Trailing line endings and EOF are handled.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`PathBuf`] cannot be created from the line, or something other than a
    /// line ending or EOF is encountered afterwards.
    fn parse_path(input: &mut &str) -> ModalResult<PathBuf> {
        // Parse until the end of the line and attempt conversion to PathBuf.
        // Make sure that the string is not empty!
        alt((
            (space1, line_ending)
                .take()
                .and_then(cut_err(fail))
                .context(StrContext::Expected(StrContextValue::Description(
                    "relative path not consisting of whitespaces and/or tabs",
                ))),
            till_line_ending,
        ))
        .verify(|s: &str| !s.is_empty())
        .context(StrContext::Label("relative path"))
        .parse_to()
        .parse_next(input)
    }

    /// Recognizes [alpm-files] data in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - `input` is not empty and the first line does not contain the required section header
    ///   "%FILES%",
    /// - or there are lines following the section header, but they cannot be parsed as a [`Vec`] of
    ///   [`PathBuf`].
    ///
    /// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
    pub(crate) fn parser(input: &mut &str) -> ModalResult<Self> {
        // Return early if the input is empty.
        // This may be the case in an alpm-db-files file if a package contains no files.
        if input.is_empty() {
            return Ok(Self(Vec::new()));
        }

        // Consume the required section header "%FILES%".
        // Optionally consume one following line ending.
        cut_err(terminated(Self::SECTION_KEYWORD, alt((line_ending, eof))))
            .context(StrContext::Label("alpm-files section header"))
            .context(StrContext::Expected(StrContextValue::Description(
                Self::SECTION_KEYWORD,
            )))
            .parse_next(input)?;

        // Return early if there is only the section header.
        // This may be the case in an alpm-repo-files file if a package contains no files.
        if input.is_empty() {
            return Ok(Self(Vec::new()));
        }

        // Consider all following lines as paths.
        // Optionally consume one following line ending.
        let paths: Vec<PathBuf> =
            repeat(0.., terminated(Self::parse_path, alt((line_ending, eof)))).parse_next(input)?;

        // Consume any trailing whitespaces or new lines.
        multispace0.parse_next(input)?;

        // Fail if there are any further non-whitespace characters.
        let _opt: Option<&str> =
            opt(not(eof)
                .take()
                .and_then(cut_err(fail).context(StrContext::Expected(
                    StrContextValue::Description("no further path after newline"),
                ))))
            .parse_next(input)?;

        Ok(Self(paths))
    }

    /// Returns the paths.
    pub fn paths(self) -> Vec<PathBuf> {
        self.0
    }
}

/// A collection of paths that are invalid in the context of a [`FilesV1`].
///
/// A [`FilesV1`] must not contain duplicate paths or (non top-level) paths that do not have a
/// parent in the same set of paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FilesV1PathErrors {
    pub(crate) absolute: HashSet<PathBuf>,
    pub(crate) without_parent: HashSet<PathBuf>,
    pub(crate) duplicate: HashSet<PathBuf>,
}

impl FilesV1PathErrors {
    /// Creates a new [`FilesV1PathErrors`].
    pub(crate) fn new() -> Self {
        Self {
            absolute: HashSet::new(),
            without_parent: HashSet::new(),
            duplicate: HashSet::new(),
        }
    }

    /// Adds a new absolute path.
    pub(crate) fn add_absolute(&mut self, path: PathBuf) -> bool {
        self.absolute.insert(path)
    }

    /// Adds a new (non top-level) path that does not have a parent.
    pub(crate) fn add_without_parent(&mut self, path: PathBuf) -> bool {
        self.without_parent.insert(path)
    }

    /// Adds a new duplicate path.
    pub(crate) fn add_duplicate(&mut self, path: PathBuf) -> bool {
        self.duplicate.insert(path)
    }

    /// Fails if `self` tracks any invalid paths.
    pub(crate) fn fail(&self) -> Result<(), Error> {
        if !(self.absolute.is_empty()
            && self.without_parent.is_empty()
            && self.duplicate.is_empty())
        {
            Err(Error::InvalidFilesPaths {
                message: self.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

impl Display for FilesV1PathErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_invalid_set(
            f: &mut std::fmt::Formatter<'_>,
            message: String,
            set: &HashSet<PathBuf>,
        ) -> std::fmt::Result {
            if !set.is_empty() {
                writeln!(f, "{message}:")?;
                let mut set = set.iter().collect::<Vec<_>>();
                set.sort();
                for path in set.iter() {
                    writeln!(f, "{}", path.as_path().display())?;
                }
            }
            Ok(())
        }

        write_invalid_set(f, t!("filesv1-path-errors-absolute-paths"), &self.absolute)?;
        write_invalid_set(
            f,
            t!("filesv1-path-errors-paths-without-a-parent"),
            &self.without_parent,
        )?;
        write_invalid_set(
            f,
            t!("filesv1-path-errors-duplicate-paths"),
            &self.duplicate,
        )?;

        Ok(())
    }
}

/// The representation of [alpm-files] data (version 1).
///
/// [alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FilesV1(Vec<PathBuf>);

impl AsRef<[PathBuf]> for FilesV1 {
    /// Returns a reference to the inner [`Vec`] of [`PathBuf`]s.
    fn as_ref(&self) -> &[PathBuf] {
        &self.0
    }
}

impl FilesStyleToString for FilesV1 {
    /// Returns the [`String`] representation of the [`FilesV1`].
    ///
    /// The formatting of the returned string depends on the provided [`FilesStyle`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use alpm_files::files::{FilesStyle, FilesStyleToString, FilesV1};
    ///
    /// # fn main() -> Result<(), alpm_files::files::Error> {
    /// // An empty alpm-db-files.
    /// let expected = "";
    /// let files = FilesV1::try_from(Vec::new())?;
    /// assert_eq!(files.to_string(FilesStyle::Db), expected);
    ///
    /// // An alpm-db-files with entries.
    /// let expected = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    ///
    /// "#;
    /// let files = FilesV1::try_from(vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ])?;
    /// assert_eq!(files.to_string(FilesStyle::Db), expected);
    ///
    /// // An empty alpm-repo-files.
    /// let expected = "%FILES%\n";
    /// let files = FilesV1::try_from(Vec::new())?;
    /// assert_eq!(files.to_string(FilesStyle::Repo), expected);
    ///
    /// // An alpm-repo-files with entries.
    /// let expected = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = FilesV1::try_from(vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ])?;
    /// assert_eq!(files.to_string(FilesStyle::Repo), expected);
    /// # Ok(())
    /// # }
    /// ```
    fn to_string(&self, style: FilesStyle) -> String {
        let mut output = String::new();

        // Return empty string if no paths are tracked and the targeted file format requires no
        // section header.
        if self.0.is_empty() && matches!(style, FilesStyle::Db) {
            return output;
        }

        output.push_str(FilesSection::SECTION_KEYWORD);
        output.push('\n');

        if self.0.is_empty() && matches!(style, FilesStyle::Repo) {
            return output;
        }

        for path in self.0.iter() {
            output.push_str(&format!("{}", path.to_string_lossy()));
            output.push('\n');
        }

        // The alpm-db-files style adds a trailing newline.
        if matches!(style, FilesStyle::Db) {
            output.push('\n');
        }

        output
    }
}

impl FromStr for FilesV1 {
    type Err = Error;

    /// Creates a new [`FilesV1`] from a string slice.
    ///
    /// # Note
    ///
    /// Delegates to the [`TryFrom`] [`Vec`] of [`PathBuf`] implementation, after the string slice
    /// has been parsed as a [`Vec`] of [`PathBuf`].
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - `value` is not empty and the first line does not contain the section header ("%FILES%"),
    /// - there are lines following the section header, but they cannot be parsed as a [`Vec`] of
    ///   [`PathBuf`],
    /// - or [`Self::try_from`] [`Vec`] of [`PathBuf`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{path::PathBuf, str::FromStr};
    ///
    /// use alpm_files::files::FilesV1;
    /// use winnow::Parser;
    ///
    /// # fn main() -> Result<(), alpm_files::files::Error> {
    /// # let expected: Vec<PathBuf> = Vec::new();
    /// // No files according to alpm-db-files.
    /// let data = "";
    /// let files = FilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// // No files according to alpm-repo-files.
    /// let data = "%FILES%";
    /// let files = FilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    /// let data = "%FILES%\n";
    /// let files = FilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// # let expected: Vec<PathBuf> = vec![
    /// #     PathBuf::from("usr/"),
    /// #     PathBuf::from("usr/bin/"),
    /// #     PathBuf::from("usr/bin/foo"),
    /// # ];
    /// // Files according to alpm-repo-files.
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo"#;
    /// let files = FilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// // Files according to alpm-db-files.
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = FilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected.as_slice());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let files_section = FilesSection::parser.parse(s)?;
        FilesV1::try_from(files_section.paths())
    }
}

impl TryFrom<PathBuf> for FilesV1 {
    type Error = Error;

    /// Creates a new [`FilesV1`] from all files and directories in a directory.
    ///
    /// # Note
    ///
    /// Delegates to [`alpm_common::relative_files`] to get a sorted list of all files and
    /// directories in the directory `value` (relative to `value`).
    /// Afterwards, tries to construct a [`FilesV1`] from this list.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - [`alpm_common::relative_files`] fails,
    /// - or [`TryFrom`] [`Vec`] of [`PathBuf`] for [`FilesV1`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::{File, create_dir_all},
    ///     path::PathBuf,
    /// };
    ///
    /// use alpm_files::files::FilesV1;
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let temp_dir = tempdir()?;
    /// let path = temp_dir.path();
    /// create_dir_all(path.join("usr/bin/"))?;
    /// File::create(path.join("usr/bin/foo"))?;
    ///
    /// let files = FilesV1::try_from(path.to_path_buf())?;
    /// assert_eq!(
    ///     files.as_ref(),
    ///     vec![
    ///         PathBuf::from("usr/"),
    ///         PathBuf::from("usr/bin/"),
    ///         PathBuf::from("usr/bin/foo")
    ///     ]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        FilesV1::try_from(relative_files(value, &[])?)
    }
}

impl TryFrom<Vec<PathBuf>> for FilesV1 {
    type Error = Error;

    /// Creates a new [`FilesV1`] from a [`Vec`] of [`PathBuf`].
    ///
    /// The provided `value` is sorted and checked for non top-level paths without a parent, as well
    /// as any duplicate paths.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `value` contains absolute paths,
    /// - `value` contains (non top-level) paths without a parent directory present in `value`,
    /// - or `value` contains duplicate paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use alpm_files::files::FilesV1;
    ///
    /// # fn main() -> Result<(), alpm_files::files::Error> {
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ];
    /// let files = FilesV1::try_from(paths)?;
    ///
    /// // Absolute paths are not allowed.
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("/usr/"),
    ///     PathBuf::from("/usr/bin/"),
    ///     PathBuf::from("/usr/bin/foo"),
    /// ];
    /// assert!(FilesV1::try_from(paths).is_err());
    ///
    /// // Every path (excluding top-level paths) must have a parent.
    /// let paths: Vec<PathBuf> = vec![PathBuf::from("usr/bin/"), PathBuf::from("usr/bin/foo")];
    /// assert!(FilesV1::try_from(paths).is_err());
    ///
    /// // Every path must be unique.
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ];
    /// assert!(FilesV1::try_from(paths).is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: Vec<PathBuf>) -> Result<Self, Self::Error> {
        let mut paths = value;
        paths.sort_unstable();

        let mut errors = FilesV1PathErrors::new();
        let mut path_set = HashSet::new();
        let empty_parent = PathBuf::from("");
        let root_parent = PathBuf::from("/");

        for path in paths.iter() {
            let path = path.as_path();

            // Add absolute paths as errors.
            if path.is_absolute() {
                errors.add_absolute(path.to_path_buf());
            }

            // Add non top-level, relative paths without a parent as errors.
            if let Some(parent) = path.parent() {
                if parent != empty_parent && parent != root_parent && !path_set.contains(parent) {
                    errors.add_without_parent(path.to_path_buf());
                }
            }

            // Add duplicates as errors.
            if !path_set.insert(path) {
                errors.add_duplicate(path.to_path_buf());
            }
        }

        errors.fail()?;

        Ok(Self(paths))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, create_dir_all};

    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    /// Ensures that a [`FilesV1`] can be successfully created from a directory.
    #[test]
    fn filesv1_try_from_pathbuf_succeeds() -> TestResult {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();
        create_dir_all(path.join("usr/bin/"))?;
        File::create(path.join("usr/bin/foo"))?;

        let files = FilesV1::try_from(path.to_path_buf())?;

        assert_eq!(
            files.as_ref(),
            vec![
                PathBuf::from("usr/"),
                PathBuf::from("usr/bin/"),
                PathBuf::from("usr/bin/foo")
            ]
        );

        Ok(())
    }

    #[rstest]
    #[case::dirs_and_files(vec![PathBuf::from("usr/"), PathBuf::from("usr/bin/"), PathBuf::from("usr/bin/foo")], 3)]
    #[case::empty(Vec::new(), 0)]
    fn filesv1_try_from_pathbufs_succeeds(
        #[case] paths: Vec<PathBuf>,
        #[case] len: usize,
    ) -> TestResult {
        let files = FilesV1::try_from(paths)?;

        assert_eq!(files.as_ref().len(), len);

        Ok(())
    }

    #[rstest]
    #[case::absolute_paths(
        vec![
            PathBuf::from("/usr/"), PathBuf::from("/usr/bin/"), PathBuf::from("/usr/bin/foo")
        ],
        FilesV1PathErrors{
            absolute: HashSet::from_iter([
                PathBuf::from("/usr/"),
                PathBuf::from("/usr/bin/"),
                PathBuf::from("/usr/bin/foo"),
            ]),
            without_parent: HashSet::new(),
            duplicate: HashSet::new(),
        }
    )]
    #[case::without_parents(
        vec![PathBuf::from("usr/bin/"), PathBuf::from("usr/bin/foo")],
        FilesV1PathErrors{
            absolute: HashSet::new(),
            without_parent: HashSet::from_iter([
                PathBuf::from("usr/bin/"),
            ]),
            duplicate: HashSet::new(),
        }
    )]
    #[case::duplicates(
        vec![PathBuf::from("usr/"), PathBuf::from("usr/")],
        FilesV1PathErrors{
            absolute: HashSet::new(),
            without_parent: HashSet::new(),
            duplicate: HashSet::from_iter([
                PathBuf::from("usr/"),
            ]),
        }
    )]
    fn filesv1_try_from_pathbufs_fails(
        #[case] paths: Vec<PathBuf>,
        #[case] expected_errors: FilesV1PathErrors,
    ) -> TestResult {
        let result = FilesV1::try_from(paths);
        let errors = match result {
            Ok(files) => panic!(
                "Should have failed with an Error::InvalidFilesPaths, but succeeded to create a FilesV1: {files:?}"
            ),
            Err(Error::InvalidFilesPaths { message }) => message,
            Err(error) => panic!("Expected an Error::InvalidFilesPaths, but got: {error}"),
        };

        eprintln!("{errors}");
        assert_eq!(errors, expected_errors.to_string());

        Ok(())
    }
}
