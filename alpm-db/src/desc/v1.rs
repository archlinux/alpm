//! Representation of the database desc file v1 ([alpm-db-descv1]).
//!
//! [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html

use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    str::FromStr,
};

use alpm_common::{Installed, Named, RuntimeRelations, Versioned};
use alpm_types::{
    Architecture,
    BuildDate,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageInstallReason,
    PackageRelation,
    PackageValidation,
    Packager,
    RelationOrSoname,
    Url,
};
use winnow::Parser;

use crate::{
    Error,
    desc::{
        DbDescFileV2,
        Section,
        parser::{SectionKeyword, sections},
    },
};

/// DB DESC version 1
///
/// `DbDescFileV1` represents the [alpm-db-descv1] specification which is the
/// canonical format of a single package entry within an ALPM database.
///
/// It includes information such as the package's name, version, architecture,
/// and dependency relationships.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_db::desc::DbDescFileV1;
///
/// # fn main() -> Result<(), alpm_db::Error> {
/// let desc_data = r#"%NAME%
/// foo
///
/// %VERSION%
/// 1.0.0-1
///
/// %BASE%
/// foo
///
/// %DESC%
/// An example package
///
/// %URL%
/// https://example.org/
///
/// %ARCH%
/// x86_64
///
/// %BUILDDATE%
/// 1733737242
///
/// %INSTALLDATE%
/// 1733737243
///
/// %PACKAGER%
/// Foobar McFooface <foobar@mcfooface.org>
///
/// %SIZE%
/// 123
///
/// %GROUPS%
/// utils
/// cli
///
/// %REASON%
/// 1
///
/// %LICENSE%
/// MIT
/// Apache-2.0
///
/// %VALIDATION%
/// pgp
///
/// %REPLACES%
/// pkg-old
///
/// %DEPENDS%
/// glibc
///
/// %OPTDEPENDS%
/// optpkg
///
/// %CONFLICTS%
/// foo-old
///
/// %PROVIDES%
/// foo-virtual
///
/// "#;
///
/// // Parse a DB DESC file in version 1 format.
/// let db_desc = DbDescFileV1::from_str(desc_data)?;
/// // Convert back to its canonical string representation.
/// assert_eq!(db_desc.to_string(), desc_data);
/// # Ok(())
/// # }
/// ```
///
/// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
pub struct DbDescFileV1 {
    /// The name of the package.
    pub name: Name,

    /// The version of the package.
    pub version: FullVersion,

    /// The base name of the package (used in split packages).
    pub base: PackageBaseName,

    /// The description of the package.
    pub description: PackageDescription,

    /// The URL for the project of the package.
    pub url: Option<Url>,

    /// The architecture of the package.
    pub arch: Architecture,

    /// The date at which the build of the package started.
    pub builddate: BuildDate,

    /// The date at which the package has been installed on the system.
    pub installdate: BuildDate,

    /// The User ID of the entity, that built the package.
    pub packager: Packager,

    /// The optional size of the (uncompressed and unpacked) package contents in bytes.
    pub size: InstalledSize,

    /// Groups the package belongs to.
    pub groups: Vec<Group>,

    /// The reason for installing the package.
    pub reason: PackageInstallReason,

    /// Licenses that apply to the package.
    pub license: Vec<License>,

    /// Validation methods used for the package.
    pub validation: Vec<PackageValidation>,

    /// Packages this one replaces.
    pub replaces: Vec<PackageRelation>,

    /// Required runtime dependencies.
    pub depends: Vec<RelationOrSoname>,

    /// Optional dependencies that enhance the package.
    pub optdepends: Vec<OptionalDependency>,

    /// Conflicting packages that cannot be installed together.
    pub conflicts: Vec<PackageRelation>,

    /// Virtual packages or capabilities provided by this one.
    pub provides: Vec<RelationOrSoname>,
}

impl Display for DbDescFileV1 {
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

        single(f, "NAME", &self.name)?;
        single(f, "VERSION", &self.version)?;
        single(f, "BASE", &self.base)?;
        single(f, "DESC", &self.description)?;
        // Write an empty string if there is no URL value.
        single(
            f,
            "URL",
            &self
                .url
                .as_ref()
                .map_or(String::new(), |url| url.to_string()),
        )?;
        single(f, "ARCH", &self.arch)?;
        single(f, "BUILDDATE", &self.builddate)?;
        single(f, "INSTALLDATE", &self.installdate)?;
        single(f, "PACKAGER", &self.packager)?;
        // Omit %SIZE% section if its value is "0"
        if self.size != 0 {
            single(f, "SIZE", &self.size)?;
        }
        section(f, "GROUPS", &self.groups)?;
        // Omit %REASON% section if its value is "PackageInstallReason::Explicit"
        if self.reason != PackageInstallReason::Explicit {
            single(f, "REASON", &self.reason)?;
        }
        section(f, "LICENSE", &self.license)?;
        section(f, "VALIDATION", &self.validation)?;
        section(f, "REPLACES", &self.replaces)?;
        section(f, "DEPENDS", &self.depends)?;
        section(f, "OPTDEPENDS", &self.optdepends)?;
        section(f, "CONFLICTS", &self.conflicts)?;
        section(f, "PROVIDES", &self.provides)?;

        Ok(())
    }
}

impl FromStr for DbDescFileV1 {
    type Err = Error;

    /// Creates a [`DbDescFileV1`] from a string slice.
    ///
    /// Parses the input according to the [alpm-db-descv1] specification and constructs a
    /// structured [`DbDescFileV1`] representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_db::desc::DbDescFileV1;
    ///
    /// # fn main() -> Result<(), alpm_db::Error> {
    /// let desc_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// "#;
    ///
    /// let db_desc = DbDescFileV1::from_str(desc_data)?;
    /// assert_eq!(db_desc.name.to_string(), "foo");
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
    /// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = sections.parse(s)?;
        Self::try_from(sections)
    }
}

impl TryFrom<Vec<Section>> for DbDescFileV1 {
    type Error = Error;

    /// Tries to create a [`DbDescFileV1`] from a list of parsed [`Section`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - any required field is missing,
    /// - a section appears more than once,
    /// - or a section violates the expected format for version 1.
    fn try_from(sections: Vec<Section>) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut version = None;
        let mut base = None;
        let mut description = None;
        let mut url = None;
        let mut arch = None;
        let mut builddate = None;
        let mut installdate = None;
        let mut packager = None;
        let mut size = None;

        let mut groups: Vec<Group> = Vec::new();
        let mut reason = None;
        let mut license: Vec<License> = Vec::new();
        let mut validation = None;
        let mut replaces: Vec<PackageRelation> = Vec::new();
        let mut depends: Vec<RelationOrSoname> = Vec::new();
        let mut optdepends: Vec<OptionalDependency> = Vec::new();
        let mut conflicts: Vec<PackageRelation> = Vec::new();
        let mut provides: Vec<RelationOrSoname> = Vec::new();

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
        macro_rules! set_vec_once {
            ($field:ident, $val:expr, $kw:expr) => {{
                if !$field.is_empty() {
                    return Err(Error::DuplicateSection($kw));
                }
                $field = $val;
            }};
        }

        for section in sections {
            match section {
                Section::Name(v) => set_once!(name, v, SectionKeyword::Name),
                Section::Version(v) => set_once!(version, v, SectionKeyword::Version),
                Section::Base(v) => set_once!(base, v, SectionKeyword::Base),
                Section::Desc(v) => set_once!(description, v, SectionKeyword::Desc),
                Section::Url(v) => set_once!(url, v, SectionKeyword::Url),
                Section::Arch(v) => set_once!(arch, v, SectionKeyword::Arch),
                Section::BuildDate(v) => set_once!(builddate, v, SectionKeyword::BuildDate),
                Section::InstallDate(v) => set_once!(installdate, v, SectionKeyword::InstallDate),
                Section::Packager(v) => set_once!(packager, v, SectionKeyword::Packager),
                Section::Size(v) => set_once!(size, v, SectionKeyword::Size),
                Section::Groups(v) => set_vec_once!(groups, v, SectionKeyword::Groups),
                Section::Reason(v) => set_once!(reason, v, SectionKeyword::Reason),
                Section::License(v) => set_vec_once!(license, v, SectionKeyword::License),
                Section::Validation(v) => set_once!(validation, v, SectionKeyword::Validation),
                Section::Replaces(v) => set_vec_once!(replaces, v, SectionKeyword::Replaces),
                Section::Depends(v) => set_vec_once!(depends, v, SectionKeyword::Depends),
                Section::OptDepends(v) => set_vec_once!(optdepends, v, SectionKeyword::OptDepends),
                Section::Conflicts(v) => set_vec_once!(conflicts, v, SectionKeyword::Conflicts),
                Section::Provides(v) => set_vec_once!(provides, v, SectionKeyword::Provides),
                Section::XData(_) => {}
            }
        }

        Ok(DbDescFileV1 {
            name: name.ok_or(Error::MissingSection(SectionKeyword::Name))?,
            version: version.ok_or(Error::MissingSection(SectionKeyword::Version))?,
            base: base.ok_or(Error::MissingSection(SectionKeyword::Base))?,
            description: description.ok_or(Error::MissingSection(SectionKeyword::Desc))?,
            url: url.ok_or(Error::MissingSection(SectionKeyword::Url))?,
            arch: arch.ok_or(Error::MissingSection(SectionKeyword::Arch))?,
            builddate: builddate.ok_or(Error::MissingSection(SectionKeyword::BuildDate))?,
            installdate: installdate.ok_or(Error::MissingSection(SectionKeyword::InstallDate))?,
            packager: packager.ok_or(Error::MissingSection(SectionKeyword::Packager))?,
            size: size.unwrap_or_default(),
            groups,
            reason: reason.unwrap_or(PackageInstallReason::Explicit),
            license,
            validation: validation
                .filter(|v| !v.is_empty())
                .ok_or(Error::MissingSection(SectionKeyword::Validation))?,
            replaces,
            depends,
            optdepends,
            conflicts,
            provides,
        })
    }
}

impl From<DbDescFileV2> for DbDescFileV1 {
    /// Converts a [`DbDescFileV2`] into a [`DbDescFileV1`].
    ///
    /// # Note
    ///
    /// This drops the `xdata` field of the [`DbDescFileV2`], which provides additional information
    /// about a package.
    fn from(v2: DbDescFileV2) -> Self {
        DbDescFileV1 {
            name: v2.name,
            version: v2.version,
            base: v2.base,
            description: v2.description,
            url: v2.url,
            arch: v2.arch,
            builddate: v2.builddate,
            installdate: v2.installdate,
            packager: v2.packager,
            size: v2.size,
            groups: v2.groups,
            reason: v2.reason,
            license: v2.license,
            validation: v2.validation,
            replaces: v2.replaces,
            depends: v2.depends,
            optdepends: v2.optdepends,
            conflicts: v2.conflicts,
            provides: v2.provides,
        }
    }
}

impl Named for DbDescFileV1 {
    fn get_name(&self) -> &Name {
        &self.name
    }
}

impl Versioned for DbDescFileV1 {
    fn get_version(&self) -> &FullVersion {
        &self.version
    }
}

impl RuntimeRelations for DbDescFileV1 {
    fn get_run_time_dependencies(&self) -> &[RelationOrSoname] {
        &self.depends
    }

    fn get_optional_dependencies(&self) -> &[OptionalDependency] {
        &self.optdepends
    }

    fn get_provisions(&self) -> &[RelationOrSoname] {
        &self.provides
    }

    fn get_conflicts(&self) -> &[PackageRelation] {
        &self.conflicts
    }

    fn get_replacements(&self) -> &[PackageRelation] {
        &self.replaces
    }
}

impl Installed for DbDescFileV1 {
    fn install_reason(&self) -> PackageInstallReason {
        self.reason
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::*;
    use testresult::TestResult;

    use super::*;

    /// An alpm-db-desc string with all sections explicitly populated.
    const DESC_FULL: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%GROUPS%
utils
cli

%REASON%
1

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
sha256
pgp

%REPLACES%
pkg-old

%DEPENDS%
glibc
libwlroots-0.19.so=libwlroots-0.19.so-64
lib:libexample.so.1

%OPTDEPENDS%
optpkg

%CONFLICTS%
foo-old

%PROVIDES%
foo-virtual
libwlroots-0.19.so=libwlroots-0.19.so-64
lib:libexample.so.1

"#;

    /// An alpm-db-desc string with all list sections set, but empty.
    const DESC_EMPTY_LIST_SECTIONS: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%GROUPS%

%LICENSE%

%VALIDATION%
pgp

%REPLACES%

%DEPENDS%

%OPTDEPENDS%

%CONFLICTS%

%PROVIDES%

"#;

    /// An alpm-db-desc string with the minimum set of sections.
    ///
    /// All list sections and sections that can be omitted, are omitted.
    const DESC_MINIMAL: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%VALIDATION%
pgp

"#;

    /// A minimal alpm-db-desc string with empty `%DESC%` and `%URL%` sections.
    ///
    /// All list sections and sections that can be omitted, are omitted for brevity.
    const DESC_EMPTY_DESC_AND_URL: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%


%URL%


%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%VALIDATION%
pgp

"#;

    /// A minimal alpm-db-desc string with the `%REASON%` section explicitly set to "0".
    ///
    /// All list sections and sections that can be omitted, are omitted for brevity.
    const DESC_REASON_EXPLICITLY_ZERO: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REASON%
0

%VALIDATION%
pgp

"#;

    /// A minimal alpm-db-desc string with the `%SIZE%` section explicitly set to "0".
    ///
    /// All list sections and sections that can be omitted, are omitted for brevity.
    const DESC_SIZE_EXPLICITLY_ZERO: &str = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
0

%VALIDATION%
pgp

"#;

    #[rstest]
    #[case::full(
        DESC_FULL,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from("An example package"),
            url: Some(Url::from_str("https://example.org/")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 123,
            groups: vec!["utils".into(), "cli".into()],
            reason: PackageInstallReason::Depend,
            license: vec![License::from_str("MIT")?, License::from_str("Apache-2.0")?],
            validation: vec![
                PackageValidation::from_str("sha256")?,
                PackageValidation::from_str("pgp")?,
            ],
            replaces: vec![PackageRelation::from_str("pkg-old")?],
            depends: vec![
                RelationOrSoname::from_str("glibc")?,
                RelationOrSoname::from_str("libwlroots-0.19.so=libwlroots-0.19.so-64")?,
                RelationOrSoname::from_str("lib:libexample.so.1")?,
            ],
            optdepends: vec![OptionalDependency::from_str("optpkg")?],
            conflicts: vec![PackageRelation::from_str("foo-old")?],
            provides: vec![
                RelationOrSoname::from_str("foo-virtual")?,
                RelationOrSoname::from_str("libwlroots-0.19.so=libwlroots-0.19.so-64")?,
                RelationOrSoname::from_str("lib:libexample.so.1")?,
            ],
        },
        DESC_FULL,
    )]
    #[case::empty_list_sections(
        DESC_EMPTY_LIST_SECTIONS,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from("An example package"),
            url: Some(Url::from_str("https://example.org/")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 0,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: Vec::new(),
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
        },
        DESC_MINIMAL,
    )]
    #[case::minimal(
        DESC_MINIMAL,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from("An example package"),
            url: Some(Url::from_str("https://example.org/")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 0,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: Vec::new(),
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
        },
        DESC_MINIMAL,
    )]
    #[case::empty_desc_and_url(
        DESC_EMPTY_DESC_AND_URL,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from(""),
            url: None,
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 0,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: Vec::new(),
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
        },
        DESC_EMPTY_DESC_AND_URL,
    )]
    #[case::reason_explicitly_zero(
        DESC_REASON_EXPLICITLY_ZERO,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from("An example package"),
            url: Some(Url::from_str("https://example.org/")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 0,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: Vec::new(),
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
        },
        DESC_MINIMAL,
    )]
    #[case::size_explicitly_zero(
        DESC_SIZE_EXPLICITLY_ZERO,
        DbDescFileV1 {
            name: Name::new("foo")?,
            version: FullVersion::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: PackageDescription::from("An example package"),
            url: Some(Url::from_str("https://example.org/")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 0,
            groups: Vec::new(),
            reason: PackageInstallReason::Explicit,
            license: Vec::new(),
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: Vec::new(),
            depends: Vec::new(),
            optdepends: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
        },
        DESC_MINIMAL,
    )]
    fn parse_valid_v1_desc(
        #[case] input_data: &str,
        #[case] expected: DbDescFileV1,
        #[case] expected_output_data: &str,
    ) -> TestResult {
        let desc = DbDescFileV1::from_str(input_data)?;
        assert_eq!(desc, expected);
        assert_eq!(expected_output_data, desc.to_string());
        Ok(())
    }

    #[test]
    fn depends_and_provides_accept_sonames() -> TestResult {
        let desc = DbDescFileV1::from_str(DESC_FULL)?;
        assert!(matches!(desc.depends[1], RelationOrSoname::SonameV1(_)));
        assert!(matches!(desc.depends[2], RelationOrSoname::SonameV2(_)));
        assert!(matches!(desc.provides[1], RelationOrSoname::SonameV1(_)));
        assert!(matches!(desc.provides[2], RelationOrSoname::SonameV2(_)));
        Ok(())
    }

    #[rstest]
    #[case("%UNKNOWN%\nvalue", "invalid section name")]
    #[case("%VERSION%\n1.0.0-1\n", "Missing section: %NAME%")]
    fn invalid_desc_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = DbDescFileV1::from_str(input);
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
        let result = DbDescFileV1::from_str(input);
        assert!(matches!(result, Err(Error::MissingSection(s)) if s == SectionKeyword::Name));
    }
}
