use pyo3::prelude::*;

pub mod checksum;
pub mod env;
pub mod error;
pub mod license;
pub mod openpgp;
pub mod path;
pub mod relation;
pub mod requirement;
pub mod source;
pub mod system;
pub mod url;
pub mod version;

pub use error::{ALPMError, Error};

#[pymodule(gil_used = false, name = "alpm_types", submodule)]
pub mod py_types {
    #[pymodule_export]
    use ALPMError;
    #[pymodule_export]
    use checksum::Blake2b512Checksum;
    #[pymodule_export]
    use checksum::Crc32CksumChecksum;
    #[pymodule_export]
    use checksum::Md5Checksum;
    #[pymodule_export]
    use checksum::Sha1Checksum;
    #[pymodule_export]
    use checksum::Sha224Checksum;
    #[pymodule_export]
    use checksum::Sha256Checksum;
    #[pymodule_export]
    use checksum::Sha384Checksum;
    #[pymodule_export]
    use checksum::Sha512Checksum;
    #[pymodule_export]
    use checksum::SkippableBlake2b512Checksum;
    #[pymodule_export]
    use checksum::SkippableCrc32CksumChecksum;
    #[pymodule_export]
    use checksum::SkippableMd5Checksum;
    #[pymodule_export]
    use checksum::SkippableSha1Checksum;
    #[pymodule_export]
    use checksum::SkippableSha224Checksum;
    #[pymodule_export]
    use checksum::SkippableSha256Checksum;
    #[pymodule_export]
    use checksum::SkippableSha384Checksum;
    #[pymodule_export]
    use checksum::SkippableSha512Checksum;
    #[pymodule_export]
    use env::BuildEnvironmentOption;
    #[pymodule_export]
    use env::PackageOption;
    #[pymodule_export]
    use env::makepkg_option_from_str;
    #[pymodule_export]
    use license::License;
    #[pymodule_export]
    use openpgp::OpenPGPKeyId;
    #[pymodule_export]
    use openpgp::OpenPGPv4Fingerprint;
    #[pymodule_export]
    use openpgp::openpgp_identifier_from_str;
    #[pymodule_export]
    use path::RelativeFilePath;
    #[pymodule_export]
    use relation::OptionalDependency;
    #[pymodule_export]
    use relation::PackageRelation;
    #[pymodule_export]
    use relation::Soname;
    #[pymodule_export]
    use relation::SonameV1;
    #[pymodule_export]
    use relation::SonameV1Type;
    #[pymodule_export]
    use relation::SonameV2;
    #[pymodule_export]
    use relation::relation_or_soname_from_str;
    #[pymodule_export]
    use requirement::VersionComparison;
    #[pymodule_export]
    use requirement::VersionRequirement;
    #[pymodule_export]
    use source::Source;
    #[pymodule_export]
    use system::Architecture;
    #[pymodule_export]
    use system::Architectures;
    #[pymodule_export]
    use system::ElfArchitectureFormat;
    #[pymodule_export]
    use system::KnownArchitecture;
    #[pymodule_export]
    use system::UnknownArchitecture;
    #[pymodule_export]
    use url::BzrInfo;
    #[pymodule_export]
    use url::FossilInfo;
    #[pymodule_export]
    use url::GitInfo;
    #[pymodule_export]
    use url::HgInfo;
    #[pymodule_export]
    use url::SourceUrl;
    #[pymodule_export]
    use url::SvnInfo;
    #[pymodule_export]
    use url::Url;
    #[pymodule_export]
    use version::Epoch;
    #[pymodule_export]
    use version::FullVersion;
    #[pymodule_export]
    use version::PackageRelease;
    #[pymodule_export]
    use version::PackageVersion;
    #[pymodule_export]
    use version::SchemaVersion;
    #[pymodule_export]
    use version::Version;

    use super::*;
}
