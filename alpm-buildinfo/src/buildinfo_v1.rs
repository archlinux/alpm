use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use alpm_types::digests::Sha256;
use alpm_types::Architecture;
use alpm_types::BuildDate;
use alpm_types::BuildDir;
use alpm_types::BuildEnv;
use alpm_types::Checksum;
use alpm_types::InstalledPackage;
use alpm_types::Name;
use alpm_types::PackageOption;
use alpm_types::Packager;
use alpm_types::SchemaVersion;
use alpm_types::Version;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::schema::Schema;
use crate::Error;

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
/// let buildinfo_data = r#"format = 1
/// pkgname = foo
/// pkgbase = foo
/// pkgver = 1:1.0.0-1
/// pkgarch = any
/// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
/// packager = Foobar McFooface <foobar@mcfooface.org>
/// builddate = 1
/// builddir = /build
/// buildenv = envfoo
/// buildenv = envbar
/// options = some_option
/// options = !other_option
/// installed = bar-1.2.3-1-any
/// installed = beh-2.2.3-4-any
/// "#;
///
/// let buildinfo = BuildInfoV1::from_str(buildinfo_data).unwrap();
/// assert_eq!(buildinfo.to_string(), buildinfo_data);
/// ```
// NOTE: The fields are defined as `pub(crate)` to allow for internal reuse of this
// struct in `BuildInfoV2`. In other words, we can construct this struct without
// validating the format version, which is done in the `new` function.
#[serde_as]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInfoV1 {
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) format: Schema,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) pkgname: Name,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) pkgbase: Name,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) pkgver: Version,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) pkgarch: Architecture,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) pkgbuild_sha256sum: Checksum<Sha256>,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) packager: Packager,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) builddate: BuildDate,

    #[serde_as(as = "DisplayFromStr")]
    pub(crate) builddir: BuildDir,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub(crate) buildenv: Vec<BuildEnv>,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub(crate) options: Vec<PackageOption>,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub(crate) installed: Vec<InstalledPackage>,
}

impl BuildInfoV1 {
    /// Create a new BuildInfoV1 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        builddate: BuildDate,
        builddir: BuildDir,
        buildenv: Vec<BuildEnv>,
        format: SchemaVersion,
        installed: Vec<InstalledPackage>,
        options: Vec<PackageOption>,
        packager: Packager,
        pkgarch: Architecture,
        pkgbase: Name,
        pkgbuild_sha256sum: Checksum<Sha256>,
        pkgname: Name,
        pkgver: Version,
    ) -> Result<Self, Error> {
        if format.inner().major != 1 {
            return Err(Error::WrongSchemaVersion(format));
        }
        Ok(BuildInfoV1 {
            builddate,
            builddir,
            buildenv,
            format: Schema::try_from(format)?,
            installed,
            options,
            packager,
            pkgarch,
            pkgbase,
            pkgbuild_sha256sum,
            pkgname,
            pkgver,
        })
    }

    /// Returns the schema version of the package format.
    pub fn format(&self) -> &SchemaVersion {
        self.format.inner()
    }

    /// Returns the package name.
    pub fn pkgname(&self) -> &Name {
        &self.pkgname
    }

    /// Returns the base name of the package.
    pub fn pkgbase(&self) -> &Name {
        &self.pkgbase
    }

    /// Returns the package version.
    pub fn pkgver(&self) -> &Version {
        &self.pkgver
    }

    /// Returns the architecture of the package.
    pub fn pkgarch(&self) -> &Architecture {
        &self.pkgarch
    }

    /// Returns the SHA256 checksum of the PKGBUILD file.
    pub fn pkgbuild_sha256sum(&self) -> &Checksum<Sha256> {
        &self.pkgbuild_sha256sum
    }

    /// Returns information about the packager.
    pub fn packager(&self) -> &Packager {
        &self.packager
    }

    /// Returns the build date of the package.
    pub fn builddate(&self) -> &BuildDate {
        &self.builddate
    }

    /// Returns the directory where the build took place.
    pub fn builddir(&self) -> &BuildDir {
        &self.builddir
    }

    /// Returns the build environment variables.
    pub fn buildenv(&self) -> &Vec<BuildEnv> {
        &self.buildenv
    }

    /// Returns the options used during package building.
    pub fn options(&self) -> &Vec<PackageOption> {
        &self.options
    }

    /// Returns a list of installed dependencies or components.
    pub fn installed(&self) -> &Vec<InstalledPackage> {
        &self.installed
    }
}

impl FromStr for BuildInfoV1 {
    type Err = Error;
    /// Create a BuildInfoV1 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `BuildInfoV1` or their respective own specification.
    fn from_str(input: &str) -> Result<BuildInfoV1, Self::Err> {
        let buildinfo: BuildInfoV1 = alpm_parsers::custom_ini::from_str(input)?;
        if buildinfo.format().inner().major != 1 {
            return Err(Error::WrongSchemaVersion(buildinfo.format().clone()));
        }
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
            self.format().inner().major,
            self.pkgname(),
            self.pkgbase(),
            self.pkgver(),
            self.pkgarch(),
            self.pkgbuild_sha256sum(),
            self.packager(),
            self.builddate(),
            self.builddir(),
            self.buildenv()
                .iter()
                .map(|v| format!("buildenv = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
            self.options()
                .iter()
                .map(|v| format!("options = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
            self.installed()
                .iter()
                .map(|v| format!("installed = {v}"))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::fixture;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    #[fixture]
    fn valid_buildinfov1() -> String {
        r#"builddate = 1
builddir = /build
buildenv = envfoo
buildenv = envbar
format = 1
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
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
    fn buildinfov1() -> TestResult {
        assert!(BuildInfoV1::new(
            1,
            BuildDir::from_str("/build")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::from_str("1")?,
            vec![InstalledPackage::from_str("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo".to_string())?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo".to_string())?,
            Version::from_str("1:1.0.0-1")?,
        )
        .is_ok());
        Ok(())
    }

    #[rstest]
    fn buildinfov1_invalid_schemaversion() -> TestResult {
        assert!(BuildInfoV1::new(
            1,
            BuildDir::from_str("/build")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::from_str("2")?,
            vec![InstalledPackage::from_str("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo".to_string())?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo".to_string())?,
            Version::from_str("1:1.0.0-1")?,
        )
        .is_err());
        Ok(())
    }

    #[rstest]
    fn buildinfov1_from_str(valid_buildinfov1: String) {
        assert!(BuildInfoV1::from_str(&valid_buildinfov1).is_ok());
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
