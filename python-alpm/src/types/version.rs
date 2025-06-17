use std::str::FromStr;

use deriving_via::DerivingVia;
use pyo3::{prelude::*, types::PyType};

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

#[derive(Debug, DerivingVia)]
#[deriving(From)]
pub struct SemverError(semver::Error);

impl From<SemverError> for PyErr {
    fn from(err: SemverError) -> PyErr {
        crate::types::ALPMError::new_err(err.to_string())
    }
}

#[pyclass(frozen, eq, ord)]
#[derive(Debug, DerivingVia, PartialEq, PartialOrd)]
pub struct SchemaVersion(alpm_types::SchemaVersion);

#[pymethods]
impl SchemaVersion {
    #[new]
    #[pyo3(signature = (major = 0, minor = 0, patch = 0, pre = "", build = ""))]
    fn new(
        major: u64,
        minor: u64,
        patch: u64,
        pre: &str,
        build: &str,
    ) -> Result<Self, SemverError> {
        let inner = semver::Version {
            major,
            minor,
            patch,
            pre: semver::Prerelease::new(pre)?,
            build: semver::BuildMetadata::new(build)?,
        };
        Ok(Self(alpm_types::SchemaVersion::new(inner)))
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, text: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::SchemaVersion::from_str(text);
        Ok(Self(inner?))
    }

    fn __repr__(&self) -> String {
        format!(
            "SchemaVersion(major={}, minor={}, patch={}, pre='{}', build='{}')",
            self.inner().major,
            self.inner().minor,
            self.inner().patch,
            self.inner().pre,
            self.inner().build,
        )
    }

    fn __str__(&self) -> String {
        self.inner().to_string()
    }

    #[getter]
    fn major(&self) -> u64 {
        self.inner().major
    }

    #[getter]
    fn minor(&self) -> u64 {
        self.inner().minor
    }

    #[getter]
    fn patch(&self) -> u64 {
        self.inner().patch
    }

    #[getter]
    fn pre(&self) -> String {
        self.inner().pre.to_string()
    }

    #[getter]
    fn build(&self) -> String {
        self.inner().build.to_string()
    }
}
