use std::str::FromStr;

use pyo3::{prelude::*, types::PyType};
use strum::Display;

#[pyclass(frozen, eq, ord, hash)]
#[derive(Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd)]
// Uses Python's enum variant naming convention.
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
pub enum Architecture {
    AARCH64,
    ANY,
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

#[pymethods]
impl Architecture {
    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, arch: &str) -> PyResult<Architecture> {
        alpm_types::Architecture::from_str(arch)
            .map(Architecture::from)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

impl From<Architecture> for alpm_types::Architecture {
    fn from(arch: Architecture) -> alpm_types::Architecture {
        match arch {
            Architecture::AARCH64 => alpm_types::Architecture::Aarch64,
            Architecture::ANY => alpm_types::Architecture::Any,
            Architecture::ARM => alpm_types::Architecture::Arm,
            Architecture::ARMV6H => alpm_types::Architecture::Armv6h,
            Architecture::ARMV7H => alpm_types::Architecture::Armv7h,
            Architecture::I386 => alpm_types::Architecture::I386,
            Architecture::I486 => alpm_types::Architecture::I486,
            Architecture::I686 => alpm_types::Architecture::I686,
            Architecture::PENTIUM4 => alpm_types::Architecture::Pentium4,
            Architecture::RISCV32 => alpm_types::Architecture::Riscv32,
            Architecture::RISCV64 => alpm_types::Architecture::Riscv64,
            Architecture::X86_64 => alpm_types::Architecture::X86_64,
            Architecture::X86_64_V2 => alpm_types::Architecture::X86_64V2,
            Architecture::X86_64_V3 => alpm_types::Architecture::X86_64V3,
            Architecture::X86_64_V4 => alpm_types::Architecture::X86_64V4,
        }
    }
}

impl From<alpm_types::Architecture> for Architecture {
    fn from(arch: alpm_types::Architecture) -> Architecture {
        match arch {
            alpm_types::Architecture::Aarch64 => Architecture::AARCH64,
            alpm_types::Architecture::Any => Architecture::ANY,
            alpm_types::Architecture::Arm => Architecture::ARM,
            alpm_types::Architecture::Armv6h => Architecture::ARMV6H,
            alpm_types::Architecture::Armv7h => Architecture::ARMV7H,
            alpm_types::Architecture::I386 => Architecture::I386,
            alpm_types::Architecture::I486 => Architecture::I486,
            alpm_types::Architecture::I686 => Architecture::I686,
            alpm_types::Architecture::Pentium4 => Architecture::PENTIUM4,
            alpm_types::Architecture::Riscv32 => Architecture::RISCV32,
            alpm_types::Architecture::Riscv64 => Architecture::RISCV64,
            alpm_types::Architecture::X86_64 => Architecture::X86_64,
            alpm_types::Architecture::X86_64V2 => Architecture::X86_64_V2,
            alpm_types::Architecture::X86_64V3 => Architecture::X86_64_V3,
            alpm_types::Architecture::X86_64V4 => Architecture::X86_64_V4,
        }
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
    #[classmethod]
    fn from_str(_cls: &Bound<'_, PyType>, format: &str) -> PyResult<ElfArchitectureFormat> {
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
