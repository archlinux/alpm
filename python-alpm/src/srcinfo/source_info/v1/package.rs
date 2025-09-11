use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Package(alpm_srcinfo::source_info::v1::package::Package);

impl_from!(Package, alpm_srcinfo::source_info::v1::package::Package);

#[pymodule(gil_used = false, name = "package", submodule)]
pub mod py_package {
    #[pymodule_export]
    use super::Package;
}
