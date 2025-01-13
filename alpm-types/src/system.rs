use strum::{Display, EnumString};

/// CPU architecture
///
/// Members of the Architecture enum can be created from `&str`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::Architecture;
///
/// // create Architecture from str
/// assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
///
/// // format as String
/// assert_eq!("aarch64", format!("{}", Architecture::Aarch64));
/// assert_eq!("any", format!("{}", Architecture::Any));
/// assert_eq!("arm", format!("{}", Architecture::Arm));
/// assert_eq!("armv6h", format!("{}", Architecture::Armv6h));
/// assert_eq!("armv7h", format!("{}", Architecture::Armv7h));
/// assert_eq!("i386", format!("{}", Architecture::I386));
/// assert_eq!("i486", format!("{}", Architecture::I486));
/// assert_eq!("i686", format!("{}", Architecture::I686));
/// assert_eq!("pentium4", format!("{}", Architecture::Pentium4));
/// assert_eq!("riscv32", format!("{}", Architecture::Riscv32));
/// assert_eq!("riscv64", format!("{}", Architecture::Riscv64));
/// assert_eq!("x86_64", format!("{}", Architecture::X86_64));
/// assert_eq!("x86_64_v2", format!("{}", Architecture::X86_64V2));
/// assert_eq!("x86_64_v3", format!("{}", Architecture::X86_64V3));
/// assert_eq!("x86_64_v4", format!("{}", Architecture::X86_64V4));
/// ```
#[derive(Clone, Copy, Debug, Display, EnumString, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
#[strum(serialize_all = "lowercase")]
pub enum Architecture {
    /// ARMv8 64-bit
    Aarch64,
    /// Any architecture
    Any,
    /// ARM
    Arm,
    /// ARMv6 hard-float
    Armv6h,
    /// ARMv7 hard-float
    Armv7h,
    /// Intel 386
    I386,
    /// Intel 486
    I486,
    /// Intel 686
    I686,
    /// Intel Pentium 4
    Pentium4,
    /// RISC-V 32-bit
    Riscv32,
    /// RISC-V 64-bit
    Riscv64,
    /// Intel x86_64
    X86_64,
    /// Intel x86_64 version 2
    #[strum(to_string = "x86_64_v2")]
    X86_64V2,
    /// Intel x86_64 version 3
    #[strum(to_string = "x86_64_v3")]
    X86_64V3,
    /// Intel x86_64 version 4
    #[strum(to_string = "x86_64_v4")]
    X86_64V4,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;
    use strum::ParseError;

    use super::*;

    #[rstest]
    #[case("aarch64", Ok(Architecture::Aarch64))]
    #[case("any", Ok(Architecture::Any))]
    #[case("arm", Ok(Architecture::Arm))]
    #[case("armv6h", Ok(Architecture::Armv6h))]
    #[case("armv7h", Ok(Architecture::Armv7h))]
    #[case("i386", Ok(Architecture::I386))]
    #[case("i486", Ok(Architecture::I486))]
    #[case("i686", Ok(Architecture::I686))]
    #[case("pentium4", Ok(Architecture::Pentium4))]
    #[case("riscv32", Ok(Architecture::Riscv32))]
    #[case("riscv64", Ok(Architecture::Riscv64))]
    #[case("x86_64", Ok(Architecture::X86_64))]
    #[case("x86_64_v2", Ok(Architecture::X86_64V2))]
    #[case("x86_64_v3", Ok(Architecture::X86_64V3))]
    #[case("x86_64_v4", Ok(Architecture::X86_64V4))]
    #[case("foo", Err(ParseError::VariantNotFound))]
    fn architecture_from_string(#[case] s: &str, #[case] arch: Result<Architecture, ParseError>) {
        assert_eq!(Architecture::from_str(s), arch);
    }

    #[rstest]
    #[case(Architecture::Aarch64, "aarch64")]
    #[case(Architecture::Any, "any")]
    #[case(Architecture::Arm, "arm")]
    #[case(Architecture::Armv6h, "armv6h")]
    #[case(Architecture::Armv7h, "armv7h")]
    #[case(Architecture::I386, "i386")]
    #[case(Architecture::I486, "i486")]
    #[case(Architecture::I686, "i686")]
    #[case(Architecture::Pentium4, "pentium4")]
    #[case(Architecture::Riscv32, "riscv32")]
    #[case(Architecture::Riscv64, "riscv64")]
    #[case(Architecture::X86_64, "x86_64")]
    #[case(Architecture::X86_64V2, "x86_64_v2")]
    #[case(Architecture::X86_64V3, "x86_64_v3")]
    #[case(Architecture::X86_64V4, "x86_64_v4")]
    fn architecture_format_string(#[case] arch: Architecture, #[case] arch_str: &str) {
        assert_eq!(arch_str, format!("{}", arch));
    }
}
