use std::collections::BTreeMap;

use pyo3::prelude::*;

use crate::{
    helpers::impl_from,
    srcinfo::source_info::v1::package::PackageArchitecture,
    types::{
        checksum::{
            SkippableBlake2b512Checksum,
            SkippableMd5Checksum,
            SkippableSha1Checksum,
            SkippableSha224Checksum,
            SkippableSha256Checksum,
            SkippableSha384Checksum,
            SkippableSha512Checksum,
        },
        env::MakepkgOption,
        license::License,
        openpgp::OpenPGPIdentifier,
        path::RelativePath,
        relation::{OptionalDependency, PackageRelation, RelationOrSoname},
        source::Source,
        system::Architecture,
        url::Url,
        version::FullVersion,
    },
};

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBase(alpm_srcinfo::source_info::v1::package_base::PackageBase);

#[pymethods]
impl PackageBase {
    #[new]
    fn new(name: String, version: FullVersion) -> Result<Self, crate::types::Error> {
        let inner = alpm_srcinfo::source_info::v1::package_base::PackageBase::new_with_defaults(
            alpm_types::Name::new(name.as_str())?,
            version.into(),
        );
        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn description(&self) -> Option<String> {
        self.0.description.to_owned().map(|desc| desc.to_string())
    }

    #[getter]
    fn url(&self) -> Option<Url> {
        self.0.url.to_owned().map(From::from)
    }

    #[getter]
    fn changelog(&self) -> Option<RelativePath> {
        self.0.changelog.to_owned().map(From::from)
    }

    #[getter]
    fn licenses(&self) -> Vec<License> {
        self.0
            .licenses
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn install(&self) -> Option<RelativePath> {
        self.0.install.clone().map(From::from)
    }

    #[getter]
    fn groups(&self) -> Vec<String> {
        self.0.groups.clone()
    }

    #[getter]
    fn options(&self) -> Vec<MakepkgOption> {
        self.0.options.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn backups(&self) -> Vec<RelativePath> {
        self.0.backups.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn version(&self) -> FullVersion {
        self.0.version.clone().into()
    }

    #[getter]
    fn pgp_fingerprints(&self) -> Vec<OpenPGPIdentifier> {
        self.0
            .pgp_fingerprints
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn architectures(&self) -> Vec<Architecture> {
        self.0
            .architectures
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn architecture_properties(&self) -> BTreeMap<Architecture, PackageBaseArchitecture> {
        self.0
            .architecture_properties
            .iter()
            .map(|(key, value)| ((*key).into(), value.clone().into()))
            .collect()
    }

    #[getter]
    fn dependencies(&self) -> Vec<RelationOrSoname> {
        self.0
            .dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn optional_dependencies(&self) -> Vec<OptionalDependency> {
        self.0
            .optional_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn provides(&self) -> Vec<RelationOrSoname> {
        self.0
            .provides
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn conflicts(&self) -> Vec<PackageRelation> {
        self.0
            .conflicts
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn replaces(&self) -> Vec<PackageRelation> {
        self.0
            .replaces
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn check_dependencies(&self) -> Vec<PackageRelation> {
        self.0
            .check_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn make_dependencies(&self) -> Vec<PackageRelation> {
        self.0
            .make_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sources(&self) -> Vec<Source> {
        self.0.sources.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn no_extracts(&self) -> Vec<String> {
        self.0.no_extracts.clone()
    }

    #[getter]
    fn b2_checksums(&self) -> Vec<SkippableBlake2b512Checksum> {
        self.0
            .b2_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn md5_checksums(&self) -> Vec<SkippableMd5Checksum> {
        self.0
            .md5_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha1_checksums(&self) -> Vec<SkippableSha1Checksum> {
        self.0
            .sha1_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha224_checksums(&self) -> Vec<SkippableSha224Checksum> {
        self.0
            .sha224_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha256_checksums(&self) -> Vec<SkippableSha256Checksum> {
        self.0
            .sha256_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha384_checksums(&self) -> Vec<SkippableSha384Checksum> {
        self.0
            .sha384_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha512_checksums(&self) -> Vec<SkippableSha512Checksum> {
        self.0
            .sha512_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }
}

impl_from!(
    PackageBase,
    alpm_srcinfo::source_info::v1::package_base::PackageBase
);

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBaseArchitecture(
    alpm_srcinfo::source_info::v1::package_base::PackageBaseArchitecture,
);

#[pymethods]
impl PackageBaseArchitecture {
    fn merge_package_properties(&mut self, properties: PackageArchitecture) {
        self.0.merge_package_properties(properties.into())
    }

    #[getter]
    fn dependencies(&self) -> Vec<RelationOrSoname> {
        self.0
            .dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn optional_dependencies(&self) -> Vec<OptionalDependency> {
        self.0
            .optional_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn provides(&self) -> Vec<RelationOrSoname> {
        self.0
            .provides
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn conflicts(&self) -> Vec<PackageRelation> {
        self.0
            .conflicts
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn replaces(&self) -> Vec<PackageRelation> {
        self.0
            .replaces
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn check_dependencies(&self) -> Vec<PackageRelation> {
        self.0
            .check_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn make_dependencies(&self) -> Vec<PackageRelation> {
        self.0
            .make_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sources(&self) -> Vec<Source> {
        self.0.sources.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn b2_checksums(&self) -> Vec<SkippableBlake2b512Checksum> {
        self.0
            .b2_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn md5_checksums(&self) -> Vec<SkippableMd5Checksum> {
        self.0
            .md5_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha1_checksums(&self) -> Vec<SkippableSha1Checksum> {
        self.0
            .sha1_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha224_checksums(&self) -> Vec<SkippableSha224Checksum> {
        self.0
            .sha224_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha256_checksums(&self) -> Vec<SkippableSha256Checksum> {
        self.0
            .sha256_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha384_checksums(&self) -> Vec<SkippableSha384Checksum> {
        self.0
            .sha384_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sha512_checksums(&self) -> Vec<SkippableSha512Checksum> {
        self.0
            .sha512_checksums
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }
}

impl_from!(
    PackageBaseArchitecture,
    alpm_srcinfo::source_info::v1::package_base::PackageBaseArchitecture
);

#[pymodule(gil_used = false, name = "package_base", submodule)]
pub mod py_package_base {
    #[pymodule_export]
    use super::PackageBase;
    #[pymodule_export]
    use super::PackageBaseArchitecture;
}
