#![doc = include_str!("../README.md")]

mod checksum;
pub use checksum::{
    Blake2b512Checksum,
    Checksum,
    ChecksumAlgorithm,
    Crc32CksumChecksum,
    DigestString as Digest,
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

pub mod url;
pub use url::{SourceUrl, Url};

/// Public re-exports of common hash functions, for use with [`Checksum`].
pub mod digests {
    pub use blake2::Blake2b512;
    pub use md5::Md5;
    pub use sha1::Sha1;
    pub use sha2::{Sha224, Sha256, Sha384, Sha512};

    pub use crate::checksum::{Crc32Cksum, DigestEncoding, DigestString as Digest};
}

mod compression;
pub use compression::CompressionAlgorithmFileExtension;

mod date;
pub use date::{BuildDate, FromOffsetDateTime};

mod env;
pub use env::{BuildEnvironmentOption, InstalledPackage, MakepkgOption, PackageOption};

mod file_type;
pub use file_type::FileTypeIdentifier;

mod error;
pub use error::Error;

mod license;
pub use license::License;

mod name;
pub use name::{BuildTool, Name, SharedObjectName};

mod package;
pub use package::{
    contents::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName},
    error::Error as PackageError,
    file_name::PackageFileName,
    installation::PackageInstallReason,
    source::{PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME},
    validation::PackageValidation,
};

mod path;
pub use path::{
    AbsolutePath,
    Backup,
    BuildDirectory,
    Changelog,
    Install,
    RelativeFilePath,
    RelativePath,
    SonameLookupDirectory,
    StartDirectory,
};

mod openpgp;
pub use openpgp::{
    Base64OpenPGPSignature,
    OpenPGPIdentifier,
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
    Packager,
};

mod pkg;
pub use pkg::{ExtraData, ExtraDataEntry, PackageBaseName, PackageDescription, PackageType};

mod relation;
pub use relation::{
    Group,
    OptionalDependency,
    PackageRelation,
    RelationOrSoname,
    SharedLibraryPrefix,
    Soname,
    SonameV1,
    SonameV2,
    VersionOrSoname,
};

mod size;
pub use size::{CompressedSize, InstalledSize};

mod system;
pub use system::{
    Architecture,
    Architectures,
    ElfArchitectureFormat,
    SystemArchitecture,
    UnknownArchitecture,
};

mod version;
pub use version::{
    base::{Epoch, PackageRelease, PackageVersion},
    buildtool::BuildToolVersion,
    comparison::{VersionSegment, VersionSegments},
    pkg_full::FullVersion,
    pkg_generic::Version,
    pkg_minimal::MinimalVersion,
    requirement::{VersionComparison, VersionRequirement},
    schema::SchemaVersion,
};

/// Public re-exports for use with [`SchemaVersion`].
pub mod semver_version {
    pub use semver::Version;
}

fluent_i18n::i18n!("locales");
