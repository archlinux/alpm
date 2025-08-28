use pyo3::prelude::*;

mod error;
mod source_info;

#[pymodule(gil_used = false, name = "alpm_srcinfo", submodule)]
pub mod py_srcinfo {
    #[pymodule_export]
    use super::error::SourceInfoError;
    #[pymodule_export]
    use super::error::py_error;
    #[pymodule_export]
    use super::source_info::py_source_info;
    #[pymodule_export]
    use super::source_info::v1::SourceInfoV1;
    #[pymodule_export]
    use super::source_info::v1::merged::MergedPackage;
}
