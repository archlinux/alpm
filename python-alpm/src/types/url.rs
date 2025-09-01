use std::str::FromStr;

use pyo3::prelude::*;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Url(alpm_types::Url);

#[pymethods]
impl Url {
    #[new]
    fn new(url: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Url::from_str(url)?;
        Ok(inner.into())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Url('{}')", self.0)
    }
}

impl From<alpm_types::Url> for Url {
    fn from(value: alpm_types::Url) -> Self {
        Url(value)
    }
}
