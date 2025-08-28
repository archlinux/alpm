use pyo3::prelude::*;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Package(alpm_srcinfo::source_info::v1::package::Package);

impl From<alpm_srcinfo::source_info::v1::package::Package> for Package {
    fn from(value: alpm_srcinfo::source_info::v1::package::Package) -> Self {
        Package(value)
    }
}

#[pymodule(gil_used = false, name = "package", submodule)]
pub mod py_package {
    #[pymodule_export]
    use super::Package;
}
