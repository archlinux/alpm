use std::str::FromStr;

use deriving_via::DerivingVia;
use pyo3::prelude::*;

#[pyclass(frozen, eq, ord)]
#[derive(Debug, DerivingVia, PartialEq, PartialOrd)]
pub struct PackageVersion(alpm_types::PackageVersion);

#[pymethods]
impl PackageVersion {
    #[new]
    fn new(pkgver: String) -> Result<Self, crate::types::Error> {
        Ok(Self(alpm_types::PackageVersion::new(pkgver)?))
    }

    fn __repr__(&self) -> String {
        format!("PackageVersion('{}')", **self)
    }

    fn __str__(&self) -> String {
        self.inner().to_string()
    }
}
