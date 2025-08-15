use pyo3::{PyErr, create_exception};

create_exception!(
    types,
    ALPMError,
    pyo3::exceptions::PyException,
    "The ALPM error type."
);

/// Error wrapper for alpm_types::Error, so that we can convert it to [`PyErr`].
#[derive(Debug)]
pub struct Error(alpm_types::Error);

impl From<alpm_types::Error> for Error {
    fn from(err: alpm_types::Error) -> Self {
        Error(err)
    }
}

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        ALPMError::new_err(err.0.to_string())
    }
}
