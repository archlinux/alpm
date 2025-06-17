#![doc = include_str!("../README.md")]
use pyo3::prelude::*;

mod types;

#[pymodule(gil_used = false)]
mod alpm {
    #[pymodule_export]
    pub use crate::types::types;
}
