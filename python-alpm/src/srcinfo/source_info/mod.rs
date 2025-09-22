use alpm_common::MetadataFile;
use pyo3::prelude::*;

use crate::srcinfo::schema::SourceInfoSchema;

pub mod v1;

#[derive(FromPyObject, IntoPyObject)]
pub enum SourceInfo {
    V1(v1::SourceInfoV1),
}

impl From<alpm_srcinfo::SourceInfo> for SourceInfo {
    fn from(v: alpm_srcinfo::SourceInfo) -> Self {
        match v {
            alpm_srcinfo::SourceInfo::V1(v) => SourceInfo::V1(v.into()),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (s, schema = None))]
pub fn source_info_from_str(
    s: &str,
    schema: Option<SourceInfoSchema>,
) -> Result<SourceInfo, crate::srcinfo::error::Error> {
    let schema: Option<alpm_srcinfo::SourceInfoSchema> = schema.map(From::from);
    let inner = alpm_srcinfo::SourceInfo::from_str_with_schema(s, schema)?;
    Ok(inner.into())
}

#[pyfunction]
#[pyo3(signature = (path, schema = None))]
pub fn source_info_from_file(
    path: std::path::PathBuf,
    schema: Option<SourceInfoSchema>,
) -> Result<SourceInfo, crate::srcinfo::error::Error> {
    let schema: Option<alpm_srcinfo::SourceInfoSchema> = schema.map(From::from);
    let inner = alpm_srcinfo::SourceInfo::from_file_with_schema(&path, schema)?;
    Ok(inner.into())
}

#[pymodule(gil_used = false, name = "source_info", submodule)]
pub mod py_source_info {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::source_info_from_file;
    #[pymodule_export]
    use super::source_info_from_str;
    #[pymodule_export]
    use super::v1::py_v1;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let modules = PyModule::import(m.py(), "sys")?.getattr("modules")?;
        modules.set_item("alpm.alpm_srcinfo.source_info.v1", m.getattr("v1")?)?;
        Ok(())
    }
}
