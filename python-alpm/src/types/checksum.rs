use std::str::FromStr;

use pyo3::prelude::*;

use crate::macros::impl_from;

#[allow(dead_code)]
#[derive(FromPyObject, IntoPyObject)]
pub enum Checksum {
    Blake2b512(Blake2b512Checksum),
    Md5(Md5Checksum),
    Sha1(Sha1Checksum),
    Sha224(Sha224Checksum),
    Sha256(Sha256Checksum),
    Sha384(Sha384Checksum),
    Sha512(Sha512Checksum),
}

macro_rules! define_checksum {
    ($name:ident, $type:ty) => {
        #[pyclass(frozen, eq, ord)]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name($type);

        #[pymethods]
        impl $name {
            #[new]
            fn new(value: &str) -> Result<Self, crate::types::Error> {
                let inner = <$type>::from_str(value)?;
                Ok(Self(inner))
            }

            fn __str__(&self) -> String {
                self.0.to_string()
            }

            fn __repr__(&self) -> String {
                format!("{}({})", stringify!($name), self.0)
            }
        }

        impl_from!($name, $type);
    };
}

define_checksum!(Blake2b512Checksum, alpm_types::Blake2b512Checksum);
define_checksum!(Md5Checksum, alpm_types::Md5Checksum);
define_checksum!(Sha1Checksum, alpm_types::Sha1Checksum);
define_checksum!(Sha224Checksum, alpm_types::Sha224Checksum);
define_checksum!(Sha256Checksum, alpm_types::Sha256Checksum);
define_checksum!(Sha384Checksum, alpm_types::Sha384Checksum);
define_checksum!(Sha512Checksum, alpm_types::Sha512Checksum);
