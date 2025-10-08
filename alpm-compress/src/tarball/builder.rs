use std::{fmt, fmt::Debug, fs::File};

use tar::Builder;

use crate::{
    Error,
    compression::{CompressionEncoder, CompressionSettings},
};

/// TODO docs
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
    /// TODO docs
    pub fn new(file: File, settings: &CompressionSettings) -> Result<Self, Error> {
        CompressionEncoder::new(file, settings).map(Self::from)
    }

    /// TODO docs
    pub fn inner_mut(&mut self) -> &mut Builder<CompressionEncoder<'c>> {
        &mut self.inner
    }

    /// TODO docs
    pub fn finish(self) -> Result<(), Error> {
        self.inner.into_inner().map_err(|source| Error::IoWrite {
            context: "finishing the tarball",
            source,
        })?;
        Ok(())
    }
}

impl<'c> From<CompressionEncoder<'c>> for TarballBuilder<'c> {
    fn from(encoder: CompressionEncoder<'c>) -> Self {
        Self {
            inner: Builder::new(encoder),
        }
    }
}
