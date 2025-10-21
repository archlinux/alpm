use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct License(alpm_types::License);

#[pymethods]
impl License {
    #[new]
    fn new(license: &str) -> Result<Self, crate::types::Error> {
        // This should be infallible, as any non-spdx string is valid as an unknown license.
        // That's why the error type here is undocumented.
        let inner = alpm_types::License::new(license.into())?;
        Ok(inner.into())
    }

    #[staticmethod]
    fn from_valid_spdx(identifier: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::License::from_valid_spdx(identifier.into())?;
        Ok(Self(inner))
    }

    #[getter]
    fn is_spdx(&self) -> bool {
        self.0.is_spdx()
    }

    fn __repr__(&self) -> String {
        format!("License('{}')", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(License, alpm_types::License);
