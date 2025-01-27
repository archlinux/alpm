use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{
    digests::Sha256,
    Architecture,
    BuildDate,
    BuildDirectory,
    BuildEnv,
    BuildTool,
    BuildToolVersion,
    Checksum,
    InstalledPackage,
    Name,
    PackageOption,
    Packager,
    SchemaVersion,
    StartDir,
    Version,
};
use serde_with::{serde_as, DisplayFromStr};

use crate::{buildinfo_v1::generate_buildinfo, Error, Schema};

generate_buildinfo! {
    /// BUILDINFO version 2
    ///
    /// `BuildInfoV2` is (exclusively) compatible with data following the v2 specification of the
    /// BUILDINFO file.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_buildinfo::BuildInfoV2;
    ///
    /// # fn main() -> Result<(), alpm_buildinfo::Error> {
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
    /// let buildinfo = BuildInfoV2::from_str(buildinfo_data)?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    BuildInfoV2 {
        #[serde_as(as = "DisplayFromStr")]
        startdir: StartDir,

        #[serde_as(as = "DisplayFromStr")]
        buildtool: BuildTool,

        #[serde_as(as = "DisplayFromStr")]
        buildtoolver: BuildToolVersion,
    }
}

impl BuildInfoV2 {
    /// Create a new BuildInfoV2 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        builddate: BuildDate,
        builddir: BuildDirectory,
        startdir: StartDir,
        buildtool: BuildTool,
        buildtoolver: BuildToolVersion,
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
            startdir,
            buildtool,
            buildtoolver,
        })
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
    pub fn buildtoolver(&self) -> &BuildToolVersion {
        &self.buildtoolver
    }
}

impl FromStr for BuildInfoV2 {
    type Err = Error;
    /// Create a BuildInfoV2 from a &str
    ///
    /// # Errors
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
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    // Test data
    const VALID_BUILDINFOV2_CASE1: &str = r#"
builddate = 1
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
"#;

    // Test data without multiple values
    const VALID_BUILDINFOV2_CASE2: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
format = 2
installed = bar-1.2.3-1-any
options = some_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#;

    #[rstest]
    #[case(VALID_BUILDINFOV2_CASE1)]
    #[case(VALID_BUILDINFOV2_CASE2)]
    fn buildinfov2_from_str(#[case] buildinfo: &str) -> TestResult {
        BuildInfoV2::from_str(buildinfo)?;
        Ok(())
    }

    #[rstest]
    fn buildinfov2() -> TestResult {
        BuildInfoV2::new(
            1,
            BuildDirectory::from_str("/build")?,
            StartDir::from_str("/startdir/")?,
            BuildTool::from_str("devtools")?,
            BuildToolVersion::from_str("1:1.2.1-1-any")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::from_str("2")?,
            vec![InstalledPackage::from_str("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo")?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo")?,
            Version::from_str("1:1.0.0-1")?,
        )?;
        Ok(())
    }

    #[rstest]
    fn buildinfov2_invalid_schemaversion() -> TestResult {
        assert!(BuildInfoV2::new(
            1,
            BuildDirectory::from_str("/build")?,
            StartDir::from_str("/startdir/")?,
            BuildTool::from_str("devtools")?,
            BuildToolVersion::from_str("1:1.2.1-1-any")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::from_str("1")?,
            vec![InstalledPackage::from_str("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::from_str("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo")?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo")?,
            Version::from_str("1:1.0.0-1")?,
        )
        .is_err());
        Ok(())
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
    fn buildinfov2_from_str_duplicate_fail(#[case] duplicate: &str) {
        let mut buildinfov2 = VALID_BUILDINFOV2_CASE1.to_string();
        buildinfov2.push_str(duplicate);
        assert!(BuildInfoV2::from_str(&buildinfov2).is_err());
    }
}
