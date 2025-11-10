//! The [PKGINFOv2] file format.
//!
//! [PKGINFOv2]: https://alpm.archlinux.page/specifications/PKGINFOv2.5.html

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    Backup,
    BuildDate,
    ExtraData,
    ExtraDataEntry,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    Packager,
    Url,
};
use serde_with::{DisplayFromStr, TryFromInto, serde_as};

use crate::{Error, RelationOrSoname};

/// PKGINFO version 2
///
/// `PackageInfoV2` is (exclusively) compatible with data following the v2 specification of the
/// PKGINFO file.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_pkginfo::PackageInfoV2;
///
/// # fn main() -> Result<(), alpm_pkginfo::Error> {
/// let pkginfo_data = r#"pkgname = example
/// pkgbase = example
/// xdata = pkgtype=pkg
/// pkgver = 1:1.0.0-1
/// pkgdesc = A project that does something
/// url = https://example.org/
/// builddate = 1729181726
/// packager = John Doe <john@example.org>
/// size = 181849963
/// arch = any
/// license = GPL-3.0-or-later
/// license = LGPL-3.0-or-later
/// replaces = other-package>0.9.0-3
/// group = package-group
/// group = other-package-group
/// conflict = conflicting-package<1.0.0
/// conflict = other-conflicting-package<1.0.0
/// provides = some-component
/// provides = some-other-component=1:1.0.0-1
/// provides = libexample.so=1-64
/// provides = libunversionedexample.so=libunversionedexample.so-64
/// provides = lib:libexample.so.1
/// backup = etc/example/config.toml
/// backup = etc/example/other-config.txt
/// depend = glibc
/// depend = gcc-libs
/// depend = libother.so=0-64
/// depend = libunversioned.so=libunversioned.so-64
/// depend = lib:libother.so.0
/// optdepend = python: for special-python-script.py
/// optdepend = ruby: for special-ruby-script.rb
/// makedepend = cmake
/// makedepend = python-sphinx
/// checkdepend = extra-test-tool
/// checkdepend = other-extra-test-tool"#;
/// let pkginfo = PackageInfoV2::from_str(pkginfo_data)?;
/// assert_eq!(pkginfo.to_string(), pkginfo_data);
/// # Ok(())
/// # }
/// ```
#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageInfoV2 {
    /// The name of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub pkgname: Name,

    /// The base name of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub pkgbase: Name,

    /// The version of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub pkgver: FullVersion,

    /// The description of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub pkgdesc: PackageDescription,

    /// The URL of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub url: Url,

    /// The build date of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub builddate: BuildDate,

    /// The packager of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub packager: Packager,

    /// The size of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub size: InstalledSize,

    /// The architecture of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub arch: Architecture,

    /// The licenses of the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub license: Vec<License>,

    /// The packages this package replaces.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub replaces: Vec<PackageRelation>,

    /// The groups this package belongs to.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub group: Vec<Group>,

    /// The packages this package conflicts with.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub conflict: Vec<PackageRelation>,

    /// The packages this package provides.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub provides: Vec<RelationOrSoname>,

    /// The backup files of the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub backup: Vec<Backup>,

    /// The dependencies of the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub depend: Vec<RelationOrSoname>,

    /// The optional dependencies of the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub optdepend: Vec<OptionalDependency>,

    /// The packages required to build this package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub makedepend: Vec<PackageRelation>,

    /// The packages this package is checked with.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub checkdepend: Vec<PackageRelation>,

    /// Extra data of the package.
    #[serde_as(as = "TryFromInto<Vec<ExtraDataEntry>>")]
    pub xdata: ExtraData,
}

impl FromStr for PackageInfoV2 {
    type Err = Error;
    /// Create a PackageInfoV2 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `PackageInfoV2` or their respective own specification.
    fn from_str(input: &str) -> Result<PackageInfoV2, Self::Err> {
        let pkg_info: PackageInfoV2 = alpm_parsers::custom_ini::from_str(input)?;
        Ok(pkg_info)
    }
}

impl Display for PackageInfoV2 {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fn format_list(label: &str, items: &[impl Display]) -> String {
            if items.is_empty() {
                String::new()
            } else {
                items
                    .iter()
                    .map(|v| format!("{label} = {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n"
            }
        }
        let pkg_type = self.xdata.pkg_type();
        let other_xdata = self
            .xdata
            .as_ref()
            .iter()
            .filter(|v| v.key() != "pkgtype")
            .collect::<Vec<_>>();
        write!(
            fmt,
            "pkgname = {}\n\
            pkgbase = {}\n\
            xdata = pkgtype={pkg_type}\n\
            pkgver = {}\n\
            pkgdesc = {}\n\
            url = {}\n\
            builddate = {}\n\
            packager = {}\n\
            size = {}\n\
            arch = {}\n\
            {}\
            {}\
            {}\
            {}\
            {}\
            {}\
            {}\
            {}\
            {}\
            {}{}",
            self.pkgname,
            self.pkgbase,
            self.pkgver,
            self.pkgdesc,
            self.url,
            self.builddate,
            self.packager,
            self.size,
            self.arch,
            format_list("license", &self.license),
            format_list("replaces", &self.replaces),
            format_list("group", &self.group),
            format_list("conflict", &self.conflict),
            format_list("provides", &self.provides),
            format_list("backup", &self.backup),
            format_list("depend", &self.depend),
            format_list("optdepend", &self.optdepend),
            format_list("makedepend", &self.makedepend),
            format_list("checkdepend", &self.checkdepend).trim_end_matches('\n'),
            if other_xdata.is_empty() {
                String::new()
            } else {
                format!(
                    "\n{}",
                    other_xdata
                        .iter()
                        .map(|v| format!("xdata = {v}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use alpm_types::PackageType;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    // Test data
    const VALID_PKGINFOV2_CASE1: &str = r#"pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
provides = libexample.so=1-64
provides = libunversionedexample.so=libunversionedexample.so-64
provides = lib:libexample.so.1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
depend = libother.so=0-64
depend = libunversioned.so=libunversioned.so-64
depend = lib:libother.so.0
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool"#;

    // Test data without multiple values
    const VALID_PKGINFOV2_CASE2: &str = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
conflict = conflicting-package<1.0.0
provides = some-component
backup = etc/example/config.toml
depend = glibc
optdepend = python: for special-python-script.py
makedepend = cmake
checkdepend = extra-test-tool
"#;

    #[rstest]
    #[case(VALID_PKGINFOV2_CASE1)]
    #[case(VALID_PKGINFOV2_CASE2)]
    fn pkginfov2_from_str(#[case] pkginfo: &str) -> TestResult {
        PackageInfoV2::from_str(pkginfo)?;
        Ok(())
    }

    fn pkg_info() -> TestResult<PackageInfoV2> {
        let pkg_info = PackageInfoV2 {
            pkgname: Name::new("example")?,
            pkgbase: Name::new("example")?,
            pkgver: FullVersion::from_str("1:1.0.0-1")?,
            pkgdesc: PackageDescription::from("A project that does something"),
            url: Url::from_str("https://example.org")?,
            builddate: BuildDate::from_str("1729181726")?,
            packager: Packager::from_str("John Doe <john@example.org>")?,
            size: InstalledSize::from_str("181849963")?,
            arch: Architecture::Any,
            license: vec![
                License::from_str("GPL-3.0-or-later")?,
                License::from_str("LGPL-3.0-or-later")?,
            ],
            replaces: vec![PackageRelation::from_str("other-package>0.9.0-3")?],
            group: vec![
                Group::from_str("package-group")?,
                Group::from_str("other-package-group")?,
            ],
            conflict: vec![
                PackageRelation::from_str("conflicting-package<1.0.0")?,
                PackageRelation::from_str("other-conflicting-package<1.0.0")?,
            ],
            provides: vec![
                RelationOrSoname::from_str("some-component")?,
                RelationOrSoname::from_str("some-other-component=1:1.0.0-1")?,
                RelationOrSoname::from_str("libexample.so=1-64")?,
                RelationOrSoname::from_str("libunversionedexample.so=libunversionedexample.so-64")?,
                RelationOrSoname::from_str("lib:libexample.so.1")?,
            ],
            backup: vec![
                Backup::from_str("etc/example/config.toml")?,
                Backup::from_str("etc/example/other-config.txt")?,
            ],
            depend: vec![
                RelationOrSoname::from_str("glibc")?,
                RelationOrSoname::from_str("gcc-libs")?,
                RelationOrSoname::from_str("libother.so=0-64")?,
                RelationOrSoname::from_str("libunversioned.so=libunversioned.so-64")?,
                RelationOrSoname::from_str("lib:libother.so.0")?,
            ],
            optdepend: vec![
                OptionalDependency::from_str("python: for special-python-script.py")?,
                OptionalDependency::from_str("ruby: for special-ruby-script.rb")?,
            ],
            makedepend: vec![
                PackageRelation::from_str("cmake")?,
                PackageRelation::from_str("python-sphinx")?,
            ],
            checkdepend: vec![
                PackageRelation::from_str("extra-test-tool")?,
                PackageRelation::from_str("other-extra-test-tool")?,
            ],
            xdata: ExtraDataEntry::from_str("pkgtype=pkg")?.try_into()?,
        };
        assert_eq!(PackageType::Package, pkg_info.xdata.pkg_type());
        Ok(pkg_info)
    }

    #[rstest]
    fn pkginfov2() -> TestResult {
        let pkg_info = pkg_info()?;
        assert_eq!(pkg_info.to_string(), VALID_PKGINFOV2_CASE1);
        Ok(())
    }

    #[rstest]
    fn pkginfov2_multiple_xdata() -> TestResult {
        let mut pkg_info = pkg_info()?;
        let mut xdata = pkg_info.xdata.into_iter().collect::<Vec<_>>();
        xdata.push(ExtraDataEntry::from_str("foo=bar")?);
        xdata.push(ExtraDataEntry::from_str("baz=qux")?);
        pkg_info.xdata = xdata.try_into()?;
        assert_eq!(
            pkg_info.to_string(),
            format!("{VALID_PKGINFOV2_CASE1}\nxdata = foo=bar\nxdata = baz=qux")
        );
        Ok(())
    }

    #[rstest]
    #[case("pkgname = foo")]
    #[case("pkgbase = foo")]
    #[case("pkgver = 1:1.0.0-1")]
    #[case("packager = Foobar McFooface <foobar@mcfooface.org>")]
    #[case("pkgarch = any")]
    fn pkginfov2_from_str_duplicate_fail(#[case] duplicate: &str) {
        let mut pkginfov2 = VALID_PKGINFOV2_CASE1.to_string();
        pkginfov2.push_str(duplicate);
        assert!(PackageInfoV2::from_str(&pkginfov2).is_err());
    }
}
