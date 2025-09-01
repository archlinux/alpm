use pyo3::prelude::*;

pub mod checksum;
pub mod env;
pub mod error;
pub mod license;
pub mod openpgp;
pub mod path;
pub mod relation;
pub mod requirement;
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
    use path::RelativePath;
    #[pymodule_export]
    use relation::OptionalDependency;
    #[pymodule_export]
    use relation::PackageRelation;
    #[pymodule_export]
    use relation::SonameV1;
    #[pymodule_export]
    use relation::SonameV1Type;
    #[pymodule_export]
    use relation::relation_or_soname_from_str;
    #[pymodule_export]
    use requirement::VersionComparison;
    #[pymodule_export]
    use requirement::VersionRequirement;
    #[pymodule_export]
    use system::Architecture;
    #[pymodule_export]
    use system::ElfArchitectureFormat;
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
