//! Database desc file (v1)

use std::str::FromStr;

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
    PackageRelation,
    Packager,
    Url,
    Version,
};
use serde_with::{DisplayFromStr, serde_as};
use winnow::Parser;

use crate::{Error, PackageInstallReason, PackageValidation, Section, parser::sections};

/// Generates a struct based on the DB DESC version 1 specification with additional fields.
macro_rules! generate_dbdesc {
    // Meta: The meta information for the struct (e.g. doc comments)
    // Name: The name of the struct
    // Extra fields: Additional fields that should be added to the struct
    ($(#[$meta:meta])* $name:ident { $($extra_fields:tt)* }) => {
        $(#[$meta])*
        #[serde_as]
        #[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
        #[serde(deny_unknown_fields)]
        #[serde(rename_all = "UPPERCASE")]
        pub struct $name {
            /// The name of the package.
            #[serde_as(as = "DisplayFromStr")]
            name: Name,

            /// The version of the package.
            #[serde_as(as = "DisplayFromStr")]
            version: Version,

            /// The base name of the package (used in split packages).
            #[serde_as(as = "DisplayFromStr")]
            base: PackageBaseName,

            /// The description of the package.
            #[serde_as(as = "DisplayFromStr")]
            desc: PackageDescription,

            /// The URL for the project of the package.
            #[serde_as(as = "DisplayFromStr")]
            url: Url,

            /// The architecture of the package.
            #[serde_as(as = "DisplayFromStr")]
            arch: Architecture,

            /// The date at which the build of the package started.
            #[serde_as(as = "DisplayFromStr")]
            builddate: BuildDate,

            /// The date at which the package has been installed on the system.
            #[serde_as(as = "DisplayFromStr")]
            installdate: BuildDate,

            /// The User ID of the entity, that built the package.
            #[serde_as(as = "DisplayFromStr")]
            packager: Packager,

            /// The optional size of the (uncompressed and unpacked) package contents in bytes.
            #[serde_as(as = "DisplayFromStr")]
            size: InstalledSize,

            /// Groups the package belongs to.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            groups: Vec<Group>,

            /// Optional install reason.
            #[serde_as(as = "DisplayFromStr")]
            reason: PackageInstallReason,

            /// Licenses that apply to the package.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            license: Vec<License>,

            /// Validation methods used for the package archive.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            validation: Vec<PackageValidation>,

            /// Packages this one replaces.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            replaces: Vec<Name>,

            /// Required runtime dependencies.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            depends: Vec<PackageRelation>,

            /// Optional dependencies that enhance the package.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            optdepends: Vec<OptionalDependency>,

            /// Conflicting packages that cannot be installed together.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            conflicts: Vec<Name>,

            /// Virtual packages or capabilities provided by this one.
            #[serde_as(as = "Vec<DisplayFromStr>")]
            provides: Vec<Name>,

            $($extra_fields)*
        }

        impl $name {
            /// Returns the name of the package.
            pub fn name(&self) -> &Name {
                &self.name
            }

            /// Returns the version of the package.
            pub fn version(&self) -> &Version {
                &self.version
            }

            /// Returns the base name of the package.
            pub fn base(&self) -> &PackageBaseName {
                &self.base
            }

            /// Returns the description of the package.
            pub fn desc(&self) -> &PackageDescription {
                &self.desc
            }

            /// Returns the URL for the project of the package.
            pub fn url(&self) -> &Url {
                &self.url
            }

            /// Returns the architecture of the package.
            pub fn arch(&self) -> &Architecture {
                &self.arch
            }

            /// Returns the build date of the package.
            pub fn builddate(&self) -> &BuildDate {
                &self.builddate
            }

            /// Returns the install date of the package.
            pub fn installdate(&self) -> &BuildDate {
                &self.installdate
            }

            /// Returns the packager of the package.
            pub fn packager(&self) -> &Packager {
                &self.packager
            }

            /// Returns the size of the package.
            pub fn size(&self) -> &InstalledSize {
                &self.size
            }

            /// Returns the groups the package belongs to.
            pub fn groups(&self) -> &Vec<Group> {
                &self.groups
            }

            /// Returns the install reason of the package.
            pub fn reason(&self) -> &PackageInstallReason {
                &self.reason
            }

            /// Returns the licenses that apply to the package.
            pub fn license(&self) -> &Vec<License> {
                &self.license
            }

            /// Returns the validation methods used for the package archive.
            pub fn validation(&self) -> &Vec<PackageValidation> {
                &self.validation
            }

            /// Returns the packages this one replaces.
            pub fn replaces(&self) -> &Vec<Name> {
                &self.replaces
            }

            /// Returns the required runtime dependencies.
            pub fn depends(&self) -> &Vec<PackageRelation> {
                &self.depends
            }

            /// Returns the optional dependencies.
            pub fn optdepends(&self) -> &Vec<OptionalDependency> {
                &self.optdepends
            }

            /// Returns the conflicting packages.
            pub fn conflicts(&self) -> &Vec<Name> {
                &self.conflicts
            }

            /// Returns the packages provided by this one.
            pub fn provides(&self) -> &Vec<Name> {
                &self.provides
            }
        }
    };
}

generate_dbdesc! {
    /// DB desc version 1
    DbDescFileV1 {}
}

pub(crate) use generate_dbdesc;

impl DbDescFileV1 {
    /// Create a new DbDescFileV1.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Name,
        version: Version,
        base: PackageBaseName,
        desc: PackageDescription,
        url: Url,
        arch: Architecture,
        builddate: BuildDate,
        installdate: BuildDate,
        packager: Packager,
        size: InstalledSize,
        groups: Vec<Group>,
        reason: PackageInstallReason,
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
            desc,
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

    /// Parses a database desc file in version 1 format from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed into a valid [`DbDescFileV1`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = sections.parse(s)?;
        Self::try_from(sections)
    }
}

impl TryFrom<Vec<Section>> for DbDescFileV1 {
    type Error = Error;

    /// Tries to create a `DbDescFileV1` from a list of parsed sections.
    ///
    /// # Errors
    ///
    /// Returns an error if any required field is missing or if an unknown section is found.
    fn try_from(sections: Vec<Section>) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut version = None;
        let mut base = None;
        let mut desc = None;
        let mut url = None;
        let mut arch = None;
        let mut builddate = None;
        let mut installdate = None;
        let mut packager = None;
        let mut size = None;

        let mut groups = Vec::new();
        let mut reason = PackageInstallReason::Unknown;
        let mut license = Vec::new();
        let mut validation = Vec::new();
        let mut replaces = Vec::new();
        let mut depends = Vec::new();
        let mut optdepends = Vec::new();
        let mut conflicts = Vec::new();
        let mut provides = Vec::new();

        for section in sections {
            match section {
                Section::Name(v) => name = Some(v),
                Section::Version(v) => version = Some(v),
                Section::Base(v) => base = Some(v),
                Section::Desc(v) => desc = Some(v),
                Section::Url(v) => url = Some(v),
                Section::Arch(v) => arch = Some(v),
                Section::BuildDate(v) => builddate = Some(v),
                Section::InstallDate(v) => installdate = Some(v),
                Section::Packager(v) => packager = Some(v),
                Section::Size(v) => size = Some(v),
                Section::Groups(v) => groups = v,
                Section::Reason(v) => reason = v,
                Section::License(v) => license = v,
                Section::Validation(v) => validation = v,
                Section::Replaces(v) => replaces = v,
                Section::Depends(v) => depends = v,
                Section::OptDepends(v) => optdepends = v,
                Section::Conflicts(v) => conflicts = v,
                Section::Provides(v) => provides = v,
                Section::Unknown(v) => return Err(Error::UnknownSection(v)),
                _ => {}
            }
        }

        Ok(DbDescFileV1 {
            name: name.ok_or_else(|| Error::MissingSection("NAME".into()))?,
            version: version.ok_or_else(|| Error::MissingSection("VERSION".into()))?,
            base: base.ok_or_else(|| Error::MissingSection("BASE".into()))?,
            desc: desc.ok_or_else(|| Error::MissingSection("DESC".into()))?,
            url: url.ok_or_else(|| Error::MissingSection("URL".into()))?,
            arch: arch.ok_or_else(|| Error::MissingSection("ARCH".into()))?,
            builddate: builddate.ok_or_else(|| Error::MissingSection("BUILDDATE".into()))?,
            installdate: installdate.ok_or_else(|| Error::MissingSection("INSTALLDATE".into()))?,
            packager: packager.ok_or_else(|| Error::MissingSection("PACKAGER".into()))?,
            size: size.ok_or_else(|| Error::MissingSection("SIZE".into()))?,
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::*;
    use testresult::TestResult;

    use super::*;

    const VALID_DESC_FILE: &str = r#"
%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

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
            desc: PackageDescription::from("An example package"),
            url: Url::from_str("https://example.org")?,
            arch: Architecture::from_str("x86_64")?,
            builddate: BuildDate::from(1733737242),
            installdate: BuildDate::from(1733737243),
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            size: 123,
            groups: vec!["utils".into(), "cli".into()],
            reason: PackageInstallReason::Depend,
            license: vec![License::from_str("MIT")?, License::from_str("Apache-2.0")?],
            validation: vec![PackageValidation::from_str("pgp")?],
            replaces: vec![Name::new("pkg-old")?],
            depends: vec![PackageRelation::from_str("glibc")?],
            optdepends: vec![OptionalDependency::from_str("optpkg")?],
            conflicts: vec![Name::new("foo-old")?],
            provides: vec![Name::new("foo-virtual")?],
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[rstest]
    #[case("%UNKNOWN%\nvalue", "invalid section name")]
    #[case("%VERSION%\n1.0.0-1\n", "Missing field: NAME")]
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
        assert!(matches!(result, Err(Error::MissingSection(ref s)) if s == "NAME"));
    }
}
