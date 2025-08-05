use std::{
    fs::{File, Metadata, read_dir},
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_types::{Soname, SonameLookupDirectory, SonameV2};
use goblin::{Hint, Object, elf::Elf};
use lddtree::DependencyAnalyzer;
use log::{debug, trace};
use walkdir::WalkDir;

use crate::Error;

/// Permission mask for checking if a file is executable.
const EXECUTABLE_MASK: u32 = 0o100;

/// Options for [`Autodeps`]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutodepsOptions {
    path: PathBuf,
    lookup_dir: SonameLookupDirectory,
    provides: bool,
    depends: bool,
}

impl AutodepsOptions {
    /// Creates a new [`AutodepsOptions`].
    ///
    /// # Arguments
    ///
    /// - `path`: specifies the directory to read. This should be the input directory of a built
    ///   package.
    /// - `lookup_dir`: specifies the soname prefix of depends and provides, and which directory to
    ///   search for provides.
    pub fn new(path: PathBuf, lookup_dir: SonameLookupDirectory) -> Self {
        Self {
            path,
            lookup_dir,
            provides: true,
            depends: true,
        }
    }

    /// Setter for provides.
    ///
    /// If true, [`Autodeps`] will generate the provides of the package.
    pub fn provides(mut self, provides: bool) -> Self {
        self.provides = provides;
        self
    }

    /// Setter for depends.
    ///
    /// If true, [`Autodeps`] will generate the depends of the package.
    pub fn depends(mut self, provides: bool) -> Self {
        self.depends = provides;
        self
    }
}

/// Autodeps reads the input directory of a package and finds what shared objects
/// the package provides, and what shared objects the package depends on.
///
/// Provides represent the shared objects that a package publicly contains. Provides are found
/// by walking the package's [`SonameLookupDirectory`] and reading
/// the `soname` of each shared object.
///
/// Dependencies represent the shared objects that a package depends on. Dependencies are found
/// by reading all binaries and libraries in a package and collecting the `soname`s that they depend
/// on. If a dependent shared object is part of the package being read it is not considered a
/// dependency.
///
/// # Note
///
/// Libraries and binaries are only read if they are regular files and executable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Autodeps {
    /// The provides of a package.
    pub provides: Vec<SonameV2>,
    /// The depends of a package.
    pub depends: Vec<SonameV2>,
}

impl Autodeps {
    /// Creates a new [`Autodeps`] instance, calculating the provides and depends of a package.
    ///
    /// This function calls [`Autodeps::with_options`] with the specified path and lookup_dir.
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use std::str::FromStr;
    /// use alpm_soname::Autodeps;
    /// use alpm_types::SonameLookupDirectory;
    ///
    /// # fn main() -> Result<(), alpm_soname::Error> {
    /// // Directory containing a built pacman package:
    /// //    usr/lib/libalpm.so.15.0.0
    /// //    usr/lib/libalpm.so
    /// //    usr/lib/libalpm.so.15
    /// //    usr/bin/pacman
    /// let pacman = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_files/pacman"));
    ///
    /// // Set prefix to `lib` and the soname lookup directory to `/usr/lib`
    /// let lookup_dir = SonameLookupDirectory::from_str("lib:/usr/lib")?;
    ///
    /// // Find providers and dependencies
    /// let autodeps = Autodeps::new(pacman, lookup_dir)?;
    ///
    /// for provide in &autodeps.provides {
    ///     println!("provide = {provide}");
    /// }
    /// for depend in &autodeps.depends {
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
    /// # let provides = autodeps.provides.iter().map(|d| d.to_string()).collect::<Vec<_>>();
    /// # let provides = provides.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    /// # let depends = autodeps.depends.iter().map(|d| d.to_string()).collect::<Vec<_>>();
    /// # let depends = depends.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    /// # assert_eq!(provides, vec!["lib:libalpm.so.15"]);
    /// # assert_eq!(depends, vec!["lib:libarchive.so.13", "lib:libc.so.6", "lib:libcrypto.so.3", "lib:libcurl.so.4", "lib:libgpgme.so.45"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: PathBuf, lookup_dir: SonameLookupDirectory) -> Result<Self, Error> {
        Self::with_options(AutodepsOptions::new(path, lookup_dir))
    }

    /// Creates a new Autodeps instance, calculating the provides and depends of a package.
    ///
    /// See [`AutodepsOptions`] for the available arguments.
    pub fn with_options(options: AutodepsOptions) -> Result<Self, Error> {
        let path = options
            .path
            .canonicalize()
            .map_err(|source| crate::Error::IoPathError {
                path: options.path.clone(),
                context: "canonicalize directory",
                source,
            })?;

        let provides = if options.provides {
            Self::provisions(&path, options.lookup_dir.clone())?
        } else {
            Default::default()
        };
        let depends = if options.depends {
            Self::depends(&path, options.lookup_dir)?
        } else {
            Default::default()
        };

        Ok(Self { depends, provides })
    }

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

            let entry_path = entry.path();

            // ensure is regular file
            if !entry.file_type().is_file() {
                continue;
            }

            let stat = match entry.metadata() {
                Ok(stat) => stat,
                Err(e) => {
                    debug!("failed to stat file for autodeps depends: {e}");
                    continue;
                }
            };

            if !is_executable(&stat) {
                continue;
            }

            let Some(elf) = read_elf(entry_path, &mut buffer)? else {
                continue;
            };

            if elf.libraries.is_empty() {
                continue;
            }

            let deps = DependencyAnalyzer::new(PathBuf::from(path))
                .analyze(entry_path)
                .map_err(|source| crate::Error::LibraryDependenciesError {
                    path: entry_path.to_path_buf(),
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

    fn provisions(path: &Path, lookup_dir: SonameLookupDirectory) -> Result<Vec<SonameV2>, Error> {
        let mut sonames = Vec::new();
        let mut buffer = Vec::new();
        let libdir = lookup_dir.directory.inner();
        let libdir = path.join(libdir.strip_prefix("/").unwrap_or(libdir));

        let Ok(entries) = read_dir(&libdir) else {
            return Ok(Default::default());
        };

        for entry in entries {
            let entry = entry.map_err(|source| crate::Error::IoPathError {
                path: libdir.to_path_buf(),
                context: "read directory",
                source,
            })?;

            let stat = match entry.metadata() {
                Ok(stat) => stat,
                Err(e) => {
                    debug!("autodeps: provides: failed to stat: {e}");
                    continue;
                }
            };

            if !stat.is_file() || !is_executable(&stat) {
                continue;
            }
            let Some(elf) = read_elf(&entry.path(), &mut buffer)? else {
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

fn read_elf<'a>(path: &Path, buffer: &'a mut Vec<u8>) -> Result<Option<Elf<'a>>, Error> {
    let Ok(mut entry) = File::open(path) else {
        return Ok(None);
    };

    // Read 16 bytes for checking the header
    let mut header = [0u8; 16];

    if let Err(e) = entry.read_exact(&mut header) {
        debug!("⤷ Could not read entry header ({e}), skipping...");
        return Ok(None);
    }

    // Check the header for an ELF file
    if let Ok(Hint::Elf(_)) = goblin::peek_bytes(&header) {
        trace!("⤷ File header: {header:?}");
        debug!("⤷ Found ELF file.");
    } else {
        debug!("⤷ Not an ELF file, skipping...");
        return Ok(None);
    };

    // Read the entry int a buffer
    // Also, take the header into account
    buffer.clear();
    buffer.extend_from_slice(&header);
    entry
        .read_to_end(buffer)
        .map_err(|source| Error::IoReadError {
            context: "reading entry from archive",
            source,
        })?;

    // Parse the ELF file and collect the soname
    let object = Object::parse(buffer).map_err(|source| Error::ElfError {
        context: "parsing ELF file",
        source,
    })?;
    let Object::Elf(elf) = object else {
        return Ok(None);
    };

    Ok(Some(elf))
}

fn is_executable(stat: &Metadata) -> bool {
    stat.permissions().mode() & EXECUTABLE_MASK != 0
}
