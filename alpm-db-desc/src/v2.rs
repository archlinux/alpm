//! Database desc file (v2)

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    BuildDate,
    ExtraData,
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
use winnow::Parser;

use crate::{
    DbDescFileV1,
    Error,
    PackageInstallReason,
    PackageValidation,
    Section,
    parser::{SectionKeyword, sections},
    v1::generate_dbdesc,
};

generate_dbdesc! {
    /// DB desc version 2
    DbDescFileV2 {
        /// Structured extra metadata, implementation-defined.
        xdata: Vec<ExtraData>,
    }
}

impl Display for DbDescFileV2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Reuse v1 formatting
        let base: DbDescFileV1 = self.clone().into();
        writeln!(f, "{base}")?;

        // Write xdata section
        writeln!(f, "%XDATA%")?;
        for v in &self.xdata {
            writeln!(f, "{v}")?;
        }
        Ok(())
    }
}

impl DbDescFileV2 {
    /// Create a new DbDescFileV2.
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
        xdata: Vec<ExtraData>,
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
            xdata,
        }
    }

    /// Returns the xdata of the database desc file.
    pub fn xdata(&self) -> &[ExtraData] {
        &self.xdata
    }
}

impl FromStr for DbDescFileV2 {
    type Err = Error;

    /// Parses a database desc file in version 2 format from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed into a valid [`DbDescFileV2`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = sections.parse(s)?;
        Self::try_from(sections)
    }
}

impl TryFrom<Vec<Section>> for DbDescFileV2 {
    type Error = Error;

    /// Tries to create a `DbDescFileV2` from a vector of parsed sections.
    ///
    /// # Errors
    ///
    /// Returns an error if any required section is missing or if an unknown section is
    /// encountered.
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
        let mut groups = vec![];
        let mut reason = None;
        let mut license = vec![];
        let mut validation = vec![];
        let mut replaces = vec![];
        let mut depends = vec![];
        let mut optdepends = vec![];
        let mut conflicts = vec![];
        let mut provides = vec![];
        let mut xdata = vec![];

        for section in sections {
            match section {
                Section::Name(v) => name = Some(v),
                Section::Version(v) => version = Some(v),
                Section::Base(v) => base = Some(v),
                Section::Desc(v) => description = Some(v),
                Section::Url(v) => url = Some(v),
                Section::Arch(v) => arch = Some(v),
                Section::BuildDate(v) => builddate = Some(v),
                Section::InstallDate(v) => installdate = Some(v),
                Section::Packager(v) => packager = Some(v),
                Section::Size(v) => size = Some(v),
                Section::Groups(v) => groups = v,
                Section::Reason(v) => reason = Some(v),
                Section::License(v) => license = v,
                Section::Validation(v) => validation = v,
                Section::Replaces(v) => replaces = v,
                Section::Depends(v) => depends = v,
                Section::OptDepends(v) => optdepends = v,
                Section::Conflicts(v) => conflicts = v,
                Section::Provides(v) => provides = v,
                Section::XData(v) => xdata = v,
            }
        }

        Ok(Self {
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
            xdata,
        })
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

%XDATA%
pkgtype=pkg
"#;

    #[test]
    fn parse_valid_v2_desc() -> TestResult {
        let actual = DbDescFileV2::from_str(VALID_DESC_FILE)?;
        let expected = DbDescFileV2 {
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
            xdata: vec![ExtraData::from_str("pkgtype=pkg")?],
        };
        assert_eq!(actual, expected);
        assert_eq!(VALID_DESC_FILE, actual.to_string());
        Ok(())
    }

    #[rstest]
    #[case("%UNKNOWN%\nvalue", "invalid section name")]
    #[case("%VERSION%\n1.0.0-1\n", "Missing section: NAME")]
    fn invalid_desc_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = DbDescFileV2::from_str(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
