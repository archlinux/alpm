use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use alpm_types::Architecture;
use alpm_types::Backup;
use alpm_types::BuildDate;
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

use crate::Error;

/// Generates a struct based on the PKGINFO version 1 specification with additional fields.
macro_rules! generate_pkginfo {
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
            pkgname: Name,

            #[serde_as(as = "DisplayFromStr")]
            pkgbase: Name,

            #[serde_as(as = "DisplayFromStr")]
            pkgver: Version,

            #[serde_as(as = "DisplayFromStr")]
            pkgdesc: PkgDesc,

            #[serde_as(as = "DisplayFromStr")]
            url: Url,

            #[serde_as(as = "DisplayFromStr")]
            builddate: BuildDate,

            #[serde_as(as = "DisplayFromStr")]
            packager: Packager,

            #[serde_as(as = "DisplayFromStr")]
            size: InstalledSize,

            #[serde_as(as = "DisplayFromStr")]
            arch: Architecture,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            license: Vec<License>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            replaces: Vec<PackageRelation>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            group: Vec<Group>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            conflict: Vec<PackageRelation>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            provides: Vec<PackageRelation>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            backup: Vec<Backup>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            depend: Vec<PackageRelation>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            optdepend: Vec<OptDepend>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            makedepend: Vec<PackageRelation>,

            #[serde_as(as = "Vec<DisplayFromStr>")]
            #[serde(default)]
            checkdepend: Vec<PackageRelation>,

            $($extra_fields)*
        }

        impl $name {
            /// Returns the name of the package
            pub fn pkgname(&self) -> &Name {
                &self.pkgname
            }

            /// Returns the base name of the package
            pub fn pkgbase(&self) -> &Name {
                &self.pkgbase
            }

            /// Returns the version of the package
            pub fn pkgver(&self) -> &Version {
                &self.pkgver
            }

            /// Returns the description of the package
            pub fn pkgdesc(&self) -> &PkgDesc {
                &self.pkgdesc
            }

            /// Returns the URL of the package
            pub fn url(&self) -> &Url {
                &self.url
            }

            /// Returns the build date of the package
            pub fn builddate(&self) -> &BuildDate {
                &self.builddate
            }

            /// Returns the packager of the package
            pub fn packager(&self) -> &Packager {
                &self.packager
            }

            /// Returns the size of the package
            pub fn size(&self) -> &InstalledSize {
                &self.size
            }

            /// Returns the architecture of the package
            pub fn arch(&self) -> &Architecture {
                &self.arch
            }

            /// Returns the licenses of the package
            pub fn license(&self) -> &[License] {
                &self.license
            }

            /// Returns the packages this package replaces
            pub fn replaces(&self) -> &[PackageRelation] {
                &self.replaces
            }

            /// Returns the group of the package
            pub fn group(&self) -> &[Group] {
                &self.group
            }

            /// Returns the packages this package conflicts with
            pub fn conflict(&self) -> &[PackageRelation] {
                &self.conflict
            }

            /// Returns the packages this package provides
            pub fn provides(&self) -> &[PackageRelation] {
                &self.provides
            }

            /// Returns the backup files of the package
            pub fn backup(&self) -> &[Backup] {
                &self.backup
            }

            /// Returns the packages this package depends on
            pub fn depend(&self) -> &[PackageRelation] {
                &self.depend
            }

            /// Returns the optional dependencies of the package
            pub fn optdepend(&self) -> &[OptDepend] {
                &self.optdepend
            }

            /// Returns the packages this package is built with
            pub fn makedepend(&self) -> &[PackageRelation] {
                &self.makedepend
            }

            /// Returns the packages this package is checked with
            pub fn checkdepend(&self) -> &[PackageRelation] {
                &self.checkdepend
            }
        }
    }
}

pub(crate) use generate_pkginfo;

generate_pkginfo! {
    /// PKGINFO version 1
    ///
    /// `PkgInfoV1` is (exclusively) compatible with data following the first specification of the
    /// PKGINFO file.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_pkginfo::PkgInfoV1;
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
    /// let pkginfo = PkgInfoV1::from_str(pkginfo_data)?;
    /// assert_eq!(pkginfo.to_string(), pkginfo_data);
    /// # Ok(())
    /// # }
    /// ```
    PkgInfoV1 {}
}

impl PkgInfoV1 {
    /// Create a new PkgInfoV1 from all required components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Name,
        base: Name,
        version: Version,
        desc: PkgDesc,
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
    ) -> Self {
        Self {
            pkgname: name,
            pkgbase: base,
            pkgver: version,
            pkgdesc: desc,
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
        }
    }
}

impl FromStr for PkgInfoV1 {
    type Err = Error;
    /// Create a PkgInfoV1 from a &str
    ///
    /// ## Errors
    ///
    /// Returns an `Error` if any of the fields in `input` can not be validated according to
    /// `PkgInfoV1` or their respective own specification.
    fn from_str(input: &str) -> Result<PkgInfoV1, Self::Err> {
        let pkginfo: PkgInfoV1 = alpm_parsers::custom_ini::from_str(input)?;
        Ok(pkginfo)
    }
}

impl Display for PkgInfoV1 {
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
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::fixture;
    use rstest::rstest;
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
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
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
        PkgInfoV1::from_str(&valid_pkginfov1)?;
        Ok(())
    }

    #[rstest]
    fn pkginfov1() -> TestResult {
        let pkg_info = PkgInfoV1::new(
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
        );
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
        assert!(PkgInfoV1::from_str(&valid_pkginfov1).is_err());
    }
}
