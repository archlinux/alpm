#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod checksum;
pub use checksum::{
    Blake2b512Checksum,
    Checksum,
    Digest,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
};

mod source;
pub use source::Source;

mod url;
pub use url::Url;

/// Public re-exports of common hash functions, for use with [`Checksum`].
pub mod digests {
    pub use blake2::Blake2b512;
    pub use sha1::Sha1;
    pub use sha2::Sha224;
    pub use sha2::Sha256;
    pub use sha2::Sha384;
    pub use sha2::Sha512;
}

mod date;
pub use date::{BuildDate, FromOffsetDateTime};

mod env;
pub use env::BuildEnv;
pub use env::InstalledPackage;
pub use env::MakePkgOption;
pub use env::PackageOption;

mod error;
pub use error::Error;

mod license;
pub use license::License;

mod name;
pub use name::BuildTool;
pub use name::Name;

mod path;
pub use path::AbsolutePath;
pub use path::Backup;
pub use path::BuildDir;
pub use path::Changelog;
pub use path::Install;
pub use path::RelativePath;
pub use path::StartDir;

mod openpgp;
pub use openpgp::OpenPGPv4Fingerprint;
pub use openpgp::Packager;

mod pkg;
pub use pkg::ExtraData;
pub use pkg::PkgBase;
pub use pkg::PkgDesc;
pub use pkg::PkgType;

mod relation;
pub use relation::Group;
pub use relation::OptDepend;
pub use relation::PackageRelation;

mod size;
pub use size::CompressedSize;
pub use size::InstalledSize;

mod system;
pub use system::Architecture;

mod version;
pub use version::BuildToolVersion;
pub use version::Epoch;
pub use version::Pkgrel;
pub use version::Pkgver;
pub use version::SchemaVersion;
pub use version::Version;
pub use version::VersionComparison;
pub use version::VersionRequirement;
