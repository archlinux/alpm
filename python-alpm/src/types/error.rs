use pyo3::{PyErr, create_exception};

use crate::macros::impl_from;

create_exception!(
    alpm_types,
    ALPMError,
    pyo3::exceptions::PyException,
    "The ALPM error type."
);

/// Error wrapper for alpm_types::Error, so that we can convert it to [`PyErr`].
#[derive(Debug)]
pub struct Error(alpm_types::Error);

impl_from!(Error, alpm_types::Error);

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        ALPMError::new_err(err.0.to_string())
    }
}
