use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use alpm_types::Architecture;
use alpm_types::Backup;
use alpm_types::BuildDate;
use alpm_types::ExtraData;
use alpm_types::Group;
use alpm_types::InstalledSize;
use alpm_types::License;
use alpm_types::Name;
use alpm_types::OptDepend;
use alpm_types::PackageDescription;
use alpm_types::PackageRelation;
use alpm_types::Packager;
use alpm_types::PkgType;
use alpm_types::Url;
use alpm_types::Version;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::pkginfo_v1::generate_pkginfo;
use crate::Error;

generate_pkginfo! {
    /// PKGINFO version 2
    ///
    /// `PkgInfoV2` is (exclusively) compatible with data following the v2 specification of the
    /// PKGINFO file.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_pkginfo::PkgInfoV2;
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
    /// backup = etc/example/config.toml
    /// backup = etc/example/other-config.txt
    /// depend = glibc
    /// depend = gcc-libs
    /// optdepend = python: for special-python-script.py
    /// optdepend = ruby: for special-ruby-script.rb
    /// makedepend = cmake
    /// makedepend = python-sphinx
    /// checkdepend = extra-test-tool
    /// checkdepend = other-extra-test-tool"#;
    /// let pkginfo = PkgInfoV2::from_str(pkginfo_data)?;
    /// assert_eq!(pkginfo.to_string(), pkginfo_data);
    /// # Ok(())
    /// # }
    /// ```
    PkgInfoV2 {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        xdata: Vec<ExtraData>,
    }
}

impl PkgInfoV2 {
    /// Create a new PkgInfoV2 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pkgname: Name,
        pkgbase: Name,
        pkgver: Version,
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
        provides: Vec<PackageRelation>,
        backup: Vec<Backup>,
        depend: Vec<PackageRelation>,
        optdepend: Vec<OptDepend>,
        makedepend: Vec<PackageRelation>,
        checkdepend: Vec<PackageRelation>,
        xdata: Vec<ExtraData>,
    ) -> Result<Self, Error> {
        let pkg_info = Self {
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
        };
        pkg_info.check_pkg_type()?;
        Ok(pkg_info)
    }

    /// Get the extra data
    pub fn xdata(&self) -> &Vec<ExtraData> {
        &self.xdata
    }

    /// Returns the package type.
    ///
    /// # Panics
    ///
    /// This function panics if the `xdata` field does not contain a `pkgtype` key.
    pub fn pkg_type(&self) -> PkgType {
        self.xdata
            .iter()
            .find(|v| v.key() == "pkgtype")
            .map(|v| PkgType::from_str(v.value()).expect("Invalid package type"))
            .unwrap_or_else(|| panic!("Missing extra data"))
    }

    /// Checks if the package type exists.
    ///
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// - if the `xdata` field does not contain a `pkgtype` key.
    /// - if the `pkgtype` key does not contain a valid package type.
    fn check_pkg_type(&self) -> Result<(), Error> {
        if let Some(pkg_type) = self.xdata.iter().find(|v| v.key() == "pkgtype") {
            let _ = PkgType::from_str(pkg_type.value())?;
            Ok(())
        } else {
            Err(Error::MissingExtraData)
        }
    }
}

impl FromStr for PkgInfoV2 {
    type Err = Error;
    /// Create a PkgInfoV2 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `PkgInfoV2` or their respective own specification.
    fn from_str(input: &str) -> Result<PkgInfoV2, Self::Err> {
        let pkg_info: PkgInfoV2 = alpm_parsers::custom_ini::from_str(input)?;
        pkg_info.check_pkg_type()?;
        Ok(pkg_info)
    }
}

impl Display for PkgInfoV2 {
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
        let pkg_type = self
            .xdata
            .iter()
            .find(|v| v.key() == "pkgtype")
            .ok_or(std::fmt::Error)?;
        let other_xdata = self
            .xdata
            .iter()
            .filter(|v| v.key() != "pkgtype")
            .collect::<Vec<_>>();
        write!(
            fmt,
            "pkgname = {}\n\
            pkgbase = {}\n\
            xdata = {pkg_type}\n\
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
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
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
        PkgInfoV2::from_str(pkginfo)?;
        Ok(())
    }

    fn pkg_info() -> TestResult<PkgInfoV2> {
        let pkg_info = PkgInfoV2::new(
            Name::new("example")?,
            Name::new("example")?,
            Version::from_str("1:1.0.0-1")?,
            "A project that does something".to_string(),
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
                PackageRelation::from_str("some-component")?,
                PackageRelation::from_str("some-other-component=1:1.0.0-1")?,
            ],
            vec![
                Backup::from_str("etc/example/config.toml")?,
                Backup::from_str("etc/example/other-config.txt")?,
            ],
            vec![
                PackageRelation::from_str("glibc")?,
                PackageRelation::from_str("gcc-libs")?,
            ],
            vec![
                OptDepend::from_str("python: for special-python-script.py")?,
                OptDepend::from_str("ruby: for special-ruby-script.rb")?,
            ],
            vec![
                PackageRelation::from_str("cmake")?,
                PackageRelation::from_str("python-sphinx")?,
            ],
            vec![
                PackageRelation::from_str("extra-test-tool")?,
                PackageRelation::from_str("other-extra-test-tool")?,
            ],
            vec![ExtraData::from_str("pkgtype=pkg")?],
        )?;
        assert_eq!(PkgType::Package, pkg_info.pkg_type());
        Ok(pkg_info)
    }

    #[rstest]
    fn pkginfov2() -> TestResult {
        let pkg_info = pkg_info()?;
        assert_eq!(pkg_info.to_string(), VALID_PKGINFOV2_CASE1);
        Ok(())
    }

    #[rstest]
    fn pkginfov2_invalid_xdata_fail() -> TestResult {
        let mut pkg_info = pkg_info()?;
        pkg_info.xdata = vec![];
        assert!(pkg_info.check_pkg_type().is_err());

        pkg_info.xdata = vec![ExtraData::from_str("pkgtype=foo")?];
        assert!(pkg_info.check_pkg_type().is_err());
        Ok(())
    }

    #[rstest]
    fn pkginfov2_multiple_xdata() -> TestResult {
        let mut pkg_info = pkg_info()?;
        pkg_info.xdata.push(ExtraData::from_str("foo=bar")?);
        pkg_info.xdata.push(ExtraData::from_str("baz=qux")?);
        assert_eq!(
            pkg_info.to_string(),
            format!("{VALID_PKGINFOV2_CASE1}\nxdata = foo=bar\nxdata = baz=qux")
        );
        Ok(())
    }

    #[rstest]
    fn pkginfov2_missing_xdata_fail() -> TestResult {
        let mut pkg_info_str = VALID_PKGINFOV2_CASE1.to_string();
        pkg_info_str = pkg_info_str.replace("xdata = pkgtype=pkg\n", "");
        assert!(PkgInfoV2::from_str(&pkg_info_str).is_err());

        let pkg_info = pkg_info()?;
        assert!(PkgInfoV2::new(
            pkg_info.pkgname,
            pkg_info.pkgbase,
            pkg_info.pkgver,
            pkg_info.pkgdesc,
            pkg_info.url,
            pkg_info.builddate,
            pkg_info.packager,
            pkg_info.size,
            pkg_info.arch,
            pkg_info.license,
            pkg_info.replaces,
            pkg_info.group,
            pkg_info.conflict,
            pkg_info.provides,
            pkg_info.backup,
            pkg_info.depend,
            pkg_info.optdepend,
            pkg_info.makedepend,
            pkg_info.checkdepend,
            vec![]
        )
        .is_err());
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
        assert!(PkgInfoV2::from_str(&pkginfov2).is_err());
    }
}
