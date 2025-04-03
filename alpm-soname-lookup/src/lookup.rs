use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use alpm_common::MetadataFile;
use alpm_pkginfo::{PackageInfo, RelationOrSoname};
use alpm_types::SonameV2;
use tar::Archive;

use crate::{Error, dir::LookupDirectory};

/// Read the package information from a package file.
///
/// This functions opens the package file, decompresses it using zstd, and reads the `.PKGINFO`
/// file from the archive. It returns a `PackageInfo` object containing the package information.
fn read_package_info(package: PathBuf) -> Result<PackageInfo, Error> {
    let file = File::open(&package)
        .map_err(|e| Error::IoPathError(package.clone(), "reading package file", e))?;
    let buf_reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(buf_reader).map_err(|e| {
        Error::IoPathError(package.clone(), "creating zstd decoder for package file", e)
    })?;
    let mut archive = Archive::new(decoder);
    let pkginfo = archive
        .entries()
        .map_err(|e| Error::IoPathError(package.clone(), "reading archive", e))?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .path()
                .map(|path| path.to_string_lossy().ends_with(".PKGINFO"))
                .unwrap_or(false)
        });
    if let Some(mut entry) = pkginfo {
        let path = entry
            .path()
            .map_err(|e| Error::IoPathError(package.clone(), "reading path", e))?
            .to_path_buf();

        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .map_err(|e| Error::IoPathError(path.clone(), "reading .PKGINFO contents", e))?;

        let package_info = PackageInfo::from_str_with_schema(&contents, None)?;

        Ok(package_info)
    } else {
        Err(Error::MissingPackageInfo(package))
    }
}

/// Find the sonames provided by a package.
///
/// This function takes a package file and a lookup directory and returns a list of sonames
/// provided by the package that match the prefix of the lookup directory.
pub fn find_provision(
    package: PathBuf,
    lookup_dir: LookupDirectory,
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
            RelationOrSoname::SonameV2(soname_v2) => Some(soname_v2.clone()),
        })
        .filter(|soname| soname.prefix == lookup_dir.prefix)
        .collect::<Vec<SonameV2>>();

    Ok(sonames)
}

/// Find the sonames required by a package.
///
/// This function takes a package file and a lookup directory and returns a list of sonames
/// required by the package that match the prefix of the lookup directory.
pub fn find_dependency(
    package: PathBuf,
    lookup_dir: LookupDirectory,
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
