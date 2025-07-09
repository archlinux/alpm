//! Helper functionality for handling symlinks in VOA. In particular, resolving symlinks while
//! checking that their structure conforms to the VOA linking rules.

use std::{
    collections::HashSet,
    fs::read_link,
    path::{Path, PathBuf},
};

use log::warn;

use crate::{error::Error, load_path::LoadPath};

/// The result of resolving a symlink in a VOA structure.
///
/// The resolved target of a link is either a [`PathBuf`] to a directory or a file, or it shows
/// that the symlink points to `/dev/null` to signal "masking".
pub(crate) enum ResolvedSymlink {
    Dir(PathBuf),
    File(PathBuf),
    Masked,
}

/// Resolve an arbitrarily long chain of symlinks and check its validity under VOA rules.
///
/// For results with the variants [`ResolvedSymlink::File`] or [`ResolvedSymlink::Dir`], the
/// returned [`PathBuf`] contains a fully canonicalized path.
///
/// Symlinks that point to `/dev/null` (including in multiple hops) signal `masking` in VOA,
/// the variant [`ResolvedSymlink::Masked`] is returned for such symlinks.
/// <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#masking>
///
/// Ensures that all intermediate and final paths are located within the set of paths in
/// `legal_symlink_paths`. When symlinks point outside of this set of paths,
/// [`Error::IllegalSymlinkTarget`] is returned.
///
/// Cycles in symlink chains are detected. [`Error::CyclicSymlinks`] is returned for such erroneous
/// configurations.
pub(crate) fn resolve_symlink(
    start: &Path,
    legal_symlink_paths: &[&LoadPath],
) -> Result<ResolvedSymlink, Error> {
    if !start.is_symlink() {
        warn!("⤷ Not a symlink {start:?}");

        // This is actually an inconsistent call for this function - use a separate error?
        return Err(Error::IllegalSymlink);
    }

    let mut path = start.to_path_buf();

    // Remember all paths we've traversed to do symlink cycle detection
    let mut paths_seen = HashSet::new();
    paths_seen.insert(path.clone());

    // Loop through chains of symlinks.
    // Check legality of each intermediate hop!
    loop {
        let link_target = read_link(&path)?;

        // Are we in a symlink cycle?
        if paths_seen.contains(&link_target) {
            return Err(Error::CyclicSymlinks);
        }

        // If this is a masking symlink, we're done and return
        if link_target.as_path().to_str() == Some("/dev/null") {
            return Ok(ResolvedSymlink::Masked);
        }

        // Check that target file path is legal:
        // Symlinks may only point into locations under `legal_symlink_paths`
        if !legal_symlink_paths
            .iter()
            .any(|p| link_target.starts_with(&p.path))
        {
            return Err(Error::IllegalSymlinkTarget);
        }

        let meta = match std::fs::symlink_metadata(&link_target) {
            Ok(meta) => meta,
            Err(err) => {
                warn!("⤷ Cannot get metadata of {link_target:?}");
                return Err(err.into());
            }
        };

        // set up next loop iteration
        let file_type = meta.file_type();
        if file_type.is_file() {
            return Ok(ResolvedSymlink::File(link_target));
        } else if file_type.is_dir() {
            return Ok(ResolvedSymlink::Dir(link_target));
        } else if !file_type.is_symlink() {
            warn!("Unexpected file type {file_type:?} for {link_target:?}");
            return Err(Error::IllegalSymlink);
        }

        path = link_target;
        paths_seen.insert(path.clone());
    }
}
