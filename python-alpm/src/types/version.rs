use std::{num::NonZeroUsize, str::FromStr};

use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

#[pyclass(frozen, eq, ord)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct PackageVersion(alpm_types::PackageVersion);

#[pymethods]
impl PackageVersion {
    #[new]
    fn new(pkgver: &str) -> Result<Self, crate::types::Error> {
        Ok(alpm_types::PackageVersion::new(pkgver.to_string())?.into())
    }

    pub fn __repr__(&self) -> String {
        format!("PackageVersion('{}')", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl From<alpm_types::PackageVersion> for PackageVersion {
    fn from(inner: alpm_types::PackageVersion) -> Self {
        PackageVersion(inner)
    }
}

impl From<PackageVersion> for alpm_types::PackageVersion {
    fn from(value: PackageVersion) -> Self {
        value.0
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
        Ok(alpm_types::Epoch::new(non_zero).into())
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, epoch: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Epoch::from_str(epoch)?;
        Ok(inner.into())
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

impl From<alpm_types::Epoch> for Epoch {
    fn from(inner: alpm_types::Epoch) -> Self {
        Epoch(inner)
    }
}

impl From<Epoch> for alpm_types::Epoch {
    fn from(outer: Epoch) -> Self {
        outer.0
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
        alpm_types::PackageRelease::new(major, minor).into()
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, version: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::PackageRelease::from_str(version)?;
        Ok(inner.into())
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

impl From<alpm_types::PackageRelease> for PackageRelease {
    fn from(inner: alpm_types::PackageRelease) -> Self {
        PackageRelease(inner)
    }
}

impl From<PackageRelease> for alpm_types::PackageRelease {
    fn from(outer: PackageRelease) -> Self {
        outer.0
    }
}

#[pyclass(frozen, eq, ord)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FullVersion(alpm_types::FullVersion);

#[pymethods]
impl FullVersion {
    #[new]
    #[pyo3(signature = (pkgver, pkgrel, epoch = None))]
    fn new(pkgver: PackageVersion, pkgrel: PackageRelease, epoch: Option<Epoch>) -> Self {
        alpm_types::FullVersion::new(pkgver.0, pkgrel.0, epoch.map(From::from)).into()
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, version: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::FullVersion::from_str(version)?;
        Ok(inner.into())
    }

    #[getter]
    fn pkgver(&self) -> PackageVersion {
        self.0.pkgver.clone().into()
    }

    #[getter]
    fn pkgrel(&self) -> PackageRelease {
        self.0.pkgrel.clone().into()
    }

    #[getter]
    fn epoch(&self) -> Option<Epoch> {
        self.0.epoch.map(From::from)
    }

    pub fn vercmp(&self, other: FullVersion) -> i8 {
        self.0.vercmp(&other.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        match self.0.epoch {
            None => format!(
                "FullVersion(pkgver={}, pkgrel={})",
                self.0.pkgver, self.0.pkgrel
            ),
            Some(epoch) => format!(
                "FullVersion(pkgver={}, pkgrel={}, epoch={})",
                self.0.pkgver, self.0.pkgrel, epoch
            ),
        }
    }
}

impl From<alpm_types::FullVersion> for FullVersion {
    fn from(inner: alpm_types::FullVersion) -> Self {
        FullVersion(inner)
    }
}

#[pyclass(frozen, eq, ord)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(alpm_types::Version);

#[pymethods]
impl Version {
    #[new]
    #[pyo3(signature = (pkgver, pkgrel = None, epoch = None))]
    fn new(pkgver: PackageVersion, pkgrel: Option<PackageRelease>, epoch: Option<Epoch>) -> Self {
        alpm_types::Version::new(pkgver.0, epoch.map(From::from), pkgrel.map(From::from)).into()
    }

    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, version: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Version::from_str(version)?;
        Ok(inner.into())
    }

    #[getter]
    fn pkgver(&self) -> PackageVersion {
        self.0.pkgver.clone().into()
    }

    #[getter]
    fn pkgrel(&self) -> Option<PackageRelease> {
        self.0.pkgrel.clone().map(From::from)
    }

    #[getter]
    fn epoch(&self) -> Option<Epoch> {
        self.0.epoch.map(From::from)
    }

    pub fn vercmp(&self, other: Version) -> i8 {
        alpm_types::Version::vercmp(&self.0, &other.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        let pkgrel_str = match self.0.pkgrel {
            Some(ref pkgrel) => format!(", pkgrel={}", pkgrel),
            None => "".to_string(),
        };
        let epoch_str = match self.0.epoch {
            Some(ref epoch) => format!(", epoch={}", epoch),
            None => "".to_string(),
        };
        format!(
            "Version(pkgver={}{}{})",
            self.0.pkgver, pkgrel_str, epoch_str
        )
    }
}

impl From<alpm_types::Version> for Version {
    fn from(inner: alpm_types::Version) -> Self {
        Version(inner)
    }
}

impl From<Version> for alpm_types::Version {
    fn from(outer: Version) -> Self {
        outer.0
    }
}
