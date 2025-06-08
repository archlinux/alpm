//! Contains the second parsing and linting pass.
//!
//! The raw representation from the [`parser`](crate::source_info::parser) module is brought into a
//! proper struct-based representation that fully represents the SRCINFO data (apart from comments
//! and empty lines).
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use alpm_types::Architecture;
use serde::{Deserialize, Serialize};
use winnow::Parser;

pub mod merged;
pub mod package;
pub mod package_base;

#[cfg(doc)]
use crate::MergedPackage;
use crate::{
    error::{Error, SourceInfoError, SourceInfoErrors},
    source_info::{
        parser::SourceInfoContent,
        v1::{merged::MergedPackagesIterator, package::Package, package_base::PackageBase},
    },
};

/// The representation of SRCINFO data.
///
/// Provides access to a [`PackageBase`] which tracks all data in a `pkgbase` section and a list of
/// [`Package`] instances that provide the accumulated data of all `pkgname` sections.
///
/// This is the entry point for parsing SRCINFO files. Once created,
/// [`Self::packages_for_architecture`] can be used to create usable [`MergedPackage`]s.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SourceInfoV1 {
    /// The information of the `pkgbase` section.
    pub base: PackageBase,
    /// The information of the `pkgname` sections.
    pub packages: Vec<Package>,
}

impl SourceInfoV1 {
    /// Reads the file at the specified path and converts it into a [`SourceInfoV1`] struct.
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

    /// Parses a SRCINFO file's content into a [`SourceInfoV1`] struct.
    ///
    /// # Error
    ///
    /// This function returns two types of errors.
    /// 1. An [`Error`] is returned if the input is, for example, invalid UTF-8 or if the input
    ///    SRCINFO file couldn't be parsed due to invalid syntax.
    /// 2. If the parsing was successful, a [`SourceInfoResult`] is returned, which wraps a possibly
    ///    invalid [`SourceInfoV1`] and possible [`SourceInfoErrors`]. [`SourceInfoErrors`] contains
    ///    all errors and lint/deprecation warnings that're encountered while interpreting the
    ///    SRCINFO file.
    ///
    /// ```rust
    /// use alpm_srcinfo::SourceInfoV1;
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
    /// let source_info_result = SourceInfoV1::from_string(source_info_data)?;
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
        let (source_info, errors) = SourceInfoV1::from_raw(parsed);

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
    /// [`SourceInfoV1`].
    ///
    /// Instead of a [`Result`] this function returns a tuple of [`SourceInfoV1`] and a vector of
    /// [`SourceInfoError`]s. The caller is expected to handle the vector of [`SourceInfoError`]s,
    /// which may only consist of linting errors that can be ignored.
    pub fn from_raw(content: SourceInfoContent) -> (SourceInfoV1, Vec<SourceInfoError>) {
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

        (SourceInfoV1 { base, packages }, errors)
    }

    /// Get an iterator over all packages
    ///
    /// ```
    /// use alpm_srcinfo::{MergedPackage, SourceInfoV1};
    /// use alpm_types::{Architecture, Name, PackageDescription, PackageRelation};
    ///
    /// # fn main() -> Result<(), alpm_srcinfo::Error> {
    /// let source_info_data = r#"
    /// pkgbase = example
    ///     pkgver = 1.0.0
    ///     epoch = 1
    ///     pkgrel = 1
    ///     arch = x86_64
    ///
    /// pkgname = example
    ///     pkgdesc = Example split package
    ///
    /// pkgname = example_other
    ///     pkgdesc = The other example split package
    /// "#;
    /// // Parse the file. This might already error if the file cannot be parsed on a low level.
    /// let result = SourceInfoV1::from_string(source_info_data)?;
    /// // Make sure there're aren't unrecoverable logic errors, such as missing values.
    /// let source_info = result.source_info()?;
    ///
    /// /// Get all merged package representations for the x86_64 architecture.
    /// let mut packages = source_info.packages_for_architecture(Architecture::X86_64);
    ///
    /// let example = packages.next().unwrap();
    /// assert_eq!(
    ///     example.description,
    ///     Some(PackageDescription::new("Example split package"))
    /// );
    ///
    /// let example_other = packages.next().unwrap();
    /// assert_eq!(
    ///     example_other.description,
    ///     Some(PackageDescription::new("The other example split package"))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn packages_for_architecture(
        &self,
        architecture: Architecture,
    ) -> MergedPackagesIterator<'_> {
        MergedPackagesIterator {
            architecture,
            source_info: self,
            package_iterator: self.packages.iter(),
        }
    }
}

/// Wraps the outcome of [`SourceInfoV1::from_string`].
///
/// While building a [`SourceInfoV1`] from raw [`SourceInfoContent`], errors as well as deprecation
/// and linter warnings may be encountered.
///
/// In case no errors are encountered, the resulting [`SourceInfoV1`] may be accessed via
/// [`SourceInfoResult::source_info`].
#[derive(Clone, Debug)]
pub struct SourceInfoResult {
    source_info: SourceInfoV1,
    errors: Option<SourceInfoErrors>,
}

impl SourceInfoResult {
    /// Returns the [`SourceInfoV1`] as long as no critical errors have been encountered.
    ///
    /// # Errors
    ///
    /// Returns an error if any kind of unrecoverable logic error is encountered, such as missing
    /// properties
    pub fn source_info(self) -> Result<SourceInfoV1, Error> {
        if let Some(errors) = self.errors {
            errors.check_unrecoverable_errors()?;
        }

        Ok(self.source_info)
    }

    /// Returns the generated [`SourceInfoV1`] regardless of whether there're any errors or not.
    ///
    /// # Warning
    ///
    /// This SourceInfoV1 struct may be incomplete, could contain invalid information and/or invalid
    /// default values!
    ///
    /// Only use this if you know what you're doing and if you want to do stuff like manual
    /// auto-correction.
    pub fn incomplete_source_info(&self) -> &SourceInfoV1 {
        &self.source_info
    }

    /// Returns a the [`SourceInfoV1`] as long as there're no errors, lints or warnings of any kind.
    ///
    /// # Errors
    ///
    /// Any kind of error, warning or lint is encountered.
    pub fn lint(self) -> Result<SourceInfoV1, Error> {
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
