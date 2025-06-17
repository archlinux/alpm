use deriving_via::DerivingVia;
use pyo3::{PyErr, create_exception, prelude::*};

mod version;

create_exception!(types, ALPMError, pyo3::exceptions::PyException);

/// Error wrapper for alpm_types::Error, so that we can convert it to PyErr
#[derive(Debug, DerivingVia)]
#[deriving(From)]
pub struct Error(alpm_types::Error);

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        ALPMError::new_err(err.to_string())
    }
}

#[pymodule(gil_used = false)]
pub mod types {
    #[pymodule_export]
    use version::PackageVersion;
    #[pymodule_export]
    use version::SchemaVersion;

    use super::*;
}
