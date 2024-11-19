use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use alpm_types::digests::Sha256;
use alpm_types::Architecture;
use alpm_types::BuildDate;
use alpm_types::BuildDir;
use alpm_types::BuildEnv;
use alpm_types::BuildTool;
use alpm_types::BuildToolVer;
use alpm_types::Checksum;
use alpm_types::InstalledPackage;
use alpm_types::Name;
use alpm_types::PackageOption;
use alpm_types::Packager;
use alpm_types::SchemaVersion;
use alpm_types::StartDir;
use alpm_types::Version;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::BuildInfoV1;
use crate::Error;

/// BUILDINFO version 2
///
/// `BuildInfoV2` is (exclusively) compatible with data following the first specification of the
/// BUILDINFO file.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_buildinfo::BuildInfoV2;
///
/// let buildinfo_data = r#"format = 2
/// pkgname = foo
/// pkgbase = foo
/// pkgver = 1:1.0.0-1
/// pkgarch = any
/// pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
/// packager = Foobar McFooface <foobar@mcfooface.org>
/// builddate = 1
/// builddir = /build
/// startdir = /startdir/
/// buildtool = devtools
/// buildtoolver = 1:1.2.1-1-any
/// buildenv = envfoo
/// buildenv = envbar
/// options = some_option
/// options = !other_option
/// installed = bar-1.2.3-1-any
/// installed = beh-2.2.3-4-any
/// "#;
///
/// let buildinfo = BuildInfoV2::from_str(buildinfo_data).unwrap();
/// assert_eq!(buildinfo.to_string(), buildinfo_data);
/// ```
#[serde_as]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInfoV2 {
    // Carry over all fields from BuildInfoV1
    #[serde(flatten)]
    v1: BuildInfoV1,

    #[serde_as(as = "DisplayFromStr")]
    startdir: StartDir,

    #[serde_as(as = "DisplayFromStr")]
    buildtool: BuildTool,

    #[serde_as(as = "DisplayFromStr")]
    buildtoolver: BuildToolVer,
}

impl BuildInfoV2 {
    /// Create a new BuildInfoV2 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        builddate: BuildDate,
        builddir: BuildDir,
        startdir: StartDir,
        buildtool: BuildTool,
        buildtoolver: BuildToolVer,
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
        if format.inner().major != 2 {
            return Err(Error::WrongSchemaVersion(format));
        }
        Ok(BuildInfoV2 {
            startdir,
            buildtool,
            buildtoolver,
            v1: BuildInfoV1 {
                builddate,
                builddir,
                buildenv,
                format: format.try_into()?,
                installed,
                options,
                packager,
                pkgarch,
                pkgbase,
                pkgbuild_sha256sum,
                pkgname,
                pkgver,
            },
        })
    }

    /// Returns the schema version of the package format.
    pub fn format(&self) -> &SchemaVersion {
        self.v1.format()
    }

    /// Returns the package name.
    pub fn pkgname(&self) -> &Name {
        self.v1.pkgname()
    }

    /// Returns the base name of the package.
    pub fn pkgbase(&self) -> &Name {
        self.v1.pkgbase()
    }

    /// Returns the package version.
    pub fn pkgver(&self) -> &Version {
        self.v1.pkgver()
    }

    /// Returns the architecture of the package.
    pub fn pkgarch(&self) -> &Architecture {
        self.v1.pkgarch()
    }

    /// Returns the SHA256 checksum of the PKGBUILD file.
    pub fn pkgbuild_sha256sum(&self) -> &Checksum<Sha256> {
        self.v1.pkgbuild_sha256sum()
    }

    /// Returns information about the packager.
    pub fn packager(&self) -> &Packager {
        self.v1.packager()
    }

    /// Returns the build date of the package.
    pub fn builddate(&self) -> &BuildDate {
        self.v1.builddate()
    }

    /// Returns the directory where the build took place.
    pub fn builddir(&self) -> &BuildDir {
        self.v1.builddir()
    }

    /// Returns the build environment variables.
    pub fn buildenv(&self) -> &Vec<BuildEnv> {
        self.v1.buildenv()
    }

    /// Returns the options used during package building.
    pub fn options(&self) -> &Vec<PackageOption> {
        self.v1.options()
    }

    /// Returns a list of installed dependencies or components.
    pub fn installed(&self) -> &Vec<InstalledPackage> {
        self.v1.installed()
    }

    /// Returns the start directory of the build process.
    pub fn startdir(&self) -> &StartDir {
        &self.startdir
    }

    /// Returns the tool used for building the package.
    pub fn buildtool(&self) -> &BuildTool {
        &self.buildtool
    }

    /// Returns the version of the build tool.
    pub fn buildtoolver(&self) -> &BuildToolVer {
        &self.buildtoolver
    }
}

impl FromStr for BuildInfoV2 {
    type Err = Error;
    /// Create a BuildInfoV2 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `BuildInfoV2` or their respective own specification.
    fn from_str(input: &str) -> Result<BuildInfoV2, Self::Err> {
        let buildinfo: BuildInfoV2 = alpm_parsers::custom_ini::from_str(input)?;
        if buildinfo.format().inner().major != 2 {
            return Err(Error::WrongSchemaVersion(buildinfo.format().clone()));
        }
        Ok(buildinfo)
    }
}

impl Display for BuildInfoV2 {
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
            startdir = {}\n\
            buildtool = {}\n\
            buildtoolver = {}\n\
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
            self.startdir(),
            self.buildtool(),
            self.buildtoolver(),
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
    fn valid_buildinfov2() -> String {
        r#"builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
buildenv = envbar
format = 2
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
    fn buildinfov2() -> TestResult {
        BuildInfoV2::new(
            1,
            BuildDir::new("/build")?,
            StartDir::new("/startdir/")?,
            BuildTool::new("devtools")?,
            BuildToolVer::new("1:1.2.1-1-any")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::new("2")?,
            vec![InstalledPackage::new("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::new("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo".to_string())?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo".to_string())?,
            Version::new("1:1.0.0-1")?,
        )
        .unwrap();
        Ok(())
    }

    #[rstest]
    fn buildinfov2_invalid_schemaversion() -> TestResult {
        assert!(BuildInfoV2::new(
            1,
            BuildDir::new("/build")?,
            StartDir::new("/startdir/")?,
            BuildTool::new("devtools")?,
            BuildToolVer::new("1:1.2.1-1-any")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::new("1")?,
            vec![InstalledPackage::new("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::new("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo".to_string())?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo".to_string())?,
            Version::new("1:1.0.0-1")?,
        )
        .is_err());
        Ok(())
    }

    #[rstest]
    fn buildinfov2_from_str(valid_buildinfov2: String) {
        BuildInfoV2::from_str(&valid_buildinfov2).unwrap();
    }

    #[rstest]
    #[case("builddate = 2")]
    #[case("builddir = /build2")]
    #[case("startdir = /startdir2/")]
    #[case("buildtool = devtools2")]
    #[case("buildtoolver = 1:1.2.1-2-any")]
    #[case("format = 1")]
    #[case("packager = Foobar McFooface <foobar@mcfooface.org>")]
    #[case("pkgarch = any")]
    #[case("pkgbase = foo")]
    #[case("pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c")]
    #[case("pkgname = foo")]
    #[case("pkgver = 1:1.0.0-1")]
    fn buildinfov2_from_str_duplicate_fail(mut valid_buildinfov2: String, #[case] duplicate: &str) {
        valid_buildinfov2.push_str(duplicate);
        assert!(BuildInfoV2::from_str(&valid_buildinfov2).is_err());
    }
}
