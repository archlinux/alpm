use std::str::FromStr;

use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict},
};

use crate::macros::impl_from;

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Url(alpm_types::Url);

#[pymethods]
impl Url {
    #[new]
    fn new(url: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Url::from_str(url)?;
        Ok(inner.into())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Url('{}')", self.0)
    }
}

impl_from!(Url, alpm_types::Url);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceUrl(alpm_types::SourceUrl);

#[pymethods]
impl SourceUrl {
    #[new]
    fn new(source_url: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::SourceUrl::from_str(source_url)?;
        Ok(inner.into())
    }

    #[getter]
    fn url(&self) -> Url {
        self.0.url.clone().into()
    }

    #[getter]
    fn vcs_info(&self) -> Option<VcsInfo> {
        self.0.vcs_info.clone().map(From::from)
    }

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        match self.0.vcs_info.clone() {
            // TODO
            Some(vcs_info) => format!("SourceUrl(url='{}', vcs_info='{:?}')", self.0.url, vcs_info),
            None => format!("SourceUrl(url='{}')", self.0.url),
        }
    }
}

impl_from!(SourceUrl, alpm_types::SourceUrl);

// Python union type
#[derive(Debug, FromPyObject, IntoPyObject)]
pub enum VcsInfo {
    Bzr(BzrInfo),
    Fossil(FossilInfo),
    Git(GitInfo),
    Hg(HgInfo),
    Svn(SvnInfo),
}

impl From<VcsInfo> for alpm_types::url::VcsInfo {
    fn from(vcs_info: VcsInfo) -> Self {
        match vcs_info {
            VcsInfo::Bzr(bzr_info) => alpm_types::url::VcsInfo::Bzr {
                fragment: bzr_info.into(),
            },
            VcsInfo::Fossil(fossil_info) => alpm_types::url::VcsInfo::Fossil {
                fragment: fossil_info.into(),
            },
            VcsInfo::Git(git_info) => git_info.into(),
            VcsInfo::Hg(hg_info) => alpm_types::url::VcsInfo::Hg {
                fragment: hg_info.into(),
            },
            VcsInfo::Svn(svn_info) => alpm_types::url::VcsInfo::Svn {
                fragment: svn_info.into(),
            },
        }
    }
}

impl From<alpm_types::url::VcsInfo> for VcsInfo {
    fn from(value: alpm_types::url::VcsInfo) -> Self {
        match value {
            alpm_types::url::VcsInfo::Bzr { fragment } => VcsInfo::Bzr(fragment.into()),
            alpm_types::url::VcsInfo::Fossil { fragment } => VcsInfo::Fossil(fragment.into()),
            alpm_types::url::VcsInfo::Git { fragment, signed } => {
                VcsInfo::Git((fragment, signed).into())
            }
            alpm_types::url::VcsInfo::Hg { fragment } => VcsInfo::Hg(fragment.into()),
            alpm_types::url::VcsInfo::Svn { fragment } => VcsInfo::Svn(fragment.into()),
        }
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BzrInfo(Option<alpm_types::url::BzrFragment>);

impl_from!(BzrInfo, Option<alpm_types::url::BzrFragment>);

#[pymethods]
impl BzrInfo {
    #[getter]
    fn fragment<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        self.clone().into_py_dict(py)
    }
}

impl IntoPyDict<'_> for BzrInfo {
    fn into_py_dict(self, py: Python) -> PyResult<Bound<PyDict>> {
        let dict = PyDict::new(py);
        if let Some(fragment) = self.0 {
            match fragment {
                alpm_types::url::BzrFragment::Revision(revision) => {
                    dict.set_item("revision", revision)?;
                }
            }
        }
        Ok(dict)
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FossilInfo(Option<alpm_types::url::FossilFragment>);

impl_from!(FossilInfo, Option<alpm_types::url::FossilFragment>);

#[pymethods]
impl FossilInfo {
    #[getter]
    fn fragment<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        self.clone().into_py_dict(py)
    }
}

impl IntoPyDict<'_> for FossilInfo {
    fn into_py_dict(self, py: Python) -> PyResult<Bound<PyDict>> {
        let dict = PyDict::new(py);
        if let Some(fragment) = self.0 {
            match fragment {
                alpm_types::url::FossilFragment::Branch(branch) => {
                    dict.set_item("branch", branch)?;
                }
                alpm_types::url::FossilFragment::Commit(commit) => {
                    dict.set_item("commit", commit)?;
                }
                alpm_types::url::FossilFragment::Tag(tag) => {
                    dict.set_item("tag", tag)?;
                }
            }
        }
        Ok(dict)
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitInfo {
    fragment: Option<alpm_types::url::GitFragment>,
    signed: bool,
}

impl From<(Option<alpm_types::url::GitFragment>, bool)> for GitInfo {
    fn from(value: (Option<alpm_types::url::GitFragment>, bool)) -> Self {
        Self {
            fragment: value.0,
            signed: value.1,
        }
    }
}

impl From<GitInfo> for alpm_types::url::VcsInfo {
    fn from(git_info: GitInfo) -> Self {
        alpm_types::url::VcsInfo::Git {
            fragment: git_info.fragment,
            signed: git_info.signed,
        }
    }
}

#[pymethods]
impl GitInfo {
    #[getter]
    fn fragment<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        self.clone().into_py_dict(py)
    }

    #[getter]
    fn signed(&self) -> bool {
        self.signed
    }
}

impl IntoPyDict<'_> for GitInfo {
    fn into_py_dict(self, py: Python) -> PyResult<Bound<PyDict>> {
        let dict = PyDict::new(py);
        if let Some(fragment) = self.fragment {
            match fragment {
                alpm_types::url::GitFragment::Branch(branch) => {
                    dict.set_item("branch", branch)?;
                }
                alpm_types::url::GitFragment::Commit(commit) => {
                    dict.set_item("commit", commit)?;
                }
                alpm_types::url::GitFragment::Tag(tag) => {
                    dict.set_item("tag", tag)?;
                }
            }
        }
        Ok(dict)
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HgInfo(Option<alpm_types::url::HgFragment>);

impl_from!(HgInfo, Option<alpm_types::url::HgFragment>);

#[pymethods]
impl HgInfo {
    #[getter]
    fn fragment<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        self.clone().into_py_dict(py)
    }
}

impl IntoPyDict<'_> for HgInfo {
    fn into_py_dict(self, py: Python) -> PyResult<Bound<PyDict>> {
        let dict = PyDict::new(py);
        if let Some(fragment) = self.0 {
            match fragment {
                alpm_types::url::HgFragment::Branch(branch) => {
                    dict.set_item("branch", branch)?;
                }
                alpm_types::url::HgFragment::Revision(commit) => {
                    dict.set_item("revision", commit)?;
                }
                alpm_types::url::HgFragment::Tag(tag) => {
                    dict.set_item("tag", tag)?;
                }
            }
        }
        Ok(dict)
    }
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvnInfo(Option<alpm_types::url::SvnFragment>);

impl_from!(SvnInfo, Option<alpm_types::url::SvnFragment>);

#[pymethods]
impl SvnInfo {
    #[getter]
    fn fragment<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        self.clone().into_py_dict(py)
    }
}

impl IntoPyDict<'_> for SvnInfo {
    fn into_py_dict(self, py: Python) -> PyResult<Bound<PyDict>> {
        let dict = PyDict::new(py);
        if let Some(fragment) = self.0 {
            match fragment {
                alpm_types::url::SvnFragment::Revision(revision) => {
                    dict.set_item("revision", revision)?;
                }
            }
        }
        Ok(dict)
    }
}
