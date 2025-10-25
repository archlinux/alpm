//! Representation of the database desc file v1 ([alpm-db-descv1]).
//!
//! [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html

use std::{
    fmt::{Display, Formatter, Result as FmtResult, Write},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    BuildDate,
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
    Url,
    Version,
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

/// Generates a struct based on the DB DESC version 1 specification with additional fields.
macro_rules! generate_dbdesc {
    // Meta: The meta information for the struct (e.g. doc comments)
    // Name: The name of the struct
    // Extra fields: Additional fields that should be added to the struct
    ($(#[$meta:meta])* $name:ident { $($extra_fields:tt)* }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
        #[serde(deny_unknown_fields)]
        #[serde(rename_all = "lowercase")]
        pub struct $name {
            /// The name of the package.
            pub name: Name,

            /// The version of the package.
            pub version: Version,

            /// The base name of the package (used in split packages).
            pub base: PackageBaseName,

            /// The description of the package.
            pub description: Option<PackageDescription>,

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

            /// Optional install reason.
            pub reason: Option<PackageInstallReason>,

            /// Licenses that apply to the package.
            pub license: Vec<License>,

            /// Validation methods used for the package archive.
            pub validation: Vec<PackageValidation>,

            /// Packages this one replaces.
            pub replaces: Vec<Name>,

            /// Required runtime dependencies.
            pub depends: Vec<PackageRelation>,

            /// Optional dependencies that enhance the package.
            pub optdepends: Vec<OptionalDependency>,

            /// Conflicting packages that cannot be installed together.
            pub conflicts: Vec<Name>,

            /// Virtual packages or capabilities provided by this one.
            pub provides: Vec<Name>,

            $($extra_fields)*
        }
    };
}

generate_dbdesc! {
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
    DbDescFileV1 {}
}

pub(crate) use generate_dbdesc;

impl Display for DbDescFileV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Helper function to write a single value section
        fn single<T: Display, W: Write>(f: &mut W, key: &str, val: &T) -> FmtResult {
            writeln!(f, "%{key}%\n{val}\n")
        }

        // Helper function to write an optional value section
        fn opt<T: Display, W: Write>(f: &mut W, key: &str, val: &Option<T>) -> FmtResult {
            if let Some(v) = val {
                writeln!(f, "%{key}%\n{v}\n")?;
            }
            Ok(())
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
        opt(f, "DESC", &self.description)?;
        opt(f, "URL", &self.url)?;
        single(f, "ARCH", &self.arch)?;
        single(f, "BUILDDATE", &self.builddate)?;
        single(f, "INSTALLDATE", &self.installdate)?;
        single(f, "PACKAGER", &self.packager)?;
        // Omit %SIZE% if it is zero
        if self.size != 0 {
            single(f, "SIZE", &self.size)?;
        }
        section(f, "GROUPS", &self.groups)?;
        opt(f, "REASON", &self.reason)?;
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

impl DbDescFileV1 {
    /// Create a new [`DbDescFileV1`] from all required components.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Name,
        version: Version,
        base: PackageBaseName,
        description: Option<PackageDescription>,
        url: Option<Url>,
        arch: Architecture,
        builddate: BuildDate,
        installdate: BuildDate,
        packager: Packager,
        size: InstalledSize,
        groups: Vec<Group>,
        reason: Option<PackageInstallReason>,
        license: Vec<License>,
        validation: Vec<PackageValidation>,
        replaces: Vec<Name>,
        depends: Vec<PackageRelation>,
        optdepends: Vec<OptionalDependency>,
        conflicts: Vec<Name>,
        provides: Vec<Name>,
    ) -> Self {
        Self {
            name,
            version,
            base,
            description,
            url,
            arch,
            builddate,
            installdate,
            packager,
            size,
            groups,
            reason,
            license,
            validation,
            replaces,
            depends,
            optdepends,
            conflicts,
            provides,
        }
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
        let mut validation: Vec<PackageValidation> = Vec::new();
        let mut replaces: Vec<Name> = Vec::new();
        let mut depends: Vec<PackageRelation> = Vec::new();
        let mut optdepends: Vec<OptionalDependency> = Vec::new();
        let mut conflicts: Vec<Name> = Vec::new();
        let mut provides: Vec<Name> = Vec::new();

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
                Section::Validation(v) => set_vec_once!(validation, v, SectionKeyword::Validation),
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
            description,
            url,
            arch: arch.ok_or(Error::MissingSection(SectionKeyword::Arch))?,
            builddate: builddate.ok_or(Error::MissingSection(SectionKeyword::BuildDate))?,
            installdate: installdate.ok_or(Error::MissingSection(SectionKeyword::InstallDate))?,
            packager: packager.ok_or(Error::MissingSection(SectionKeyword::Packager))?,
            size: size.ok_or(Error::MissingSection(SectionKeyword::Size))?,
            groups,
            reason,
            license,
            validation,
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::*;
    use testresult::TestResult;

    use super::*;

    const VALID_DESC_FILE: &str = r#"%NAME%
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
pgp

%REPLACES%
pkg-old

%DEPENDS%
glibc

%OPTDEPENDS%
optpkg

%CONFLICTS%
foo-old

%PROVIDES%
foo-virtual

"#;

    #[test]
    fn parse_valid_v1_desc() -> TestResult {
        let actual = DbDescFileV1::from_str(VALID_DESC_FILE)?;
        let expected = DbDescFileV1 {
            name: Name::new("foo")?,
            version: Version::from_str("1.0.0-1")?,
            base: PackageBaseName::new("foo")?,
            description: Some(PackageDescription::from("An example package")),
            url: Some(Url::from_str("https://example.org")?),
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 123,
            groups: vec!["utils".into(), "cli".into()],
            reason: Some(PackageInstallReason::Depend),
            license: vec![License::from_str("MIT")?, License::from_str("Apache-2.0")?],
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: vec![Name::new("pkg-old")?],
            depends: vec![PackageRelation::from_str("glibc")?],
            optdepends: vec![OptionalDependency::from_str("optpkg")?],
            conflicts: vec![Name::new("foo-old")?],
            provides: vec![Name::new("foo-virtual")?],
        };
        assert_eq!(actual, expected);
        assert_eq!(VALID_DESC_FILE, actual.to_string());
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
