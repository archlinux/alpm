//! Package creation configuration.

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use alpm_compress::compression::CompressionSettings;
#[cfg(doc)]
use alpm_pkginfo::PackageInfo;
use alpm_types::PackageFileName;
use fluent_i18n::t;

use crate::input::PackageInput;
#[cfg(doc)]
use crate::package::Package;

/// An output directory that is guaranteed to be an absolute, writable directory.
#[derive(Clone, Debug)]
pub struct OutputDir(PathBuf);

impl OutputDir {
    /// Creates a new [`OutputDir`] from `path`.
    ///
    /// Creates a directory at `path` if it does not exist yet.
    /// Also creates any missing parent directories.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `path` is not absolute,
    /// - `path` does not exist and cannot be created,
    /// - the metadata of `path` cannot be retrieved,
    /// - `path` is not a directory,
    /// - or `path` is only read-only.
    pub fn new(path: PathBuf) -> Result<Self, crate::Error> {
        if !path.is_absolute() {
            return Err(alpm_common::Error::NonAbsolutePaths {
                paths: vec![path.clone()],
            }
            .into());
        }

        if !path.exists() {
            create_dir_all(&path).map_err(|source| crate::Error::IoPath {
                path: path.clone(),
                context: t!("error-io-create-output-dir"),
                source,
            })?;
        }

        let metadata = path.metadata().map_err(|source| crate::Error::IoPath {
            path: path.clone(),
            context: t!("error-io-get-metadata"),
            source,
        })?;

        if !metadata.is_dir() {
            return Err(alpm_common::Error::NotADirectory { path: path.clone() }.into());
        }

        if metadata.permissions().readonly() {
            return Err(crate::Error::PathIsReadOnly { path: path.clone() });
        }

        Ok(Self(path))
    }

    /// Coerces to a Path slice.
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Converts a Path to an owned PathBuf.
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.to_path_buf()
    }

    /// Creates an owned PathBuf with path adjoined to self.
    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }
}

impl AsRef<Path> for OutputDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

/// A config that tracks the components needed for the creation of an [alpm-package] from input
/// directory.
///
/// Tracks a [`PackageInput`], optional [`CompressionSettings`] and an [`OutputDir`] in which an
/// [alpm-package] is placed after creation.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(Clone, Debug)]
pub struct PackageCreationConfig {
    package_input: PackageInput,
    output_dir: OutputDir,
    compression: CompressionSettings,
}

impl PackageCreationConfig {
    /// Creates a new [`PackageCreationConfig`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - `package_input.input_dir` is equal to `output_dir`,
    /// - `package_input.input_dir` is located inside of `output_dir`,
    /// - or `output_dir` is located inside of `package_input.input_dir`.
    pub fn new(
        package_input: PackageInput,
        output_dir: OutputDir,
        compression: CompressionSettings,
    ) -> Result<Self, crate::Error> {
        if package_input.input_dir() == output_dir.as_path() {
            return Err(crate::Error::InputDirIsOutputDir {
                path: package_input.input_dir().to_path_buf(),
            });
        }
        if output_dir.as_path().starts_with(package_input.input_dir()) {
            return Err(crate::Error::OutputDirInInputDir {
                input_path: package_input.input_dir().to_path_buf(),
                output_path: output_dir.to_path_buf(),
            });
        }
        if package_input.input_dir().starts_with(output_dir.as_path()) {
            return Err(crate::Error::InputDirInOutputDir {
                input_path: package_input.input_dir().to_path_buf(),
                output_path: output_dir.to_path_buf(),
            });
        }

        Ok(Self {
            compression,
            package_input,
            output_dir,
        })
    }

    /// Returns a reference to the [`PackageInput`].
    pub fn package_input(&self) -> &PackageInput {
        &self.package_input
    }

    /// Returns a reference to the [`OutputDir`].
    pub fn output_dir(&self) -> &OutputDir {
        &self.output_dir
    }

    /// Returns a reference to the [`CompressionSettings`].
    pub fn compression(&self) -> &CompressionSettings {
        &self.compression
    }
}

impl From<&PackageCreationConfig> for PackageFileName {
    /// Creates a [`PackageFileName`] from a [`PackageCreationConfig`] reference.
    fn from(value: &PackageCreationConfig) -> Self {
        Self::new(
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgname.clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgname.clone(),
            },
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgver.clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgver.clone(),
            },
            match value.package_input.package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.arch.clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.arch.clone(),
            },
            (&value.compression).into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    /// Ensures that [`OutputDir::new`] creates non-existing, absolute directories.
    #[test]
    fn output_dir_new_creates_dir() -> TestResult {
        let temp_dir = tempdir()?;
        let non_existing_path = temp_dir.path().join("non-existing");
        if let Err(error) = OutputDir::new(non_existing_path) {
            return Err(format!("Failed although it should have succeeded:\n{error}").into());
        }

        Ok(())
    }

    /// Ensures that [`OutputDir::new`] fails on relative paths and non-directory paths.
    #[test]
    fn output_dir_new_fails() -> TestResult {
        assert!(matches!(
            OutputDir::new(PathBuf::from("test")),
            Err(crate::Error::AlpmCommon(
                alpm_common::Error::NonAbsolutePaths { paths: _ }
            ))
        ));

        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("non-existing");
        let _file = File::create(&file_path)?;
        assert!(matches!(
            OutputDir::new(file_path),
            Err(crate::Error::AlpmCommon(
                alpm_common::Error::NotADirectory { path: _ }
            ))
        ));

        Ok(())
    }

    /// Ensures that [`OutputDir::as_ref`] works.
    #[test]
    fn output_dir_as_ref() -> TestResult {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();

        let output_dir = OutputDir::new(path.to_path_buf())?;

        assert_eq!(output_dir.as_ref(), path);

        Ok(())
    }
}
