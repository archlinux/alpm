//! Package lookup handling
use std::{fs::File, io::BufReader, path::PathBuf};

use alpm_common::MetadataFile;
use alpm_pkginfo::{PackageInfo, RelationOrSoname};
use alpm_types::{SonameLookupDirectory, SonameV2};
use tar::Archive;

use crate::Error;

/// Reads the [PKGINFO] data from a package file.
///
/// This functions opens the package file, decompresses it using zstd, and reads the `.PKGINFO`
/// file from the archive. It returns a `PackageInfo` object containing the package information.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
///
/// # Errors
///
/// Returns an error if
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
fn read_package_info(path: PathBuf) -> Result<PackageInfo, Error> {
    let file = File::open(&path).map_err(|source| Error::IoPathError {
        path: path.clone(),
        context: "opening package file",
        source,
    })?;
    let buf_reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(buf_reader).map_err(|source| Error::IoPathError {
        path: path.clone(),
        context: "creating zstd decoder for package file",
        source,
    })?;
    let mut archive = Archive::new(decoder);
    let pkginfo = archive
        .entries()
        .map_err(|source| Error::IoPathError {
            path: path.clone(),
            context: "reading archive",
            source,
        })?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .path()
                .map(|path| path == PathBuf::from(".PKGINFO"))
                .unwrap_or(false)
        });

    let Some(mut entry) = pkginfo else {
        return Err(Error::MissingPackageInfo { path });
    };

    let package_info = PackageInfo::from_reader(&mut entry)?;

    Ok(package_info)
}

/// Finds the **soname** data provided by a package.
///
/// This function takes a package file and a lookup directory and extracts a list of [`SonameV2`].
/// provided by the package that match the prefix of the lookup directory.
///
/// # Errors
///
/// Returns an error if
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
pub fn find_provisions(
    package: PathBuf,
    lookup_dir: SonameLookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    let package_info = read_package_info(package)?;
    let provides = match package_info {
        PackageInfo::V1(package_info_v1) => package_info_v1.provides().to_vec(),
        PackageInfo::V2(package_info_v2) => package_info_v2.provides().to_vec(),
    };

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
/// # Errors
///
/// Returns an error if
/// - the `package` can not be opened for reading,
/// - the `package` can not be decoded using zstd compression,
/// - the `package` can not be read as archive,
/// - the `package` does not contain a `.PKGINFO` file,
/// - or the `.PKGINFO` file in `package` could not be read.
pub fn find_dependencies(
    package: PathBuf,
    lookup_dir: SonameLookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    let package_info = read_package_info(package)?;

    let depends = match package_info {
        PackageInfo::V1(package_info_v1) => package_info_v1.depend().to_vec(),
        PackageInfo::V2(package_info_v2) => package_info_v2.depend().to_vec(),
    };

    let sonames = depends
        .iter()
        .filter_map(|p| match p {
            RelationOrSoname::Relation(_) => None,
            RelationOrSoname::SonameV1(_) => None,
            RelationOrSoname::SonameV2(soname_v2) => Some(soname_v2.clone()),
        })
        .filter(|soname| soname.prefix == lookup_dir.prefix)
        .collect::<Vec<SonameV2>>();

    Ok(sonames)
}
