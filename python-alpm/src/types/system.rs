use std::str::FromStr;

use pyo3::{exceptions::PyStopIteration, prelude::*};
use strum::Display;

use crate::macros::{impl_from, vec_convert};

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
// Uses Python's enum variant naming convention.
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum KnownArchitecture {
    AARCH64,
    ARM,
    ARMV6H,
    ARMV7H,
    I386,
    I486,
    I686,
    PENTIUM4,
    RISCV32,
    RISCV64,
    X86_64,
    X86_64_V2,
    X86_64_V3,
    X86_64_V4,
}

impl From<KnownArchitecture> for alpm_types::SystemArchitecture {
    fn from(arch: KnownArchitecture) -> alpm_types::SystemArchitecture {
        match arch {
            KnownArchitecture::AARCH64 => alpm_types::SystemArchitecture::Aarch64,
            KnownArchitecture::ARM => alpm_types::SystemArchitecture::Arm,
            KnownArchitecture::ARMV6H => alpm_types::SystemArchitecture::Armv6h,
            KnownArchitecture::ARMV7H => alpm_types::SystemArchitecture::Armv7h,
            KnownArchitecture::I386 => alpm_types::SystemArchitecture::I386,
            KnownArchitecture::I486 => alpm_types::SystemArchitecture::I486,
            KnownArchitecture::I686 => alpm_types::SystemArchitecture::I686,
            KnownArchitecture::PENTIUM4 => alpm_types::SystemArchitecture::Pentium4,
            KnownArchitecture::RISCV32 => alpm_types::SystemArchitecture::Riscv32,
            KnownArchitecture::RISCV64 => alpm_types::SystemArchitecture::Riscv64,
            KnownArchitecture::X86_64 => alpm_types::SystemArchitecture::X86_64,
            KnownArchitecture::X86_64_V2 => alpm_types::SystemArchitecture::X86_64V2,
            KnownArchitecture::X86_64_V3 => alpm_types::SystemArchitecture::X86_64V3,
            KnownArchitecture::X86_64_V4 => alpm_types::SystemArchitecture::X86_64V4,
        }
    }
}

#[pymethods]
impl KnownArchitecture {
    #[getter]
    fn value(&self) -> String {
        self.to_string()
    }

    fn __str__(&self) -> String {
        self.to_string()
    }
}

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnknownArchitecture(alpm_types::UnknownArchitecture);

impl_from!(UnknownArchitecture, alpm_types::UnknownArchitecture);

#[pymethods]
impl UnknownArchitecture {
    #[getter]
    fn value(&self) -> String {
        self.0.to_string()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("UnknownArchitecture('{}')", self.0)
    }
}

impl From<UnknownArchitecture> for alpm_types::SystemArchitecture {
    fn from(arch: UnknownArchitecture) -> alpm_types::SystemArchitecture {
        alpm_types::SystemArchitecture::Unknown(arch.into())
    }
}

/// Python union: `KnownArchitecture | str`
#[derive(
    FromPyObject, IntoPyObject, Clone, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub enum SystemArchitecture {
    Known(KnownArchitecture),
    Unknown(UnknownArchitecture),
}

impl From<SystemArchitecture> for alpm_types::SystemArchitecture {
    fn from(arch: SystemArchitecture) -> alpm_types::SystemArchitecture {
        match arch {
            SystemArchitecture::Known(known) => known.into(),
            SystemArchitecture::Unknown(unknown) => unknown.into(),
        }
    }
}

impl From<alpm_types::SystemArchitecture> for SystemArchitecture {
    fn from(arch: alpm_types::SystemArchitecture) -> SystemArchitecture {
        match arch {
            alpm_types::SystemArchitecture::Aarch64 => {
                SystemArchitecture::Known(KnownArchitecture::AARCH64)
            }
            alpm_types::SystemArchitecture::Arm => {
                SystemArchitecture::Known(KnownArchitecture::ARM)
            }
            alpm_types::SystemArchitecture::Armv6h => {
                SystemArchitecture::Known(KnownArchitecture::ARMV6H)
            }
            alpm_types::SystemArchitecture::Armv7h => {
                SystemArchitecture::Known(KnownArchitecture::ARMV7H)
            }
            alpm_types::SystemArchitecture::I386 => {
                SystemArchitecture::Known(KnownArchitecture::I386)
            }
            alpm_types::SystemArchitecture::I486 => {
                SystemArchitecture::Known(KnownArchitecture::I486)
            }
            alpm_types::SystemArchitecture::I686 => {
                SystemArchitecture::Known(KnownArchitecture::I686)
            }
            alpm_types::SystemArchitecture::Pentium4 => {
                SystemArchitecture::Known(KnownArchitecture::PENTIUM4)
            }
            alpm_types::SystemArchitecture::Riscv32 => {
                SystemArchitecture::Known(KnownArchitecture::RISCV32)
            }
            alpm_types::SystemArchitecture::Riscv64 => {
                SystemArchitecture::Known(KnownArchitecture::RISCV64)
            }
            alpm_types::SystemArchitecture::X86_64 => {
                SystemArchitecture::Known(KnownArchitecture::X86_64)
            }
            alpm_types::SystemArchitecture::X86_64V2 => {
                SystemArchitecture::Known(KnownArchitecture::X86_64_V2)
            }
            alpm_types::SystemArchitecture::X86_64V3 => {
                SystemArchitecture::Known(KnownArchitecture::X86_64_V3)
            }
            alpm_types::SystemArchitecture::X86_64V4 => {
                SystemArchitecture::Known(KnownArchitecture::X86_64_V4)
            }
            alpm_types::SystemArchitecture::Unknown(unknown) => {
                SystemArchitecture::Unknown(unknown.into())
            }
        }
    }
}

/// Union used as an argument for [`Architecture`] constructor.
///
/// [`RawArchitecture::String`] is converted to either [`KnownArchitecture`],
/// [`UnknownArchitecture`] or "any".
#[derive(Clone, Debug, FromPyObject, IntoPyObject)]
pub enum RawArchitecture {
    Known(KnownArchitecture),
    Unknown(UnknownArchitecture),
    String(String),
}

impl TryFrom<RawArchitecture> for alpm_types::Architecture {
    type Error = crate::types::Error;

    fn try_from(arch: RawArchitecture) -> Result<alpm_types::Architecture, Self::Error> {
        Ok(match arch {
            RawArchitecture::Known(known) => alpm_types::Architecture::Some(known.into()),
            RawArchitecture::Unknown(unknown) => alpm_types::Architecture::Some(unknown.into()),
            RawArchitecture::String(s) => alpm_types::Architecture::from_str(s.as_str())?,
        })
    }
}

impl TryFrom<RawArchitecture> for Architecture {
    type Error = crate::types::Error;

    fn try_from(arch: RawArchitecture) -> Result<Architecture, Self::Error> {
        Ok(Architecture(arch.try_into()?))
    }
}

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Architecture(alpm_types::Architecture);

impl_from!(Architecture, alpm_types::Architecture);

#[pymethods]
impl Architecture {
    #[new]
    #[pyo3(signature = (arch = RawArchitecture::String("any".to_string())))]
    fn new(arch: RawArchitecture) -> Result<Architecture, crate::types::Error> {
        arch.try_into()
    }

    #[getter]
    fn is_any(&self) -> bool {
        matches!(self.0, alpm_types::Architecture::Any)
    }

    #[getter]
    fn system_arch(&self) -> Option<SystemArchitecture> {
        match &self.0 {
            alpm_types::Architecture::Any => None,
            alpm_types::Architecture::Some(arch) => Some(arch.clone().into()),
        }
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Architecture('{}')", self.0)
    }
}

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Architectures(alpm_types::Architectures);

impl_from!(Architectures, alpm_types::Architectures);

impl TryFrom<Vec<Architecture>> for Architectures {
    type Error = crate::types::Error;

    fn try_from(architectures: Vec<Architecture>) -> Result<Architectures, Self::Error> {
        if architectures.is_empty() {
            Ok(Architectures(alpm_types::Architectures::Any))
        } else {
            let inner_vec: Vec<alpm_types::Architecture> = vec_convert!(architectures);
            let inner: alpm_types::Architectures = alpm_types::Architectures::try_from(inner_vec)?;
            Ok(inner.into())
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct ArchitecturesIterator(std::vec::IntoIter<alpm_types::Architecture>);

#[pymethods]
impl ArchitecturesIterator {
    fn __iter__(&self) -> Self {
        self.clone()
    }

    fn __next__(&mut self) -> PyResult<Architecture> {
        self.0
            .next()
            .map(From::from)
            .ok_or(PyStopIteration::new_err(()))
    }
}

#[pymethods]
impl Architectures {
    #[new]
    #[pyo3(signature = (architectures = vec![]))]
    fn new(
        architectures: Option<Vec<RawArchitecture>>,
    ) -> Result<Architectures, crate::types::Error> {
        match architectures {
            Some(architectures) => {
                let archs_vec: Vec<Architecture> = architectures
                    .into_iter()
                    .map(|arch| arch.try_into())
                    .collect::<Result<Vec<Architecture>, crate::types::Error>>()?;
                archs_vec.try_into()
            }
            None => Ok(alpm_types::Architectures::Any.into()),
        }
    }

    #[getter]
    fn is_any(&self) -> bool {
        matches!(self.0, alpm_types::Architectures::Any)
    }

    fn __iter__(&self) -> ArchitecturesIterator {
        let inner: alpm_types::Architectures = self.clone().into();
        ArchitecturesIterator(inner.into_iter())
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "Architectures([{}])",
            self.0
                .into_iter()
                .map(|arch| Architecture::from(arch).__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
// Uses Python's enum variant naming convention.
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum ElfArchitectureFormat {
    #[strum(to_string = "32")]
    BIT_32,
    #[strum(to_string = "64")]
    BIT_64,
}

#[pymethods]
impl ElfArchitectureFormat {
    #[staticmethod]
    fn from_str(format: &str) -> PyResult<ElfArchitectureFormat> {
        alpm_types::ElfArchitectureFormat::from_str(format)
            .map(ElfArchitectureFormat::from)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

impl From<ElfArchitectureFormat> for alpm_types::ElfArchitectureFormat {
    fn from(format: ElfArchitectureFormat) -> alpm_types::ElfArchitectureFormat {
        match format {
            ElfArchitectureFormat::BIT_32 => alpm_types::ElfArchitectureFormat::Bit32,
            ElfArchitectureFormat::BIT_64 => alpm_types::ElfArchitectureFormat::Bit64,
        }
    }
}

impl From<alpm_types::ElfArchitectureFormat> for ElfArchitectureFormat {
    fn from(format: alpm_types::ElfArchitectureFormat) -> ElfArchitectureFormat {
        match format {
            alpm_types::ElfArchitectureFormat::Bit32 => ElfArchitectureFormat::BIT_32,
            alpm_types::ElfArchitectureFormat::Bit64 => ElfArchitectureFormat::BIT_64,
        }
    }
}
