use alpm_srcinfo::source_info::v1::package as alpm_srcinfo_package;
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{
    helpers::impl_from,
    types::{license::License, path::RelativePath, url::Url},
};

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Package(alpm_srcinfo_package::Package);

impl_from!(Package, alpm_srcinfo_package::Package);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageArchitecture(alpm_srcinfo_package::PackageArchitecture);

#[pymethods]
impl PackageArchitecture {
    // TODO getters
}

impl_from!(
    PackageArchitecture,
    alpm_srcinfo_package::PackageArchitecture
);

#[derive(Clone, Debug, FromPyObject, IntoPyObject, PartialEq)]
pub enum Overridable {
    String(String),
    Url(Url),
    RelativePath(RelativePath),
    Licenses(Vec<License>),
}

#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq)]
pub struct Override(Option<Overridable>);

macro_rules! impl_tryfrom_override {
    ($type:ty, $matcher:pat_param => $body:expr) => {
        impl TryFrom<Override> for alpm_srcinfo_package::Override<$type> {
            type Error = PyErr;
            fn try_from(value: Override) -> Result<Self, Self::Error> {
                match value.0 {
                    Some($matcher) => Ok(alpm_srcinfo_package::Override::Yes {
                        value: $body.into(),
                    }),
                    None => Ok(alpm_srcinfo_package::Override::Clear),
                    _ => Err(PyValueError::new_err("TODO".to_string())),
                }
            }
        }
    };
}

impl_tryfrom_override!(String, Overridable::String(s) => s.as_str());
impl_tryfrom_override!(alpm_types::Url, Overridable::Url(u) => u);
impl_tryfrom_override!(alpm_types::RelativePath, Overridable::RelativePath(p) => p);
impl_tryfrom_override!(Vec<alpm_types::License>, Overridable::Licenses(l) => {
    l.into_iter().map(|lic| lic.into()).collect::<Vec<_>>()
});

#[pymodule(gil_used = false, name = "package", submodule)]
pub mod py_package {
    #[pymodule_export]
    use super::Package;
    #[pymodule_export]
    use super::PackageArchitecture;
}
