use std::str::FromStr;

use pyo3::prelude::*;

use crate::macros::impl_from;

// Union type `BuildEnvironmentOption | PackageOption`
#[derive(Clone, Debug, FromPyObject, IntoPyObject, PartialEq)]
pub enum MakepkgOption {
    BuildEnvironment(BuildEnvironmentOption),
    Package(PackageOption),
}

impl From<alpm_types::MakepkgOption> for MakepkgOption {
    fn from(value: alpm_types::MakepkgOption) -> Self {
        match value {
            alpm_types::MakepkgOption::BuildEnvironment(opt) => {
                MakepkgOption::BuildEnvironment(BuildEnvironmentOption(opt))
            }
            alpm_types::MakepkgOption::Package(opt) => MakepkgOption::Package(PackageOption(opt)),
        }
    }
}

impl From<MakepkgOption> for alpm_types::MakepkgOption {
    fn from(value: MakepkgOption) -> Self {
        match value {
            MakepkgOption::BuildEnvironment(opt) => {
                alpm_types::MakepkgOption::BuildEnvironment(opt.0)
            }
            MakepkgOption::Package(opt) => alpm_types::MakepkgOption::Package(opt.0),
        }
    }
}

// Returns union type `BuildEnvironmentOption | PackageOption`.
// Equivalent to `alpm_types::MakepkgOption::new` in Rust.
#[pyfunction]
pub fn makepkg_option_from_str(option: &str) -> Result<MakepkgOption, crate::types::Error> {
    let inner_ident = alpm_types::MakepkgOption::from_str(option)?;
    Ok(inner_ident.into())
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildEnvironmentOption(alpm_types::BuildEnvironmentOption);

#[pymethods]
impl BuildEnvironmentOption {
    #[new]
    fn new(option: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::BuildEnvironmentOption::new(option)?;
        Ok(Self(inner))
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name().to_string()
    }

    #[getter]
    fn on(&self) -> bool {
        self.0.on()
    }

    fn __repr__(&self) -> String {
        format!(
            "BuildEnvironmentOption('{}', {})",
            self.0.name(),
            self.0.on()
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(BuildEnvironmentOption, alpm_types::BuildEnvironmentOption);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageOption(alpm_types::PackageOption);

#[pymethods]
impl PackageOption {
    #[new]
    fn new(option: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::PackageOption::new(option)?;
        Ok(Self(inner))
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name().to_string()
    }

    #[getter]
    fn on(&self) -> bool {
        self.0.on()
    }

    fn __repr__(&self) -> String {
        format!("PackageOption('{}', {})", self.0.name(), self.0.on())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(PackageOption, alpm_types::PackageOption);
