use pyo3::prelude::*;

mod checksum;
mod env;
mod error;
mod license;
mod openpgp;
mod path;
mod system;
mod url;
mod version;

pub use error::{ALPMError, Error};

#[pymodule(gil_used = false, name = "alpm_types")]
pub mod types {
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
    use env::parse_makepkg_option;
    #[pymodule_export]
    use license::License;
    #[pymodule_export]
    use openpgp::OpenPGPKeyId;
    #[pymodule_export]
    use openpgp::OpenPGPv4Fingerprint;
    #[pymodule_export]
    use openpgp::parse_openpgp_identifier;
    #[pymodule_export]
    use path::RelativePath;
    #[pymodule_export]
    use system::Architecture;
    #[pymodule_export]
    use url::Url;
    #[pymodule_export]
    use version::Epoch;
    #[pymodule_export]
    use version::PackageRelease;
    #[pymodule_export]
    use version::PackageVersion;
    #[pymodule_export]
    use version::SchemaVersion;

    use super::*;
}
