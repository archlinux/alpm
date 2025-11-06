use std::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::DerefMut,
    str::FromStr,
};

use digest::{Digest, FixedOutput, HashMarker, Output, OutputSizeUser, Update};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumString, VariantArray, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    ascii::dec_uint,
    combinator::{alt, cut_err, eof, repeat, terminated},
    error::{StrContext, StrContextValue},
    token::one_of,
};

use crate::{
    Error,
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};

/// Defines the string representation format of a checksum digest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestEncoding {
    /// Checksum digest represented by a hexadecimal string.
    Hex,
    /// Checksum digest represented by a decimal string.
    Dec,
}

/// [`Digest`] extension providing a [`Self::ENCODING`] constant defining the string representation
/// of the digest used for parsing and formatting.
pub trait DigestString: Digest {
    /// The format used for string representation of the digest.
    const ENCODING: DigestEncoding;
}

impl DigestString for Blake2b512 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Md5 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Sha1 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Sha224 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Sha256 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Sha384 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Sha512 {
    const ENCODING: DigestEncoding = DigestEncoding::Hex;
}

impl DigestString for Crc32Cksum {
    const ENCODING: DigestEncoding = DigestEncoding::Dec;
}

// Convenience type aliases for the supported checksums

/// A checksum using the Blake2b512 algorithm
pub type Blake2b512Checksum = Checksum<Blake2b512>;

/// A checksum using the Md5 algorithm
pub type Md5Checksum = Checksum<Md5>;

/// A checksum using the Sha1 algorithm
pub type Sha1Checksum = Checksum<Sha1>;

/// A checksum using the Sha224 algorithm
pub type Sha224Checksum = Checksum<Sha224>;

/// A checksum using the Sha256 algorithm
pub type Sha256Checksum = Checksum<Sha256>;

/// A checksum using the Sha384 algorithm
pub type Sha384Checksum = Checksum<Sha384>;

/// A checksum using the Sha512 algorithm
pub type Sha512Checksum = Checksum<Sha512>;

/// A checksum using CRC-32/CKSUM algorithm
pub type Crc32CksumChecksum = Checksum<Crc32Cksum>;

/// This enum represents all accepted checksum algorithms used in the Arch Linux distribution.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    VariantNames,
    VariantArray,
)]
pub enum ChecksumAlgorithm {
    /// Blake2b-512 cryptographic hash algorithm
    Blake2b512,
    /// Md5 hash algorithm (deprecated)
    Md5,
    /// Sha1 hash algorithm (deprecated)
    Sha1,
    /// Sha224 hash algorithm
    Sha224,
    /// Sha256 hash algorithm
    Sha256,
    /// Sha384 hash algorithm
    Sha384,
    /// Sha512 hash algorithm
    Sha512,
    /// CRC-32/CKSUM hash algorithm
    Crc32Cksum,
}

impl ChecksumAlgorithm {
    /// Determines if a checksum algorithm is considered deprecated for security reasons.
    ///
    /// Returns `true` for cryptographically unsafe algorithms that should be avoided.
    /// These algorithms are still supported for backwards compatibility but their use is strongly
    /// discouraged.
    ///
    /// Currently deprecated algorithms:
    ///
    /// - [`ChecksumAlgorithm::Md5`]: Vulnerable to collision attacks
    /// - [`ChecksumAlgorithm::Sha1`]: Vulnerable to collision attacks
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::ChecksumAlgorithm;
    ///
    /// // Deprecated algorithms
    /// assert!(ChecksumAlgorithm::Md5.is_deprecated());
    /// assert!(ChecksumAlgorithm::Sha1.is_deprecated());
    ///
    /// // Safe algorithms
    /// assert!(!ChecksumAlgorithm::Sha256.is_deprecated());
    /// assert!(!ChecksumAlgorithm::Blake2b512.is_deprecated());
    /// ```
    pub fn is_deprecated(&self) -> bool {
        match self {
            ChecksumAlgorithm::Md5 | ChecksumAlgorithm::Sha1 | ChecksumAlgorithm::Crc32Cksum => {
                true
            }
            ChecksumAlgorithm::Blake2b512
            | ChecksumAlgorithm::Sha224
            | ChecksumAlgorithm::Sha256
            | ChecksumAlgorithm::Sha384
            | ChecksumAlgorithm::Sha512 => false,
        }
    }

    /// Returns a list of [`ChecksumAlgorithm`] variants that are not considered deprecated.
    pub fn non_deprecated_checksums(&self) -> Vec<ChecksumAlgorithm> {
        <ChecksumAlgorithm as VariantArray>::VARIANTS
            .iter()
            .filter(|algo| !algo.is_deprecated())
            .copied()
            .collect::<Vec<ChecksumAlgorithm>>()
    }
}

/// A [checksum] using a supported algorithm
///
/// Checksums are created using one of the supported algorithms:
///
/// - `Blake2b512`
/// - `Md5` (**WARNING**: Use of this algorithm is highly discouraged, because it is
///   cryptographically unsafe)
/// - `Sha1` (**WARNING**: Use of this algorithm is highly discouraged, because it is
///   cryptographically unsafe)
/// - `Sha224`
/// - `Sha256`
/// - `Sha384`
/// - `Sha512`
/// - `Crc32Cksum` (**WARNING**: Use of this algorithm is highly discouraged, because it is
///   cryptographically unsafe)
///
/// ## Note
///
/// There are two ways to use a checksum:
///
/// 1. Generically over a digest (e.g. `Checksum::<Blake2b512>`)
/// 2. Using the convenience type aliases (e.g. `Blake2b512Checksum`)
///
/// ## Examples
///
/// ```
/// use std::str::FromStr;
/// use alpm_types::{digests::Blake2b512, Checksum};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// let checksum = Checksum::<Blake2b512>::calculate_from("foo\n");
/// let digest = vec![
///     210, 2, 215, 149, 29, 242, 196, 183, 17, 202, 68, 180, 188, 201, 215, 179, 99, 250, 66,
///     82, 18, 126, 5, 140, 26, 145, 14, 192, 91, 108, 208, 56, 215, 28, 194, 18, 33, 192, 49,
///     192, 53, 159, 153, 62, 116, 107, 7, 245, 150, 92, 248, 197, 195, 116, 106, 88, 51, 122,
///     217, 171, 101, 39, 142, 119,
/// ];
/// assert_eq!(checksum.inner(), digest);
/// assert_eq!(
///     format!("{}", checksum),
///     "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77",
/// );
///
/// // create checksum from hex string
/// let checksum = Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77")?;
/// assert_eq!(checksum.inner(), digest);
/// # Ok(())
/// # }
/// ```
///
/// # Developer Note
///
/// In case you want to wrap this type and make the parent `Serialize`able, please note the
/// following:
///
/// Serde automatically adds a `Serialize` trait bound on top of it trait bounds in wrapper
/// types. **However**, that's not needed as we use `D` simply as a phantom marker that
/// isn't serialized in the first place.
/// To fix this in your wrapper type, make use of the [bound container attribute], e.g.:
///
/// [checksum]: https://en.wikipedia.org/wiki/Checksum
/// ```
/// use alpm_types::{Checksum, digests::Digest};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Wrapper<D: Digest> {
///     #[serde(bound = "D: Digest")]
///     checksum: Checksum<D>,
/// }
/// ```
#[derive(Clone)]
pub struct Checksum<D: Digest> {
    digest: Vec<u8>,
    _marker: PhantomData<D>,
}

impl<D: DigestString> Serialize for Checksum<D> {
    /// Serialize a [`Checksum`] into a hex `String` representation.
    ///
    /// We chose hex as byte vectors are imperformant and considered bad practice for non-binary
    /// formats like `JSON` or `YAML`
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, D: DigestString> Deserialize<'de> for Checksum<D> {
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Checksum::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl<D: DigestString> Checksum<D> {
    /// Calculate a new Checksum for data that may be represented as a list of bytes
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::{digests::Blake2b512, Checksum};
    ///
    /// assert_eq!(
    ///     format!("{}", Checksum::<Blake2b512>::calculate_from("foo\n")),
    ///     "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77",
    /// );
    /// ```
    pub fn calculate_from(input: impl AsRef<[u8]>) -> Self {
        let mut hasher = D::new();
        hasher.update(input);

        Checksum {
            digest: hasher.finalize()[..].to_vec(),
            _marker: PhantomData,
        }
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &[u8] {
        &self.digest
    }

    /// Recognizes an ASCII hexadecimal [`Checksum`] from a string slice.
    ///
    /// Consumes all input.
    /// See [`Checksum::from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not the output of a _hash function_
    /// in hexadecimal (or decimal in case of CRC-32/CKSUM) form.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        /// Consume 1 hex digit and return its hex value.
        ///
        /// Accepts uppercase or lowercase.
        #[inline]
        fn hex_digit(input: &mut &str) -> ModalResult<u8> {
            one_of(('0'..='9', 'a'..='f', 'A'..='F'))
                .map(|d: char|
                    // unwraps are unreachable: their invariants are always
                    // upheld because the above character set can never
                    // consume anything but a single valid hex digit
                    d.to_digit(16).unwrap().try_into().unwrap())
                .context(StrContext::Expected(StrContextValue::Description(
                    "ASCII hex digit",
                )))
                .parse_next(input)
        }

        let hex_pair = (hex_digit, hex_digit).map(|(first, second)|
            // shift is infallible because hex_digit cannot return >0b00001111
            (first << 4) + second);

        // output size in bytes
        let digest_bytes = <D as Digest>::output_size();

        let digest = match D::ENCODING {
            DigestEncoding::Hex => {
                // Consume exactly the number of hex pairs that our Digest type expects
                let digest = cut_err(repeat(digest_bytes, hex_pair))
                    .context(StrContext::Label("hash digest"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "a hex hash digest with the appropriate length for the given algorithm.",
                    )))
                    .parse_next(input)?;

                cut_err(eof)
                    .context(StrContext::Expected(StrContextValue::Description(
                        "end of checksum. Checksum is too long.",
                    )))
                    .parse_next(input)?;

                digest
            }
            DigestEncoding::Dec => {
                // output size in bits
                let digest_bits = digest_bytes * 8;

                // The following logic parses a decimal integer for consumption by a digest.
                // We chose to use a [`u128::MAX`] as this is the currently largest number type in
                // the rust std library. In reality we only use this for CRC-32/CKSUM which is
                // 4 bytes, but it's nice to keep this a bit more generic.

                // Determine the maximum allowed value based on the number of allowed
                // `digest_bytes`.
                let max_value: u128 = if digest_bits >= 128 {
                    // Since we're parsing into a `u128`, we don't allow digests that use more bytes
                    // than that. If we ever were to add such a digest, this
                    // logic needs to be adjusted.
                    u128::MAX
                } else {
                    (1u128 << digest_bits) - 1
                };

                // Parse into the u128 decimal and verify that the resulting value fits into our
                // requested digest length. E.g. CRC-32 is restricted to a u32.
                cut_err(dec_uint::<_, u128, _>)
                    .verify(move |&v| v <= max_value)
                    // Convert the u128 into a big endian byte array.
                    // Then cut the array at the highest significant byte we allow for this digest.
                    .map(move |v | v.to_be_bytes()[16 - digest_bytes..].to_vec())
                    .context(StrContext::Label("hash digest"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "a decimal hash digest with the appropriate length for the given algorithm.",
                    )))
                    .parse_next(input)?
            }
        };

        Ok(Self {
            digest,
            _marker: PhantomData,
        })
    }
}

impl<D: DigestString> FromStr for Checksum<D> {
    type Err = Error;
    /// Create a new Checksum from a hex string and return it in a Result
    ///
    /// The input is processed as a lowercase string.
    /// An Error is returned, if the input length does not match the output size for the given
    /// supported algorithm, or if the provided hex string could not be converted to a list of
    /// bytes.
    ///
    /// Delegates to [`Checksum::parser`].
    ///
    /// ## Examples
    /// ```
    /// use std::str::FromStr;
    /// use alpm_types::{digests::Blake2b512, Checksum};
    ///
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77").is_ok());
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e7").is_err());
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e7x").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Checksum<D>, Self::Err> {
        Ok(Checksum::parser.parse(s)?)
    }
}

impl<D: DigestString> Display for Checksum<D> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match D::ENCODING {
            DigestEncoding::Hex => {
                write!(
                    fmt,
                    "{}",
                    self.digest
                        .iter()
                        .map(|x| format!("{x:02x?}"))
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
            DigestEncoding::Dec => {
                // Convert a big-endian byte array into an u128.
                // The parser already assumes that the digest fits into a u128,
                // so this should be infallible.
                let value = self
                    .digest
                    .iter()
                    .fold(0u128, |acc, &byte| (acc << 8) | byte as u128);
                write!(fmt, "{}", value)
            }
        }
    }
}

/// Use [Display] as [Debug] impl, since the byte representation and [PhantomData] field aren't
/// relevant for debugging purposes.
impl<D: DigestString> Debug for Checksum<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl<D: Digest> PartialEq for Checksum<D> {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

impl<D: Digest> Eq for Checksum<D> {}

impl<D: Digest> Ord for Checksum<D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.digest.cmp(&other.digest)
    }
}

impl<D: Digest> PartialOrd for Checksum<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A [`Checksum`] that may be skipped.
///
/// Strings representing checksums are used to verify the integrity of files.
/// If the `"SKIP"` keyword is found, the integrity check is skipped.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SkippableChecksum<D: DigestString + Clone> {
    /// Sourcefile checksum validation may be skipped, which is expressed with this variant.
    Skip,
    /// The related source file should be validated via the provided checksum.
    #[serde(bound = "D: Digest + Clone")]
    Checksum {
        /// The checksum to be used for the validation.
        digest: Checksum<D>,
    },
}

impl<D: DigestString + Clone> SkippableChecksum<D> {
    /// Determines whether the [`SkippableChecksum`] is skipped.
    ///
    /// Checksums are considered skipped if they are of the variant [`SkippableChecksum::Skip`].
    pub fn is_skipped(&self) -> bool {
        matches!(self, SkippableChecksum::Skip)
    }

    /// Recognizes a [`SkippableChecksum`] from a string slice.
    ///
    /// Consumes all its input.
    /// See [`SkippableChecksum::from_str`], [`Checksum::parser`] and [`Checksum::from_str`].
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is not the output of a _hash function_
    /// in hexadecimal (or decimal in case of CRC32/CKSUM) form.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        terminated(
            alt((
                "SKIP".value(Self::Skip),
                Checksum::parser.map(|digest| Self::Checksum { digest }),
            )),
            cut_err(eof).context(StrContext::Expected(StrContextValue::Description(
                "end of checksum.",
            ))),
        )
        .parse_next(input)
    }
}

impl<D: DigestString + Clone> FromStr for SkippableChecksum<D> {
    type Err = Error;
    /// Create a new [`SkippableChecksum`] from a string slice and return it in a Result.
    ///
    /// First checks for the special `SKIP` keyword, before trying [`Checksum::from_str`].
    ///
    /// Delegates to [`SkippableChecksum::parser`].
    ///
    /// ## Examples
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_types::{SkippableChecksum, digests::Sha256};
    ///
    /// assert!(SkippableChecksum::<Sha256>::from_str("SKIP").is_ok());
    /// assert!(
    ///     SkippableChecksum::<Sha256>::from_str(
    ///         "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"
    ///     )
    ///     .is_ok()
    /// );
    /// ```
    fn from_str(s: &str) -> Result<SkippableChecksum<D>, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl<D: DigestString + Clone> Display for SkippableChecksum<D> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let output = match self {
            SkippableChecksum::Skip => "SKIP".to_string(),
            SkippableChecksum::Checksum { digest } => digest.to_string(),
        };
        write!(fmt, "{output}",)
    }
}

impl<D: DigestString + Clone> PartialEq for SkippableChecksum<D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SkippableChecksum::Skip, SkippableChecksum::Skip) => true,
            (SkippableChecksum::Skip, SkippableChecksum::Checksum { .. }) => false,
            (SkippableChecksum::Checksum { .. }, SkippableChecksum::Skip) => false,
            (
                SkippableChecksum::Checksum { digest },
                SkippableChecksum::Checksum {
                    digest: digest_other,
                },
            ) => digest == digest_other,
        }
    }
}

/// CRC-32/CKSUM hasher state.
///
/// This implementation tracks the length of the input data and appends it to the checksum
/// calculation similarly to the Unix `cksum` utility.
#[derive(Clone, Debug)]
pub struct Crc32Cksum {
    digest: crc_fast::Digest,
    len: u64,
}

impl HashMarker for Crc32Cksum {}

impl Default for Crc32Cksum {
    fn default() -> Self {
        Self {
            digest: crc_fast::Digest::new(crc_fast::CrcAlgorithm::Crc32Cksum),
            len: 0,
        }
    }
}

impl Update for Crc32Cksum {
    fn update(&mut self, data: &[u8]) {
        self.digest.update(data);
        self.len += data.len() as u64;
    }
}

impl OutputSizeUser for Crc32Cksum {
    type OutputSize = digest::consts::U4;
}

impl FixedOutput for Crc32Cksum {
    fn finalize_into(mut self, out: &mut Output<Self>) {
        if self.len != 0 {
            let len_bytes = self.len.to_be_bytes();

            // Skip leading zero bytes and append the length to the digest...
            let start = len_bytes.iter().position(|&b| b != 0).unwrap_or(7);
            self.digest.update(&len_bytes[start..]);
        }

        let crc = self.digest.finalize() as u32;
        out.deref_mut().clone_from_slice(&crc.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_checksum_blake2b512_from_string(string in r"[a-f0-9]{128}") {
            prop_assert_eq!(&string, &format!("{}", Blake2b512Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_blake2b512_bigger_size(string in r"[a-f0-9]{129}") {
            assert!(Blake2b512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_blake2b512_smaller_size(string in r"[a-f0-9]{127}") {
            assert!(Blake2b512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_blake2b512_wrong_chars(string in r"[e-z0-9]{128}") {
            assert!(Blake2b512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha1_from_string(string in r"[a-f0-9]{40}") {
            prop_assert_eq!(&string, &format!("{}", Sha1Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha1_from_string_bigger_size(string in r"[a-f0-9]{41}") {
            assert!(Sha1Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha1_from_string_smaller_size(string in r"[a-f0-9]{39}") {
            assert!(Sha1Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha1_from_string_wrong_chars(string in r"[e-z0-9]{40}") {
            assert!(Sha1Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha224_from_string(string in r"[a-f0-9]{56}") {
            prop_assert_eq!(&string, &format!("{}", Sha224Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha224_from_string_bigger_size(string in r"[a-f0-9]{57}") {
            assert!(Sha224Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha224_from_string_smaller_size(string in r"[a-f0-9]{55}") {
            assert!(Sha224Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha224_from_string_wrong_chars(string in r"[e-z0-9]{56}") {
            assert!(Sha224Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha256_from_string(string in r"[a-f0-9]{64}") {
            prop_assert_eq!(&string, &format!("{}", Sha256Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha256_from_string_bigger_size(string in r"[a-f0-9]{65}") {
            assert!(Sha256Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha256_from_string_smaller_size(string in r"[a-f0-9]{63}") {
            assert!(Sha256Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha256_from_string_wrong_chars(string in r"[e-z0-9]{64}") {
            assert!(Sha256Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha384_from_string(string in r"[a-f0-9]{96}") {
            prop_assert_eq!(&string, &format!("{}", Sha384Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha384_from_string_bigger_size(string in r"[a-f0-9]{97}") {
            assert!(Sha384Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha384_from_string_smaller_size(string in r"[a-f0-9]{95}") {
            assert!(Sha384Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha384_from_string_wrong_chars(string in r"[e-z0-9]{96}") {
            assert!(Sha384Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha512_from_string(string in r"[a-f0-9]{128}") {
            prop_assert_eq!(&string, &format!("{}", Sha512Checksum::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha512_from_string_bigger_size(string in r"[a-f0-9]{129}") {
            assert!(Sha512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha512_from_string_smaller_size(string in r"[a-f0-9]{127}") {
            assert!(Sha512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha512_from_string_wrong_chars(string in r"[e-z0-9]{128}") {
            assert!(Sha512Checksum::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_crc32cksum(sum in 0u32..=u32::MAX) {
            let decimal_str = format!("{sum}");
            prop_assert_eq!(
                &decimal_str,
                &format!("{}", Crc32CksumChecksum::from_str(decimal_str.as_str()).unwrap())
            );
        }

        #[test]
        fn invalid_checksum_crc32cksum_bigger_size(sum in (u32::MAX as u128)..=u128::MAX) {
            let decimal_str = format!("{sum}");
            assert!(Crc32CksumChecksum::from_str(decimal_str.as_str()).is_err());
        }

        #[test]
        fn invalid_checksum_crc32cksum_wrong_chars(string in r"[a-f]{9}") {
            assert!(Crc32CksumChecksum::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_crc32cksum_negative(string in r"-[1-9]{9}") {
            assert!(Crc32CksumChecksum::from_str(&string).is_err());
        }
    }

    #[rstest]
    fn checksum_blake2b512() {
        let data = "foo\n";
        let digest = vec![
            210, 2, 215, 149, 29, 242, 196, 183, 17, 202, 68, 180, 188, 201, 215, 179, 99, 250, 66,
            82, 18, 126, 5, 140, 26, 145, 14, 192, 91, 108, 208, 56, 215, 28, 194, 18, 33, 192, 49,
            192, 53, 159, 153, 62, 116, 107, 7, 245, 150, 92, 248, 197, 195, 116, 106, 88, 51, 122,
            217, 171, 101, 39, 142, 119,
        ];
        let hex_digest = "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77";

        let checksum = Blake2b512Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Blake2b512Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);
    }

    #[rstest]
    fn checksum_sha1() {
        let data = "foo\n";
        let digest = vec![
            241, 210, 210, 249, 36, 233, 134, 172, 134, 253, 247, 179, 108, 148, 188, 223, 50, 190,
            236, 21,
        ];
        let hex_digest = "f1d2d2f924e986ac86fdf7b36c94bcdf32beec15";

        let checksum = Sha1Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Sha1Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);
    }

    #[rstest]
    fn checksum_sha224() {
        let data = "foo\n";
        let digest = vec![
            231, 213, 227, 110, 141, 71, 12, 62, 81, 3, 254, 221, 46, 79, 42, 165, 195, 10, 178,
            127, 102, 41, 189, 195, 40, 111, 157, 210,
        ];
        let hex_digest = "e7d5e36e8d470c3e5103fedd2e4f2aa5c30ab27f6629bdc3286f9dd2";

        let checksum = Sha224Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Sha224Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);
    }

    #[rstest]
    fn checksum_sha256() {
        let data = "foo\n";
        let digest = vec![
            181, 187, 157, 128, 20, 160, 249, 177, 214, 30, 33, 231, 150, 215, 141, 204, 223, 19,
            82, 242, 60, 211, 40, 18, 244, 133, 11, 135, 138, 228, 148, 76,
        ];
        let hex_digest = "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c";

        let checksum = Sha256Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Sha256Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);
    }

    #[rstest]
    fn checksum_sha384() {
        let data = "foo\n";
        let digest = vec![
            142, 255, 218, 191, 225, 68, 22, 33, 74, 37, 15, 147, 85, 5, 37, 11, 217, 145, 241, 6,
            6, 93, 137, 157, 182, 225, 155, 220, 139, 246, 72, 243, 172, 15, 25, 53, 196, 246, 95,
            232, 247, 152, 40, 155, 26, 13, 30, 6,
        ];
        let hex_digest = "8effdabfe14416214a250f935505250bd991f106065d899db6e19bdc8bf648f3ac0f1935c4f65fe8f798289b1a0d1e06";

        let checksum = Sha384Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Sha384Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);
    }

    #[rstest]
    fn checksum_sha512() {
        let data = "foo\n";
        let digest = vec![
            12, 249, 24, 10, 118, 74, 186, 134, 58, 103, 182, 215, 47, 9, 24, 188, 19, 28, 103,
            114, 100, 44, 178, 220, 229, 163, 79, 10, 112, 47, 148, 112, 221, 194, 191, 18, 92, 18,
            25, 139, 25, 149, 194, 51, 195, 75, 74, 253, 52, 108, 84, 162, 51, 76, 53, 10, 148,
            138, 81, 182, 232, 180, 230, 182,
        ];
        let hex_digest = "0cf9180a764aba863a67b6d72f0918bc131c6772642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6";

        let checksum = Sha512Checksum::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest);

        let checksum = Sha512Checksum::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest);
    }

    #[rstest]
    fn checksum_crc32cksum() {
        let data = "foo\n";
        let digest = 3915528286u32;
        let digest_string = format!("{digest}");

        let checksum = Crc32CksumChecksum::calculate_from(data);
        assert_eq!(digest.to_be_bytes(), checksum.inner());
        assert_eq!(format!("{}", &checksum), digest_string);

        let checksum = Crc32CksumChecksum::from_str(digest_string.as_str()).unwrap();
        assert_eq!(digest.to_be_bytes(), checksum.inner());
        assert_eq!(format!("{}", &checksum), digest_string);
    }

    #[rstest]
    #[case::non_hex_digits(
        "0cf9180a764aba863a67b6d72f0918bc13gggggg642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6",
        "expected ASCII hex digit"
    )]
    #[case::incomplete_pair(" b ", "expected ASCII hex digit")]
    #[case::incomplete_digest("0cf9180a764aba863a67b6d72f0918bca", "expected ASCII hex digit")]
    #[case::whitespace(
        "d2 02 d7 95 1d f2 c4 b7 11 ca 44 b4 bc c9 d7 b3 63 fa 42 52 12 7e 05 8c 1a 91 0e c0 5b 6c d0 38 d7 1c c2 12 21 c0 31 c0 35 9f 99 3e 74 6b 07 f5 96 5c f8 c5 c3 74 6a 58 33 7a d9 ab 65 27 8e 77",
        "expected ASCII hex digit"
    )]
    fn checksum_parse_error(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = Sha512Checksum::from_str(input) else {
            panic!("'{input}' did not fail to parse as expected")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    #[rstest]
    fn skippable_checksum_sha256() {
        let hex_digest = "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c";
        let checksum = SkippableChecksum::<Sha256>::from_str(hex_digest).unwrap();
        assert_eq!(format!("{}", &checksum), hex_digest);
    }

    #[rstest]
    fn skippable_checksum_skip() {
        let hex_digest = "SKIP";
        let checksum = SkippableChecksum::<Sha256>::from_str(hex_digest).unwrap();

        assert_eq!(SkippableChecksum::Skip, checksum);
        assert_eq!(format!("{}", &checksum), hex_digest);
    }
}
