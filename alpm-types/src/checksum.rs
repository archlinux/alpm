use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

use digest::Digest;

use crate::Error;

/// A checksum using a supported algorithm
///
/// Checksums are created using one of the supported algorithms:
/// - `Blake2b512`
/// - `Md5` (WARNING: Use is highly discouraged, because it is cryptographically unsafe)
/// - `Sha1` (WARNING: Use is highly discouraged, because it is cryptographically unsafe)
/// - `Sha224`
/// - `Sha256`
/// - `Sha384`
/// - `Sha512`
///
/// NOTE: Contrary to makepkg/pacman, this crate *does not* support using cksum-style CRC-32 as it
/// is non-standard (different implementations throughout libraries) and cryptographically unsafe.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
/// use alpm_types::{digests::Blake2b512, Checksum};
///
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
/// let checksum = Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77").unwrap();
/// assert_eq!(checksum.inner(), digest);
/// ```
#[derive(Debug, Clone)]
pub struct Checksum<D: Digest> {
    digest: Vec<u8>,
    _marker: PhantomData<*const D>,
}

impl<D: Digest> Checksum<D> {
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
}

impl<D: Digest> FromStr for Checksum<D> {
    type Err = Error;
    /// Create a new Checksum from a hex string and return it in a Result
    ///
    /// All whitespaces are removed from the input and it is processed as a lowercase string.
    /// An Error is returned, if the input length does not match the output size for the given
    /// supported algorithm, or if the provided hex string could not be converted to a list of
    /// bytes.
    ///
    /// ## Examples
    /// ```
    /// use std::str::FromStr;
    /// use alpm_types::{digests::Blake2b512, Checksum};
    ///
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77").is_ok());
    /// assert!(Checksum::<Blake2b512>::from_str("d2 02 d7 95 1d f2 c4 b7 11 ca 44 b4 bc c9 d7 b3 63 fa 42 52 12 7e 05 8c 1a 91 0e c0 5b 6c d0 38 d7 1c c2 12 21 c0 31 c0 35 9f 99 3e 74 6b 07 f5 96 5c f8 c5 c3 74 6a 58 33 7a d9 ab 65 27 8e 77").is_ok());
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e7").is_err());
    /// assert!(Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e7x").is_err());
    /// ```
    fn from_str(input: &str) -> Result<Checksum<D>, Self::Err> {
        let input = input.replace(' ', "").to_lowercase();
        // the input does not have the correct length
        if input.len() != <D as Digest>::output_size() * 2 {
            return Err(Error::IncorrectLength {
                length: input.len(),
                expected: <D as Digest>::output_size() * 2,
            });
        }

        let mut digest = vec![];
        for i in (0..input.len()).step_by(2) {
            let src = &input[i..i + 2];
            match u8::from_str_radix(src, 16) {
                Ok(byte) => digest.push(byte),
                Err(e) => {
                    return Err(Error::InvalidInteger {
                        kind: e.kind().clone(),
                    })
                }
            }
        }

        Ok(Checksum {
            digest,
            _marker: PhantomData,
        })
    }
}

impl<D: Digest> Display for Checksum<D> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            self.digest
                .iter()
                .map(|x| format!("{:02x?}", x))
                .collect::<Vec<String>>()
                .join("")
        )
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::digests::Blake2b512;
    use crate::digests::Sha1;
    use crate::digests::Sha224;
    use crate::digests::Sha256;
    use crate::digests::Sha384;
    use crate::digests::Sha512;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_checksum_blake2b512_from_string(string in r"[a-f0-9]{128}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Blake2b512>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_blake2b512_bigger_size(string in r"[a-f0-9]{129}") {
            assert!(Checksum::<Blake2b512>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_blake2b512_smaller_size(string in r"[a-f0-9]{127}") {
            assert!(Checksum::<Blake2b512>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_blake2b512_wrong_chars(string in r"[e-z0-9]{128}") {
            assert!(Checksum::<Blake2b512>::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha1_from_string(string in r"[a-f0-9]{40}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Sha1>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha1_from_string_bigger_size(string in r"[a-f0-9]{41}") {
            assert!(Checksum::<Sha1>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha1_from_string_smaller_size(string in r"[a-f0-9]{39}") {
            assert!(Checksum::<Sha1>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha1_from_string_wrong_chars(string in r"[e-z0-9]{40}") {
            assert!(Checksum::<Sha1>::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha224_from_string(string in r"[a-f0-9]{56}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Sha224>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha224_from_string_bigger_size(string in r"[a-f0-9]{57}") {
            assert!(Checksum::<Sha224>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha224_from_string_smaller_size(string in r"[a-f0-9]{55}") {
            assert!(Checksum::<Sha224>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha224_from_string_wrong_chars(string in r"[e-z0-9]{56}") {
            assert!(Checksum::<Sha224>::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha256_from_string(string in r"[a-f0-9]{64}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Sha256>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha256_from_string_bigger_size(string in r"[a-f0-9]{65}") {
            assert!(Checksum::<Sha256>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha256_from_string_smaller_size(string in r"[a-f0-9]{63}") {
            assert!(Checksum::<Sha256>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha256_from_string_wrong_chars(string in r"[e-z0-9]{64}") {
            assert!(Checksum::<Sha256>::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha384_from_string(string in r"[a-f0-9]{96}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Sha384>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha384_from_string_bigger_size(string in r"[a-f0-9]{97}") {
            assert!(Checksum::<Sha384>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha384_from_string_smaller_size(string in r"[a-f0-9]{95}") {
            assert!(Checksum::<Sha384>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha384_from_string_wrong_chars(string in r"[e-z0-9]{96}") {
            assert!(Checksum::<Sha384>::from_str(&string).is_err());
        }

        #[test]
        fn valid_checksum_sha512_from_string(string in r"[a-f0-9]{128}") {
            prop_assert_eq!(&string, &format!("{}", Checksum::<Sha512>::from_str(&string).unwrap()));
        }

        #[test]
        fn invalid_checksum_sha512_from_string_bigger_size(string in r"[a-f0-9]{129}") {
            assert!(Checksum::<Sha512>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha512_from_string_smaller_size(string in r"[a-f0-9]{127}") {
            assert!(Checksum::<Sha512>::from_str(&string).is_err());
        }

        #[test]
        fn invalid_checksum_sha512_from_string_wrong_chars(string in r"[e-z0-9]{128}") {
            assert!(Checksum::<Sha512>::from_str(&string).is_err());
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

        let checksum = Checksum::<Blake2b512>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Checksum::<Blake2b512>::from_str(hex_digest).unwrap();
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

        let checksum = Checksum::<Sha1>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Checksum::<Sha1>::from_str(hex_digest).unwrap();
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

        let checksum = Checksum::<Sha224>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Checksum::<Sha224>::from_str(hex_digest).unwrap();
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

        let checksum = Checksum::<Sha256>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Checksum::<Sha256>::from_str(hex_digest).unwrap();
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

        let checksum = Checksum::<Sha384>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest,);

        let checksum = Checksum::<Sha384>::from_str(hex_digest).unwrap();
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

        let checksum = Checksum::<Sha512>::calculate_from(data);
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest);

        let checksum = Checksum::<Sha512>::from_str(hex_digest).unwrap();
        assert_eq!(digest, checksum.inner());
        assert_eq!(format!("{}", &checksum), hex_digest);
    }
}
