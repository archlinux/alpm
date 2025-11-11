use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    BuildDate,
    BuildDirectory,
    BuildEnvironmentOption,
    Checksum,
    FullVersion,
    InstalledPackage,
    Name,
    PackageOption,
    Packager,
    SchemaVersion,
    digests::Sha256,
    semver_version::Version as SemverVersion,
};
use serde_with::{DisplayFromStr, serde_as};

use crate::{BuildInfoSchema, Error, build_info::format::BuildInfoFormat};

/// BUILDINFO version 1
///
/// `BuildInfoV1` is (exclusively) compatible with data following the first specification of the
/// BUILDINFO file.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_buildinfo::BuildInfoV1;
///
/// # fn main() -> Result<(), alpm_buildinfo::Error> {
/// let buildinfo_data = r#"format = 1
/// pkgname = foo
/// pkgbase = foo
/// pkgver = 1:1.0.0-1
/// pkgarch = any
/// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
/// packager = Foobar McFooface <foobar@mcfooface.org>
/// builddate = 1
/// builddir = /build
/// buildenv = ccache
/// buildenv = color
/// options = lto
/// options = !strip
/// installed = bar-1.2.3-1-any
/// installed = beh-2.2.3-4-any
/// "#;
///
/// let buildinfo = BuildInfoV1::from_str(buildinfo_data)?;
/// assert_eq!(buildinfo.to_string(), buildinfo_data);
/// # Ok(())
/// # }
/// ```
#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde_more::SerializeMore)]
#[more(key = "format", position = "front")]
pub struct BuildInfoV1 {
    /// The package name
    #[serde_as(as = "DisplayFromStr")]
    pub pkgname: Name,

    /// The package base name
    #[serde_as(as = "DisplayFromStr")]
    pub pkgbase: Name,

    /// The package version
    #[serde_as(as = "DisplayFromStr")]
    pub pkgver: FullVersion,

    /// The package architecture
    #[serde_as(as = "DisplayFromStr")]
    pub pkgarch: Architecture,

    /// The package build SHA-256 checksum
    #[serde_as(as = "DisplayFromStr")]
    pub pkgbuild_sha256sum: Checksum<Sha256>,

    /// The packager
    #[serde_as(as = "DisplayFromStr")]
    pub packager: Packager,

    /// The build date
    #[serde_as(as = "DisplayFromStr")]
    pub builddate: BuildDate,

    /// The build directory
    #[serde_as(as = "DisplayFromStr")]
    pub builddir: BuildDirectory,

    /// The build environment
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub buildenv: Vec<BuildEnvironmentOption>,

    /// The package options
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub options: Vec<PackageOption>,

    /// The installed packages
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub installed: Vec<InstalledPackage>,
}

impl BuildInfoV1 {
    /// Used by serde_more to serialize the additional `format` field.
    fn format(&self) -> String {
        BuildInfoSchema::V1(SchemaVersion::new(SemverVersion::new(1, 0, 0))).to_string()
    }
}

impl FromStr for BuildInfoV1 {
    type Err = Error;
    /// Create a BuildInfoV1 from a &str
    ///
    /// # Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `BuildInfoV1` or their respective own specification.
    fn from_str(input: &str) -> Result<BuildInfoV1, Self::Err> {
        let build_info_format: BuildInfoFormat = alpm_parsers::custom_ini::from_str(input)?;
        let schema_version: SchemaVersion = build_info_format.into();
        if schema_version.inner().major != 1 {
            return Err(Error::WrongSchemaVersion(schema_version));
        }

        let buildinfo: BuildInfoV1 = alpm_parsers::custom_ini::from_str(input)?;
        Ok(buildinfo)
    }
}

impl Display for BuildInfoV1 {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "format = {}\n\
            pkgname = {}\n\
            pkgbase = {}\n\
            pkgver = {}\n\
            pkgarch = {}\n\
            pkgbuild_sha256sum = {}\n\
            packager = {}\n\
            builddate = {}\n\
            builddir = {}\n\
            {}\n\
            {}\n\
            {}\n\
            ",
            self.format(),
            self.pkgname,
            self.pkgbase,
            self.pkgver,
            self.pkgarch,
            self.pkgbuild_sha256sum,
            self.packager,
            self.builddate,
            self.builddir,
            self.buildenv
                .iter()
                .map(|v| format!("buildenv = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
            self.options
                .iter()
                .map(|v| format!("options = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
            self.installed
                .iter()
                .map(|v| format!("installed = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};
    use testresult::TestResult;

    use super::*;

    #[fixture]
    fn valid_buildinfov1() -> String {
        r#"format = 1
builddate = 1
builddir = /build
buildenv = ccache
buildenv = color
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#
        .to_string()
    }

    #[fixture]
    fn invalid_format_buildinfov1() -> String {
        r#"format = 2
builddate = 1
builddir = /build
buildenv = ccache
buildenv = color
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#
        .to_string()
    }

    #[rstest]
    fn buildinfov1_from_str(valid_buildinfov1: String) -> TestResult {
        BuildInfoV1::from_str(&valid_buildinfov1)?;
        Ok(())
    }

    #[rstest]
    fn buildinfov1() -> TestResult {
        BuildInfoV1 {
            builddate: 1,
            builddir: BuildDirectory::from_str("/build")?,
            buildenv: vec![BuildEnvironmentOption::new("check")?],
            installed: vec![InstalledPackage::from_str("bar-1:1.0.0-2-any")?],
            options: vec![PackageOption::new("lto")?],
            packager: Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            pkgarch: Architecture::Any,
            pkgbase: Name::new("foo")?,
            pkgbuild_sha256sum: Checksum::<Sha256>::calculate_from("foo"),
            pkgname: Name::new("foo")?,
            pkgver: FullVersion::from_str("1:1.0.0-1")?,
        };
        Ok(())
    }

    #[rstest]
    fn buildinfov1_invalid_schemaversion(invalid_format_buildinfov1: String) -> TestResult {
        assert!(BuildInfoV1::from_str(&invalid_format_buildinfov1).is_err());
        Ok(())
    }

    #[rstest]
    #[case("builddate = 2")]
    #[case("builddir = /build2")]
    #[case("format = 1")]
    #[case("packager = Foobar McFooface <foobar@mcfooface.org>")]
    #[case("pkgarch = any")]
    #[case("pkgbase = foo")]
    #[case("pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c")]
    #[case("pkgname = foo")]
    #[case("pkgver = 1:1.0.0-1")]
    fn buildinfov1_from_str_duplicate_fail(mut valid_buildinfov1: String, #[case] duplicate: &str) {
        valid_buildinfov1.push_str(duplicate);
        assert!(BuildInfoV1::from_str(&valid_buildinfov1).is_err());
    }
}
