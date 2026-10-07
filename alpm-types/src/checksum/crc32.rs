//! A `cksum` compatible CRC-32 Hasher.
//!
//! The actual CRC calculation is done by the [crc] crate. [`Crc32Cksum`] adds special handling used
//! by `cksum`. On top of it, this module provides all traits necessary for `Digest`, which is
//! needed for compatibility with the other hashing algorithms.
//!
//! `makepkg` still supports `cksum`'s as a legacy checksum algorithm, which we sadly have to
//! support for backwards compatibility. In practice, nobody except a few packages in the AUR use
//! this any longer.

use std::{fmt::Formatter, ops::DerefMut};

use crc::{CRC_32_CKSUM, Crc, Digest};
use digest::{FixedOutput, HashMarker, Output, OutputSizeUser, Update};

/// The CRC-32/CKSUM algorithm
static CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

/// A [`cksum`] compatible CRC-32 hasher.
///
/// This tracks the length of the input data and appends it to the checksum calculation, just like
/// the Unix `cksum` utility does.
///
/// [`cksum`]: https://man.archlinux.org/man/cksum.1
#[derive(Clone)]
pub struct Crc32Cksum {
    /// The ongoing CRC-32/CKSUM calculation.
    digest: Digest<'static, u32>,
    /// The number of bytes that have been fed into `digest` so far.
    len: u64,
}

impl Default for Crc32Cksum {
    fn default() -> Self {
        Self {
            digest: CRC.digest(),
            len: 0,
        }
    }
}

impl std::fmt::Debug for Crc32Cksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Crc32Cksum")
            .field("len", &self.len)
            .finish()
    }
}

impl HashMarker for Crc32Cksum {}

impl Update for Crc32Cksum {
    /// Update the digest with a new betch of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the input data exceeds ~18.44 exabytes on systems with `usize > 64bits`.
    // TODO(cleanup): Investigate the arithmetic_side_effects
    #[expect(clippy::expect_used, clippy::arithmetic_side_effects)]
    fn update(&mut self, data: &[u8]) {
        self.digest.update(data);
        self.len += u64::try_from(data.len())
            .expect("the number of bytes in the input slice fit into a u64");
    }
}

impl OutputSizeUser for Crc32Cksum {
    type OutputSize = digest::consts::U4;
}

impl FixedOutput for Crc32Cksum {
    fn finalize_into(mut self, out: &mut Output<Self>) {
        // Feed the length of the input as octets into the digest, with its least significant octet
        // first. The smallest amount of non-zero octets is to be used.
        // Apparently this is used to differentiate between some outputs that would otherwise result
        // in the same checksum.
        //
        // See the `cksum` specification for details, specifically the end of the `1.` paragraph:
        // https://man.archlinux.org/man/cksum.1p#DESCRIPTION
        let mut len = self.len;
        while len != 0 {
            self.digest.update(&[len as u8]);
            len >>= 8;
        }

        let crc = self.digest.finalize();
        out.deref_mut().clone_from_slice(&crc.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    use rstest::rstest;
    use testresult::TestResult;
    use which::which;

    use crate::Crc32CksumChecksum;

    /// Ensures that our [`Crc32CksumChecksum`] implementation produces the same digests as the
    /// `cksum` binary from coreutils.
    #[rstest]
    #[case::empty(Vec::new())]
    #[case::single_byte(b"a".to_vec())]
    #[case::utf8("ÄÖÜ äöü ß 🦆:3".as_bytes().to_vec())]
    #[case::really_long_input(vec![b'a'; 5000])]
    fn checksum_matches_cksum(#[case] data: Vec<u8>) -> TestResult {
        let cksum = which("cksum").unwrap_or_else(|_| {
            panic!("cksum: command not found");
        });

        let output = {
            let mut child = Command::new(cksum)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            // Send the input to the `cksum`'s stdin.
            child
                .stdin
                .take()
                .expect("the stdin of cksum to be piped")
                .write_all(&data)?;

            child.wait_with_output()?
        };
        assert!(
            output.status.success(),
            "cksum exited with {}",
            output.status
        );

        // `cksum` returns "{digest} {input_bytes}" when reading from stdin.
        // We simply split by space to get the digest
        let stdout = String::from_utf8_lossy(&output.stdout);
        let digest: &str = stdout
            .split_whitespace()
            .next()
            .expect("cksum to print a digest");

        // Make sure both implementations create the same checksum.
        let checksum = Crc32CksumChecksum::calculate_from(&data);
        assert_eq!(&format!("{checksum}"), digest);

        Ok(())
    }
}
