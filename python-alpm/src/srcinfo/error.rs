use pyo3::{create_exception, prelude::*};

use crate::macros::impl_from;

create_exception!(
    alpm_srcinfo,
    SourceInfoError,
    pyo3::exceptions::PyException,
    "The high-level exception that can occur when using alpm_srcinfo module."
);

/// Error wrapper for alpm_srcinfo::Error, so that we can convert it to [`PyErr`].
#[derive(Debug)]
pub struct Error(alpm_srcinfo::Error);

impl_from!(Error, alpm_srcinfo::Error);

impl From<Error> for PyErr {
    fn from(value: Error) -> PyErr {
        SourceInfoError::new_err(value.0.to_string())
    }
}

#[pymodule(gil_used = false, name = "error", submodule)]
pub mod py_error {
    #[pymodule_export]
    use super::SourceInfoError;
}
