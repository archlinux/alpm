#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod checksum;
pub use checksum::Checksum;
#[allow(deprecated)]
pub use checksum::Md5Sum;

mod source;
pub use source::Filename;
pub use source::Source;
pub use source::SourceLocation;

/// Public re-exports of common hash functions, for use with [`Checksum`].
pub mod digests {
    pub use blake2::Blake2b512;
    pub use md5::Md5;
    pub use sha1::Sha1;
    pub use sha2::Sha224;
    pub use sha2::Sha256;
    pub use sha2::Sha384;
    pub use sha2::Sha512;
}

mod date;
pub use date::BuildDate;

mod env;
pub use env::BuildEnv;
pub use env::BuildOption;
pub use env::Installed;
pub use env::PackageOption;

mod error;
pub use error::Error;

mod macros;
use macros::regex_once;

mod name;
pub use name::BuildTool;
pub use name::Name;

mod path;
pub use path::AbsolutePath;
pub use path::BuildDir;
pub use path::StartDir;

mod pkg;
pub use pkg::Packager;
pub use pkg::PkgType;

mod size;
pub use size::CompressedSize;
pub use size::InstalledSize;

mod system;
pub use system::Architecture;

mod version;
pub use version::BuildToolVer;
pub use version::Epoch;
pub use version::Pkgrel;
pub use version::Pkgver;
pub use version::SchemaVersion;
pub use version::Version;
pub use version::VersionComparison;
pub use version::VersionRequirement;
