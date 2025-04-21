//! Package lookup handling
use std::{io::Read, path::PathBuf, str::FromStr};

use alpm_package::Package;
use alpm_pkginfo::{PackageInfo, RelationOrSoname};
use alpm_types::{Soname, SonameLookupDirectory, SonameV2};
use goblin::{Hint, Object};
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::Error;

/// Represents a shared library and its associated sonames.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ElfSonames {
    /// The path to the ELF file in the package archive.
    pub path: PathBuf,
    /// The list of sonames extracted from the ELF file.
    pub sonames: Vec<Soname>,
}

/// Extracts the **sonames** from ELF files contained in a package.
///
/// This function opens the package file, decompresses it, and reads the ELF files from
/// the archive.
/// From each ELF file it then extracts the shared object dependencies and returns them as a
/// vector of [`ElfSonames`].
///
/// # Errors
///
/// Returns an error if:
///
/// - the package cannot be opened for reading (see [`Package::try_from`]),
/// - the ELF files in `package` cannot be read/parsed,
/// - or the found shared objects cannot be parsed as [`Soname`].
pub fn extract_elf_sonames(path: PathBuf) -> Result<Vec<ElfSonames>, Error> {
    let package = Package::try_from(path.as_path())?;
    let mut reader = package.into_reader()?;
    let mut elf_sonames = Vec::new();
    for entry in reader.data_entries()? {
        let mut entry = entry?;
        let path_in_archive = entry.path().to_path_buf();
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
            trace!("⤷ Not an ELF file, skipping...");
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

/// Finds the **soname** data provided by a package.
///
/// This function takes a package file and a lookup directory and extracts a list of [`SonameV2`]
/// provided by the package, that match the prefix of the lookup directory.
///
/// # Errors
///
/// Returns an error if:
///
/// - the input `path` is a directory,
/// - the package cannot be opened for reading (see [`Package::try_from`]),
/// - the `package` does not contain a [PKGINFO] file,
/// - or the [PKGINFO] file in `package` cannot be read.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
pub fn find_provisions(
    path: PathBuf,
    lookup_dir: SonameLookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    if path.is_dir() {
        return Err(Error::InputDirectoryNotSupported { path });
    }
    let package = Package::try_from(path.as_path())?;
    let package_info = package.read_pkginfo()?;
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

/// Finds the **soname** dependencies required by a package.
///
/// This function takes a package file `path` and a lookup directory `lookup_dir` and extracts a
/// list of [`SonameV2`] used by the package that match the prefix of the lookup directory.
///
/// Dependencies are extracted from the dynamic section of all ELF files contained in the package,
/// (see [`extract_elf_sonames`]) and the sonames are then compared to the **soname** dependencies
/// encoded in the package's [PKGINFO] data.
///
/// If `all` is `false`, this function returns only the [`SonameV2`] for which a match exists in the
/// [PKGINFO] data of the package. If `all` is `true`, this function returns all dependencies, also
/// those without a matching entry in the package's [PKGINFO] data.
///
/// # Errors
///
/// Returns an error if:
///
/// - the input `path` is a directory,
/// - the package cannot be opened for reading (see [`Package::try_from`]),
/// - the `package` does not contain a [PKGINFO] file,
/// - the ELF files in `package` cannot be read/parsed (see [`extract_elf_sonames`]),
/// - or the [PKGINFO] file in `package` could not be read.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
pub fn find_dependencies(
    path: PathBuf,
    lookup_dir: SonameLookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    if path.is_dir() {
        return Err(Error::InputDirectoryNotSupported { path });
    }
    let package = Package::try_from(path.as_path())?;
    let package_info = package.read_pkginfo()?;
    let depends = match package_info {
        PackageInfo::V1(package_info_v1) => package_info_v1.depend().to_vec(),
        PackageInfo::V2(package_info_v2) => package_info_v2.depend().to_vec(),
    };
    debug!(
        "Package dependencies according to PKGINFO data: {}",
        depends
            .iter()
            .map(|depend| depend.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let elf_sonames = extract_elf_sonames(path)?;
    let sonames = depends
        .iter()
        .filter_map(|p| match p {
            RelationOrSoname::Relation(_) => None,
            RelationOrSoname::SonameV1(_) => None,
            RelationOrSoname::SonameV2(soname_v2) => Some(soname_v2.clone()),
        })
        .filter(|soname| {
            let matches_prefix = soname.prefix == lookup_dir.prefix;
            let found_dependencies: Vec<&ElfSonames> = elf_sonames
                .iter()
                .filter(|dependency| dependency.sonames.contains(&soname.soname))
                .collect();
            trace!("Found dependencies: {found_dependencies:?}");
            matches_prefix && !found_dependencies.is_empty()
        })
        .collect::<Vec<SonameV2>>();

    Ok(sonames)
}
