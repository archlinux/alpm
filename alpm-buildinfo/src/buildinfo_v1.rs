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
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::schema::Schema;
use crate::Error;

/// Generates a struct based on the BUILDINFO version 1 specification with additional fields.
macro_rules! generate_buildinfo {
    // Meta: The meta information for the struct (e.g. doc comments)
    // Name: The name of the struct
    // Extra fields: Additional fields that should be added to the struct
    ($(#[$meta:meta])* $name:ident { $($extra_fields:tt)* }) => {
        $(#[$meta])*
        #[serde_as]
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            #[serde_as(as = "DisplayFromStr")]
            format: Schema,

            #[serde_as(as = "DisplayFromStr")]
            pkgname: Name,

            #[serde_as(as = "DisplayFromStr")]
            pkgbase: Name,

            #[serde_as(as = "DisplayFromStr")]
            pkgver: Version,

            #[serde_as(as = "DisplayFromStr")]
            pkgarch: Architecture,

            #[serde_as(as = "DisplayFromStr")]
            pkgbuild_sha256sum: Checksum<Sha256>,

            #[serde_as(as = "DisplayFromStr")]
            packager: Packager,

            #[serde_as(as = "DisplayFromStr")]
            builddate: BuildDate,

            #[serde_as(as = "DisplayFromStr")]
            builddir: BuildDir,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            buildenv: Vec<BuildEnv>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            options: Vec<PackageOption>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            installed: Vec<InstalledPackage>,

            $($extra_fields)*
        }

        impl $name {
            /// Returns the format of the BUILDINFO file
            pub fn format(&self) -> &SchemaVersion {
                self.format.inner()
            }

            /// Returns the package name
            pub fn pkgname(&self) -> &Name {
                &self.pkgname
            }

            /// Returns the package base
            pub fn pkgbase(&self) -> &Name {
                &self.pkgbase
            }

            /// Returns the package version
            pub fn pkgver(&self) -> &Version {
                &self.pkgver
            }

            /// Returns the package architecture
            pub fn pkgarch(&self) -> &Architecture {
                &self.pkgarch
            }

            /// Returns the package build SHA-256 checksum
            pub fn pkgbuild_sha256sum(&self) -> &Checksum<Sha256> {
                &self.pkgbuild_sha256sum
            }

            /// Returns the packager
            pub fn packager(&self) -> &Packager {
                &self.packager
            }

            /// Returns the build date
            pub fn builddate(&self) -> &BuildDate {
                &self.builddate
            }

            /// Returns the build directory
            pub fn builddir(&self) -> &BuildDir {
                &self.builddir
            }

            /// Returns the build environment
            pub fn buildenv(&self) -> &Vec<BuildEnv> {
                &self.buildenv
            }

            /// Returns the package options
            pub fn options(&self) -> &Vec<PackageOption> {
                &self.options
            }

            /// Returns the installed packages
            pub fn installed(&self) -> &Vec<InstalledPackage> {
                &self.installed
            }
        }
    }
}

pub(crate) use generate_buildinfo;

generate_buildinfo! {
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
    /// buildenv = envfoo
    /// buildenv = envbar
    /// options = some_option
    /// options = !other_option
    /// installed = bar-1.2.3-1-any
    /// installed = beh-2.2.3-4-any
    /// "#;
    ///
    /// let buildinfo = BuildInfoV1::from_str(buildinfo_data)?;
    /// assert_eq!(buildinfo.to_string(), buildinfo_data);
    /// # Ok(())
    /// # }
    /// ```
    BuildInfoV1 {}
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
    fn buildinfov1_from_str(valid_buildinfov1: String) -> TestResult {
        BuildInfoV1::from_str(&valid_buildinfov1)?;
        Ok(())
    }

    #[rstest]
    fn buildinfov1() -> TestResult {
        BuildInfoV1::new(
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
        )?;
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
