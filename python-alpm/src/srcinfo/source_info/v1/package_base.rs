use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBase(alpm_srcinfo::source_info::v1::package_base::PackageBase);

impl_from!(
    PackageBase,
    alpm_srcinfo::source_info::v1::package_base::PackageBase
);

#[pymodule(gil_used = false, name = "package_base", submodule)]
pub mod py_package_base {
    #[pymodule_export]
    use super::PackageBase;
}
