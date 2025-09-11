use std::str::FromStr;

use pyo3::prelude::*;

use crate::macros::impl_from;

// Union type `OpenPGPKeyId | OpenPGPv4Fingerprint`
#[derive(Debug, FromPyObject, IntoPyObject)]
pub enum OpenPGPIdentifier {
    OpenPGPKeyId(OpenPGPKeyId),
    OpenPGPv4Fingerprint(OpenPGPv4Fingerprint),
}

impl From<alpm_types::OpenPGPIdentifier> for OpenPGPIdentifier {
    fn from(ident: alpm_types::OpenPGPIdentifier) -> Self {
        match ident {
            alpm_types::OpenPGPIdentifier::OpenPGPKeyId(key_id) => {
                OpenPGPIdentifier::OpenPGPKeyId(OpenPGPKeyId(key_id))
            }
            alpm_types::OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint) => {
                OpenPGPIdentifier::OpenPGPv4Fingerprint(OpenPGPv4Fingerprint(fingerprint))
            }
        }
    }
}

impl From<OpenPGPIdentifier> for alpm_types::OpenPGPIdentifier {
    fn from(value: OpenPGPIdentifier) -> Self {
        match value {
            OpenPGPIdentifier::OpenPGPKeyId(key_id) => {
                alpm_types::OpenPGPIdentifier::OpenPGPKeyId(key_id.0)
            }
            OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint) => {
                alpm_types::OpenPGPIdentifier::OpenPGPv4Fingerprint(fingerprint.0)
            }
        }
    }
}

// Returns union type `OpenPGPKeyId | OpenPGPv4Fingerprint`.
// Equivalent to `alpm_types::OpenPGPIdentifier::from_str` in Rust.
#[pyfunction]
pub fn openpgp_identifier_from_str(
    identifier: &str,
) -> Result<OpenPGPIdentifier, crate::types::Error> {
    let inner_ident = alpm_types::OpenPGPIdentifier::from_str(identifier)?;
    Ok(inner_ident.into())
}

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenPGPKeyId(alpm_types::OpenPGPKeyId);

#[pymethods]
impl OpenPGPKeyId {
    #[new]
    fn new(key_id: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::OpenPGPKeyId::new(key_id.into())?;
        Ok(Self(inner))
    }

    fn __repr__(&self) -> String {
        format!("OpenPGPKeyId('{}')", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(OpenPGPKeyId, alpm_types::OpenPGPKeyId);

#[pyclass(frozen, eq)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenPGPv4Fingerprint(alpm_types::OpenPGPv4Fingerprint);

#[pymethods]
impl OpenPGPv4Fingerprint {
    #[new]
    fn new(fingerprint: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_types::OpenPGPv4Fingerprint::new(fingerprint.into())?;
        Ok(Self(inner))
    }

    fn __repr__(&self) -> String {
        format!("OpenPGPv4Fingerprint('{}')", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl_from!(OpenPGPv4Fingerprint, alpm_types::OpenPGPv4Fingerprint);
