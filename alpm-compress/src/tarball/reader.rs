use std::{
    fmt,
    fmt::Debug,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use tar::{Archive, Entries, Entry, EntryType};

use crate::{
    Error,
    decompression::{CompressionDecoder, DecompressionSettings},
};

/// TODO docs
pub struct TarballReader<'c> {
    archive: Archive<CompressionDecoder<'c>>,
}

impl Debug for TarballReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TarballReader")
            .field("archive", &"Archive<CompressionDecoder>")
            .finish()
    }
}

impl<'c> TarballReader<'c> {
    /// TODO docs
    pub fn new(decoder: CompressionDecoder<'c>) -> Self {
        Self {
            archive: Archive::new(decoder),
        }
    }

    /// TODO docs
    pub fn entries<'a>(&'a mut self) -> Result<TarballEntries<'a, 'c>, Error> {
        let raw_entries = self.archive.entries().map_err(|source| Error::IoRead {
            context: "reading archive entries",
            source,
        })?;
        Ok(raw_entries.into())
    }

    /// TODO docs
    pub fn read_entry<'a, P: AsRef<Path>>(
        &'a mut self,
        path: P,
    ) -> Result<Option<TarballEntry<'a, 'c>>, Error> {
        for entry in self.entries()? {
            let entry = entry?;
            if entry.path() == path.as_ref() {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }
}

impl TryFrom<&Path> for TarballReader<'_> {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(|source| Error::IoRead {
            context: "opening archive for reading",
            source,
        })?;
        let settings = DecompressionSettings::try_from(path)?;
        let decoder = CompressionDecoder::new(file, settings)?;
        Ok(Self::new(decoder))
    }
}

impl TryFrom<PathBuf> for TarballReader<'_> {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

/// An entry in a tarball.
pub struct TarballEntry<'a, 'c> {
    /// The path of the entry in the archive.
    path: PathBuf,
    /// The raw tar entry.
    entry: Entry<'a, CompressionDecoder<'c>>,
}

impl Debug for TarballEntry<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TarballEntry")
            .field("path", &self.path)
            .field("entry", &"tar::Entry<CompressionDecoder>")
            .finish()
    }
}

impl<'a, 'c> TarballEntry<'a, 'c> {
    /// Returns the path of the entry in the archive.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the content of the entry.
    ///
    /// # Errors
    ///
    /// Returns an error if [`Entry::read_to_end`] fails.
    pub fn content(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        self.entry
            .read_to_end(&mut buffer)
            .map_err(|source| crate::Error::IoRead {
                context: "reading archive entry content",
                source,
            })?;
        Ok(buffer)
    }

    /// Checks whether the [`TarballEntry`] represents a directory.
    ///
    /// Returns `true` if the [`TarballEntry`] represents a directory, `false` otherwise.
    ///
    /// # Note
    ///
    /// This is a convenience method for comparing the [`EntryType`] of the [`Entry::header`]
    /// contained in the [`TarballEntry`] with [`EntryType::Directory`].
    pub fn is_dir(&self) -> bool {
        self.entry.header().entry_type() == EntryType::Directory
    }

    /// Checks whether the [`TarballEntry`] represents a regular file.
    ///
    /// Returns `true` if the [`TarballEntry`] represents a regular file, `false` otherwise.
    ///
    /// # Note
    ///
    /// This is a convenience method for comparing the [`EntryType`] of the [`Entry::header`]
    /// contained in the [`TarballEntry`] with [`EntryType::Regular`].
    pub fn is_file(&self) -> bool {
        self.entry.header().entry_type() == EntryType::Regular
    }

    /// Checks whether the [`TarballEntry`] represents a symlink.
    ///
    /// Returns `true` if the [`TarballEntry`] represents a symlink, `false` otherwise.
    ///
    /// # Note
    ///
    /// This is a convenience method for comparing the [`EntryType`] of the [`Entry::header`]
    /// contained in the [`TarballEntry`] with [`EntryType::Symlink`].
    pub fn is_symlink(&self) -> bool {
        self.entry.header().entry_type() == EntryType::Symlink
    }

    /// Returns the access permissions that apply for the [`TarballEntry`].
    ///
    /// # Notes
    ///
    /// - This is a convenience method for retrieving the mode of the [`Entry::header`] contained in
    ///   the [`TarballEntry`].
    /// - It returns the mode masked with `0o7777` to ensure only the permission bits are returned.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving the mode from the entry's header fails.
    pub fn permissions(&self) -> Result<u32, Error> {
        Ok(self.entry.header().mode().map_err(|source| Error::IoRead {
            context: "retrieving permissions of archive entry",
            source,
        })? & 0o7777)
    }

    /// Returns a reference to the underlying tar [`Entry`].
    ///
    /// This is useful for accessing metadata of the entry, such as its header or path.
    pub fn raw(&self) -> &Entry<'a, CompressionDecoder<'c>> {
        &self.entry
    }
}

impl Read for TarballEntry<'_, '_> {
    /// Reads data from the entry into the provided buffer.
    ///
    /// Delegates to [`Entry::read`].
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the entry fails.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.entry.read(buf)
    }
}

/// TODO docs
pub struct TarballEntries<'a, 'c> {
    inner: Entries<'a, CompressionDecoder<'c>>,
}

impl Debug for TarballEntries<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TarballEntries")
            .field("inner", &"Entries<CompressionDecoder>")
            .finish()
    }
}

impl<'a, 'c> Iterator for TarballEntries<'a, 'c> {
    type Item = Result<TarballEntry<'a, 'c>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|entry| {
            let entry = entry.map_err(|source| Error::IoRead {
                context: "reading archive entry",
                source,
            })?;

            let path = entry
                .path()
                .map_err(|source| Error::IoRead {
                    context: "retrieving path of archive entry",
                    source,
                })?
                .to_path_buf();

            Ok(TarballEntry { path, entry })
        })
    }
}

impl<'a, 'c> From<Entries<'a, CompressionDecoder<'c>>> for TarballEntries<'a, 'c> {
    fn from(inner: Entries<'a, CompressionDecoder<'c>>) -> Self {
        Self { inner }
    }
}
