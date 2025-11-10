//! The [PKGINFOv1] file format.
//!
//! [PKGINFOv1]: https://alpm.archlinux.page/specifications/PKGINFOv1.5.html

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{
    Architecture,
    Backup,
    BuildDate,
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
use serde_with::{DisplayFromStr, serde_as};

use crate::{Error, RelationOrSoname};

/// PKGINFO version 1
///
/// `PackageInfoV1` is (exclusively) compatible with data following the first specification of the
/// PKGINFO file.
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
///
/// use alpm_pkginfo::PackageInfoV1;
///
/// # fn main() -> Result<(), alpm_pkginfo::Error> {
/// let pkginfo_data = r#"pkgname = example
/// pkgbase = example
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
/// let pkginfo = PackageInfoV1::from_str(pkginfo_data)?;
/// assert_eq!(pkginfo.to_string(), pkginfo_data);
/// # Ok(())
/// # }
/// ```
#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageInfoV1 {
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
}

impl FromStr for PackageInfoV1 {
    type Err = Error;
    /// Create a PackageInfoV1 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `PackageInfoV1` or their respective own specification.
    fn from_str(input: &str) -> Result<PackageInfoV1, Self::Err> {
        let pkginfo: PackageInfoV1 = alpm_parsers::custom_ini::from_str(input)?;
        Ok(pkginfo)
    }
}

impl Display for PackageInfoV1 {
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
        write!(
            fmt,
            "pkgname = {}\n\
            pkgbase = {}\n\
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
            {}",
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
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use testresult::TestResult;

    use super::*;

    #[fixture]
    fn valid_pkginfov1() -> String {
        r#"pkgname = example
pkgbase = example
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
checkdepend = other-extra-test-tool"#
            .to_string()
    }

    #[rstest]
    fn pkginfov1_from_str(valid_pkginfov1: String) -> TestResult {
        PackageInfoV1::from_str(&valid_pkginfov1)?;
        Ok(())
    }

    #[rstest]
    fn pkginfov1() -> TestResult {
        let pkg_info = PackageInfoV1 {
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
        };
        assert_eq!(pkg_info.to_string(), valid_pkginfov1());
        Ok(())
    }

    #[rstest]
    #[case("pkgname = foo")]
    #[case("pkgbase = foo")]
    #[case("pkgver = 1:1.0.0-1")]
    #[case("packager = Foobar McFooface <foobar@mcfooface.org>")]
    #[case("pkgarch = any")]
    fn pkginfov1_from_str_duplicate_fail(mut valid_pkginfov1: String, #[case] duplicate: &str) {
        valid_pkginfov1.push_str(duplicate);
        assert!(PackageInfoV1::from_str(&valid_pkginfov1).is_err());
    }
}
