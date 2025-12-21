//! The representation of [alpm-db-files] files (version 1).
//!
//! [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html

use std::{collections::HashSet, fmt::Display, path::PathBuf, str::FromStr};

use alpm_common::relative_files;
use alpm_types::{Md5Checksum, RelativeFilePath, RelativePath};
use fluent_i18n::t;
use winnow::{
    ModalResult,
    Parser,
    ascii::{line_ending, multispace0, space1, till_line_ending},
    combinator::{alt, cut_err, eof, fail, not, opt, repeat, separated_pair, terminated},
    error::{StrContext, StrContextValue},
    token::take_while,
};

use crate::files::Error;

/// The raw data section in [alpm-db-files] data.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Debug)]
pub(crate) struct FilesSection(Vec<RelativePath>);

impl FilesSection {
    /// The section keyword ("%FILES%").
    pub(crate) const SECTION_KEYWORD: &str = "%FILES%";

    /// Recognizes a [`RelativePath`] in a single line.
    ///
    /// # Note
    ///
    /// This parser only consumes till the end of a line and attempts to parse a [`RelativePath`]
    /// from it. Trailing line endings and EOF are handled.
    ///
    /// # Errors
    ///
    /// Returns an error if a [`RelativePath`] cannot be created from the line, or something other
    /// than a line ending or EOF is encountered afterwards.
    fn parse_path(input: &mut &str) -> ModalResult<RelativePath> {
        // Parse until the end of the line and attempt conversion to RelativePath.
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

    /// Recognizes [alpm-db-files] data in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error, if
    ///
    /// - `input` is not empty and the first line does not contain the required section header
    ///   "%FILES%",
    /// - or there are lines following the section header, but they cannot be parsed as a [`Vec`] of
    ///   [`RelativePath`].
    ///
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    pub(crate) fn parser(input: &mut &str) -> ModalResult<Self> {
        // Return early if the input is empty.
        // This may be the case in an alpm-db-files file if a package contains no files.
        if input.is_empty() {
            return Ok(Self(Vec::new()));
        }

        // Consume the required section header "%FILES%".
        // Optionally consume one following line ending.
        cut_err(terminated(Self::SECTION_KEYWORD, alt((line_ending, eof))))
            .context(StrContext::Label("alpm-db-files section header"))
            .context(StrContext::Expected(StrContextValue::Description(
                Self::SECTION_KEYWORD,
            )))
            .parse_next(input)?;

        // Return early if there is only the section header.
        if input.is_empty() {
            return Ok(Self(Vec::new()));
        }

        // Consider all following lines as paths.
        // Optionally consume one following line ending.
        let paths: Vec<RelativePath> =
            repeat(0.., terminated(Self::parse_path, alt((line_ending, eof)))).parse_next(input)?;

        // Consume any trailing whitespaces or new lines.
        multispace0.parse_next(input)?;

        // If a BACKUP section follows, leave the rest of the input to that parser.
        if input.is_empty() || input.starts_with(BackupSection::SECTION_KEYWORD) {
            return Ok(Self(paths));
        }

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
        self.0.into_iter().map(RelativePath::into_inner).collect()
    }
}

/// A path that should be tracked for backup together with its checksum.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct BackupEntry {
    /// The path to the file that is backed up.
    pub path: RelativeFilePath,
    /// The MD5 checksum of the backed up file as stored in the package.
    pub md5: Md5Checksum,
}

impl BackupEntry {
    /// Recognizes a single backup entry.
    ///
    /// Each entry consists of a relative path, a tab, and a 32 character hexadecimal MD5 digest.
    ///
    /// # Note
    ///
    /// As a special edge case, the parser does not fail if it encounters the keyword `(null)`
    /// instead of an MD-5 hash digest. The `(null)` keyword may be present in [alpm-db-files]
    /// files, due to how [pacman] handles package metadata with invalid `backup` entries.
    /// Specifically, if a package is created from a [PKGBUILD] that tracks files in its `backup`
    /// array, which are not in the package, then pacman creates an invalid `%BACKUP%` entry upon
    /// installation of the package, instead of skipping the invalid entries.
    ///
    /// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
    /// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
    /// [pacman]: https://man.archlinux.org/man/pacman.8
    pub(crate) fn parser(input: &mut &str) -> ModalResult<Option<Self>> {
        let mut line = till_line_ending.parse_next(input)?;
        separated_pair(
            take_while(1.., |c: char| c != '\t' && c != '\n' && c != '\r')
                .verify(|s: &str| !s.chars().all(|c| c.is_whitespace()))
                .context(StrContext::Label("relative path"))
                .parse_to(),
            '\t',
            alt((
                // Some alpm-db-files metadata may contain "(null)" instead of a hash digest for a
                // backup entry. This happens if a file that is not contained in a
                // package is added to the package's PKGBUILD and pacman adds an (unused) backup
                // entry for it nonetheless.
                "(null)".value(None),
                Md5Checksum::parser.map(Some),
            )),
        )
        .map(|(path, md5)| md5.map(|md5| BackupEntry { path, md5 }))
        .parse_next(&mut line)
    }
}

/// The raw backup section in [alpm-db-files] data.
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Debug)]
pub(crate) struct BackupSection(Vec<BackupEntry>);

impl BackupSection {
    /// The section keyword ("%BACKUP%").
    pub(crate) const SECTION_KEYWORD: &str = "%BACKUP%";

    /// Recognizes the optional `%BACKUP%` section.
    ///
    /// # Errors
    ///
    /// Returns an error if the section header is missing or malformed, or if any entry cannot be
    /// parsed.
    pub(crate) fn parser(input: &mut &str) -> ModalResult<Self> {
        cut_err(terminated(Self::SECTION_KEYWORD, alt((line_ending, eof))))
            .context(StrContext::Label("alpm-db-files backup section header"))
            .context(StrContext::Expected(StrContextValue::Description(
                Self::SECTION_KEYWORD,
            )))
            .parse_next(input)?;

        if input.is_empty() {
            return Ok(Self(Vec::new()));
        }

        let entries: Vec<BackupEntry> = repeat(
            0..,
            terminated(BackupEntry::parser, alt((line_ending, eof))),
        )
        .map(|entries: Vec<Option<BackupEntry>>| entries.into_iter().flatten().collect::<Vec<_>>())
        .parse_next(input)?;

        // Consume any trailing whitespaces or new lines.
        multispace0.parse_next(input)?;

        // Fail if there are any further non-whitespace characters.
        let _opt: Option<&str> =
            opt(not(eof)
                .take()
                .and_then(cut_err(fail).context(StrContext::Expected(
                    StrContextValue::Description("no further backup entry after newline"),
                ))))
            .parse_next(input)?;

        Ok(Self(entries))
    }

    /// Returns the parsed entries.
    pub fn entries(self) -> Vec<BackupEntry> {
        self.0
    }
}

/// A collection of paths that are invalid in the context of a [`DbFilesV1`].
///
/// A [`DbFilesV1`] must not contain duplicate paths or (non top-level) paths that do not have a
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

/// A collection of invalid backup entries for a [`DbFilesV1`].
///
/// A [`DbFilesV1`] must not contain duplicate backup paths or backup paths that are not listed in
/// the `%FILES%` section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BackupV1Errors {
    pub(crate) not_in_files: HashSet<RelativeFilePath>,
    pub(crate) duplicate: HashSet<RelativeFilePath>,
}

impl BackupV1Errors {
    /// Creates a new [`BackupV1Errors`].
    pub(crate) fn new() -> Self {
        Self {
            not_in_files: HashSet::new(),
            duplicate: HashSet::new(),
        }
    }

    /// Adds a new path that is not tracked by the `%FILES%` section.
    pub(crate) fn add_not_in_files(&mut self, path: RelativeFilePath) -> bool {
        self.not_in_files.insert(path)
    }

    /// Adds a new duplicate path.
    pub(crate) fn add_duplicate(&mut self, path: RelativeFilePath) -> bool {
        self.duplicate.insert(path)
    }

    /// Fails if `self` tracks any invalid backup entries.
    pub(crate) fn fail(&self) -> Result<(), Error> {
        if !(self.not_in_files.is_empty() && self.duplicate.is_empty()) {
            Err(Error::InvalidBackupEntries {
                message: self.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

impl Display for BackupV1Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_invalid_set(
            f: &mut std::fmt::Formatter<'_>,
            message: String,
            set: &HashSet<RelativeFilePath>,
        ) -> std::fmt::Result {
            if !set.is_empty() {
                writeln!(f, "{message}:")?;
                let mut set = set.iter().collect::<Vec<_>>();
                set.sort_by(|a, b| a.inner().cmp(b.inner()));
                for path in set.iter() {
                    writeln!(f, "{path}")?;
                }
            }
            Ok(())
        }

        write_invalid_set(
            f,
            t!("backupv1-errors-not-in-files-section"),
            &self.not_in_files,
        )?;
        write_invalid_set(f, t!("backupv1-errors-duplicate-paths"), &self.duplicate)?;

        Ok(())
    }
}

/// The representation of [alpm-db-files] data (version 1).
///
/// [alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
#[derive(Clone, Debug, serde::Serialize)]
pub struct DbFilesV1 {
    files: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    backup: Vec<BackupEntry>,
}

impl AsRef<[PathBuf]> for DbFilesV1 {
    /// Returns a reference to the inner [`Vec`] of [`PathBuf`]s.
    fn as_ref(&self) -> &[PathBuf] {
        &self.files
    }
}

impl DbFilesV1 {
    /// Returns the backup entries tracked for this file listing.
    pub fn backups(&self) -> &[BackupEntry] {
        &self.backup
    }

    fn try_from_parts(
        mut paths: Vec<PathBuf>,
        mut backup: Vec<BackupEntry>,
    ) -> Result<Self, Error> {
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
            if !path_set.insert(path.to_path_buf()) {
                errors.add_duplicate(path.to_path_buf());
            }
        }

        errors.fail()?;

        let mut backup_errors = BackupV1Errors::new();
        let mut backup_set: HashSet<RelativeFilePath> = HashSet::new();

        for entry in backup.iter() {
            if !path_set.contains(entry.path.inner()) {
                backup_errors.add_not_in_files(entry.path.clone());
            }

            if !backup_set.insert(entry.path.clone()) {
                backup_errors.add_duplicate(entry.path.clone());
            }
        }

        backup_errors.fail()?;

        backup.sort_unstable_by(|a, b| a.path.inner().cmp(b.path.inner()));

        Ok(Self {
            files: paths,
            backup,
        })
    }
}

impl Display for DbFilesV1 {
    /// Returns the [`String`] representation of the [`DbFilesV1`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use alpm_db::files::DbFilesV1;
    ///
    /// # fn main() -> Result<(), alpm_db::files::Error> {
    /// // An empty alpm-db-files.
    /// let expected = "";
    /// let files = DbFilesV1::try_from(Vec::new())?;
    /// assert_eq!(files.to_string(), expected);
    ///
    /// // An alpm-db-files with entries.
    /// let expected = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    ///
    /// "#;
    /// let files = DbFilesV1::try_from(vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ])?;
    /// assert_eq!(files.to_string(), expected);
    /// # Ok(())
    /// # }
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Return empty string if no paths or backups exist and no section is required.
        if self.files.is_empty() && self.backup.is_empty() {
            return Ok(());
        }

        // %FILES% section
        writeln!(f, "{}", FilesSection::SECTION_KEYWORD)?;

        for path in &self.files {
            writeln!(f, "{}", path.to_string_lossy())?;
        }

        // The spec requires a *trailing* blank line after %FILES%
        writeln!(f)?;

        // Optional %BACKUP% section
        if !self.backup.is_empty() {
            writeln!(f, "{}", BackupSection::SECTION_KEYWORD)?;

            for entry in &self.backup {
                writeln!(f, "{}\t{}", entry.path, entry.md5)?;
            }
        }

        Ok(())
    }
}

impl FromStr for DbFilesV1 {
    type Err = Error;

    /// Creates a new [`DbFilesV1`] from a string slice.
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
    /// use alpm_db::files::DbFilesV1;
    /// use winnow::Parser;
    ///
    /// # fn main() -> Result<(), alpm_db::files::Error> {
    /// # let expected: Vec<PathBuf> = Vec::new();
    /// // No files according to alpm-db-files.
    /// let data = "";
    /// let files = DbFilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// // No files according to alpm-db-files.
    /// let data = "%FILES%";
    /// let files = DbFilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    /// let data = "%FILES%\n";
    /// let files = DbFilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// # let expected: Vec<PathBuf> = vec![
    /// #     PathBuf::from("usr/"),
    /// #     PathBuf::from("usr/bin/"),
    /// #     PathBuf::from("usr/bin/foo"),
    /// # ];
    /// // DbFiles according to alpm-db-files.
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo"#;
    /// let files = DbFilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected);
    ///
    /// // DbFiles according to alpm-db-files.
    /// let data = r#"%FILES%
    /// usr/
    /// usr/bin/
    /// usr/bin/foo
    /// "#;
    /// let files = DbFilesV1::from_str(data)?;
    /// # assert_eq!(files.as_ref(), expected.as_slice());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (files_section, backup_section) =
            (|input: &mut &str| -> ModalResult<(FilesSection, BackupSection)> {
                let files_section = FilesSection::parser.parse_next(input)?;
                let backup_section = if input.is_empty() {
                    BackupSection(Vec::new())
                } else {
                    BackupSection::parser.parse_next(input)?
                };
                Ok((files_section, backup_section))
            })
            .parse(s)?;

        DbFilesV1::try_from_parts(files_section.paths(), backup_section.entries())
    }
}

impl TryFrom<PathBuf> for DbFilesV1 {
    type Error = Error;

    /// Creates a new [`DbFilesV1`] from all files and directories in a directory.
    ///
    /// # Note
    ///
    /// Delegates to [`alpm_common::relative_files`] to get a sorted list of all files and
    /// directories in the directory `value` (relative to `value`).
    /// Afterwards, tries to construct a [`DbFilesV1`] from this list.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - [`alpm_common::relative_files`] fails,
    /// - or [`TryFrom`] [`Vec`] of [`PathBuf`] for [`DbFilesV1`] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::{File, create_dir_all},
    ///     path::PathBuf,
    /// };
    ///
    /// use alpm_db::files::DbFilesV1;
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let temp_dir = tempdir()?;
    /// let path = temp_dir.path();
    /// create_dir_all(path.join("usr/bin/"))?;
    /// File::create(path.join("usr/bin/foo"))?;
    ///
    /// let files = DbFilesV1::try_from(path.to_path_buf())?;
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
        DbFilesV1::try_from_parts(relative_files(value, &[])?, Vec::new())
    }
}

impl TryFrom<Vec<PathBuf>> for DbFilesV1 {
    type Error = Error;

    /// Creates a new [`DbFilesV1`] from a [`Vec`] of [`PathBuf`].
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
    /// use alpm_db::files::DbFilesV1;
    ///
    /// # fn main() -> Result<(), alpm_db::files::Error> {
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ];
    /// let files = DbFilesV1::try_from(paths)?;
    ///
    /// // Absolute paths are not allowed.
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("/usr/"),
    ///     PathBuf::from("/usr/bin/"),
    ///     PathBuf::from("/usr/bin/foo"),
    /// ];
    /// assert!(DbFilesV1::try_from(paths).is_err());
    ///
    /// // Every path (excluding top-level paths) must have a parent.
    /// let paths: Vec<PathBuf> = vec![PathBuf::from("usr/bin/"), PathBuf::from("usr/bin/foo")];
    /// assert!(DbFilesV1::try_from(paths).is_err());
    ///
    /// // Every path must be unique.
    /// let paths: Vec<PathBuf> = vec![
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/"),
    ///     PathBuf::from("usr/bin/"),
    ///     PathBuf::from("usr/bin/foo"),
    /// ];
    /// assert!(DbFilesV1::try_from(paths).is_err());
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(value: Vec<PathBuf>) -> Result<Self, Self::Error> {
        DbFilesV1::try_from_parts(value, Vec::new())
    }
}

impl TryFrom<(Vec<PathBuf>, Vec<BackupEntry>)> for DbFilesV1 {
    type Error = Error;

    /// Creates a new [`DbFilesV1`] from a [`Vec`] of [`PathBuf`] and backup entries.
    fn try_from(value: (Vec<PathBuf>, Vec<BackupEntry>)) -> Result<Self, Self::Error> {
        let (paths, backup) = value;
        DbFilesV1::try_from_parts(paths, backup)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{File, create_dir_all},
        str::FromStr,
    };

    use alpm_types::{Md5Checksum, RelativeFilePath};
    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    /// Ensures that a [`DbFilesV1`] can be successfully created from a directory.
    #[test]
    fn filesv1_try_from_pathbuf_succeeds() -> TestResult {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();
        create_dir_all(path.join("usr/bin/"))?;
        File::create(path.join("usr/bin/foo"))?;

        let files = DbFilesV1::try_from(path.to_path_buf())?;

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
        let files = DbFilesV1::try_from(paths)?;

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
        let result = DbFilesV1::try_from(paths);
        let errors = match result {
            Ok(files) => panic!(
                "Should have failed with an Error::InvalidFilesPaths, but succeeded to create a DbFilesV1: {files:?}"
            ),
            Err(Error::InvalidFilesPaths { message }) => message,
            Err(error) => panic!("Expected an Error::InvalidFilesPaths, but got: {error}"),
        };

        eprintln!("{errors}");
        assert_eq!(errors, expected_errors.to_string());

        Ok(())
    }

    #[test]
    fn filesv1_try_from_paths_and_backups_succeeds() -> TestResult {
        let paths = vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo"),
        ];
        let backup = vec![BackupEntry {
            path: RelativeFilePath::from_str("usr/bin/foo")?,
            md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e")?,
        }];

        let files = DbFilesV1::try_from((paths, backup))?;

        assert_eq!(files.backups().len(), 1);

        Ok(())
    }

    #[rstest]
    #[case::backup_not_in_files(
        vec![PathBuf::from("usr/")],
        vec![BackupEntry {
            path: RelativeFilePath::from_str("usr/bin/foo").unwrap(),
            md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e").unwrap(),
        }],
        BackupV1Errors{
            not_in_files: HashSet::from_iter([RelativeFilePath::from_str("usr/bin/foo").unwrap()]),
            duplicate: HashSet::new(),
        }
    )]
    #[case::duplicate_backup_entries(
        vec![
            PathBuf::from("usr/"),
            PathBuf::from("usr/bin/"),
            PathBuf::from("usr/bin/foo")
        ],
        vec![
            BackupEntry {
                path: RelativeFilePath::from_str("usr/bin/foo").unwrap(),
                md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e").unwrap(),
            },
            BackupEntry {
                path: RelativeFilePath::from_str("usr/bin/foo").unwrap(),
                md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e").unwrap(),
            }
        ],
        BackupV1Errors{
            not_in_files: HashSet::new(),
            duplicate: HashSet::from_iter([RelativeFilePath::from_str("usr/bin/foo").unwrap()]),
        }
    )]
    fn filesv1_try_from_paths_and_backups_fails(
        #[case] paths: Vec<PathBuf>,
        #[case] backup: Vec<BackupEntry>,
        #[case] expected_errors: BackupV1Errors,
    ) -> TestResult {
        let result = DbFilesV1::try_from((paths, backup));
        let errors = match result {
            Ok(files) => panic!(
                "Should have failed with an Error::InvalidBackupEntries, but succeeded to create a DbFilesV1: {files:?}"
            ),
            Err(Error::InvalidBackupEntries { message }) => message,
            Err(error) => panic!("Expected an Error::InvalidBackupEntries, but got: {error}"),
        };

        eprintln!("{errors}");
        assert_eq!(errors, expected_errors.to_string());

        Ok(())
    }

    #[test]
    fn filesv1_from_str_rejects_absolute_paths() -> TestResult {
        let data = "%FILES%\n/usr/bin/foo\n";

        match DbFilesV1::from_str(data) {
            Err(Error::ParseError(_)) => Ok(()),
            Err(error) => panic!("expected ParseError, got {error}"),
            Ok(files) => panic!("expected parse failure, got {files:?}"),
        }
    }

    #[test]
    fn filesv1_from_str_skips_null_backup_entries() -> TestResult {
        let data = r#"%FILES%
etc/
etc/foo/
etc/foo/foo.conf

%BACKUP%
etc/foo/foo.conf	d41d8cd98f00b204e9800998ecf8427e
etc/foo/bar.conf	(null)
"#;

        let files = DbFilesV1::from_str(data)?;

        assert_eq!(
            files.backups(),
            &[BackupEntry {
                path: RelativeFilePath::from_str("etc/foo/foo.conf")?,
                md5: Md5Checksum::from_str("d41d8cd98f00b204e9800998ecf8427e")?
            }]
        );

        Ok(())
    }
}
