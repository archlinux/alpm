use alpm_srcinfo::source_info::v1::merged as alpm_srcinfo_merged;
use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen)]
#[derive(Clone, Debug)]
pub struct MergedPackage(alpm_srcinfo_merged::MergedPackage);

#[pymethods]
impl MergedPackage {
    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn description(&self) -> Option<String> {
        self.0.description.as_ref().map(ToString::to_string)
    }

    #[getter]
    fn url(&self) -> Option<crate::types::url::Url> {
        self.0.url.clone().map(From::from)
    }

    #[getter]
    fn licenses(&self) -> Vec<crate::types::license::License> {
        self.0
            .licenses
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn architecture(&self) -> crate::types::system::Architecture {
        self.0.architecture.into()
    }

    #[getter]
    fn changelog(&self) -> Option<crate::types::path::RelativePath> {
        self.0.changelog.clone().map(From::from)
    }

    #[getter]
    fn install(&self) -> Option<crate::types::path::RelativePath> {
        self.0.install.clone().map(From::from)
    }

    #[getter]
    fn groups(&self) -> Vec<String> {
        self.0.groups.clone()
    }

    #[getter]
    fn options(&self) -> Vec<crate::types::env::MakepkgOption> {
        self.0.options.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn backups(&self) -> Vec<crate::types::path::RelativePath> {
        self.0.backups.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn version(&self) -> crate::types::version::FullVersion {
        self.0.version.clone().into()
    }

    #[getter]
    fn pgp_fingerprints(&self) -> Vec<crate::types::openpgp::OpenPGPIdentifier> {
        self.0
            .pgp_fingerprints
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn dependencies(&self) -> Vec<crate::types::relation::RelationOrSoname> {
        self.0
            .dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn optional_dependencies(&self) -> Vec<crate::types::relation::OptionalDependency> {
        self.0
            .optional_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn provides(&self) -> Vec<crate::types::relation::RelationOrSoname> {
        self.0
            .provides
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn conflicts(&self) -> Vec<crate::types::relation::PackageRelation> {
        self.0
            .conflicts
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn replaces(&self) -> Vec<crate::types::relation::PackageRelation> {
        self.0
            .replaces
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn check_dependencies(&self) -> Vec<crate::types::relation::PackageRelation> {
        self.0
            .check_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn make_dependencies(&self) -> Vec<crate::types::relation::PackageRelation> {
        self.0
            .make_dependencies
            .clone()
            .into_iter()
            .map(From::from)
            .collect()
    }

    #[getter]
    fn sources(&self) -> Vec<MergedSource> {
        self.0.sources.clone().into_iter().map(From::from).collect()
    }

    #[getter]
    fn no_extracts(&self) -> Vec<String> {
        self.0.no_extracts.clone()
    }
}

impl_from!(MergedPackage, alpm_srcinfo_merged::MergedPackage);

#[pyclass(frozen)]
#[derive(Clone, Debug)]
pub struct MergedSource(alpm_srcinfo_merged::MergedSource);

impl_from!(MergedSource, alpm_srcinfo_merged::MergedSource);

#[pymodule(gil_used = false, name = "merged", submodule)]
pub mod py_merged {
    #[pymodule_export]
    use super::MergedPackage;
    #[pymodule_export]
    use super::MergedSource;
}
