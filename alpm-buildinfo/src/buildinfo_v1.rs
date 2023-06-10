// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use alpm_types::digests::Sha256;
use alpm_types::Architecture;
use alpm_types::BuildDate;
use alpm_types::BuildDir;
use alpm_types::BuildEnv;
use alpm_types::Checksum;
use alpm_types::Installed;
use alpm_types::Name;
use alpm_types::PackageOption;
use alpm_types::Packager;
use alpm_types::SchemaVersion;
use alpm_types::Version;

use crate::common::ensure_mandatory_field;
use crate::common::get_multiple;
use crate::common::get_once;
use crate::common::get_once_strum;
use crate::common::keyword_with_list_entries;
use crate::common::KeyAssign;
use crate::Error;

/// BUILDINFO version 1
///
/// `BuildInfoV1` is (exclusively) compatible with data following the first specification of the BUILDINFO file.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
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
#[derive(Clone, Debug)]
pub struct BuildInfoV1 {
    builddate: BuildDate,
    builddir: BuildDir,
    buildenv: Vec<BuildEnv>,
    format: SchemaVersion,
    installed: Vec<Installed>,
    options: Vec<PackageOption>,
    packager: Packager,
    pkgarch: Architecture,
    pkgbase: Name,
    pkgbuild_sha256sum: Checksum<Sha256>,
    pkgname: Name,
    pkgver: Version,
}

impl BuildInfoV1 {
    /// Create a new BuildInfoV1 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        builddate: BuildDate,
        builddir: BuildDir,
        buildenv: Vec<BuildEnv>,
        format: SchemaVersion,
        installed: Vec<Installed>,
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
            format,
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
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to `BuildInfoV1` or their
    /// respective own specification.
    fn from_str(input: &str) -> Result<BuildInfoV1, Self::Err> {
        let buildinfo_version = "1";

        let builddate_keyassign = KeyAssign::new("builddate".to_string());
        let builddir_keyassign = KeyAssign::new("builddir".to_string());
        let buildenv_keyassign = KeyAssign::new("buildenv".to_string());
        let format_keyassign = KeyAssign::new("format".to_string());
        let installed_keyassign = KeyAssign::new("installed".to_string());
        let option_keyassign = KeyAssign::new("options".to_string());
        let packager_keyassign = KeyAssign::new("packager".to_string());
        let pkgarch_keyassign = KeyAssign::new("pkgarch".to_string());
        let pkgbase_keyassign = KeyAssign::new("pkgbase".to_string());
        let pkgbuild_sha256sum_keyassign = KeyAssign::new("pkgbuild_sha256sum".to_string());
        let pkgname_keyassign = KeyAssign::new("pkgname".to_string());
        let pkgver_keyassign = KeyAssign::new("pkgver".to_string());

        let mut builddate: Option<BuildDate> = None;
        let mut builddir: Option<BuildDir> = None;
        let mut buildenv: Vec<BuildEnv> = vec![];
        let mut format: Option<SchemaVersion> = None;
        let mut installed: Vec<Installed> = vec![];
        let mut options: Vec<PackageOption> = vec![];
        let mut packager: Option<Packager> = None;
        let mut pkgarch: Option<Architecture> = None;
        let mut pkgbase: Option<Name> = None;
        let mut pkgbuild_sha256sum: Option<Checksum<Sha256>> = None;
        let mut pkgname: Option<Name> = None;
        let mut pkgver: Option<Version> = None;

        for (number, line) in input.lines().enumerate() {
            if line.starts_with(&builddate_keyassign.to_string()) {
                builddate = get_once(
                    &builddate_keyassign,
                    builddate,
                    line,
                    number,
                    buildinfo_version,
                )?;
            } else if line.starts_with(&builddir_keyassign.to_string()) {
                builddir = get_once(
                    &builddir_keyassign,
                    builddir,
                    line,
                    number,
                    buildinfo_version,
                )?;
            } else if line.starts_with(&buildenv_keyassign.to_string()) {
                buildenv.push(get_multiple::<BuildEnv>(
                    &buildenv_keyassign,
                    line,
                    number,
                    buildinfo_version,
                )?);
            } else if line.starts_with(&format_keyassign.to_string()) {
                format = get_once(&format_keyassign, format, line, number, buildinfo_version)?;
            } else if line.starts_with(&installed_keyassign.to_string()) {
                installed.push(get_multiple::<Installed>(
                    &installed_keyassign,
                    line,
                    number,
                    buildinfo_version,
                )?);
            } else if line.starts_with(&option_keyassign.to_string()) {
                options.push(get_multiple::<PackageOption>(
                    &option_keyassign,
                    line,
                    number,
                    buildinfo_version,
                )?);
            } else if line.starts_with(&packager_keyassign.to_string()) {
                packager = get_once(
                    &packager_keyassign,
                    packager,
                    line,
                    number,
                    buildinfo_version,
                )?;
            } else if line.starts_with(&pkgarch_keyassign.to_string()) {
                pkgarch =
                    get_once_strum(&pkgarch_keyassign, pkgarch, line, number, buildinfo_version)?;
            } else if line.starts_with(&pkgbase_keyassign.to_string()) {
                pkgbase = get_once(&pkgbase_keyassign, pkgbase, line, number, buildinfo_version)?;
            } else if line.starts_with(&pkgbuild_sha256sum_keyassign.to_string()) {
                pkgbuild_sha256sum = get_once(
                    &pkgbuild_sha256sum_keyassign,
                    pkgbuild_sha256sum,
                    line,
                    number,
                    buildinfo_version,
                )?;
            }
            if line.starts_with(&pkgname_keyassign.to_string()) {
                pkgname = get_once(&pkgname_keyassign, pkgname, line, number, buildinfo_version)?;
            }
            if line.starts_with(&pkgver_keyassign.to_string()) {
                pkgver = get_once(&pkgver_keyassign, pkgver, line, number, buildinfo_version)?;
            }
        }

        BuildInfoV1::new(
            ensure_mandatory_field(builddate, builddate_keyassign.inner(), buildinfo_version)?,
            ensure_mandatory_field(builddir, builddir_keyassign.inner(), buildinfo_version)?,
            buildenv,
            ensure_mandatory_field(format, format_keyassign.inner(), buildinfo_version)?,
            installed,
            options,
            ensure_mandatory_field(packager, packager_keyassign.inner(), buildinfo_version)?,
            ensure_mandatory_field(pkgarch, pkgarch_keyassign.inner(), buildinfo_version)?,
            ensure_mandatory_field(pkgbase, pkgbase_keyassign.inner(), buildinfo_version)?,
            ensure_mandatory_field(
                pkgbuild_sha256sum,
                pkgbuild_sha256sum_keyassign.inner(),
                buildinfo_version,
            )?,
            ensure_mandatory_field(pkgname, pkgname_keyassign.inner(), buildinfo_version)?,
            ensure_mandatory_field(pkgver, pkgver_keyassign.inner(), buildinfo_version)?,
        )
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
            self.format.inner().major,
            self.pkgname,
            self.pkgbase,
            self.pkgver,
            self.pkgarch,
            self.pkgbuild_sha256sum,
            self.packager,
            self.builddate,
            self.builddir,
            keyword_with_list_entries(&KeyAssign::new("buildenv".to_string()), &self.buildenv),
            keyword_with_list_entries(&KeyAssign::new("options".to_string()), &self.options),
            keyword_with_list_entries(&KeyAssign::new("installed".to_string()), &self.installed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::fixture;
    use rstest::rstest;
    use testresult::TestResult;

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
            BuildDate::new(1),
            BuildDir::new("/build")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::new("1")?,
            vec![Installed::new("bar-1:1.0.0-2-any")?],
            vec![PackageOption::new("buildoption")?],
            Packager::new("Foobar McFooface <foobar@mcfooface.org>")?,
            Architecture::Any,
            Name::new("foo".to_string())?,
            Checksum::<Sha256>::calculate_from("foo"),
            Name::new("foo".to_string())?,
            Version::new("1:1.0.0-1")?,
        )
        .is_ok());
        Ok(())
    }

    #[rstest]
    fn buildinfov1_invalid_schemaversion() -> TestResult {
        assert!(BuildInfoV1::new(
            BuildDate::new(1),
            BuildDir::new("/build")?,
            vec![BuildEnv::new("some")?],
            SchemaVersion::new("2")?,
            vec![Installed::new("bar-1:1.0.0-2-any")?],
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
