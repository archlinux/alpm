use std::str::FromStr;

use alpm_common::FileFormatSchema;
use pyo3::prelude::*;

use crate::{macros::impl_from, types::version::SchemaVersion};

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct SourceInfoSchema(alpm_srcinfo::SourceInfoSchema);

#[pymethods]
impl SourceInfoSchema {
    #[new]
    fn new(version: SchemaVersionOrStr) -> PyResult<SourceInfoSchema> {
        let schema_version: alpm_types::SchemaVersion = version.try_into()?;
        let inner = alpm_srcinfo::SourceInfoSchema::try_from(schema_version)
            .map_err(crate::srcinfo::error::Error::from)?;
        Ok(inner.into())
    }

    #[staticmethod]
    fn derive_from_str(srcinfo: &str) -> Result<Self, crate::srcinfo::error::Error> {
        let inner = alpm_srcinfo::SourceInfoSchema::derive_from_str(srcinfo)?;
        Ok(inner.into())
    }

    #[staticmethod]
    fn derive_from_file(path: std::path::PathBuf) -> Result<Self, crate::srcinfo::error::Error> {
        let inner = alpm_srcinfo::SourceInfoSchema::derive_from_file(&path)?;
        Ok(inner.into())
    }

    #[getter]
    fn version(&self) -> SchemaVersion {
        match self.0.clone() {
            alpm_srcinfo::SourceInfoSchema::V1(version) => version.into(),
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("SourceInfoSchema({})", self.0)
    }
}

impl_from!(SourceInfoSchema, alpm_srcinfo::SourceInfoSchema);

#[derive(Debug, FromPyObject, IntoPyObject)]
pub enum SchemaVersionOrStr {
    SchemaVersion(SchemaVersion),
    Str(String),
}

impl TryFrom<SchemaVersionOrStr> for alpm_types::SchemaVersion {
    type Error = crate::types::Error;

    fn try_from(value: SchemaVersionOrStr) -> Result<Self, Self::Error> {
        match value {
            SchemaVersionOrStr::SchemaVersion(v) => Ok(v.into()),
            SchemaVersionOrStr::Str(s) => {
                let v = alpm_types::SchemaVersion::from_str(&s)?;
                Ok(v)
            }
        }
    }
}

#[pymodule(gil_used = false, name = "schema", submodule)]
pub mod py_schema {
    #[pymodule_export]
    use super::SourceInfoSchema;
}
