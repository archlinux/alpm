#![doc = include_str!("../README.md")]
#![allow(rustdoc::broken_intra_doc_links)]

use pyo3::prelude::*;

mod types;

#[pymodule(gil_used = false, name = "alpm")]
mod py_alpm {
    #[pymodule_export]
    use crate::types::ALPMError;
    #[pymodule_export]
    use crate::types::py_types;
}
