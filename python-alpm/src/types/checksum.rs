use std::str::FromStr;

use pyo3::prelude::*;

use crate::macros::impl_from;

macro_rules! define_checksum {
    ($checksum:ident, $skippable_checksum: ident, $digest:ty) => {
        #[pyclass(frozen, eq, ord)]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $checksum(alpm_types::Checksum<$digest>);

        #[pymethods]
        impl $checksum {
            #[new]
            fn new(value: &str) -> Result<Self, crate::types::Error> {
                let inner = <alpm_types::Checksum<$digest>>::from_str(value)?;
                Ok(inner.into())
            }

            fn __str__(&self) -> String {
                self.0.to_string()
            }

            fn __repr__(&self) -> String {
                format!("{}({})", stringify!($checksum), self.0)
            }
        }

        impl_from!($checksum, alpm_types::Checksum<$digest>);

        #[pyclass(frozen)]
        #[derive(Clone, Debug)]
        pub struct $skippable_checksum(alpm_types::SkippableChecksum<$digest>);

        #[pymethods]
        impl $skippable_checksum {
            #[new]
            #[pyo3(signature = (value=None))]
            fn new(value: Option<&str>) -> Result<Self, crate::types::Error> {
                let inner: alpm_types::SkippableChecksum<$digest> = match value {
                    Some(v) => alpm_types::SkippableChecksum::from_str(v)?,
                    None => alpm_types::SkippableChecksum::Skip,
                };
                Ok(inner.into())
            }

            #[getter]
            fn is_skipped(&self) -> bool {
                self.0.is_skipped()
            }

            fn __str__(&self) -> String {
                self.0.to_string()
            }

            fn __repr__(&self) -> String {
                format!("{}({})", stringify!($skippable_checksum), self.0)
            }
        }

        impl_from!($skippable_checksum, alpm_types::SkippableChecksum<$digest>);
    };
}

define_checksum!(
    Blake2b512Checksum,
    SkippableBlake2b512Checksum,
    alpm_types::digests::Blake2b512
);
define_checksum!(Md5Checksum, SkippableMd5Checksum, alpm_types::digests::Md5);
define_checksum!(
    Sha1Checksum,
    SkippableSha1Checksum,
    alpm_types::digests::Sha1
);
define_checksum!(
    Sha224Checksum,
    SkippableSha224Checksum,
    alpm_types::digests::Sha224
);
define_checksum!(
    Sha256Checksum,
    SkippableSha256Checksum,
    alpm_types::digests::Sha256
);
define_checksum!(
    Sha384Checksum,
    SkippableSha384Checksum,
    alpm_types::digests::Sha384
);
define_checksum!(
    Sha512Checksum,
    SkippableSha512Checksum,
    alpm_types::digests::Sha512
);
define_checksum!(
    Crc32CksumChecksum,
    SkippableCrc32CksumChecksum,
    alpm_types::digests::Crc32Cksum
);
