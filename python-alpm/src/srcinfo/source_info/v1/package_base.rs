use pyo3::prelude::*;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBase(alpm_srcinfo::source_info::v1::package_base::PackageBase);

impl From<alpm_srcinfo::source_info::v1::package_base::PackageBase> for PackageBase {
    fn from(value: alpm_srcinfo::source_info::v1::package_base::PackageBase) -> Self {
        PackageBase(value)
    }
}

#[pymodule(gil_used = false, name = "package_base", submodule)]
pub mod py_package_base {
    #[pymodule_export]
    use super::PackageBase;
}
