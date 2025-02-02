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
    SkippableChecksum,
};

mod source;
pub use source::Source;

mod url;
pub use url::Url;

/// Public re-exports of common hash functions, for use with [`Checksum`].
pub mod digests {
    pub use blake2::Blake2b512;
    pub use digest::Digest;
    pub use md5::Md5;
    pub use sha1::Sha1;
    pub use sha2::Sha224;
    pub use sha2::Sha256;
    pub use sha2::Sha384;
    pub use sha2::Sha512;
}

mod date;
pub use date::{BuildDate, FromOffsetDateTime};

mod env;
pub use env::BuildEnvironmentOption;
pub use env::InstalledPackage;
pub use env::MakepkgOption;
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
pub use path::BuildDirectory;
pub use path::Changelog;
pub use path::Install;
pub use path::RelativePath;
pub use path::StartDirectory;

mod openpgp;
pub use openpgp::OpenPGPIdentifier;
pub use openpgp::OpenPGPKeyId;
pub use openpgp::OpenPGPv4Fingerprint;
pub use openpgp::Packager;

mod pkg;
pub use pkg::ExtraData;
pub use pkg::PackageBaseName;
pub use pkg::PackageDescription;
pub use pkg::PackageType;

mod relation;
pub use relation::Group;
pub use relation::OptionalDependency;
pub use relation::PackageRelation;

mod size;
pub use size::CompressedSize;
pub use size::InstalledSize;

mod system;
pub use system::Architecture;

mod version;
pub use version::BuildToolVersion;
pub use version::Epoch;
pub use version::PackageRelease;
pub use version::PackageVersion;
pub use version::SchemaVersion;
pub use version::Version;
pub use version::VersionComparison;
pub use version::VersionRequirement;
