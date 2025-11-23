use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::{FileFormatSchema, MetadataFile, RuntimeRelationLookupData};
#[cfg(doc)]
use alpm_types::RelationLookup;
use alpm_types::{PackageRelation, VersionRequirement};
use fluent_i18n::t;

use crate::{
    Error,
    desc::{DbDescFileV1, DbDescFileV2, DbDescSchema},
};

/// A representation of the [alpm-db-desc] file format.
///
/// Tracks all supported schema versions (`v1` and `v2`) of the database description file.
/// Each variant corresponds to a distinct layout of the format.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum DbDescFile {
    /// The [alpm-db-descv1] file format.
    ///
    /// [alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
    V1(DbDescFileV1),
    /// The [alpm-db-descv2] file format.
    ///
    /// This revision of the file format, adds the `%XDATA%` section.
    ///
    /// [alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
    V2(DbDescFileV2),
}

impl MetadataFile<DbDescSchema> for DbDescFile {
    type Err = Error;

    /// Creates a [`DbDescFile`] from a file on disk, optionally validated using a [`DbDescSchema`].
    ///
    /// Opens the file and defers to [`DbDescFile::from_reader_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with DB desc data (v1)
    /// let (file, desc_data) = {
    ///     let desc_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (file, desc_data)
    /// };
    ///
    /// let db_desc = DbDescFile::from_file_with_schema(
    ///     file.path(),
    ///     Some(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// assert_eq!(db_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file cannot be opened for reading,
    /// - the contents cannot be parsed into any known [`DbDescFile`] variant,
    /// - or the provided [`DbDescSchema`] does not match the contents of the file.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<DbDescSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| Error::IoPathError {
                path: PathBuf::from(file),
                context: t!("error-io-path-open-file"),
                source,
            })?,
            schema,
        )
    }

    /// Creates a [`DbDescFile`] from any readable stream, optionally validated using a
    /// [`DbDescSchema`].
    ///
    /// Reads the `reader` to a string buffer and defers to [`DbDescFile::from_str_with_schema`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a reader with DB desc data (v2)
    /// let (reader, desc_data) = {
    ///     let desc_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///     let file = tempfile::NamedTempFile::new()?;
    ///     let mut output = File::create(&file)?;
    ///     write!(output, "{}", desc_data)?;
    ///     (File::open(&file.path())?, desc_data)
    /// };
    ///
    /// let db_desc = DbDescFile::from_reader_with_schema(
    ///     reader,
    ///     Some(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// assert_eq!(db_desc.to_string(), desc_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the `reader` cannot be read to string,
    /// - the data cannot be parsed into a known [`DbDescFile`] variant,
    /// - or the provided [`DbDescSchema`] does not match the parsed content.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<DbDescSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::IoReadError {
                context: t!("error-io-read-db-desc"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`DbDescFile`] from a string slice, optionally validated using a [`DbDescSchema`].
    ///
    /// If `schema` is [`None`], automatically infers the schema version by inspecting the input
    /// (`v1` = no `%XDATA%` section, `v2` = has `%XDATA%`).
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_common::MetadataFile;
    /// use alpm_db::desc::{DbDescFile, DbDescSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let v1_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// "#;
    ///
    /// let dbdesc_v1 = DbDescFile::from_str_with_schema(
    ///     v1_data,
    ///     Some(DbDescSchema::V1(SchemaVersion::new(Version::new(1, 0, 0)))),
    /// )?;
    /// assert_eq!(dbdesc_v1.to_string(), v1_data);
    ///
    /// let v2_data = r#"%NAME%
    /// foo
    ///
    /// %VERSION%
    /// 1.0.0-1
    ///
    /// %BASE%
    /// foo
    ///
    /// %DESC%
    /// An example package
    ///
    /// %URL%
    /// https://example.org/
    ///
    /// %ARCH%
    /// x86_64
    ///
    /// %BUILDDATE%
    /// 1733737242
    ///
    /// %INSTALLDATE%
    /// 1733737243
    ///
    /// %PACKAGER%
    /// Foobar McFooface <foobar@mcfooface.org>
    ///
    /// %SIZE%
    /// 123
    ///
    /// %VALIDATION%
    /// pgp
    ///
    /// %XDATA%
    /// pkgtype=pkg
    ///
    /// "#;
    ///
    /// let dbdesc_v2 = DbDescFile::from_str_with_schema(
    ///     v2_data,
    ///     Some(DbDescSchema::V2(SchemaVersion::new(Version::new(2, 0, 0)))),
    /// )?;
    /// assert_eq!(dbdesc_v2.to_string(), v2_data);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the input cannot be parsed into a valid [`DbDescFile`],
    /// - or the derived or provided schema does not match the detected format.
    fn from_str_with_schema(s: &str, schema: Option<DbDescSchema>) -> Result<Self, Error> {
        let schema = match schema {
            Some(schema) => schema,
            None => DbDescSchema::derive_from_str(s)?,
        };

        match schema {
            DbDescSchema::V1(_) => Ok(DbDescFile::V1(DbDescFileV1::from_str(s)?)),
            DbDescSchema::V2(_) => Ok(DbDescFile::V2(DbDescFileV2::from_str(s)?)),
        }
    }
}

impl Display for DbDescFile {
    /// Returns the textual representation of the [`DbDescFile`] in its corresponding
    /// [alpm-db-desc] format.
    ///
    /// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1(file) => write!(f, "{file}"),
            Self::V2(file) => write!(f, "{file}"),
        }
    }
}

impl FromStr for DbDescFile {
    type Err = Error;

    /// Creates a [`DbDescFile`] from a string slice.
    ///
    /// Internally calls [`DbDescFile::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`DbDescFile::from_str_with_schema`] fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}

impl RuntimeRelationLookupData for DbDescFile {
    /// Adds each [run-time dependency] to a [`RelationLookup`].
    ///
    /// [run-time dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
    fn add_run_time_dependencies_to_lookup(&self, lookup: &mut alpm_types::RelationLookup) {
        let (name, relations) = match self {
            Self::V1(db_desc) => (&db_desc.name, &db_desc.depends),
            Self::V2(db_desc) => (&db_desc.name, &db_desc.depends),
        };

        for relation in relations.iter() {
            lookup.insert_relation_or_soname(relation, Some(name.clone()));
        }
    }

    /// Adds each [optional dependency] to a [`RelationLookup`].
    ///
    /// [optional dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#optional-dependency
    fn add_optional_dependencies_to_lookup(&self, lookup: &mut alpm_types::RelationLookup) {
        let (name, optionals) = match self {
            Self::V1(db_desc) => (&db_desc.name, &db_desc.optdepends),
            Self::V2(db_desc) => (&db_desc.name, &db_desc.optdepends),
        };

        for optional in optionals.iter() {
            lookup.insert_package_relation(optional.package_relation(), Some(name.clone()))
        }
    }

    /// Adds each [provision] to a [`RelationLookup`].
    ///
    /// Adds the name and version tracked by the [`DbDescFile`] as a strict [`PackageRelation`]
    /// (e.g. "example=1.0.0-1") in addition to any [provision], because a package always provides
    /// itself.
    ///
    /// [provision]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#provision
    fn add_provisions_to_lookup(&self, lookup: &mut alpm_types::RelationLookup) {
        let (name, version, relations) = match self {
            Self::V1(db_desc) => (&db_desc.name, &db_desc.version, &db_desc.provides),
            Self::V2(db_desc) => (&db_desc.name, &db_desc.version, &db_desc.provides),
        };

        // Add the package name and version itself to the list of provisions, as a package always
        // provides itself in the specific version.
        lookup.insert_package_relation(
            &PackageRelation {
                name: name.clone(),
                version_requirement: Some(VersionRequirement {
                    comparison: alpm_types::VersionComparison::Equal,
                    version: version.into(),
                }),
            },
            Some(name.clone()),
        );

        for relation in relations.iter() {
            lookup.insert_relation_or_soname(relation, Some(name.clone()));
        }
    }

    /// Adds each [conflict] to a [`RelationLookup`].
    ///
    /// [conflict]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#conflict
    fn add_conflicts_to_lookup(&self, lookup: &mut alpm_types::RelationLookup) {
        let (name, relations) = match self {
            Self::V1(db_desc) => (&db_desc.name, &db_desc.conflicts),
            Self::V2(db_desc) => (&db_desc.name, &db_desc.conflicts),
        };

        for package_relation in relations.iter() {
            lookup.insert_package_relation(package_relation, Some(name.clone()))
        }
    }

    /// Adds each [replacement] to a [`RelationLookup`].
    ///
    /// [replacement]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#replacement
    fn add_replacements_to_lookup(&self, lookup: &mut alpm_types::RelationLookup) {
        let (name, relations) = match self {
            Self::V1(db_desc) => (&db_desc.name, &db_desc.replaces),
            Self::V2(db_desc) => (&db_desc.name, &db_desc.replaces),
        };

        for package_relation in relations.iter() {
            lookup.insert_package_relation(package_relation, Some(name.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use alpm_types::{Name, RelationLookup, SonameV1, SonameV2, Version};
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    /// An alpm-db-descv1 string with all sections explicitly populated.
    const DESC_V1: &str = r#"%NAME%
example

%VERSION%
1:1.0.0-1

%BASE%
example

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%GROUPS%
utils
cli

%REASON%
1

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
pgp

%REPLACES%
other-package>0.9.0-3

%DEPENDS%
glibc
lib-other-example-0.19.so=lib-other-example-0.19.so-64
lib:lib-other-example.so.1

%OPTDEPENDS%
python: for special-python-script.py

%CONFLICTS%
conflicting-package<1.0.0

%PROVIDES%
example-virtual
libexample-test-0.1.so=libexample-test-0.1.so-64
lib:libexample.so.1

"#;

    /// An alpm-db-descv2 string with all sections explicitly populated.
    const DESC_V2: &str = r#"%NAME%
example

%VERSION%
1:1.0.0-1

%BASE%
example

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%GROUPS%
utils
cli

%REASON%
1

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
pgp

%REPLACES%
other-package>0.9.0-3

%DEPENDS%
glibc
lib-other-example-0.19.so=lib-other-example-0.19.so-64
lib:lib-other-example.so.1

%OPTDEPENDS%
python: for special-python-script.py

%CONFLICTS%
conflicting-package<1.0.0

%PROVIDES%
example-virtual
libexample-test-0.1.so=libexample-test-0.1.so-64
lib:libexample.so.1

%XDATA%
pkgtype=pkg

"#;

    #[rstest]
    #[case::v1(DESC_V1)]
    #[case::v1(DESC_V2)]
    fn package_info_add_run_time_dependencies_to_lookup(#[case] input: &str) -> TestResult {
        let package_info = DbDescFile::from_str(input)?;
        let mut lookup = RelationLookup::default();

        package_info.add_run_time_dependencies_to_lookup(&mut lookup);

        eprintln!("{lookup:?}");

        assert_eq!(lookup.len(), 3);
        assert!(
            lookup.satisfies_name_and_version(&Name::from_str("glibc")?, &Version::from_str("1")?,)
        );
        assert!(lookup.satisfies_sonamev1(&SonameV1::from_str(
            "lib-other-example-0.19.so=lib-other-example-0.19.so-64"
        )?));
        assert!(lookup.satisfies_sonamev2(&SonameV2::from_str("lib:lib-other-example.so.1")?));

        Ok(())
    }

    #[rstest]
    #[case::v1(DESC_V1)]
    #[case::v1(DESC_V2)]
    fn package_info_add_optional_dependencies_to_lookup(#[case] input: &str) -> TestResult {
        let package_info = DbDescFile::from_str(input)?;
        let mut lookup = RelationLookup::default();

        package_info.add_optional_dependencies_to_lookup(&mut lookup);

        assert_eq!(lookup.len(), 1);
        assert!(
            lookup
                .satisfies_name_and_version(&Name::from_str("python")?, &Version::from_str("1")?,)
        );

        Ok(())
    }

    #[rstest]
    #[case::v1(DESC_V1)]
    #[case::v1(DESC_V2)]
    fn package_info_add_provisions_to_lookup(#[case] input: &str) -> TestResult {
        let package_info = DbDescFile::from_str(input)?;
        let mut lookup = RelationLookup::default();

        package_info.add_provisions_to_lookup(&mut lookup);

        assert_eq!(lookup.len(), 4);
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("example")?,
            &Version::from_str("1:1.0.0-1")?,
        ));
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("example-virtual")?,
            &Version::from_str("1:1.0.0-1")?,
        ));
        assert!(lookup.satisfies_sonamev1(&SonameV1::from_str(
            "libexample-test-0.1.so=libexample-test-0.1.so-64"
        )?));
        assert!(lookup.satisfies_sonamev2(&SonameV2::from_str("lib:libexample.so.1")?));

        Ok(())
    }

    #[rstest]
    #[case::v1(DESC_V1)]
    #[case::v1(DESC_V2)]
    fn package_info_add_conflicts_to_lookup(#[case] input: &str) -> TestResult {
        let package_info = DbDescFile::from_str(input)?;
        let mut lookup = RelationLookup::default();

        package_info.add_conflicts_to_lookup(&mut lookup);

        assert_eq!(lookup.len(), 1);
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("conflicting-package")?,
            &Version::from_str("0.9.0")?,
        ));

        Ok(())
    }

    #[rstest]
    #[case::v1(DESC_V1)]
    #[case::v1(DESC_V2)]
    fn package_info_add_replacements_to_lookup(#[case] input: &str) -> TestResult {
        let package_info = DbDescFile::from_str(input)?;
        let mut lookup = RelationLookup::default();

        package_info.add_replacements_to_lookup(&mut lookup);

        assert_eq!(lookup.len(), 1);
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("other-package")?,
            &Version::from_str("1.0.0")?,
        ));

        Ok(())
    }
}
