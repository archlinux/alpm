use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use alpm_types::{
    digests::{Blake2b512, Digest, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
    Architecture,
    Checksum,
    Epoch,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelativePath,
    Source,
    Url,
    Version,
};
use winnow::Parser;

/// Representation of [`Package`] (`pkgname`) specific overrides.
pub mod package;
/// Representation of the [`PackageBase`] (`pkgbase`) default declarations of a SRCINFO file.
pub mod package_base;

/// Common and reusable linter logic.
mod lints;

use crate::{
    error::{lint, unrecoverable, Error, SourceInfoError, SourceInfoErrors},
    merged::MergedPackagesIterator,
    parser::{self, PackageBaseProperty, SharedMetaProperty, SourceInfoContent},
    source_info::{package::Package, package_base::PackageBase},
};

/// Represent a checksum check that is allowed to be skipped.
/// If the `SKIP` keyword is found, a source file won't be checked for this type of checksum.
#[derive(Debug, Clone)]
pub enum SkippableChecksum<D: Digest + Clone> {
    Skip,
    Checksum(Checksum<D>),
}

/// The accurate depiction of a SRCINFO file.
/// This is the entry point for parsing SRCINFO files. Once created,
/// [`Self::packages_for_architecture`] can be used to create usable
/// [`MergedPackage`](crate::merged::MergedPackage)s.
///
/// The `SourceInfo` struct is not very usable in this form.
#[derive(Debug)]
pub struct SourceInfo {
    pub base: PackageBase,
    pub packages: Vec<Package>,
}

impl SourceInfo {
    /// Read the file at the specified and convert it into a [`SourceInfo`] struct.
    ///
    /// # Errors
    ///
    /// Throws an error if the file cannot be read or parsed.
    /// Returns an error array with potentially un/-recoverable errors, this needs to be explicitly
    /// handled by the user.
    pub fn from_file(path: &Path) -> Result<(SourceInfo, Option<SourceInfoErrors>), Error> {
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

    pub fn from_string(content: &str) -> Result<(SourceInfo, Option<SourceInfoErrors>), Error> {
        // Parse the given srcinfo content.
        let parsed = parser::srcinfo
            .parse(content)
            .map_err(|err| Error::ParseError(format!("{err}")))?;

        // Bring it into a proper structural representation and run linting checks.
        let (source_info, errors) = SourceInfo::from_parsed(parsed);

        // If there're some errors, create a SourceInfoErrors to also capture the file content for
        // context.
        let errors = if !errors.is_empty() {
            Some(SourceInfoErrors {
                inner: errors,
                file_content: content.to_string(),
            })
        } else {
            None
        };

        Ok((source_info, errors))
    }

    /// Read the raw [`SourceInfoContent`] from the first parsing step and convert it into a proper
    /// `struct` representation.
    ///
    /// This function returns error array instead of a `Result` as those errors may only be linting
    /// errors that can be ignored.
    /// The consumer has handle those errors and decide what to do do based on their type.
    pub fn from_parsed(content: SourceInfoContent) -> (SourceInfo, Vec<SourceInfoError>) {
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
