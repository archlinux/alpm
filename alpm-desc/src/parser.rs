use std::collections::BTreeMap;

use alpm_parsers::custom_ini::{Deserializer, parser::Item};
use alpm_types::{
    Architecture, BuildDate, CompressedSize, ExtraData, Group, InstalledSize, License, Name,
    OptionalDependency, PackageBaseName, PackageDescription, PackageRelation, Packager,
    Sha256Checksum, Url, Version,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_with::{DisplayFromStr, serde_as};
use winnow::{
    ModalResult, Parser,
    ascii::{line_ending, space0, till_line_ending},
    combinator::{alt, delimited, eof, preceded, repeat_till, terminated},
    token::take_till,
};

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    serde::Serialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum PackageValidation {
    None,
    #[strum(to_string = "md5")]
    Md5,
    #[strum(to_string = "sha256")]
    Sha256,
    #[strum(to_string = "pgp")]
    Pgp,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
)]
#[repr(u8)]
pub enum PackageInstallReason {
    /// Explicitly requested by the user.
    #[strum(to_string = "0")]
    Explicit = 0,
    /// Installed as a dependency for another package.
    #[strum(to_string = "1")]
    Depend = 1,
    /// Failed parsing of local database.
    #[strum(to_string = "2")]
    Unknown = 2,
}

#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "UPPERCASE")]
pub struct DescFile {
    /// The filename of the package archive.
    ///
    /// TODO: Use PackageFileName (<https://gitlab.archlinux.org/archlinux/alpm/alpm/-/merge_requests/197/>)
    pub filename: String,

    /// The name of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub name: Name,

    /// The base name of the package (used in split packages).
    #[serde_as(as = "DisplayFromStr")]
    pub base: PackageBaseName,

    /// The version of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub version: Version,

    /// A short description of the package.
    #[serde_as(as = "DisplayFromStr")]
    pub desc: PackageDescription,

    /// The compressed size of the package archive.
    #[serde_as(as = "DisplayFromStr")]
    pub csize: CompressedSize,

    /// The size the package will occupy once installed.
    #[serde_as(as = "DisplayFromStr")]
    pub isize: InstalledSize,

    /// The SHA256 checksum of the package archive contents.
    #[serde_as(as = "DisplayFromStr")]
    pub sha256sum: Sha256Checksum,

    /// The upstream URL for the package.
    #[serde_as(as = "DisplayFromStr")]
    pub url: Url,

    /// Licenses that apply to the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub license: Vec<License>,

    /// The architecture for which the package was built.
    #[serde_as(as = "DisplayFromStr")]
    pub arch: Architecture,

    /// The timestamp when the package was built.
    #[serde_as(as = "DisplayFromStr")]
    pub builddate: BuildDate,

    /// Optional timestamp when the package was installed.
    #[serde_as(as = "DisplayFromStr")]
    pub installdate: BuildDate,

    /// The packager of the package, in the format `Name <email>`.
    #[serde_as(as = "DisplayFromStr")]
    pub packager: Packager,

    /// Optional install reason (0 = explicit, 1 = dependency).
    #[serde_as(as = "DisplayFromStr")]
    pub reason: PackageInstallReason,

    /// Validation methods used for the package archive.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub validation: Vec<PackageValidation>,

    /// Deprecated: size in bytes (use `isize` instead).
    #[serde_as(as = "DisplayFromStr")]
    pub size: InstalledSize,

    /// Groups the package belongs to.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub groups: Vec<Group>,

    /// Required runtime dependencies.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub depends: Vec<PackageRelation>,

    /// Optional dependencies that enhance the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub optdepends: Vec<OptionalDependency>,

    /// Dependencies needed to build the package.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub makedepends: Vec<PackageRelation>,

    /// Dependencies required to run the test suite.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub checkdepends: Vec<PackageRelation>,

    /// Packages this one replaces.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub replaces: Vec<Name>,

    /// Conflicting packages that cannot be installed together.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub conflicts: Vec<Name>,

    /// Virtual packages or capabilities provided by this one.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub provides: Vec<Name>,

    /// Structured extra metadata, implementation-defined.
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub xdata: Vec<ExtraData>,
}

#[derive(Debug)]
pub enum MetadataField<'a> {
    Section(&'a str),
    Value(&'a str),
    EmptyLine,
}

fn section<'a>(input: &mut &'a str) -> ModalResult<MetadataField<'a>> {
    let section = delimited("%", take_till(1.., |c| c == '%'), "%").map(MetadataField::Section);
    terminated(preceded(space0, section), line_ending).parse_next(input)
}

fn value<'a>(input: &mut &'a str) -> ModalResult<MetadataField<'a>> {
    let value = till_line_ending.map(MetadataField::Value);
    terminated(preceded(space0, value), line_ending).parse_next(input)
}

fn empty_line<'a>(input: &mut &'a str) -> ModalResult<MetadataField<'a>> {
    preceded(space0, line_ending)
        .map(|_| MetadataField::EmptyLine)
        .parse_next(input)
}

fn parser<'a>(input: &mut &'a str) -> ModalResult<BTreeMap<String, Item>> {
    let (fields, _eof): (Vec<MetadataField<'a>>, _) =
        repeat_till(0.., alt((empty_line, section, value)), eof).parse_next(input)?;

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current_section: Option<&'a str> = None;

    for field in fields {
        match field {
            MetadataField::Section(s) => current_section = Some(s),
            MetadataField::Value(v) => {
                if let Some(key) = current_section {
                    map.entry(key.to_string()).or_default().push(v.to_string());
                }
            }
            MetadataField::EmptyLine => {}
        }
    }

    let collapsed = map
        .into_iter()
        .map(|(key, mut values)| {
            if values.len() == 1 {
                (key, Item::Value(values.remove(0)))
            } else {
                (key, Item::List(values))
            }
        })
        .collect();

    Ok(collapsed)
}

pub fn from_str<T: DeserializeOwned>(s: &str) -> T {
    let input = parser.parse(s).unwrap();
    let mut de = Deserializer { input };
    Deserialize::deserialize(&mut de).unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;

    const TEST_DESC_DATA: &str = r#"
%FILENAME%
example-1.0.0-1-x86_64.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%CSIZE%
475255

%ISIZE%
1165163

%SHA256SUM%
b3948da79bee3aa25e1a58ee5946355b6ba822679e51a48253620dbfac510e9d

%URL%
https://gitlab.archlinux.org/archlinux/alpm

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733738255

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REASON%
0

%VALIDATION%
sha256
pgp

%SIZE%
1165163

%GROUPS%
base-devel
utils

%DEPENDS%
gcc-libs
zlib

%OPTDEPENDS%
man-db: for reading man pages
vim: for editing configuration

%MAKEDEPENDS%
cargo
pkgconf

%CHECKDEPENDS%
bats

%REPLACES%
old-example

%CONFLICTS%
another-example

%PROVIDES%
example

%XDATA%
pkgtype=pkg
"#;

    #[test]
    fn parse_desc_format() {
        let desc_file = from_str::<DescFile>(TEST_DESC_DATA);
        println!("{desc_file:#?}");
    }
}
