//! Package lookup handling
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use alpm_common::MetadataFile;
use alpm_pkginfo::{PackageInfo, RelationOrSoname};
use alpm_types::{Soname, SonameLookupDirectory, SonameV2};
use goblin::{Hint, Object};
use log::{debug, trace};
use tar::{Archive, EntryType};

use crate::Error;

/// Helper function to open a zstd compressed archive.
///
/// This function takes a path to a zstd compressed archive and returns an [`Archive`] object.
///
/// # Errors
///
/// Returns an error if:
///
/// - the `path` can not be opened for reading,
/// - the `path` can not be decoded using zstd compression,
/// - the `path` can not be read as archive.
fn open_zstd_archive(
    path: &Path,
) -> Result<Archive<zstd::Decoder<'_, BufReader<BufReader<File>>>>, Error> {
    debug!("Opening package file: {path:?}");
    let file = File::open(path).map_err(|source| Error::IoPathError {
        path: path.to_path_buf(),
        context: "opening package file",
        source,
    })?;
    let buf_reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(buf_reader).map_err(|source| Error::IoPathError {
        path: path.to_path_buf(),
        context: "creating zstd decoder for package file",
        source,
    })?;
    let archive = Archive::new(decoder);
    Ok(archive)
}

/// Represents a shared library and its associated sonames.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ElfSonames {
    /// The path to the ELF file in the package archive.
    path: PathBuf,
    /// The list of sonames extracted from the ELF file.
    sonames: Vec<Soname>,
}

/// Reads the **soname** dependencies from a package file.
///
/// This function opens the package file, decompresses it using zstd, and reads the ELF files from
/// the archive. Then it extracts the dynamic libraries from the ELF files and returns them as a
/// vector of [`ElfSonames`].
///
/// # Errors
///
/// Returns an error if:
///
/// - the package archive cannot be opened (see [`open_zstd_archive`]),
/// - the ELF files in `package` could not be read/parsed,
/// - or the shared libraries could not be parsed as [`Soname`].
fn read_soname_dependencies(path: PathBuf) -> Result<Vec<ElfSonames>, Error> {
    let mut archive = open_zstd_archive(&path)?;
    let mut elf_sonames = Vec::new();
    for entry in archive.entries().map_err(|source| Error::IoPathError {
        path: path.to_path_buf(),
        context: "reading archive",
        source,
    })? {
        // Read the entry from the archive
        let mut entry = entry.map_err(|source| Error::IoPathError {
            path: path.to_path_buf(),
            context: "reading entry from tar archive",
            source,
        })?;

        // Skip directories, symlinks, and other non-regular entries
        if entry.header().entry_type() != EntryType::Regular {
            continue;
        }

        let path_in_archive = entry
            .path()
            .map_err(|source| Error::IoPathError {
                path: path.to_path_buf(),
                context: "getting entry path",
                source,
            })?
            .to_path_buf();
        debug!("Package entry: {path_in_archive:?}");

        // Read 16 bytes for checking the header
        let mut header = [0u8; 16];
        if let Err(e) = entry.read_exact(&mut header) {
            debug!("⤷ Could not read entry header ({e}), skipping...");
            continue;
        }

        // Check the header for an ELF file
        if let Ok(Hint::Elf(_)) = goblin::peek_bytes(&header) {
            trace!("⤷ File header: {header:?}");
            debug!("⤷ Found ELF file.");
        } else {
            debug!("⤷ Not an ELF file, skipping...");
            continue;
        };

        // Read the entry into a buffer
        // Also, take the header into account
        let mut buffer = header.to_vec();
        entry
            .read_to_end(&mut buffer)
            .map_err(|source| Error::IoReadError {
                context: "reading entry from archive",
                source,
            })?;

        // Parse the ELF file and collect the dependencies
        let object = Object::parse(&buffer).map_err(|source| Error::ElfError {
            context: "parsing ELF file",
            source,
        })?;
        if let Object::Elf(elf) = object {
            debug!("⤷ Dependencies: {:?}", elf.libraries);
            let mut sonames = Vec::new();
            for library in elf.libraries.iter() {
                let soname = Soname::from_str(library)?;
                sonames.push(soname);
            }
            elf_sonames.push(ElfSonames {
                path: path_in_archive,
                sonames,
            });
        }
    }
    Ok(elf_sonames)
}

/// Reads the [PKGINFO] data from a package file.
///
/// This functions opens the package file, decompresses it using zstd, and reads the `.PKGINFO`
/// file from the archive. It returns a `PackageInfo` object containing the package information.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
///
/// # Errors
///
/// Returns an error if:
///
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
fn read_package_info(path: &Path) -> Result<PackageInfo, Error> {
    let mut archive = open_zstd_archive(path)?;
    let mut entries = archive.entries().map_err(|source| Error::IoPathError {
        path: path.to_path_buf(),
        context: "reading archive",
        source,
    })?;
    let mut pkginfo = None;
    for entry in &mut entries {
        let entry = entry.map_err(|source| Error::IoPathError {
            path: path.to_path_buf(),
            context: "reading entry",
            source,
        })?;
        let path_in_archive = entry.path().map_err(|source| Error::IoPathError {
            path: path.to_path_buf(),
            context: "getting entry path",
            source,
        })?;
        if path_in_archive == PathBuf::from(".PKGINFO") {
            pkginfo = Some(entry);
            break;
        }
    }

    let Some(mut entry) = pkginfo else {
        return Err(Error::MissingPackageInfo {
            path: path.to_path_buf(),
        });
    };

    let package_info = PackageInfo::from_reader(&mut entry)?;
    trace!("Package info:\n{package_info:?}");

    Ok(package_info)
}

/// Finds the **soname** data provided by a package.
///
/// This function takes a package file and a lookup directory and extracts a list of [`SonameV2`].
/// provided by the package that match the prefix of the lookup directory.
///
/// # Errors
///
/// Returns an error if:
///
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
pub fn find_provisions(
    package: PathBuf,
    lookup_dir: SonameLookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    let package_info = read_package_info(&package)?;
    let provides = match package_info {
        PackageInfo::V1(package_info_v1) => package_info_v1.provides().to_vec(),
        PackageInfo::V2(package_info_v2) => package_info_v2.provides().to_vec(),
    };
    debug!("Package provisions: {provides:?}");

    let sonames = provides
        .iter()
        .filter_map(|p| match p {
            RelationOrSoname::Relation(_) => None,
            RelationOrSoname::SonameV1(_) => None,
            RelationOrSoname::SonameV2(soname_v2) => {
                if soname_v2.prefix == lookup_dir.prefix {
                    Some(soname_v2.clone())
                } else {
                    None
                }
            }
        })
        .collect::<Vec<SonameV2>>();

    Ok(sonames)
}

/// Find the dependencies provided by a package.
///
/// This function takes a package file and a lookup directory and extracts a list of [`SonameV2`].
/// used by the package that match the prefix of the lookup directory.
///
/// It collects the dependencies from the ELF files (from the dynamic section) and also
/// from the `.PKGINFO` file.
///
/// If `all` is `true`, it will return all dependencies, even those without matching provisions.
///
/// # Errors
///
/// Returns an error if:
///
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
pub fn find_dependencies(
    package: PathBuf,
    lookup_dir: SonameLookupDirectory,
    all: bool,
) -> Result<Vec<SonameV2>, Error> {
    let package_info = read_package_info(&package)?;
    let dependencies = read_soname_dependencies(package.clone())?;

    let depends = match package_info {
        PackageInfo::V1(package_info_v1) => package_info_v1.depend().to_vec(),
        PackageInfo::V2(package_info_v2) => package_info_v2.depend().to_vec(),
    };
    debug!("Package dependencies: {depends:?}");

    let sonames = depends
        .iter()
        .filter_map(|p| match p {
            RelationOrSoname::Relation(_) => None,
            RelationOrSoname::SonameV1(_) => None,
            RelationOrSoname::SonameV2(soname_v2) => Some(soname_v2.clone()),
        })
        .filter(|soname| {
            let matches_prefix = soname.prefix == lookup_dir.prefix;
            let found_dependencies: Vec<&ElfSonames> = dependencies
                .iter()
                .filter(|dependency| dependency.sonames.contains(&soname.soname))
                .collect();
            trace!("Found dependencies: {found_dependencies:?}");
            if all {
                matches_prefix
            } else {
                matches_prefix && !found_dependencies.is_empty()
            }
        })
        .collect::<Vec<SonameV2>>();

    Ok(sonames)
}
