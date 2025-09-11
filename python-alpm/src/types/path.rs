use std::path::PathBuf;

use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelativePath(alpm_types::RelativePath);

#[pymethods]
impl RelativePath {
    // PyO3 handles the conversion from Rust's `std::path::PathBuf` to Python's `pathlib.Path`.
    #[new]
    fn new(path: PathBuf) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::RelativePath::new(path)?;
        Ok(inner.into())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("RelativePath('{}')", self.0)
    }
}

impl_from!(RelativePath, alpm_types::RelativePath);
