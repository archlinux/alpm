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

use alpm_pkgbuild::bridge::BridgeOutput;
use alpm_types::Architecture;
use fluent_i18n::t;
use serde::{Deserialize, Serialize};
use winnow::Parser;
use writer::{pkgbase_section, pkgname_section};

pub mod merged;
pub mod package;
pub mod package_base;
pub mod writer;

#[cfg(doc)]
use crate::MergedPackage;
use crate::{
    error::Error,
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SourceInfoV1 {
    /// The information of the `pkgbase` section.
    pub base: PackageBase,
    /// The information of the `pkgname` sections.
    pub packages: Vec<Package>,
}

impl SourceInfoV1 {
    /// Returns the [SRCINFO] representation.
    ///
    /// ```
    /// use std::{env::var, path::PathBuf};
    ///
    /// use alpm_srcinfo::SourceInfoV1;
    ///
    /// const TEST_FILE: &str = include_str!("../../../tests/unit_test_files/normal.srcinfo");
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Read a .SRCINFO file and bring it into the `SourceInfoV1` representation.
    /// let source_info = SourceInfoV1::from_string(TEST_FILE)?;
    /// // Convert the `SourceInfoV1` back into it's alpm `.SRCINFO` format.
    /// println!("{}", source_info.as_srcinfo());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    pub fn as_srcinfo(&self) -> String {
        let mut srcinfo = String::new();

        pkgbase_section(&self.base, &mut srcinfo);
        for package in &self.packages {
            srcinfo.push('\n');
            pkgname_section(package, &self.base.architectures, &mut srcinfo);
        }

        srcinfo
    }

    /// Reads the file at the specified path and converts it into a [`SourceInfoV1`] struct.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be read or parsed.
    pub fn from_file(path: &Path) -> Result<SourceInfoV1, Error> {
        let mut buffer = Vec::new();
        let file = File::open(path).map_err(|source| Error::IoPath {
            path: path.to_path_buf(),
            context: t!("error-io-path-opening-file"),
            source,
        })?;
        let mut buf_reader = BufReader::new(file);
        buf_reader
            .read_to_end(&mut buffer)
            .map_err(|source| Error::IoPath {
                path: path.to_path_buf(),
                context: t!("error-io-path-reading-file"),
                source,
            })?;

        let content = String::from_utf8(buffer)?.to_string();

        Self::from_string(&content)
    }

    /// Creates a [`SourceInfoV1`] from a [`PKGBUILD`] file.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - The `PKGBUILD` cannot be read.
    /// - a required field is not set,
    /// - a `package` functions exists, but does not correspond to a declared [alpm-split-package],
    /// - a `package` function without an [alpm-package-name] suffix exists in an
    ///   [alpm-split-package] setup,
    /// - a value cannot be turned into its [`alpm_types`] equivalent,
    /// - multiple values exist for a field that only accepts a singular value,
    /// - an [alpm-architecture] is duplicated,
    /// - an [alpm-architecture] is cleared in `package` function,
    /// - or an [alpm-architecture] suffix is set on a keyword that does not support it.
    ///
    /// [`PKGBUILD`]: https://man.archlinux.org/man/PKGBUILD.5
    /// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
    /// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
    /// [alpm-split-package]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
    pub fn from_pkgbuild(pkgbuild_path: &Path) -> Result<SourceInfoV1, Error> {
        let output = BridgeOutput::from_file(pkgbuild_path)?;
        let source_info: SourceInfoV1 = output.try_into()?;

        Ok(source_info)
    }

    /// Parses a SRCINFO file's content into a [`SourceInfoV1`] struct.
    ///
    /// # Error
    ///
    /// This function returns two types of errors.
    /// 1. An [`Error`] is returned if the input is, for example, invalid UTF-8 or if the input
    ///    SRCINFO file couldn't be parsed due to invalid syntax.
    /// 2. An [`Error`] is returned if the parsed data is incomplete or otherwise invalid.
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
    /// // Parse the file. This errors if the file cannot be parsed, is missing data or contains invalid data.
    /// let source_info = SourceInfoV1::from_string(source_info_data)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_string(content: &str) -> Result<SourceInfoV1, Error> {
        // A temporary fix for <https://github.com/winnow-rs/winnow/issues/847>
        let content_no_tabs = content.replace('\t', " ");

        // Parse the given srcinfo content.
        let parsed = SourceInfoContent::parser
            .parse(content_no_tabs.as_str())
            .map_err(|err| Error::ParseError(format!("{err}")))?;

        // Bring it into a proper structural representation
        let source_info = SourceInfoV1::from_raw(parsed)?;

        Ok(source_info)
    }

    /// Reads raw [`SourceInfoContent`] from a first parsing step and converts it into a
    /// [`SourceInfoV1`].
    pub fn from_raw(content: SourceInfoContent) -> Result<SourceInfoV1, Error> {
        let base = PackageBase::from_parsed(content.package_base)?;

        let mut packages = Vec::new();
        for package in content.packages {
            let package = Package::from_parsed(package)?;
            packages.push(package);
        }

        Ok(SourceInfoV1 { base, packages })
    }

    /// Get an iterator over all packages
    ///
    /// ```
    /// use alpm_srcinfo::{MergedPackage, SourceInfoV1};
    /// use alpm_types::{Name, PackageDescription, PackageRelation, SystemArchitecture};
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
    /// // Parse the file. This errors if the file cannot be parsed, is missing data or contains invalid data.
    /// let source_info = SourceInfoV1::from_string(source_info_data)?;
    ///
    /// /// Get all merged package representations for the x86_64 architecture.
    /// let mut packages = source_info.packages_for_architecture(SystemArchitecture::X86_64);
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
    pub fn packages_for_architecture<A: Into<Architecture>>(
        &self,
        architecture: A,
    ) -> MergedPackagesIterator<'_> {
        MergedPackagesIterator {
            architecture: architecture.into(),
            source_info: self,
            package_iterator: self.packages.iter(),
        }
    }
}
