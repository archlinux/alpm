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
use alpm_types::PackageRelation;
use alpm_types::Packager;
use alpm_types::PkgDesc;
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
        pkgdesc: PkgDesc,
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
    pub fn xdata(&self) -> &Vec<ExtraData> {
        &self.xdata
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
        let pkginfo: PkgInfoV2 = alpm_parsers::custom_ini::from_str(input)?;
        Ok(pkginfo)
    }
}

impl Display for PkgInfoV2 {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fn format_list(label: &str, items: &[impl Display]) -> String {
            items
                .iter()
                .map(|v| format!("{} = {}", label, v))
                .collect::<Vec<_>>()
                .join("\n")
        }
        write!(
            fmt,
            "pkgname = {}\n\
            pkgbase = {}\n\
            {}\n\
            pkgver = {}\n\
            pkgdesc = {}\n\
            url = {}\n\
            builddate = {}\n\
            packager = {}\n\
            size = {}\n\
            arch = {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}\n\
            {}",
            self.pkgname(),
            self.pkgbase(),
            format_list("xdata", self.xdata()),
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
            format_list("checkdepend", self.checkdepend()),
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

    #[rstest]
    fn pkginfov2() -> TestResult {
        let pkg_info = PkgInfoV2::new(
            Name::new("example".to_string())?,
            Name::new("example".to_string())?,
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
        );
        assert_eq!(pkg_info.to_string(), VALID_PKGINFOV2_CASE1);
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
