//! Representation of the package repository desc file v1 ([alpm-repo-descv1]).
//!
//! [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html

use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    Base64OpenPGPSignature,
    BuildDate,
    CompressedSize,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Md5Checksum,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageFileName,
    PackageRelation,
    Packager,
    RelationOrSoname,
    Sha256Checksum,
    Url,
};
use winnow::Parser;

use crate::{
    Error,
    desc::{
        Section,
        parser::{SectionKeyword, sections},
    },
};

/// Representation of files following the [alpm-repo-descv1] specification.
///
/// This file format is used to describe a single package entry within an [alpm-repo-db].
///
/// It includes information such as the package's name, version, architecture,
/// and dependency relationships.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_repo_db::desc::RepoDescFileV1;
///
/// # fn main() -> Result<(), alpm_repo_db::Error> {
/// let desc_data = r#"%FILENAME%
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
/// // Parse a REPO DESC file in version 1 format.
/// let repo_desc = RepoDescFileV1::from_str(desc_data)?;
/// // Convert back to its canonical string representation.
/// assert_eq!(repo_desc.to_string(), desc_data);
/// # Ok(())
/// # }
/// ```
///
/// [alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html
/// [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
pub struct RepoDescFileV1 {
    /// The file name of the package.
    pub file_name: PackageFileName,

    /// The name of the package.
    pub name: Name,

    /// The name of the package base, from which this package originates.
    pub base: PackageBaseName,

    /// The version of the package.
    pub version: FullVersion,

    /// The description of the package.
    ///
    /// Can be 0 or more characters.
    pub description: PackageDescription,

    /// The groups this package belongs to.
    ///
    /// If the package does not belong to any group, this will be an empty list.
    pub groups: Vec<Group>,

    /// The compressed size of the package in bytes.
    pub compressed_size: CompressedSize,

    /// The size of the uncompressed and unpacked package contents in bytes.
    ///
    /// Multiple hard-linked files are only counted once.
    pub installed_size: InstalledSize,

    /// The MD5 checksum of the package file.
    pub md5_checksum: Md5Checksum,

    /// The SHA256 checksum of the package file.
    pub sha256_checksum: Sha256Checksum,

    /// The base64 encoded OpenPGP detached signature of the package file.
    pub pgp_signature: Base64OpenPGPSignature,

    /// The optional URL associated with the package.
    pub url: Option<Url>,

    /// Set of licenses under which the package is distributed.
    ///
    /// Can be empty.
    pub license: Vec<License>,

    /// The architecture of the package.
    pub arch: Architecture,

    /// The date at wchich the build of the package started.
    pub build_date: BuildDate,

    /// The User ID of the entity, that built the package.
    pub packager: Packager,

    /// Virtual components or packages that this package replaces upon installation.
    ///
    /// Can be empty.
    pub replaces: Vec<PackageRelation>,

    /// Virtual components or packages that this package conflicts with.
    ///
    /// Can be empty.
    pub conflicts: Vec<PackageRelation>,

    /// Virtual components or packages that this package provides.
    ///
    /// Can be empty.
    pub provides: Vec<RelationOrSoname>,

    /// Run-time dependencies required by the package.
    ///
    /// Can be empty.
    pub dependencies: Vec<RelationOrSoname>,

    /// Optional dependencies that are not strictly required by the package.
    ///
    /// Can be empty.
    pub optional_dependencies: Vec<OptionalDependency>,

    /// Dependencies for building the upstream software of the package.
    ///
    /// Can be empty.
    pub make_dependencies: Vec<PackageRelation>,

    /// A dependency for running tests of the package's upstream project.
    ///
    /// Can be empty.
    pub check_dependencies: Vec<PackageRelation>,
}

impl Display for RepoDescFileV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Helper function to write a single value section
        fn single<T: Display, W: Write>(f: &mut W, key: &str, val: &T) -> FmtResult {
            writeln!(f, "%{key}%\n{val}\n")
        }

        // Helper function to write a multi-value section
        fn section<T: Display, W: Write>(f: &mut W, key: &str, vals: &[T]) -> FmtResult {
            if vals.is_empty() {
                return Ok(());
            }
            writeln!(f, "%{key}%")?;
            for v in vals {
                writeln!(f, "{v}")?;
            }
            writeln!(f)
        }

        single(f, "FILENAME", &self.file_name)?;
        single(f, "NAME", &self.name)?;
        single(f, "BASE", &self.base)?;
        single(f, "VERSION", &self.version)?;
        if !self.description.as_ref().is_empty() {
            single(f, "DESC", &self.description)?;
        }
        section(f, "GROUPS", &self.groups)?;
        single(f, "CSIZE", &self.compressed_size)?;
        single(f, "ISIZE", &self.installed_size)?;
        single(f, "MD5SUM", &self.md5_checksum)?;
        single(f, "SHA256SUM", &self.sha256_checksum)?;
        single(f, "PGPSIG", &self.pgp_signature)?;
        if let Some(url) = &self.url {
            single(f, "URL", url)?;
        }
        section(f, "LICENSE", &self.license)?;
        single(f, "ARCH", &self.arch)?;
        single(f, "BUILDDATE", &self.build_date)?;
        single(f, "PACKAGER", &self.packager)?;
        section(f, "REPLACES", &self.replaces)?;
        section(f, "CONFLICTS", &self.conflicts)?;
        section(f, "PROVIDES", &self.provides)?;
        section(f, "DEPENDS", &self.dependencies)?;
        section(f, "OPTDEPENDS", &self.optional_dependencies)?;
        section(f, "MAKEDEPENDS", &self.make_dependencies)?;
        section(f, "CHECKDEPENDS", &self.check_dependencies)?;
        Ok(())
    }
}

impl FromStr for RepoDescFileV1 {
    type Err = Error;

    /// Creates a [`RepoDescFileV1`] from a string slice.
    ///
    /// Parses the input according to the [alpm-repo-descv1] specification and constructs a
    /// structured [`RepoDescFileV1`] representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_repo_db::desc::RepoDescFileV1;
    ///
    /// # fn main() -> Result<(), alpm_repo_db::Error> {
    /// let desc_data = r#"%FILENAME%
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
    /// let repo_desc = RepoDescFileV1::from_str(desc_data)?;
    /// assert_eq!(repo_desc.name.to_string(), "example-meta");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input cannot be parsed into valid sections,
    /// - or required fields are missing or malformed.
    ///
    /// [alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = sections.parse(s)?;
        Self::try_from(sections)
    }
}

impl TryFrom<Vec<Section>> for RepoDescFileV1 {
    type Error = Error;

    /// Tries to create a [`RepoDescFileV1`] from a list of parsed [`Section`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - any required field is missing,
    /// - a section appears more than once,
    /// - or a section violates the expected format for version 1.
    fn try_from(sections: Vec<Section>) -> Result<Self, Self::Error> {
        let mut file_name = None;
        let mut name = None;
        let mut base = None;
        let mut version = None;
        let mut description = None;
        let mut groups: Vec<Group> = Vec::new();
        let mut compressed_size = None;
        let mut installed_size = None;
        let mut md5_checksum = None;
        let mut sha256_checksum = None;
        let mut pgp_signature = None;
        let mut url = None;
        let mut license: Vec<License> = Vec::new();
        let mut arch = None;
        let mut build_date = None;
        let mut packager = None;
        let mut replaces: Vec<PackageRelation> = Vec::new();
        let mut conflicts: Vec<PackageRelation> = Vec::new();
        let mut provides: Vec<RelationOrSoname> = Vec::new();
        let mut dependencies: Vec<RelationOrSoname> = Vec::new();
        let mut optional_dependencies: Vec<OptionalDependency> = Vec::new();
        let mut make_dependencies: Vec<PackageRelation> = Vec::new();
        let mut check_dependencies: Vec<PackageRelation> = Vec::new();

        /// Helper macro to set a field only once, returning an error if it was already set.
        macro_rules! set_once {
            ($field:ident, $val:expr, $kw:expr) => {{
                if $field.is_some() {
                    return Err(Error::DuplicateSection($kw));
                }
                $field = Some($val);
            }};
        }

        /// Helper macro to set a vector field only once, returning an error if it was already set.
        /// Additionally, ensures that the provided value is not empty.
        macro_rules! set_vec_once {
            ($field:ident, $val:expr, $kw:expr) => {{
                if !$field.is_empty() {
                    return Err(Error::DuplicateSection($kw));
                }
                if $val.is_empty() {
                    return Err(Error::EmptySection($kw));
                }
                $field = $val;
            }};
        }

        for section in sections {
            match section {
                Section::Filename(val) => set_once!(file_name, val, SectionKeyword::Filename),
                Section::Name(val) => set_once!(name, val, SectionKeyword::Name),
                Section::Base(val) => set_once!(base, val, SectionKeyword::Base),
                Section::Version(val) => set_once!(version, val, SectionKeyword::Version),
                Section::Desc(val) => set_once!(description, val, SectionKeyword::Desc),
                Section::Groups(val) => set_vec_once!(groups, val, SectionKeyword::Groups),
                Section::CSize(val) => set_once!(compressed_size, val, SectionKeyword::CSize),
                Section::ISize(val) => set_once!(installed_size, val, SectionKeyword::ISize),
                Section::Md5Sum(val) => set_once!(md5_checksum, val, SectionKeyword::Md5Sum),
                Section::Sha256Sum(val) => {
                    set_once!(sha256_checksum, val, SectionKeyword::Sha256Sum)
                }
                Section::PgpSig(val) => set_once!(pgp_signature, val, SectionKeyword::PgpSig),
                Section::Url(val) => set_once!(url, val, SectionKeyword::Url),
                Section::License(val) => set_vec_once!(license, val, SectionKeyword::License),
                Section::Arch(val) => set_once!(arch, val, SectionKeyword::Arch),
                Section::BuildDate(val) => set_once!(build_date, val, SectionKeyword::BuildDate),
                Section::Packager(val) => set_once!(packager, val, SectionKeyword::Packager),
                Section::Replaces(val) => set_vec_once!(replaces, val, SectionKeyword::Replaces),
                Section::Conflicts(val) => set_vec_once!(conflicts, val, SectionKeyword::Conflicts),
                Section::Provides(val) => set_vec_once!(provides, val, SectionKeyword::Provides),
                Section::Depends(val) => set_vec_once!(dependencies, val, SectionKeyword::Depends),
                Section::OptDepends(val) => {
                    set_vec_once!(optional_dependencies, val, SectionKeyword::OptDepends)
                }
                Section::MakeDepends(val) => {
                    set_vec_once!(make_dependencies, val, SectionKeyword::MakeDepends)
                }
                Section::CheckDepends(val) => {
                    set_vec_once!(check_dependencies, val, SectionKeyword::CheckDepends)
                }
            }
        }

        Ok(RepoDescFileV1 {
            file_name: file_name.ok_or(Error::MissingSection(SectionKeyword::Filename))?,
            name: name.ok_or(Error::MissingSection(SectionKeyword::Name))?,
            base: base.ok_or(Error::MissingSection(SectionKeyword::Base))?,
            version: version.ok_or(Error::MissingSection(SectionKeyword::Version))?,
            description: description.unwrap_or_default(),
            groups,
            compressed_size: compressed_size.ok_or(Error::MissingSection(SectionKeyword::CSize))?,
            installed_size: installed_size.ok_or(Error::MissingSection(SectionKeyword::ISize))?,
            md5_checksum: md5_checksum.ok_or(Error::MissingSection(SectionKeyword::Md5Sum))?,
            sha256_checksum: sha256_checksum
                .ok_or(Error::MissingSection(SectionKeyword::Sha256Sum))?,
            pgp_signature: pgp_signature.ok_or(Error::MissingSection(SectionKeyword::PgpSig))?,
            url: url.unwrap_or(None),
            license,
            arch: arch.ok_or(Error::MissingSection(SectionKeyword::Arch))?,
            build_date: build_date.ok_or(Error::MissingSection(SectionKeyword::BuildDate))?,
            packager: packager.ok_or(Error::MissingSection(SectionKeyword::Packager))?,
            replaces,
            conflicts,
            provides,
            dependencies,
            optional_dependencies,
            make_dependencies,
            check_dependencies,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::*;
    use testresult::TestResult;

    use super::*;

    const VALID_DESC_FILE: &str = r#"%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%GROUPS%
example-group
other-group

%CSIZE%
1818463

%ISIZE%
18184634

%MD5SUM%
d3b07384d113edec49eaa6238ad5ff00

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%
https://example.org/

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REPLACES%
other-pkg-replaced

%CONFLICTS%
other-pkg-conflicts

%PROVIDES%
example-component
lib:libexample.so.1

%DEPENDS%
glibc
gcc-libs
libdep.so=1-64

%OPTDEPENDS%
bash: for a script

%MAKEDEPENDS%
cmake

%CHECKDEPENDS%
bats

"#;

    const VALID_DESC_FILE_MINIMAL: &str = r#"%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%CSIZE%
1818463

%ISIZE%
18184634

%MD5SUM%
d3b07384d113edec49eaa6238ad5ff00

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

"#;

    const VALID_DESC_FILE_EMPTY_FIELDS: &str = r#"%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%

%CSIZE%
1818463

%ISIZE%
18184634

%MD5SUM%
d3b07384d113edec49eaa6238ad5ff00

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

"#;

    #[test]
    fn parse_valid_v1_desc() -> TestResult {
        let actual = RepoDescFileV1::from_str(VALID_DESC_FILE)?;
        let expected = RepoDescFileV1 {
            file_name: PackageFileName::from_str("example-1.0.0-1-any.pkg.tar.zst")?,
            name: Name::from_str("example")?,
            base: PackageBaseName::from_str("example")?,
            version: FullVersion::from_str("1.0.0-1")?,
            description: PackageDescription::from("An example package"),
            groups: vec![
                Group::from_str("example-group")?,
                Group::from_str("other-group")?,
            ],
            compressed_size: CompressedSize::from_str("1818463")?,
            installed_size: InstalledSize::from_str("18184634")?,
            md5_checksum: Md5Checksum::from_str("d3b07384d113edec49eaa6238ad5ff00")?,
            sha256_checksum: Sha256Checksum::from_str(
                "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
            )?,
            pgp_signature: Base64OpenPGPSignature::from_str(
                "iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=",
            )?,
            url: Some(Url::from_str("https://example.org")?),
            license: vec![License::from_str("MIT")?, License::from_str("Apache-2.0")?],
            arch: Architecture::from_str("x86_64")?,
            build_date: BuildDate::from_str("1729181726")?,
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            replaces: vec![PackageRelation::from_str("other-pkg-replaced")?],
            conflicts: vec![PackageRelation::from_str("other-pkg-conflicts")?],
            provides: vec![
                RelationOrSoname::from_str("example-component")?,
                RelationOrSoname::from_str("lib:libexample.so.1")?,
            ],
            dependencies: vec![
                RelationOrSoname::from_str("glibc")?,
                RelationOrSoname::from_str("gcc-libs")?,
                RelationOrSoname::from_str("libdep.so=1-64")?,
            ],
            optional_dependencies: vec![OptionalDependency::from_str("bash: for a script")?],
            make_dependencies: vec![PackageRelation::from_str("cmake")?],
            check_dependencies: vec![PackageRelation::from_str("bats")?],
        };
        assert_eq!(actual, expected);
        assert_eq!(VALID_DESC_FILE, actual.to_string());
        Ok(())
    }

    #[test]
    fn parse_valid_v1_desc_minimal() -> TestResult {
        let actual = RepoDescFileV1::from_str(VALID_DESC_FILE_MINIMAL)?;
        let expected = RepoDescFileV1 {
            file_name: PackageFileName::from_str("example-1.0.0-1-any.pkg.tar.zst")?,
            name: Name::from_str("example")?,
            base: PackageBaseName::from_str("example")?,
            version: FullVersion::from_str("1.0.0-1")?,
            description: PackageDescription::from(""),
            groups: vec![],
            compressed_size: CompressedSize::from_str("1818463")?,
            installed_size: InstalledSize::from_str("18184634")?,
            md5_checksum: Md5Checksum::from_str("d3b07384d113edec49eaa6238ad5ff00")?,
            sha256_checksum: Sha256Checksum::from_str(
                "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
            )?,
            pgp_signature: Base64OpenPGPSignature::from_str(
                "iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=",
            )?,
            url: None,
            license: vec![],
            arch: Architecture::from_str("x86_64")?,
            build_date: BuildDate::from_str("1729181726")?,
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            replaces: vec![],
            conflicts: vec![],
            provides: vec![],
            dependencies: vec![],
            optional_dependencies: vec![],
            make_dependencies: vec![],
            check_dependencies: vec![],
        };
        assert_eq!(actual, expected);
        assert_eq!(VALID_DESC_FILE_MINIMAL, actual.to_string());
        Ok(())
    }

    #[rstest]
    #[case(VALID_DESC_FILE, VALID_DESC_FILE)]
    #[case(VALID_DESC_FILE_MINIMAL, VALID_DESC_FILE_MINIMAL)]
    // Empty optional fields are omitted in output
    #[case(VALID_DESC_FILE_EMPTY_FIELDS, VALID_DESC_FILE_MINIMAL)]
    fn parser_roundtrip(#[case] input: &str, #[case] expected: &str) -> TestResult {
        let desc = RepoDescFileV1::from_str(input)?;
        let output = desc.to_string();
        assert_eq!(output, expected);
        let desc_roundtrip = RepoDescFileV1::from_str(&output)?;
        assert_eq!(desc, desc_roundtrip);
        Ok(())
    }

    #[rstest]
    #[case("%UNKNOWN%\nvalue", "invalid section name")]
    #[case("%VERSION%\n1.0.0-1\n", "Missing section: %FILENAME%")]
    fn invalid_desc_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = RepoDescFileV1::from_str(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }

    #[test]
    fn missing_required_section_should_fail() {
        let input = "%VERSION%\n1.0.0-1\n";
        let result = RepoDescFileV1::from_str(input);
        assert!(matches!(result, Err(Error::MissingSection(s)) if s == SectionKeyword::Filename));
    }
}
