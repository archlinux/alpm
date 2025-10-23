//! Creation of tarballs.

use std::{fmt, fmt::Debug, fs::File};

use fluent_i18n::t;
use tar::Builder;

use crate::{
    Error,
    compression::{CompressionEncoder, CompressionSettings},
};

/// Wraps a [`Builder`] that writes to a [`CompressionEncoder`].
///
/// As [`CompressionEncoder`] has an uncompressed variant, this can be used to create
/// either compressed tarballs `.tar.*` or uncompressed tar archives `.tar`.
pub struct TarballBuilder<'c> {
    inner: Builder<CompressionEncoder<'c>>,
}

impl Debug for TarballBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TarballBuilder")
            .field("inner", &"Builder<CompressionEncoder>")
            .finish()
    }
}

impl<'c> TarballBuilder<'c> {
    /// Creates a new [`TarballBuilder`] that writes to the given [`File`] with the given
    /// [`CompressionSettings`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`CompressionEncoder`] initialization fails.
    pub fn new(file: File, settings: &CompressionSettings) -> Result<Self, Error> {
        CompressionEncoder::new(file, settings).map(Self::from)
    }

    /// Returns a mutable reference to the inner [`Builder`].
    ///
    /// This can be used to set options to the builder or append files to the tarball.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tempfile::{tempfile, NamedTempFile};
    /// # use alpm_compress::{tarball::TarballBuilder, compression::CompressionSettings};
    /// # use testresult::TestResult;
    /// # fn main() -> TestResult {
    /// # let mut builder = TarballBuilder::new(tempfile()?, &CompressionSettings::None)?;
    /// builder.inner_mut().follow_symlinks(false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn inner_mut(&mut self) -> &mut Builder<CompressionEncoder<'c>> {
        &mut self.inner
    }

    /// Finishes writing the tarball.
    ///
    /// Delegates to [`CompressionEncoder::finish`] of the inner [`Builder`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`CompressionEncoder`] fails to finish the compression stream.
    pub fn finish(self) -> Result<(), Error> {
        self.inner
            .into_inner()
            .map_err(|source| Error::IoWrite {
                context: t!("error-io-write-archive"),
                source,
            })?
            .finish()?;
        Ok(())
    }
}

impl<'c> From<CompressionEncoder<'c>> for TarballBuilder<'c> {
    /// Creates a [`TarballBuilder`] from a [`CompressionEncoder`].
    fn from(encoder: CompressionEncoder<'c>) -> Self {
        Self {
            inner: Builder::new(encoder),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use rstest::rstest;
    use tempfile::{NamedTempFile, tempfile};
    use testresult::TestResult;

    use super::*;
    use crate::compression::{
        Bzip2CompressionLevel,
        CompressionSettings,
        GzipCompressionLevel,
        XzCompressionLevel,
        ZstdCompressionLevel,
        ZstdThreads,
    };

    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() })]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() })]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default() })]
    #[case::zstd(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::all() })]
    #[case::no_compression(CompressionSettings::None)]
    fn test_tarball_builder_write_file(
        #[case] compression_settings: CompressionSettings,
    ) -> TestResult {
        let mut builder = TarballBuilder::new(tempfile()?, &compression_settings)?;
        let test_file = NamedTempFile::new()?;
        {
            let mut f = test_file.reopen()?;
            f.write_all(b"alpm4ever")?;
            f.flush()?;
        }

        builder
            .inner_mut()
            .append_path_with_name(test_file.path(), "testfile")?;
        builder.finish()?;

        Ok(())
    }

    #[rstest]
    #[case::bzip2(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() })]
    #[case::gzip(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() })]
    #[case::xz(CompressionSettings::Xz { compression_level: XzCompressionLevel::default() })]
    #[case::zstd(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::all() })]
    #[case::no_compression(CompressionSettings::None)]
    fn test_tarball_builder_debug(#[case] compression_settings: CompressionSettings) -> TestResult {
        let builder = TarballBuilder::new(tempfile()?, &compression_settings)?;
        let dbg = format!("{:?}", builder);
        assert!(dbg.contains("TarballBuilder"));
        Ok(())
    }
}
