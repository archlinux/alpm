use std::{path::PathBuf, str::FromStr};

use pyo3::prelude::*;

use crate::{macros::impl_from, types::url::SourceUrl};

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source(alpm_types::Source);

#[pymethods]
impl Source {
    #[new]
    fn new(source: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::Source::from_str(source)?;
        Ok(inner.into())
    }

    #[getter]
    fn filename(&self) -> Option<&PathBuf> {
        self.0.filename()
    }

    #[getter]
    fn source_url(&self) -> Option<SourceUrl> {
        match &self.0 {
            alpm_types::Source::SourceUrl { source_url, .. } => Some(source_url.clone().into()),
            alpm_types::Source::File { .. } => None,
        }
    }

    #[getter]
    fn location(&self) -> Option<&PathBuf> {
        match &self.0 {
            alpm_types::Source::File { location, .. } => Some(location),
            alpm_types::Source::SourceUrl { .. } => None,
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        let mut optional_properties = String::default();

        if let Some(source_url) = self.source_url() {
            optional_properties.push_str(&format!("source_url='{}'", source_url.__str__()));
        }

        if let Some(location) = self.location() {
            optional_properties.push_str(&format!("location='{}'", location.display()));
        }

        if let Some(filename) = self.filename() {
            optional_properties.push_str(&format!(", filename='{}'", filename.display()));
        }

        format!("Source({})", optional_properties)
    }
}

impl_from!(Source, alpm_types::Source);
