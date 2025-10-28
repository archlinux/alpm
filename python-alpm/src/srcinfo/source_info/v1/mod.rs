pub mod merged;
pub mod package;
pub mod package_base;

use std::path::PathBuf;

use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct SourceInfoV1(alpm_srcinfo::SourceInfoV1);

#[pymethods]
impl SourceInfoV1 {
    #[new]
    fn new(srcinfo: &str) -> Result<Self, crate::srcinfo::error::Error> {
        let inner = alpm_srcinfo::SourceInfoV1::from_string(srcinfo)?;
        Ok(inner.into())
    }

    #[staticmethod]
    fn from_file(path: PathBuf) -> Result<Self, crate::srcinfo::error::Error> {
        let inner = alpm_srcinfo::SourceInfoV1::from_file(&path)?;
        Ok(inner.into())
    }

    #[staticmethod]
    fn from_pkgbuild(path: PathBuf) -> Result<Self, crate::srcinfo::error::Error> {
        let inner = alpm_srcinfo::SourceInfoV1::from_pkgbuild(&path)?;
        Ok(inner.into())
    }

    #[getter]
    fn base(&self) -> package_base::PackageBase {
        self.0.base.clone().into()
    }

    #[getter]
    fn packages(&self) -> Vec<package::Package> {
        self.0.packages.iter().map(|p| p.clone().into()).collect()
    }

    pub fn packages_for_architecture(
        &self,
        architecture: crate::types::system::Architecture,
    ) -> Vec<merged::MergedPackage> {
        self.0
            .packages_for_architecture(architecture)
            .map(From::from)
            .collect()
    }

    pub fn as_srcinfo(&self) -> String {
        self.0.as_srcinfo()
    }

    fn __str__(&self) -> String {
        self.as_srcinfo()
    }

    fn __repr__(&self) -> String {
        format!("SourceInfoV1(srcinfo={})", self.as_srcinfo())
    }
}

impl_from!(SourceInfoV1, alpm_srcinfo::SourceInfoV1);

#[pymodule(gil_used = false, name = "v1", submodule)]
pub mod py_v1 {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::SourceInfoV1;
    #[pymodule_export]
    use super::merged::py_merged;
    #[pymodule_export]
    use super::package::py_package;
    #[pymodule_export]
    use super::package_base::py_package_base;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let modules = PyModule::import(m.py(), "sys")?.getattr("modules")?;
        modules.set_item(
            "alpm.alpm_srcinfo.source_info.v1.merged",
            m.getattr("merged")?,
        )?;
        modules.set_item(
            "alpm.alpm_srcinfo.source_info.v1.package",
            m.getattr("package")?,
        )?;
        modules.set_item(
            "alpm.alpm_srcinfo.source_info.v1.package_base",
            m.getattr("package_base")?,
        )?;
        Ok(())
    }
}
