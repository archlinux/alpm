use std::str::FromStr;

use pyo3::prelude::*;

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionRequirement(alpm_types::VersionRequirement);

#[pymethods]
impl VersionRequirement {
    #[new]
    fn new(comparison: VersionComparison, version: crate::types::version::Version) -> Self {
        alpm_types::VersionRequirement::new(comparison.into(), version.into()).into()
    }

    #[staticmethod]
    fn from_str(req: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::VersionRequirement::from_str(req)?;
        Ok(inner.into())
    }

    pub fn is_satisfied_by(&self, ver: crate::types::version::Version) -> bool {
        self.0.is_satisfied_by(&ver.into())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "VersionRequirement(comparison={}, version={})",
            self.0.comparison, self.0.version
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(VersionRequirement, alpm_types::VersionRequirement);

#[pyclass(frozen, eq)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// Uses Python's enum variant naming convention.
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum VersionComparison {
    LESS_OR_EQUAL,
    GREATER_OR_EQUAL,
    EQUAL,
    LESS,
    GREATER,
}

#[pymethods]
impl VersionComparison {
    #[staticmethod]
    fn from_str(comparison: &str) -> PyResult<VersionComparison> {
        alpm_types::VersionComparison::from_str(comparison)
            .map(VersionComparison::from)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

impl From<alpm_types::VersionComparison> for VersionComparison {
    fn from(value: alpm_types::VersionComparison) -> Self {
        match value {
            alpm_types::VersionComparison::LessOrEqual => VersionComparison::LESS_OR_EQUAL,
            alpm_types::VersionComparison::GreaterOrEqual => VersionComparison::GREATER_OR_EQUAL,
            alpm_types::VersionComparison::Equal => VersionComparison::EQUAL,
            alpm_types::VersionComparison::Less => VersionComparison::LESS,
            alpm_types::VersionComparison::Greater => VersionComparison::GREATER,
        }
    }
}

impl From<VersionComparison> for alpm_types::VersionComparison {
    fn from(value: VersionComparison) -> Self {
        match value {
            VersionComparison::LESS_OR_EQUAL => alpm_types::VersionComparison::LessOrEqual,
            VersionComparison::GREATER_OR_EQUAL => alpm_types::VersionComparison::GreaterOrEqual,
            VersionComparison::EQUAL => alpm_types::VersionComparison::Equal,
            VersionComparison::LESS => alpm_types::VersionComparison::Less,
            VersionComparison::GREATER => alpm_types::VersionComparison::Greater,
        }
    }
}
