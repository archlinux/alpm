use strum_macros::Display;
use strum_macros::EnumString;

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
#[derive(Clone, Debug, Display, EnumString, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Architecture {
    #[strum(to_string = "aarch64")]
    Aarch64,
    #[strum(to_string = "any")]
    Any,
    #[strum(to_string = "arm")]
    Arm,
    #[strum(to_string = "armv6h")]
    Armv6h,
    #[strum(to_string = "armv7h")]
    Armv7h,
    #[strum(to_string = "i486")]
    I486,
    #[strum(to_string = "i686")]
    I686,
    #[strum(to_string = "pentium4")]
    Pentium4,
    #[strum(to_string = "riscv32")]
    Riscv32,
    #[strum(to_string = "riscv64")]
    Riscv64,
    #[strum(to_string = "x86_64")]
    X86_64,
    #[strum(to_string = "x86_64_v2")]
    X86_64V2,
    #[strum(to_string = "x86_64_v3")]
    X86_64V3,
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
    fn architecture_from_string(
        #[case] from_str: &str,
        #[case] arch: Result<Architecture, ParseError>,
    ) {
        assert_eq!(Architecture::from_str(from_str), arch);
    }

    #[rstest]
    #[case(Architecture::Aarch64, "aarch64")]
    #[case(Architecture::Any, "any")]
    #[case(Architecture::Arm, "arm")]
    #[case(Architecture::Armv6h, "armv6h")]
    #[case(Architecture::Armv7h, "armv7h")]
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
