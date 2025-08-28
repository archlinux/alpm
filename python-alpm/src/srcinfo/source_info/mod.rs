use pyo3::prelude::*;

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

#[pymodule(gil_used = false, name = "source_info", submodule)]
pub mod py_source_info {
    #[pymodule_export]
    use super::v1::py_v1;
}
