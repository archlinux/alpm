//! Soname-based dependency and provision detection.

use std::{
    fs::{File, read_dir},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_package::InputDir;
use alpm_types::{Soname, SonameLookupDirectory, SonameV2};
use lddtree::DependencyAnalyzer;
use log::debug;
use walkdir::WalkDir;

use crate::{Error, elf::read_elf};

/// Options for [`SonameDetection`].
///
/// Controls how soname-based dependency and provision scanning is performed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SonameDetectionOptions {
    path: InputDir,
    lookup_dir: SonameLookupDirectory,
    provides: bool,
    depends: bool,
}

impl SonameDetectionOptions {
    /// Creates a new [`SonameDetectionOptions`].
    ///
    /// Accepts a `path`, that specifies the package input directory to read from and a `lookup_dir`
    /// specifying the subdirectory in `path` below which **soname** data is detected.
    pub fn new(path: InputDir, lookup_dir: SonameLookupDirectory) -> Self {
        Self {
            path,
            lookup_dir,
            provides: true,
            depends: true,
        }
    }

    /// Sets the option for detecting run-time dependencies based on [soname] data.
    ///
    /// If `option` is `true` (the default) indicates that [`SonameDetection`] should detect any
    /// [alpm-package-relation] of type [run-time dependency]. If `option` is `false`, indicates
    /// that [`SonameDetection`] should not detect [alpm-package-relation] of type [run-time
    /// dependency].
    ///
    /// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
    /// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
    /// [run-time dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
    /// [soname]: https://en.wikipedia.org/wiki/Soname
    pub fn provides(mut self, option: bool) -> Self {
        self.provides = option;
        self
    }

    /// Sets the option for detecting provisions based on [soname] data.
    ///
    /// If `option` is `true` (the default) indicates that [`SonameDetection`] should detect any
    /// [alpm-package-relation] of type [provision].
    /// If `option` is `false`, indicates that [`SonameDetection`] should not detect
    /// [alpm-package-relation] of type [provision].
    ///
    /// [alpm-package-relation]:
    /// https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
    /// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
    /// [provision]:
    /// https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#provision
    /// [soname]: https://en.wikipedia.org/wiki/Soname
    pub fn depends(mut self, option: bool) -> Self {
        self.depends = option;
        self
    }
}

/// Detect dependencies and provisions based on [soname] data.
///
/// Based on a [`SonameDetectionOptions`] reads files in a subdirectory of an [`InputDir`] and based
/// on [soname] data in binaries and shared objects detects [run-time dependencies] and
/// [provisions]. The required **soname** data is retrieved from the dynamic section of relevant
/// [ELF] files in the package input directory. After initialization, the `provisions` field
/// contains a list of [`SonameV2`] objects that represent all sonames that the shared objects in
/// the package provide publicly. Further, the `dependencies` field contains a list of [`SonameV2`]
/// objects, that represent all dependencies towards sonames, that binaries and shared object files
/// in the package input directory rely upon during run-time.
///
/// # Note
///
/// A soname is only considered one of the [run-time dependencies] if the package input directory
/// does not also provide this soname.
///
/// [ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
/// [provisions]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#provision
/// [run-time dependencies]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
/// [soname]: https://en.wikipedia.org/wiki/Soname
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SonameDetection {
    /// The provides of a package.
    pub provides: Vec<SonameV2>,
    /// The depends of a package.
    pub depends: Vec<SonameV2>,
}

impl SonameDetection {
    /// Creates a new [`SonameDetection`] instance, calculating the provides and depends of a
    /// package.
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use std::path::PathBuf;
    /// use alpm_soname::{SonameDetection, SonameDetectionOptions};
    /// use alpm_types::SonameLookupDirectory;
    /// use alpm_package::InputDir;
    ///
    /// # fn main() -> Result<(), alpm_soname::Error> {
    /// // Directory containing a built pacman package:
    /// //    usr/lib/libalpm.so.15.0.0
    /// //    usr/lib/libalpm.so
    /// //    usr/lib/libalpm.so.15
    /// //    usr/bin/pacman
    /// let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_files/pacman"));
    /// let pacman = InputDir::new(path)?;
    ///
    /// // Set prefix to `lib` and the soname lookup directory to `/usr/lib`
    /// let lookup_dir = SonameLookupDirectory::from_str("lib:/usr/lib")?;
    ///
    /// // Create soname detection options
    /// let options = SonameDetectionOptions::new(pacman, lookup_dir);
    ///
    /// // Find providers and dependencies
    /// let soname_detection = SonameDetection::new(options)?;
    ///
    /// for provide in &soname_detection.provides {
    ///     println!("provide = {provide}");
    /// }
    /// for depend in &soname_detection.depends {
    ///     println!("depend = {depend}");
    /// }
    ///
    /// // Output:
    /// //    depends = lib:libarchive.so.13
    /// //    depends = lib:libc.so.6
    /// //    depends = lib:libcrypto.so.3
    /// //    depends = lib:libcurl.so.4
    /// //    depends = lib:libgpgme.so.45
    /// //    provide = lib:libalpm.so.15
    /// # let provides = soname_detection.provides.iter().map(|d| d.to_string()).collect::<Vec<_>>();
    /// # let provides = provides.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    /// # let depends = soname_detection.depends.iter().map(|d| d.to_string()).collect::<Vec<_>>();
    /// # let depends = depends.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    /// # assert_eq!(provides, vec!["lib:libalpm.so.15"]);
    /// # assert_eq!(depends, vec!["lib:libarchive.so.13", "lib:libc.so.6", "lib:libcrypto.so.3", "lib:libcurl.so.4", "lib:libgpgme.so.45"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(options: SonameDetectionOptions) -> Result<Self, Error> {
        let provides = if options.provides {
            Self::provisions(options.path.as_path(), options.lookup_dir.clone())?
        } else {
            Default::default()
        };
        let depends = if options.depends {
            Self::depends(options.path.as_path(), options.lookup_dir)?
        } else {
            Default::default()
        };

        Ok(Self { depends, provides })
    }

    /// Detects all soname-based **dependencies** in ELF binaries and shared objects.
    ///
    /// Only includes libraries not also provided within the same package.
    ///
    /// # Errors
    ///
    /// Returns an error if reading files or parsing ELF binaries fails.
    fn depends(path: &Path, lookup_dir: SonameLookupDirectory) -> Result<Vec<SonameV2>, Error> {
        let mut sonames = Vec::new();
        let mut buffer = Vec::new();

        let entries = WalkDir::new(path);

        for entry in entries {
            let entry = entry.map_err(|source| crate::Error::IoPathError {
                path: path.to_path_buf(),
                context: "read directory",
                source: source.into(),
            })?;

            // ensure is regular file
            if !entry.file_type().is_file() {
                continue;
            }

            let mut file =
                File::open(entry.path()).map_err(|source| crate::Error::IoPathError {
                    path: entry.path().to_path_buf(),
                    context: "read entry",
                    source,
                })?;

            let Some(elf) = read_elf(&mut file, &mut buffer)? else {
                continue;
            };

            if elf.libraries.is_empty() {
                continue;
            }

            let deps = DependencyAnalyzer::new(PathBuf::from(path))
                .analyze(entry.path())
                .map_err(|source| crate::Error::DependencyError {
                    path: entry.path().to_path_buf(),
                    source,
                })?;

            for lib in elf.libraries {
                // Only add the dependency if the library does not live in the package
                if let Some(dep) = deps.libraries.get(lib)
                    && dep.realpath.is_none()
                {
                    sonames.push(SonameV2 {
                        prefix: lookup_dir.prefix.clone(),
                        soname: Soname::from_str(lib)?,
                    });
                }
            }
        }

        sonames.sort();
        sonames.dedup();

        Ok(sonames)
    }

    /// Detects all soname-based **provisions** within shared libraries.
    ///
    /// Scans the soname lookup directory (e.g., `/usr/lib`) inside the package.
    ///
    /// # Errors
    ///
    /// Returns an error if a directory cannot be read or an ELF file cannot be parsed.
    fn provisions(path: &Path, lookup_dir: SonameLookupDirectory) -> Result<Vec<SonameV2>, Error> {
        let mut sonames = Vec::new();
        let mut buffer = Vec::new();
        let lib_dir = lookup_dir.directory.inner();
        let libdir = path.join(lib_dir.strip_prefix("/").unwrap_or(lib_dir));

        let Ok(entries) = read_dir(&libdir) else {
            return Ok(Default::default());
        };

        for entry in entries {
            let entry = entry.map_err(|source| crate::Error::IoPathError {
                path: libdir.to_path_buf(),
                context: "read directory",
                source,
            })?;

            let metadata = match entry.metadata() {
                Ok(stat) => stat,
                Err(e) => {
                    debug!("autodeps: provides: failed to stat: {e}");
                    continue;
                }
            };

            if !metadata.is_file() {
                continue;
            }

            let mut file =
                File::open(entry.path()).map_err(|source| crate::Error::IoPathError {
                    path: entry.path().to_path_buf(),
                    context: "read entry",
                    source,
                })?;

            let Some(elf) = read_elf(&mut file, &mut buffer)? else {
                continue;
            };

            if let Some(soname) = elf.soname
                && elf.is_lib
            {
                sonames.push(SonameV2 {
                    prefix: lookup_dir.prefix.clone(),
                    soname: Soname::from_str(soname)?,
                });
            }
        }

        sonames.sort();
        sonames.dedup();

        Ok(sonames)
    }
}
