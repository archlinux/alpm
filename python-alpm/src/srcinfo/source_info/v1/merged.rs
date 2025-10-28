use std::path::PathBuf;

use alpm_srcinfo::source_info::v1::merged as alpm_srcinfo_merged;
use pyo3::prelude::*;

use crate::{
    macros::{impl_from, vec_convert},
    srcinfo::source_info::v1::{package::Package, package_base::PackageBase},
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
        relation::{OptionalDependency, PackageRelation, RelationOrSoname},
        source::Source,
        system::Architecture,
        url::Url,
        version::FullVersion,
    },
};

#[derive(Debug, FromPyObject, IntoPyObject)]
// Price paid for Python (we can't `Box` a `Package` as it's passed from Python)
#[allow(clippy::large_enum_variant)]
pub enum PackageOrName {
    Package(Package),
    Name(String),
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct MergedPackage(alpm_srcinfo_merged::MergedPackage);

#[pymethods]
impl MergedPackage {
    #[new]
    fn new(
        architecture: Architecture,
        base: PackageBase,
        package_or_name: PackageOrName,
    ) -> Result<Self, crate::types::Error> {
        let inner = match package_or_name {
            PackageOrName::Package(package) => {
                alpm_srcinfo_merged::MergedPackage::from_base_and_package(
                    architecture,
                    &base.into(),
                    &package.into(),
                )
            }
            PackageOrName::Name(name_string) => alpm_srcinfo_merged::MergedPackage::from_base(
                architecture,
                alpm_types::Name::new(name_string.as_str())?,
                &base.into(),
            ),
        };

        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn description(&self) -> Option<String> {
        self.0.description.as_ref().map(ToString::to_string)
    }

    #[getter]
    fn url(&self) -> Option<Url> {
        self.0.url.clone().map(From::from)
    }

    #[getter]
    fn licenses(&self) -> Vec<License> {
        vec_convert!(self.0.licenses.clone())
    }

    #[getter]
    fn architecture(&self) -> Architecture {
        self.0.architecture.clone().into()
    }

    #[getter]
    fn changelog(&self) -> Option<PathBuf> {
        self.0
            .changelog
            .clone()
            .map(|rel_path| rel_path.inner().to_path_buf())
    }

    #[getter]
    fn install(&self) -> Option<PathBuf> {
        self.0
            .install
            .clone()
            .map(|rel_path| rel_path.inner().to_path_buf())
    }

    #[getter]
    fn groups(&self) -> Vec<String> {
        self.0.groups.clone()
    }

    #[getter]
    fn options(&self) -> Vec<MakepkgOption> {
        vec_convert!(self.0.options.clone())
    }

    #[getter]
    fn backups(&self) -> Vec<PathBuf> {
        self.0
            .backups
            .clone()
            .into_iter()
            .map(|rel_path| rel_path.inner().to_path_buf())
            .collect()
    }

    #[getter]
    fn version(&self) -> FullVersion {
        self.0.version.clone().into()
    }

    #[getter]
    fn pgp_fingerprints(&self) -> Vec<OpenPGPIdentifier> {
        vec_convert!(self.0.pgp_fingerprints.clone())
    }

    #[getter]
    fn dependencies(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.dependencies.clone())
    }

    #[getter]
    fn optional_dependencies(&self) -> Vec<OptionalDependency> {
        vec_convert!(self.0.optional_dependencies.clone())
    }

    #[getter]
    fn provides(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.provides.clone())
    }

    #[getter]
    fn conflicts(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.conflicts.clone())
    }

    #[getter]
    fn replaces(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.replaces.clone())
    }

    #[getter]
    fn check_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.check_dependencies.clone())
    }

    #[getter]
    fn make_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.make_dependencies.clone())
    }

    #[getter]
    fn sources(&self) -> Vec<MergedSource> {
        vec_convert!(self.0.sources.clone())
    }

    #[getter]
    fn no_extracts(&self) -> Vec<String> {
        self.0.no_extracts.clone()
    }

    fn __str__(&self) -> String {
        self.0.name.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "MergedPackage(architecture={}, name='{}', version={})",
            self.architecture().__repr__(),
            self.0.name,
            self.version().__repr__()
        )
    }
}

impl_from!(MergedPackage, alpm_srcinfo_merged::MergedPackage);

#[pyclass(frozen)]
#[derive(Clone, Debug)]
pub struct MergedSource(alpm_srcinfo_merged::MergedSource);

#[pymethods]
impl MergedSource {
    #[getter]
    fn source(&self) -> Source {
        self.0.source.clone().into()
    }

    #[getter]
    fn b2_checksum(&self) -> Option<SkippableBlake2b512Checksum> {
        self.0.b2_checksum.clone().map(From::from)
    }

    #[getter]
    fn md5_checksum(&self) -> Option<SkippableMd5Checksum> {
        self.0.md5_checksum.clone().map(From::from)
    }

    #[getter]
    fn sha1_checksum(&self) -> Option<SkippableSha1Checksum> {
        self.0.sha1_checksum.clone().map(From::from)
    }

    #[getter]
    fn sha224_checksum(&self) -> Option<SkippableSha224Checksum> {
        self.0.sha224_checksum.clone().map(From::from)
    }

    #[getter]
    fn sha256_checksum(&self) -> Option<SkippableSha256Checksum> {
        self.0.sha256_checksum.clone().map(From::from)
    }

    #[getter]
    fn sha384_checksum(&self) -> Option<SkippableSha384Checksum> {
        self.0.sha384_checksum.clone().map(From::from)
    }

    #[getter]
    fn sha512_checksum(&self) -> Option<SkippableSha512Checksum> {
        self.0.sha512_checksum.clone().map(From::from)
    }
}

impl_from!(MergedSource, alpm_srcinfo_merged::MergedSource);

#[pymodule(gil_used = false, name = "merged", submodule)]
pub mod py_merged {
    #[pymodule_export]
    use super::MergedPackage;
    #[pymodule_export]
    use super::MergedSource;
}
