#![doc = include_str!("../README.md")]
#![allow(rustdoc::broken_intra_doc_links)]

use pyo3::prelude::*;

pub(crate) mod macros;
mod srcinfo;
mod types;

#[pymodule(gil_used = false, name = "_native")]
mod py_alpm {
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::srcinfo::py_srcinfo;
    #[pymodule_export]
    use crate::types::ALPMError;
    #[pymodule_export]
    use crate::types::py_types;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let modules = PyModule::import(m.py(), "sys")?.getattr("modules")?;
        modules.set_item("alpm.alpm_types", m.getattr("alpm_types")?)?;
        modules.set_item("alpm.alpm_srcinfo", m.getattr("alpm_srcinfo")?)?;
        Ok(())
    }
}
