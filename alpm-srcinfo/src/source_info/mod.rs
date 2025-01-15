//! Contains the second parsing and linting pass.
//!
//! The raw representation from the [`parser`](crate::parser) module is brought into a proper
//! struct-based representation that fully represents the SRCINFO data (apart from comments and
//! empty lines).
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use winnow::Parser;

use crate::{
    error::{Error, SourceInfoError, SourceInfoErrors},
    parser::SourceInfoContent,
    source_info::{package::Package, package_base::PackageBase},
};

mod lints;
pub mod package;
pub mod package_base;

/// The representation of SRCINFO data.
///
/// Provides access to a [`PackageBase`] which tracks all data in a `pkgbase` section and a list of
/// [`Package`] instances that provide the accumulated data of all `pkgname` sections.
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub base: PackageBase,
    pub packages: Vec<Package>,
}

impl SourceInfo {
    /// Reads the file at the specified path and converts it into a [`SourceInfo`] struct.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be read or parsed.
    /// Returns an error array with potentially un/-recoverable errors, this needs to be explicitly
    /// handled by the user.
    pub fn from_file(path: &Path) -> Result<SourceInfoResult, Error> {
        let mut buffer = Vec::new();
        let file = File::open(path)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "opening file", err))?;
        let mut buf_reader = BufReader::new(file);
        buf_reader
            .read_to_end(&mut buffer)
            .map_err(|err| Error::IoPath(path.to_path_buf(), "reading file", err))?;

        let content = String::from_utf8(buffer)?.to_string();

        Self::from_string(&content)
    }

    /// Parses a SRCINFO file's content into a [`SourceInfo`] struct.
    ///
    /// # Error
    ///
    /// This function returns two types of errors.
    /// 1. An [`Error`] is returned if the input is, for example, invalid UTF-8 or if the input
    ///    SRCINFO file couldn't be parsed due to invalid syntax.
    /// 2. If the parsing was successful, a [`SourceInfoResult`] is returned, which wraps a possibly
    ///    invalid [`SourceInfo`] and possible [`SourceInfoErrors`]. [`SourceInfoErrors`] contains
    ///    all errors and lint/deprecation warnings that're encountered while interpreting the
    ///    SRCINFO file.
    ///
    /// ```rust
    /// use alpm_srcinfo::SourceInfo;
    /// use alpm_types::{Architecture, Name, PackageRelation};
    ///
    /// # fn main() -> Result<(), alpm_srcinfo::Error> {
    /// let source_info_data = r#"
    /// pkgbase = example
    ///     pkgver = 1.0.0
    ///     epoch = 1
    ///     pkgrel = 1
    ///     pkgdesc = A project that does something
    ///     url = https://example.org/
    ///     arch = x86_64
    ///     depends = glibc
    ///     optdepends = python: for special-python-script.py
    ///     makedepends = cmake
    ///     checkdepends = extra-test-tool
    ///
    /// pkgname = example
    ///     depends = glibc
    ///     depends = gcc-libs
    /// "#;
    ///
    /// // Parse the file. This might already error if the file cannot be parsed on a low level.
    /// let source_info_result = SourceInfo::from_string(source_info_data)?;
    /// // Make sure there're no other unrecoverable errors.
    /// let source_info = source_info_result.source_info()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_string(content: &str) -> Result<SourceInfoResult, Error> {
        // Parse the given srcinfo content.
        let parsed = SourceInfoContent::parser
            .parse(content)
            .map_err(|err| Error::ParseError(format!("{err}")))?;

        // Bring it into a proper structural representation and run linting checks.
        let (source_info, errors) = SourceInfo::from_raw(parsed);

        // If there're some errors, create a SourceInfoErrors to also capture the file content for
        // context.
        let errors = if !errors.is_empty() {
            Some(SourceInfoErrors::new(errors, content.to_string()))
        } else {
            None
        };

        Ok(SourceInfoResult {
            source_info,
            errors,
        })
    }

    /// Reads raw [`SourceInfoContent`] from a first parsing step and converts it into a
    /// [`SourceInfo`].
    ///
    /// Instead of a [`Result`] this function returns a tuple of [`SourceInfo`] and a vector of
    /// [`SourceInfoError`]s. The caller is expected to handle the vector of [`SourceInfoError`]s,
    /// which may only consist of linting errors that can be ignored.
    pub fn from_raw(content: SourceInfoContent) -> (SourceInfo, Vec<SourceInfoError>) {
        // Set the line cursor for error messages to the last line before we start with the
        // `pkgbase`
        let mut current_line = content.preceding_lines.len();
        let mut errors = Vec::new();

        // Save the number of lines in the `pkgbase` section.
        let package_base_length = content.package_base.properties.len();

        // Account for the `pkgbase` section header, which is handled separately.
        current_line += 1;
        let base = PackageBase::from_parsed(current_line, content.package_base, &mut errors);
        // Add the length of lines of the pkgbuild section.
        current_line += package_base_length;

        let mut packages = Vec::new();
        for package in content.packages {
            // Save the number of lines in the `pkgname` section.
            let package_length = package.properties.len();

            // Account for the `pkgname` section header, which is handled separately.
            current_line += 1;
            let package =
                Package::from_parsed(current_line, &base.architectures, package, &mut errors);
            // Add the number of lines of the pkgname section.
            current_line += package_length;

            packages.push(package);
        }

        (SourceInfo { base, packages }, errors)
    }
}

/// Wraps the outcome of [`SourceInfo::from_string`].
///
/// While building a [`SourceInfo`] from raw [`SourceInfoContent`], errors as well as deprecation
/// and linter warnings may be encountered.
///
/// In case no errors are encountered, the resulting [`SourceInfo`] may be accessed via
/// [`SourceInfoResult::source_info`].
#[derive(Debug, Clone)]
pub struct SourceInfoResult {
    source_info: SourceInfo,
    errors: Option<SourceInfoErrors>,
}

impl SourceInfoResult {
    /// Returns the [`SourceInfo`] as long as no critical errors have been encountered.
    ///
    /// # Errors
    ///
    /// Returns an error if any kind of unrecoverable logic error is encountered, such as missing
    /// properties
    pub fn source_info(self) -> Result<SourceInfo, Error> {
        if let Some(errors) = self.errors {
            errors.check_unrecoverable_errors()?;
        }

        Ok(self.source_info)
    }

    /// Returns the generated [`SourceInfo`] regardless of whether there're any errors or not.
    ///
    /// # Warning
    ///
    /// This SourceInfo struct may be incomplete, could contain invalid information and/or invalid
    /// default values!
    ///
    /// Only use this if you know what you're doing and if you want to do stuff like manual
    /// auto-correction.
    pub fn incomplete_source_info(&self) -> &SourceInfo {
        &self.source_info
    }

    /// Returns a the [`SourceInfo`] as long as there're no errors, lints or warnings of any kind.
    ///
    /// # Errors
    ///
    /// Any kind of error, warning or lint is encountered.
    pub fn lint(self) -> Result<SourceInfo, Error> {
        if let Some(errors) = self.errors {
            if !errors.errors().is_empty() {
                return Err(Error::SourceInfoErrors(errors));
            }
        }

        Ok(self.source_info)
    }

    /// Gets a reference to the errors of this result.
    pub fn errors(&self) -> Option<&SourceInfoErrors> {
        self.errors.as_ref()
    }
}
