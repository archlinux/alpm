use std::str::FromStr;

use pyo3::{exceptions::PyTypeError, prelude::*};
use strum::Display;

use crate::macros::impl_from;

#[derive(Debug, FromPyObject, IntoPyObject)]
pub enum VersionOrSoname {
    Version(crate::types::version::PackageVersion),
    Soname(String),
}

impl From<alpm_types::VersionOrSoname> for VersionOrSoname {
    fn from(value: alpm_types::VersionOrSoname) -> Self {
        match value {
            alpm_types::VersionOrSoname::Version(v) => VersionOrSoname::Version(v.into()),
            alpm_types::VersionOrSoname::Soname(s) => VersionOrSoname::Soname(s.to_string()),
        }
    }
}

impl TryFrom<VersionOrSoname> for alpm_types::VersionOrSoname {
    type Error = crate::types::Error;

    fn try_from(value: VersionOrSoname) -> Result<Self, Self::Error> {
        Ok(match value {
            VersionOrSoname::Version(v) => alpm_types::VersionOrSoname::Version(v.into()),
            VersionOrSoname::Soname(s) => {
                alpm_types::VersionOrSoname::Soname(alpm_types::SharedObjectName::new(s.as_str())?)
            }
        })
    }
}

#[derive(Clone, Debug, FromPyObject, IntoPyObject, PartialEq)]
pub enum RelationOrSoname {
    Relation(PackageRelation),
    SonameV1(SonameV1),
    SonameV2(SonameV2),
}

impl From<alpm_types::RelationOrSoname> for RelationOrSoname {
    fn from(value: alpm_types::RelationOrSoname) -> Self {
        match value {
            alpm_types::RelationOrSoname::Relation(r) => RelationOrSoname::Relation(r.into()),
            alpm_types::RelationOrSoname::SonameV1(s) => RelationOrSoname::SonameV1(s.into()),
            alpm_types::RelationOrSoname::SonameV2(s) => RelationOrSoname::SonameV2(s.into()),
        }
    }
}

impl From<RelationOrSoname> for alpm_types::RelationOrSoname {
    fn from(value: RelationOrSoname) -> Self {
        match value {
            RelationOrSoname::Relation(r) => alpm_types::RelationOrSoname::Relation(r.0),
            RelationOrSoname::SonameV1(s) => alpm_types::RelationOrSoname::SonameV1(s.0),
            RelationOrSoname::SonameV2(s) => alpm_types::RelationOrSoname::SonameV2(s.0),
        }
    }
}

impl From<PackageRelation> for RelationOrSoname {
    fn from(value: PackageRelation) -> Self {
        RelationOrSoname::Relation(value)
    }
}

impl TryFrom<RelationOrSoname> for PackageRelation {
    type Error = PyErr;

    fn try_from(value: RelationOrSoname) -> Result<Self, Self::Error> {
        match value {
            RelationOrSoname::Relation(r) => Ok(r),
            RelationOrSoname::SonameV1(_) | RelationOrSoname::SonameV2(_) => Err(
                PyTypeError::new_err("expected PackageRelation, found Soname"),
            ),
        }
    }
}

#[pyfunction]
pub fn relation_or_soname_from_str(s: &str) -> Result<RelationOrSoname, crate::types::Error> {
    let inner = alpm_types::RelationOrSoname::from_str(s)?;
    Ok(inner.into())
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Soname(alpm_types::Soname);

#[pymethods]
impl Soname {
    #[new]
    #[pyo3(signature = (name, version = None))]
    fn new(
        name: &str,
        version: Option<crate::types::version::PackageVersion>,
    ) -> Result<Self, crate::types::Error> {
        let name = alpm_types::SharedObjectName::new(name)?;
        let inner = alpm_types::Soname::new(name, version.map(From::from));
        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn version(&self) -> Option<crate::types::version::PackageVersion> {
        self.0.version.clone().map(From::from)
    }

    fn __repr__(&self) -> String {
        match self.version() {
            Some(version) => format!(
                "Soname(name='{}', version={})",
                self.name(),
                version.__repr__()
            ),
            None => format!("Soname(name='{}')", self.name()),
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(Soname, alpm_types::Soname);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SonameV2(alpm_types::SonameV2);

#[pymethods]
impl SonameV2 {
    #[new]
    fn new(prefix: &str, soname: Soname) -> Result<Self, crate::types::Error> {
        let prefix = alpm_types::Name::new(prefix)?;
        let inner = alpm_types::SonameV2::new(prefix, soname.into());
        Ok(inner.into())
    }

    #[getter]
    fn prefix(&self) -> String {
        self.0.prefix.to_string()
    }

    #[getter]
    fn soname(&self) -> Soname {
        self.0.soname.clone().into()
    }

    fn __repr__(&self) -> String {
        format!(
            "SonameV2(prefix='{}', soname={})",
            self.prefix(),
            self.soname().__repr__()
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(SonameV2, alpm_types::SonameV2);

#[pyclass(frozen)]
#[derive(Clone, Copy, Debug, Display, PartialEq)]
// Uses Python's enum variant naming convention.
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum SonameV1Type {
    BASIC,
    UNVERSIONED,
    EXPLICIT,
}

impl From<&alpm_types::SonameV1> for SonameV1Type {
    fn from(value: &alpm_types::SonameV1) -> Self {
        use alpm_types::SonameV1::{Basic, Explicit, Unversioned};
        match value {
            Basic(_) => SonameV1Type::BASIC,
            Unversioned { .. } => SonameV1Type::UNVERSIONED,
            Explicit { .. } => SonameV1Type::EXPLICIT,
        }
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct SonameV1(alpm_types::SonameV1);

#[pymethods]
impl SonameV1 {
    #[new]
    #[pyo3(signature = (name, version_or_soname = None, architecture = None))]
    fn new(
        name: &str,
        version_or_soname: Option<VersionOrSoname>,
        architecture: Option<crate::types::system::ElfArchitectureFormat>,
    ) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::SonameV1::new(
            alpm_types::SharedObjectName::new(name)?,
            version_or_soname.map(TryFrom::try_from).transpose()?,
            architecture.map(From::from),
        )?;
        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.shared_object_name().to_string()
    }

    #[getter]
    fn soname(&self) -> Option<String> {
        use alpm_types::SonameV1::{Basic, Explicit, Unversioned};
        match &self.0 {
            Basic(_) | Explicit { .. } => None,
            Unversioned { soname, .. } => Some(soname.to_string()),
        }
    }

    #[getter]
    fn version(&self) -> Option<crate::types::version::PackageVersion> {
        use alpm_types::SonameV1::{Basic, Explicit, Unversioned};
        match &self.0 {
            Basic(_) | Unversioned { .. } => None,
            Explicit { version, .. } => Some(version.clone().into()),
        }
    }

    #[getter]
    fn architecture(&self) -> Option<crate::types::system::ElfArchitectureFormat> {
        use alpm_types::SonameV1::{Basic, Explicit, Unversioned};
        match self.0 {
            Basic(_) | Unversioned { .. } => None,
            Explicit { architecture, .. } => Some(architecture.into()),
        }
    }

    #[getter]
    fn form(&self) -> SonameV1Type {
        (&self.0).into()
    }

    fn __repr__(&self) -> String {
        let mut optional_properties = String::new();

        if let Some(soname) = self.soname() {
            optional_properties.push_str(&format!(", soname='{}'", soname));
        }

        if let Some(version) = self.version() {
            optional_properties.push_str(&format!(", version={})", version.__repr__()));
        }

        if let Some(arch) = self.architecture() {
            optional_properties.push_str(&format!(", architecture={})", arch));
        }

        format!(
            "SonameV1(form={}, name={}{})",
            self.form(),
            self.name(),
            optional_properties
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(SonameV1, alpm_types::SonameV1);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRelation(alpm_types::PackageRelation);

#[pymethods]
impl PackageRelation {
    #[new]
    #[pyo3(signature = (name, version_requirement = None))]
    fn new(
        name: &str,
        version_requirement: Option<crate::types::requirement::VersionRequirement>,
    ) -> Result<Self, crate::types::Error> {
        let name = alpm_types::Name::new(name)?;
        let inner = alpm_types::PackageRelation::new(name, version_requirement.map(From::from));
        Ok(inner.into())
    }

    #[staticmethod]
    fn from_str(s: &str) -> Result<PackageRelation, crate::types::Error> {
        let inner = alpm_types::PackageRelation::from_str(s)?;
        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn version_requirement(&self) -> Option<crate::types::requirement::VersionRequirement> {
        self.0.version_requirement.clone().map(From::from)
    }

    fn __repr__(&self) -> String {
        match self.version_requirement() {
            Some(vr) => format!(
                "PackageRelation(name='{}', version_requirement={})",
                self.name(),
                vr.__repr__()
            ),
            None => format!("PackageRelation(name='{}')", self.name()),
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(PackageRelation, alpm_types::PackageRelation);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionalDependency(alpm_types::OptionalDependency);

#[pymethods]
impl OptionalDependency {
    #[new]
    #[pyo3(signature = (package_relation, description = None))]
    fn new(package_relation: PackageRelation, description: Option<String>) -> Self {
        alpm_types::OptionalDependency::new(package_relation.into(), description).into()
    }

    #[staticmethod]
    fn from_str(s: &str) -> Result<OptionalDependency, crate::types::Error> {
        let inner = alpm_types::OptionalDependency::from_str(s)?;
        Ok(inner.into())
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name().to_string()
    }

    #[getter]
    fn version_requirement(&self) -> Option<crate::types::requirement::VersionRequirement> {
        self.0.version_requirement().clone().map(From::from)
    }

    #[getter]
    fn description(&self) -> Option<String> {
        self.0.description().as_deref().map(ToString::to_string)
    }

    fn __repr__(&self) -> String {
        let mut optional_properties = String::new();

        if let Some(vr) = self.version_requirement() {
            optional_properties.push_str(&format!(", version_requirement={}", vr.__repr__()));
        }

        if let Some(desc) = self.description() {
            optional_properties.push_str(&format!(", description='{}'", desc));
        }

        format!(
            "OptionalDependency(name={}{})",
            self.name(),
            optional_properties
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(OptionalDependency, alpm_types::OptionalDependency);
