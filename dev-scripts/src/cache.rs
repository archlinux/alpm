//! Directory handling.
use std::path::{Path, PathBuf};

use dirs::cache_dir;
use log::debug;

use crate::{
    Error,
    consts::{PROJECT_NAME, TESTING_DIR},
};

/// The directory in which all downloads and test artifacts reside.
#[derive(Clone, Debug)]
pub struct CacheDir(PathBuf);

impl CacheDir {
    /// Creates a new [`CacheDir`] from XDG default location.
    ///
    /// Defaults to `$XDG_CACHE_HOME/alpm/testing`.
    /// If `$XDG_CACHE_HOME` is unset, falls back to `~/.cache/alpm/testing`.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory for the current user cannot be retrieved.
    pub fn from_xdg() -> Result<Self, Error> {
        let path = cache_dir()
            .ok_or(Error::CannotGetCacheDir)?
            .join(PROJECT_NAME)
            .join(TESTING_DIR);
        debug!("Using path {path:?} as cache dir.");

        Ok(Self(path))
    }
}

impl From<PathBuf> for CacheDir {
    fn from(value: PathBuf) -> Self {
        debug!("Using path {value:?} as cache dir.");

        Self(value)
    }
}

impl AsRef<Path> for CacheDir {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}
