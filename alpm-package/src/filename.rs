use std::{fmt::Display, path::PathBuf, str::FromStr};

use alpm_types::{Architecture, Name, Version};

use crate::{compression::PackageCompression, pipeline::PackagePipeline};

/// An error that can occur when dealing with filenames of alpm-packages.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A path can not be used as a [`PackageFileName`].
    #[error("The path {path} is not a valid alpm-package filename")]
    InvalidPath {
        /// The file path that is not valid.
        path: PathBuf,
    },

    /// A string is not a valid [`PackageFileName`].
    #[error("The string {string} is not a valid alpm-package filename")]
    InvalidString {
        /// The file path that is not valid.
        string: String,
    },
}

#[derive(Clone, Debug)]
pub struct PackageFileName {
    name: Name,
    version: Version,
    architecture: Architecture,
    compression: PackageCompression,
}

impl PackageFileName {
    pub fn new(
        name: Name,
        version: Version,
        architecture: Architecture,
        compression: PackageCompression,
    ) -> Self {
        Self {
            name,
            version,
            architecture,
            compression,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn architecture(&self) -> &Architecture {
        &self.architecture
    }

    pub fn compression(&self) -> &PackageCompression {
        &self.compression
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }
}

impl Display for PackageFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}.pkg.tar{}",
            self.name,
            self.version,
            self.architecture,
            match self.compression {
                PackageCompression::None => self.compression.to_string(),
                _ => format!(".{}", self.compression),
            }
        )
    }
}

impl From<&PackagePipeline> for PackageFileName {
    fn from(value: &PackagePipeline) -> Self {
        Self {
            name: match value.package_input.get_package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgname().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgname().clone(),
            },
            version: match value.package_input.get_package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => package_info.pkgver().clone(),
                alpm_pkginfo::PackageInfo::V2(package_info) => package_info.pkgver().clone(),
            },
            architecture: match value.package_input.get_package_info() {
                alpm_pkginfo::PackageInfo::V1(package_info) => *package_info.arch(),
                alpm_pkginfo::PackageInfo::V2(package_info) => *package_info.arch(),
            },
            compression: value.compression.clone(),
        }
    }
}

impl FromStr for PackageFileName {
    type Err = crate::Error;

    /// Creates a [`PackageFileName`] from string slice.
    ///
    /// TODO: replace with proper winnow parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_package::filename::PackageFileName;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let filename = "example-package-1:1.0.0-1-x86_64.pkg.tar.zst";
    /// assert_eq!(filename, PackageFileName::from_str(filename)?.to_string());
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((name_version_arch, rest)) = s.split_once(".pkg.tar") else {
            return Err(Error::InvalidString {
                string: s.to_string(),
            }
            .into());
        };

        let compression = {
            if rest.is_empty() {
                PackageCompression::None
            } else {
                let Some((_empty, compression)) = rest.split_once(".") else {
                    return Err(Error::InvalidString {
                        string: s.to_string(),
                    }
                    .into());
                };
                PackageCompression::from_str(compression)?
            }
        };

        let Some((name_version, arch)) = name_version_arch.rsplit_once("-") else {
            return Err(Error::InvalidString {
                string: s.to_string(),
            }
            .into());
        };

        let architecture = Architecture::from_str(arch)?;

        let strings: Vec<&str> = name_version.rsplitn(3, '-').collect();
        if strings.len() != 3 {
            return Err(Error::InvalidString {
                string: s.to_string(),
            }
            .into());
        }
        let version = Version::from_str(&format!("{}-{}", strings[1], strings[0]))?;
        let name = Name::from_str(strings[2])?;

        Ok(Self {
            name,
            version,
            architecture,
            compression,
        })
    }
}
