use std::{num::NonZeroUsize, str::FromStr};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

#[pyclass(frozen, eq, ord)]
#[derive(Debug, PartialEq, PartialOrd)]
pub struct PackageVersion(alpm_types::PackageVersion);

#[pymethods]
impl PackageVersion {
    #[new]
    fn new(pkgver: &str) -> Result<Self, crate::types::Error> {
        Ok(Self(alpm_types::PackageVersion::new(pkgver.to_string())?))
    }

    fn __repr__(&self) -> String {
        format!("PackageVersion('{}')", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct SemverError(semver::Error);

impl From<semver::Error> for SemverError {
    fn from(err: semver::Error) -> Self {
        SemverError(err)
    }
}

impl From<SemverError> for PyErr {
    fn from(err: SemverError) -> PyErr {
        crate::types::ALPMError::new_err(err.0.to_string())
    }
}

#[pyclass(frozen, eq, ord)]
#[derive(Debug, PartialEq, PartialOrd)]
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
    fn from_str(_cls: &Bound<'_, PyType>, version: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::SchemaVersion::from_str(version);
        Ok(Self(inner?))
    }

    fn __repr__(&self) -> String {
        format!(
            "SchemaVersion(major={}, minor={}, patch={}, pre='{}', build='{}')",
            self.0.inner().major,
            self.0.inner().minor,
            self.0.inner().patch,
            self.0.inner().pre,
            self.0.inner().build,
        )
    }

    fn __str__(&self) -> String {
        self.0.inner().to_string()
    }

    #[getter]
    fn major(&self) -> u64 {
        self.0.inner().major
    }

    #[getter]
    fn minor(&self) -> u64 {
        self.0.inner().minor
    }

    #[getter]
    fn patch(&self) -> u64 {
        self.0.inner().patch
    }

    #[getter]
    fn pre(&self) -> String {
        self.0.inner().pre.to_string()
    }

    #[getter]
    fn build(&self) -> String {
        self.0.inner().build.to_string()
    }
}

#[pyclass(frozen, eq, ord)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Epoch(alpm_types::Epoch);

#[pymethods]
impl Epoch {
    #[new]
    fn new(value: usize) -> PyResult<Self> {
        let non_zero = NonZeroUsize::new(value)
            // Since this is `Optional` in Rust, we raise `ValueError` in case of `None`,
            // as this is more idiomatic Python.
            .ok_or_else(|| PyValueError::new_err("Epoch must be a non-zero positive integer"))?;
        Ok(Self(alpm_types::Epoch::new(non_zero)))
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, epoch: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Epoch::from_str(epoch);
        Ok(Self(inner?))
    }

    /// Epoch value as a positive integer.
    #[getter]
    fn value(&self) -> usize {
        self.0.0.get()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Epoch({})", self.0.0.get())
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageRelease(alpm_types::PackageRelease);

#[pymethods]
impl PackageRelease {
    #[new]
    #[pyo3(signature = (major = 0, minor = None))]
    fn new(major: usize, minor: Option<usize>) -> Self {
        PackageRelease(alpm_types::PackageRelease::new(major, minor))
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, version: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::PackageRelease::from_str(version);
        Ok(PackageRelease(inner?))
    }

    #[getter]
    fn major(&self) -> usize {
        self.0.major
    }

    #[getter]
    fn minor(&self) -> Option<usize> {
        self.0.minor
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        match self.0.minor {
            Some(minor) => format!("PackageRelease(major={}, minor={})", self.0.major, minor),
            None => format!("PackageRelease(major={})", self.0.major),
        }
    }
}
