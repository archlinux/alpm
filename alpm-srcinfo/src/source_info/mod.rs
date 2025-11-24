//! Data representations and integrations for reading of SRCINFO data.
pub mod parser;
pub mod v1;

use std::{fs::File, path::Path, str::FromStr};

use alpm_common::{BuildRelationLookupData, MetadataFile};
use alpm_types::{Architecture, RelationLookup, SchemaVersion, semver_version::Version};
use fluent_i18n::t;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::{Error, SourceInfoSchema, SourceInfoV1};

/// The representation of SRCINFO data.
///
/// Tracks all available versions of the file format.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourceInfo {
    /// The [SRCINFO] file format.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    V1(SourceInfoV1),
}

impl MetadataFile<SourceInfoSchema> for SourceInfo {
    type Err = Error;

    /// Creates a [`SourceInfo`] from `file`, optionally validated using a [`SourceInfoSchema`].
    ///
    /// Opens the `file` and defers to [`SourceInfo::from_reader_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_file`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::{FileFormatSchema, MetadataFile};
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// // Prepare a file with SRCINFO data
    /// let srcinfo_file = tempfile::NamedTempFile::new()?;
    /// let (file, srcinfo_data) = {
    ///     let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///     let mut output = File::create(&srcinfo_file)?;
    ///     write!(output, "{}", srcinfo_data)?;
    ///     (srcinfo_file, srcinfo_data)
    /// };
    ///
    /// let srcinfo = SourceInfo::from_file_with_schema(
    ///     file.path().to_path_buf(),
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `file` cannot be opened for reading,
    /// - no variant of [`SourceInfo`] can be constructed from the contents of `file`,
    /// - or `schema` is [`Some`] and the [`SourceInfoSchema`] does not match the contents of
    ///   `file`.
    fn from_file_with_schema(
        file: impl AsRef<Path>,
        schema: Option<SourceInfoSchema>,
    ) -> Result<Self, Error> {
        let file = file.as_ref();
        Self::from_reader_with_schema(
            File::open(file).map_err(|source| Error::IoPath {
                path: file.to_path_buf(),
                context: t!("error-io-path-opening-file"),
                source,
            })?,
            schema,
        )
    }

    /// Creates a [`SourceInfo`] from a `reader`, optionally validated using a
    /// [`SourceInfoSchema`].
    ///
    /// Reads the `reader` to string and defers to [`SourceInfo::from_str_with_schema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_reader`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let srcinfo_file = tempfile::NamedTempFile::new()?;
    /// // Prepare a reader with SRCINFO data
    /// let (reader, srcinfo_data) = {
    ///     let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///     let mut output = File::create(&srcinfo_file)?;
    ///     write!(output, "{}", srcinfo_data)?;
    ///     (File::open(&srcinfo_file.path())?, srcinfo_data)
    /// };
    ///
    /// let srcinfo = SourceInfo::from_reader_with_schema(
    ///     reader,
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - the `reader` cannot be read to string,
    /// - no variant of [`SourceInfo`] can be constructed from the contents of the `reader`,
    /// - or `schema` is [`Some`] and the [`SourceInfoSchema`] does not match the contents of the
    ///   `reader`.
    fn from_reader_with_schema(
        mut reader: impl std::io::Read,
        schema: Option<SourceInfoSchema>,
    ) -> Result<Self, Error> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(|source| Error::Io {
                context: t!("error-io-read-srcinfo-data"),
                source,
            })?;
        Self::from_str_with_schema(&buf, schema)
    }

    /// Creates a [`SourceInfo`] from string slice, optionally validated using a
    /// [`SourceInfoSchema`].
    ///
    /// If `schema` is [`None`] attempts to detect the [`SourceInfoSchema`] from `s`.
    /// Attempts to create a [`SourceInfo`] variant that corresponds to the [`SourceInfoSchema`].
    ///
    /// # Note
    ///
    /// To automatically derive the [`SourceInfoSchema`], use [`SourceInfo::from_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Write};
    ///
    /// use alpm_common::MetadataFile;
    /// use alpm_srcinfo::{SourceInfo, SourceInfoSchema};
    /// use alpm_types::{SchemaVersion, semver_version::Version};
    ///
    /// # fn main() -> testresult::TestResult {
    /// let srcinfo_data = r#"
    /// pkgbase = example
    ///     pkgdesc = An example
    ///     arch = x86_64
    ///     pkgver = 0.1.0
    ///     pkgrel = 1
    ///
    /// pkgname = example
    /// "#;
    ///
    /// let srcinfo = SourceInfo::from_str_with_schema(
    ///     srcinfo_data,
    ///     Some(SourceInfoSchema::V1(SchemaVersion::new(Version::new(
    ///         1, 0, 0,
    ///     )))),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `schema` is [`Some`] and the specified variant of [`SourceInfo`] cannot be constructed
    ///   from `s`,
    /// - `schema` is [`None`] and
    ///   - a [`SourceInfoSchema`] cannot be derived from `s`,
    ///   - or the detected variant of [`SourceInfo`] cannot be constructed from `s`.
    fn from_str_with_schema(s: &str, schema: Option<SourceInfoSchema>) -> Result<Self, Error> {
        // NOTE: This does not use `SourceInfoSchema::derive_from_str`,
        // to not run the parser twice.
        // In the future, this should run `SourceInfoContent` parser directly
        // and delegate to `from_raw` instead of `from_string`.

        let schema = match schema {
            Some(schema) => schema,
            None => SourceInfoSchema::V1(SchemaVersion::new(Version::new(1, 0, 0))),
        };

        match schema {
            SourceInfoSchema::V1(_) => Ok(SourceInfo::V1(SourceInfoV1::from_string(s)?)),
        }
    }
}

impl FromStr for SourceInfo {
    type Err = Error;

    /// Creates a [`SourceInfo`] from string slice `s`.
    ///
    /// Calls [`SourceInfo::from_str_with_schema`] with `schema` set to [`None`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - a [`SourceInfoSchema`] cannot be derived from `s`,
    /// - or the detected variant of [`SourceInfo`] cannot be constructed from `s`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_schema(s, None)
    }
}

impl BuildRelationLookupData for SourceInfo {
    /// Adds each [build dependency] to a [`RelationLookup`].
    ///
    /// Considers each [build dependency] as well as each [run-time dependency] and adds the package
    /// base as origin.
    ///
    /// If `architecture` is provided, also considers each architecture-specific [build dependency]
    /// and each [run-time dependency].
    ///
    /// # Note
    ///
    /// If `architecture` does not match any architecture tracked by the [`SourceInfo`], no [build
    /// dependency] is added.
    ///
    /// [build dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#build-dependency
    /// [run-time dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
    fn add_build_dependencies_to_lookup(
        &self,
        lookup: &mut RelationLookup,
        architecture: Option<&Architecture>,
    ) {
        let (origin, architectures, run_time_dependencies, build_dependencies) = match self {
            Self::V1(source_info) => (
                &source_info.base.name,
                &source_info.base.architectures,
                source_info.base.dependencies.as_slice(),
                source_info.base.make_dependencies.as_slice(),
            ),
        };

        // Add any architecture-specific build and run-time dependencies, if they are requested.
        // NOTE: Here we are returning early, if the provided architecture is not
        // `Architecture::Any` or does not match any of the pkgbase's architectures.
        if let Some(architecture) = architecture {
            let pkgbase_architecture = match architecture {
                Architecture::Any => None,
                Architecture::Some(system_arch) => {
                    if !architectures.into_iter().any(|arch| &arch == architecture) {
                        warn!(
                            "The target architecture {architecture} for collecting build dependencies does not match any architecture package base {origin}. Skipping..."
                        );
                        return;
                    }

                    match self {
                        Self::V1(source_info) => {
                            source_info.base.architecture_properties.get(system_arch)
                        }
                    }
                }
            };

            if let Some(pkgbase_architecture) = pkgbase_architecture {
                for relation in pkgbase_architecture.dependencies.iter() {
                    lookup.insert_relation_or_soname(relation, Some(origin.clone()));
                }
                for package_relation in pkgbase_architecture.make_dependencies.iter() {
                    lookup.insert_package_relation(package_relation, Some(origin.clone()));
                }
            }
        }

        // Add architecture-agnostic dependencies
        for package_relation in build_dependencies.iter() {
            lookup.insert_package_relation(package_relation, Some(origin.clone()));
        }
        for relation in run_time_dependencies.iter() {
            lookup.insert_relation_or_soname(relation, Some(origin.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use alpm_types::{Name, RelationLookup, Version};
    use testresult::TestResult;

    use super::*;

    /// A SRCINFO string.
    const SRCINFO_V1: &str = r#"pkgbase = example
	pkgdesc = A example with all pkgbase properties set.
	pkgver = 0.1.0
	pkgrel = 1
	url = https://archlinux.org/
	arch = x86_64
	arch = aarch64
	groups = group
	groups = group_2
	license = MIT
	license = Apache-2.0
	checkdepends = default_checkdep
	checkdepends = default_checkdep_2=2.0.0
	makedepends_x86_64 = default_makedep
	makedepends_aarch64 = arm_default_makedep_2=2.0.0
	depends = default_dep
	depends_x86_64 = x86_default_dep
	depends_aarch64 = arm_default_dep_2=2.0.0
	optdepends = default_optdep
	optdepends = default_optdep_2=2.0.0: With description
	provides = default_provides
	provides = default_provides_2=2.0.0
	conflicts = default_conflict
	conflicts = default_conflict_2=2.0.0
	replaces = default_replaces
	replaces = default_replaces_2=2.0.0

pkgname = example
"#;

    #[test]
    fn source_info_add_build_dependencies_to_lookup_none() -> TestResult {
        let source_info = SourceInfo::from_str(SRCINFO_V1)?;
        let mut lookup = RelationLookup::default();

        source_info.add_build_dependencies_to_lookup(&mut lookup, None);
        assert_eq!(lookup.len(), 1);
        eprintln!("{lookup:?}");
        assert!(
            lookup.satisfies_name_and_version(
                &Name::from_str("default_dep")?,
                &Version::from_str("1")?
            )
        );

        Ok(())
    }

    #[test]
    fn source_info_add_build_dependencies_to_lookup_x86_64() -> TestResult {
        let source_info = SourceInfo::from_str(SRCINFO_V1)?;
        let mut lookup = RelationLookup::default();

        source_info.add_build_dependencies_to_lookup(&mut lookup, Some(&"x86_64".parse()?));
        assert_eq!(lookup.len(), 3);
        eprintln!("{lookup:?}");
        assert!(
            lookup.satisfies_name_and_version(
                &Name::from_str("default_dep")?,
                &Version::from_str("1")?
            )
        );
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("x86_default_dep")?,
            &Version::from_str("1")?
        ));
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("default_makedep")?,
            &Version::from_str("1")?
        ));

        Ok(())
    }

    #[test]
    fn source_info_add_build_dependencies_to_lookup_aarch64() -> TestResult {
        let source_info = SourceInfo::from_str(SRCINFO_V1)?;
        let mut lookup = RelationLookup::default();

        source_info.add_build_dependencies_to_lookup(&mut lookup, Some(&"aarch64".parse()?));
        assert_eq!(lookup.len(), 3);
        eprintln!("{lookup:?}");
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("default_dep")?,
            &Version::from_str("1.2.3-1")?
        ));
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("arm_default_makedep_2")?,
            &Version::from_str("2.0.0")?
        ));
        assert!(lookup.satisfies_name_and_version(
            &Name::from_str("arm_default_dep_2")?,
            &Version::from_str("2.0.0")?
        ));

        Ok(())
    }

    #[test]
    fn source_info_add_build_dependencies_to_lookup_riscv64() -> TestResult {
        let source_info = SourceInfo::from_str(SRCINFO_V1)?;
        let mut lookup = RelationLookup::default();

        source_info.add_build_dependencies_to_lookup(&mut lookup, Some(&"riscv64".parse()?));
        assert_eq!(lookup.len(), 0);
        eprintln!("{lookup:?}");

        Ok(())
    }
}
