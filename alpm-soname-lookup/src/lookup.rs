use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

use alpm_pkginfo::{PackageInfoV2, RelationOrSoname};
use alpm_types::SonameV2;
use tar::Archive;

use crate::{Error, dir::LookupDirectory};

/// Find the sonames provided by a package.
///
/// This function takes a package file and a lookup directory and returns a list of sonames
/// provided by the package that match the prefix of the lookup directory.
pub fn find_provision(
    package: PathBuf,
    lookup_dir: LookupDirectory,
) -> Result<Vec<SonameV2>, Error> {
    let mut found_sonames = Vec::new();
    let file = File::open(&package)
        .map_err(|e| Error::IoPathError(package.clone(), "reading package file", e))?;
    let buf_reader = BufReader::new(file);
    let decoder = zstd::Decoder::new(buf_reader).map_err(|e| {
        Error::IoPathError(package.clone(), "creating zstd decoder for package file", e)
    })?;
    let mut archive = Archive::new(decoder);
    if let Some(mut entry) = archive
        .entries()
        .map_err(|e| Error::IoPathError(package.clone(), "reading archive", e))?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .path()
                .map(|path| path.to_string_lossy().ends_with(".PKGINFO"))
                .unwrap_or(false)
        })
    {
        let path = entry
            .path()
            .map_err(|e| Error::IoPathError(package.clone(), "reading path", e))?
            .to_path_buf();

        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .map_err(|e| Error::IoPathError(path.clone(), "reading .PKGINFO contents", e))?;

        // TODO: handle v1
        let package_info = PackageInfoV2::from_str(&contents)?;

        let sonames = package_info
            .provides()
            .iter()
            .filter_map(|p| match p {
                RelationOrSoname::Relation(_) => None,
                RelationOrSoname::SonameV1(_) => None,
                RelationOrSoname::SonameV2(soname_v2) => Some(soname_v2.clone()),
            })
            .collect::<Vec<SonameV2>>();

        for soname in &sonames {
            if soname.prefix == lookup_dir.prefix {
                found_sonames.push(soname.clone());
            }
        }
    } else {
        return Err(Error::MissingPackageInfo(package));
    }

    Ok(found_sonames)
}
