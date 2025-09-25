//! Compression level structs for various compression algorithms.

use std::fmt::{Debug, Display};

use log::trace;

use crate::error::Error;

/// A macro to define a compression level struct.
///
/// Accepts the `name` of the compression level struct, its `min`, `max` and `default` values, the
/// `compression` executable it relates to and a `url`, that defines a man page for the
/// `compression` executable.
macro_rules! define_compression_level {
    (
        $name:ident,
        Min => $min:expr,
        Max => $max:expr,
        Default => $default:expr,
        $compression:literal,
        $url:literal
    ) => {
        #[doc = concat!("Compression level for ", $compression, " compression.")]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(u8);

        impl $name {
            #[doc = concat!("Creates a new [`", stringify!($name), "`] from a [`u8`].")]
            ///
            #[doc = concat!("The `level` must be in the range of [`", stringify!($name), "::min`] and [`", stringify!($name), "::max`].")]
            ///
            /// # Errors
            ///
            #[doc = concat!("Returns an error if the value is not in the range of [`", stringify!($name), "::min`] and [`", stringify!($name), "::max`].")]
            pub fn new(level: u8) -> Result<Self, Error> {
                trace!(concat!("Creating new compression level for ", $compression, " compression with {{level}}"));
                if !($name::min()..=$name::max()).contains(&level) {
                    return Err(Error::InvalidCompressionLevel {
                        level,
                        min: $name::min(),
                        max: $name::max(),
                    });
                }
                Ok(Self(level))
            }

            #[doc = concat!("Returns the default level (`", stringify!($default), "`) for [`", stringify!($name), "`].")]
            ///
            #[doc = concat!("The default level adheres to the one selected by the [", $compression, "] executable.")]
            ///
            #[doc = concat!("[", $compression, "]: ", $url)]
            pub const fn default_level() -> u8 {
                $default
            }

            #[doc = concat!("Returns the minimum allowed level (`", stringify!($min), "`) for [`", stringify!($name), "`].")]
            pub const fn min() -> u8 {
                $min
            }

            #[doc = concat!("Returns the maximum allowed level (`", stringify!($max), "`) for [`", stringify!($name), "`].")]
            pub const fn max() -> u8 {
                $max
            }
        }

        impl Default for $name {
            #[doc = concat!("Returns the default [`", stringify!($name), "`].")]
            ///
            #[doc = concat!("Delegates to [`", stringify!($name), "::default_level`] for retrieving the default compression level.")]
            fn default() -> Self {
                Self($name::default_level())
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<&$name> for i32 {
            fn from(value: &$name) -> Self {
                i32::from(value.0)
            }
        }

        impl From<&$name> for u32 {
            fn from(value: &$name) -> Self {
                 u32::from(value.0)
            }
        }

        impl TryFrom<i8> for $name {
            type Error = Error;

            fn try_from(value: i8) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i16> for $name {
            type Error = Error;

            fn try_from(value: i16) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i32> for $name {
            type Error = Error;

            fn try_from(value: i32) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<i64> for $name {
            type Error = Error;

            fn try_from(value: i64) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u8> for $name {
            type Error = Error;

            fn try_from(value: u8) -> Result<Self, Error> {
                 $name::new(value)
            }
        }

        impl TryFrom<u16> for $name {
            type Error = Error;

            fn try_from(value: u16) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u32> for $name {
            type Error = Error;

            fn try_from(value: u32) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }

        impl TryFrom<u64> for $name {
            type Error = Error;

            fn try_from(value: u64) -> Result<Self, Error> {
                 $name::new(u8::try_from(value).map_err(Error::IntegerConversion)?)
            }
        }
    };
}

// Create the bzip2 compression level struct.
define_compression_level!(
    Bzip2CompressionLevel,
    Min => 1,
    Max => 9,
    Default => 9,
    "bzip2",
    "https://man.archlinux.org/man/bzip2.1"
);

// Create the gzip compression level struct.
define_compression_level!(
    GzipCompressionLevel,
    Min => 1,
    Max => 9,
    Default => 6,
    "gzip",
    "https://man.archlinux.org/man/gzip.1"
);

// Create the xz compression level struct.
define_compression_level!(
    XzCompressionLevel,
    Min => 0,
    Max => 9,
    Default => 6,
    "xz",
    "https://man.archlinux.org/man/xz.1"
);

// Create the zstd compression level struct.
define_compression_level!(
    ZstdCompressionLevel,
    Min => 0,
    Max => 22,
    Default => 3,
    "zstd",
    "https://man.archlinux.org/man/zstd.1"
);

#[cfg(test)]
mod tests {
    use proptest::{proptest, test_runner::Config as ProptestConfig};
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_bzip2_compression_level_try_from_i8(input in 1..=9i8) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i16(input in 1..=9i16) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i32(input in 1..=9i32) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_i64(input in 1..=9i64) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u8(input in 1..=9u8) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u16(input in 1..=9u16) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u32(input in 1..=9u32) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_bzip2_compression_level_try_from_u64(input in 1..=9u64) {
            assert!(Bzip2CompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i8(input in 1..=9i8) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i16(input in 1..=9i16) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i32(input in 1..=9i32) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_i64(input in 1..=9i64) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u8(input in 1..=9u8) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u16(input in 1..=9u16) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u32(input in 1..=9u32) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_gzip_compression_level_try_from_u64(input in 1..=9u64) {
            assert!(GzipCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i8(input in 0..=9i8) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i16(input in 0..=9i16) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i32(input in 0..=9i32) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_i64(input in 0..=9i64) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u8(input in 0..=9u8) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u16(input in 0..=9u16) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u32(input in 0..=9u32) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_xz_compression_level_try_from_u64(input in 0..=9u64) {
            assert!(XzCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i8(input in 0..=22i8) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i16(input in 0..=22i16) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i32(input in 0..=22i32) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_i64(input in 0..=22i64) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u8(input in 0..=22u8) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u16(input in 0..=22u16) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u32(input in 0..=22u32) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }

        #[test]
        fn valid_zstd_compression_level_try_from_u64(input in 0..=22u64) {
            assert!(ZstdCompressionLevel::try_from(input).is_ok());
        }
    }

    #[rstest]
    #[case::too_large(Bzip2CompressionLevel::max() + 1)]
    #[case::too_small(Bzip2CompressionLevel::min() - 1)]
    fn create_bzip2_compression_level_fails(#[case] level: u8) -> TestResult {
        if let Ok(level) = Bzip2CompressionLevel::new(level) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_bzip2_compression_level_succeeds() -> TestResult {
        if let Err(error) = Bzip2CompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[rstest]
    #[case::too_large(GzipCompressionLevel::max() + 1)]
    #[case::too_small(GzipCompressionLevel::min() - 1)]
    fn create_gzip_compression_level_fails(#[case] level: u8) -> TestResult {
        if let Ok(level) = GzipCompressionLevel::new(level) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_gzip_compression_level_succeeds() -> TestResult {
        if let Err(error) = GzipCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[test]
    fn create_xz_compression_level_fails() -> TestResult {
        if let Ok(level) = XzCompressionLevel::new(XzCompressionLevel::max() + 1) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_xz_compression_level_succeeds() -> TestResult {
        if let Err(error) = XzCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }

    #[test]
    fn create_zstd_compression_level_fails() -> TestResult {
        if let Ok(level) = ZstdCompressionLevel::new(ZstdCompressionLevel::max() + 1) {
            return Err(format!("Should not have succeeded but created level: {level}").into());
        }

        Ok(())
    }

    #[test]
    fn create_zstd_compression_level_succeeds() -> TestResult {
        if let Err(error) = ZstdCompressionLevel::new(6) {
            return Err(format!("Should have succeeded but raised error:\n{error}").into());
        }

        Ok(())
    }
}
