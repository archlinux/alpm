use pyo3::prelude::*;

mod error;
mod source_info;

#[pymodule(gil_used = false, name = "alpm_srcinfo", submodule)]
pub mod py_srcinfo {
    use pyo3::prelude::*;

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

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let modules = PyModule::import(m.py(), "sys")?.getattr("modules")?;
        modules.set_item("alpm.alpm_srcinfo.error", m.getattr("error")?)?;
        modules.set_item("alpm.alpm_srcinfo.source_info", m.getattr("source_info")?)?;
        Ok(())
    }
}
