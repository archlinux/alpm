//! Representation of the database desc file v2 ([alpm-db-descv2]).
//!
//! [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html

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
        DbDescFileV1,
        Section,
        parser::{SectionKeyword, sections},
        v1::generate_dbdesc,
    },
};

generate_dbdesc! {
    /// DB desc version 2
    ///
    /// `DbDescFileV2` extends [`DbDescFileV1`] according to the second revision of the
    /// [alpm-db-desc] specification. It introduces an additional `%XDATA%` section, which allows
    /// storing structured, implementation-defined metadata.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_db::desc::DbDescFileV2;
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
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///
    /// // Parse a DB DESC file in version 2 format.
    /// let db_desc = DbDescFileV2::from_str(desc_data)?;
    /// // Convert back to its canonical string representation.
    /// assert_eq!(db_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    DbDescFileV2 {
        /// Structured extra metadata, implementation-defined.
        pub xdata: Vec<ExtraData>,
    }
}

impl Display for DbDescFileV2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Reuse v1 formatting
        let base: DbDescFileV1 = self.clone().into();
        write!(f, "{base}")?;

        // Write xdata section
        writeln!(f, "%XDATA%")?;
        for v in &self.xdata {
            writeln!(f, "{v}")?;
        }

        writeln!(f)
    }
}

impl DbDescFileV2 {
    /// Creates a new [`DbDescFileV2`] from all required components.
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

    /// Returns the xdata entries.
    pub fn xdata(&self) -> &[ExtraData] {
        &self.xdata
    }
}

impl FromStr for DbDescFileV2 {
    type Err = Error;

    /// Creates a [`DbDescFileV2`] from a string slice.
    ///
    /// Parses the input according to the [alpm-db-descv2] specification (version 2) and constructs
    /// a structured [`DbDescFileV2`] representation including the `%XDATA%` section.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_db::desc::DbDescFileV2;
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
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///
    /// let db_desc = DbDescFileV2::from_str(desc_data)?;
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
    /// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = sections.parse(s)?;
        Self::try_from(sections)
    }
}

impl TryFrom<Vec<Section>> for DbDescFileV2 {
    type Error = Error;

    /// Tries to create a [`DbDescFileV2`] from a list of parsed [`Section`]s.
    ///
    /// Reuses the parsing logic from [`DbDescFileV1`] for all common fields, and adds support for
    /// the `%XDATA%` section introduced in the [alpm-db-descv2] specification.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - any required field is missing,
    /// - a section appears more than once,
    /// - or the `%XDATA%` section is missing or malformed.
    ///
    /// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
    fn try_from(sections: Vec<Section>) -> Result<Self, Self::Error> {
        // Reuse v1 fields
        let v1 = DbDescFileV1::try_from(sections.clone())?;

        // Find xdata section
        let xdata = if let Some(Section::XData(v)) =
            sections.iter().find(|s| matches!(s, Section::XData(_)))
        {
            v.clone()
        } else {
            return Err(Error::MissingSection(SectionKeyword::XData));
        };

        Ok(Self {
            name: v1.name,
            version: v1.version,
            base: v1.base,
            description: v1.description,
            url: v1.url,
            arch: v1.arch,
            builddate: v1.builddate,
            installdate: v1.installdate,
            packager: v1.packager,
            size: v1.size,
            groups: v1.groups,
            reason: v1.reason,
            license: v1.license,
            validation: v1.validation,
            replaces: v1.replaces,
            depends: v1.depends,
            optdepends: v1.optdepends,
            conflicts: v1.conflicts,
            provides: v1.provides,
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
    #[case("%VERSION%\n1.0.0-1\n", "Missing section: %NAME%")]
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
