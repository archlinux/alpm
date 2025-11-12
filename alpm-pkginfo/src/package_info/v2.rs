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

use crate::{Error, RelationOrSoname, package_info::v1::generate_pkginfo};

generate_pkginfo! {
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
    PackageInfoV2 {
        #[serde_as(as = "TryFromInto<Vec<ExtraDataEntry>>")]
        xdata: ExtraData,
    }
}

impl PackageInfoV2 {
    /// Create a new PackageInfoV2 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pkgname: Name,
        pkgbase: Name,
        pkgver: FullVersion,
        pkgdesc: PackageDescription,
        url: Url,
        builddate: BuildDate,
        packager: Packager,
        size: InstalledSize,
        arch: Architecture,
        license: Vec<License>,
        replaces: Vec<PackageRelation>,
        group: Vec<Group>,
        conflict: Vec<PackageRelation>,
        provides: Vec<RelationOrSoname>,
        backup: Vec<Backup>,
        depend: Vec<RelationOrSoname>,
        optdepend: Vec<OptionalDependency>,
        makedepend: Vec<PackageRelation>,
        checkdepend: Vec<PackageRelation>,
        xdata: ExtraData,
    ) -> Self {
        Self {
            pkgname,
            pkgbase,
            pkgver,
            pkgdesc,
            url,
            builddate,
            packager,
            size,
            arch,
            license,
            replaces,
            group,
            conflict,
            provides,
            backup,
            depend,
            optdepend,
            makedepend,
            checkdepend,
            xdata,
        }
    }

    /// Get the extra data
    pub fn xdata(&self) -> &ExtraData {
        &self.xdata
    }
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
            self.pkgname(),
            self.pkgbase(),
            self.pkgver(),
            self.pkgdesc(),
            self.url(),
            self.builddate(),
            self.packager(),
            self.size(),
            self.arch(),
            format_list("license", self.license()),
            format_list("replaces", self.replaces()),
            format_list("group", self.group()),
            format_list("conflict", self.conflict()),
            format_list("provides", self.provides()),
            format_list("backup", self.backup()),
            format_list("depend", self.depend()),
            format_list("optdepend", self.optdepend()),
            format_list("makedepend", self.makedepend()),
            format_list("checkdepend", self.checkdepend()).trim_end_matches('\n'),
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
        let pkg_info = PackageInfoV2::new(
            Name::new("example")?,
            Name::new("example")?,
            FullVersion::from_str("1:1.0.0-1")?,
            PackageDescription::from("A project that does something"),
            Url::from_str("https://example.org")?,
            BuildDate::from_str("1729181726")?,
            Packager::from_str("John Doe <john@example.org>")?,
            InstalledSize::from_str("181849963")?,
            Architecture::Any,
            vec![
                License::from_str("GPL-3.0-or-later")?,
                License::from_str("LGPL-3.0-or-later")?,
            ],
            vec![PackageRelation::from_str("other-package>0.9.0-3")?],
            vec![
                Group::from_str("package-group")?,
                Group::from_str("other-package-group")?,
            ],
            vec![
                PackageRelation::from_str("conflicting-package<1.0.0")?,
                PackageRelation::from_str("other-conflicting-package<1.0.0")?,
            ],
            vec![
                RelationOrSoname::from_str("some-component")?,
                RelationOrSoname::from_str("some-other-component=1:1.0.0-1")?,
                RelationOrSoname::from_str("libexample.so=1-64")?,
                RelationOrSoname::from_str("libunversionedexample.so=libunversionedexample.so-64")?,
                RelationOrSoname::from_str("lib:libexample.so.1")?,
            ],
            vec![
                Backup::from_str("etc/example/config.toml")?,
                Backup::from_str("etc/example/other-config.txt")?,
            ],
            vec![
                RelationOrSoname::from_str("glibc")?,
                RelationOrSoname::from_str("gcc-libs")?,
                RelationOrSoname::from_str("libother.so=0-64")?,
                RelationOrSoname::from_str("libunversioned.so=libunversioned.so-64")?,
                RelationOrSoname::from_str("lib:libother.so.0")?,
            ],
            vec![
                OptionalDependency::from_str("python: for special-python-script.py")?,
                OptionalDependency::from_str("ruby: for special-ruby-script.rb")?,
            ],
            vec![
                PackageRelation::from_str("cmake")?,
                PackageRelation::from_str("python-sphinx")?,
            ],
            vec![
                PackageRelation::from_str("extra-test-tool")?,
                PackageRelation::from_str("other-extra-test-tool")?,
            ],
            ExtraDataEntry::from_str("pkgtype=pkg")?.try_into()?,
        );
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
