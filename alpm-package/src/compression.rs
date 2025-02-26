use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

#[cfg(doc)]
use crate::package::Package;

/// An error that can occur when dealing with compression of package files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file uses an unknown compression algorithm
    #[error("The path {path} uses an unknown compression algorithm")]
    UnknownCompression {
        /// The path of the file that uses an unknown compression.
        path: PathBuf,
    },

    /// A string represents an unknown compression algorithm.
    #[error("The string \"{string}\" represents an unknown compression algorithm")]
    UnknownCompressionString {
        /// The unknown compression string.
        string: String,
    },
}

/// The compression algorithm used for a [`Package`].
///
/// TODO: impl From<zip::write::FileOptions> or so
#[derive(Clone, Debug, strum::Display)]
pub enum PackageCompression {
    /// The package is compressed using the zstandard compression algorithm.
    #[strum(to_string = "bz2")]
    Bzip {
        /// The used compression level.
        compression_level: Option<i64>,
    },
    /// The package is compressed using the gzip compression algorithm.
    #[strum(to_string = "gz")]
    Gzip {
        /// The used compression level.
        compression_level: Option<u32>,
    },
    /// The package is not compressed.
    #[strum(to_string = "")]
    None,
    /// The package is compressed using the XZ compression algorithm.
    #[strum(to_string = "xz")]
    Xz {
        /// The used compression level.
        compression_level: Option<i64>,
    },
    /// The package is compressed using the zstandard compression algorithm.
    #[strum(to_string = "zst")]
    Zstandard {
        /// The used compression level.
        compression_level: Option<i64>,
    },
}

impl TryFrom<&Path> for PackageCompression {
    type Error = crate::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let extension = {
            let Some(str) = value.extension() else {
                return Err(Error::UnknownCompression {
                    path: value.to_path_buf(),
                }
                .into());
            };
            let Some(str) = str.to_str() else {
                return Err(Error::UnknownCompression {
                    path: value.to_path_buf(),
                }
                .into());
            };
            str
        };

        Ok(match extension {
            "gz" => Self::Gzip {
                compression_level: None,
            },
            // TODO: implement all
            _ => {
                return Err(Error::UnknownCompression {
                    path: value.to_path_buf(),
                }
                .into());
            }
        })
    }
}

impl FromStr for PackageCompression {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "" => PackageCompression::None,
            "bz2" => PackageCompression::Bzip {
                compression_level: None,
            },
            "gz" => PackageCompression::Gzip {
                compression_level: None,
            },
            "xz" => PackageCompression::Xz {
                compression_level: None,
            },
            "zst" => PackageCompression::Zstandard {
                compression_level: None,
            },
            _ => {
                return Err(Error::UnknownCompressionString {
                    string: s.to_string(),
                }
                .into());
            }
        })
    }
}
