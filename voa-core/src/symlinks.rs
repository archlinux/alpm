//! Helper functionality for handling symlinks in VOA, and checking their validity within the
//! VOA linking rules.

use std::{
    collections::HashSet,
    fs::read_link,
    path::{Path, PathBuf},
};

use log::{trace, warn};

use crate::{
    load_path::LoadPath,
    types::{Error, VerifierSourcePath},
};

/// The result of resolving a symlink in a VOA structure.
///
/// The resolved target of a link is either a [PathBuf] to a directory or a file, or it shows
/// that the symlink points to `/dev/null` to signal that the source is "masked".
pub(crate) enum ResolveSym {
    Dir(PathBuf),
    File(PathBuf),
    Masked,
}

/// Check and resolve an arbitrarily long chain of symlinks.
///
/// Can detect cycles in symlink chains and will return an Error in case of a cycle.
/// Detects masking (symlinks to `/dev/null`) and returns `ResolveSym::Masked` for this case.
///
/// For results with the variants `ResolveSym::File` or `ResolveSym::Dir`, the returned `PathBuf`
/// contains a fully canonicalized path.
pub(crate) fn resolve_symlink(
    start: &Path,
    legal_symlink_paths: &[&LoadPath],
) -> Result<ResolveSym, Error> {
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
        let link_target = read_link(&path).unwrap(); // FIXME: map to our error

        // Are we in a symlink cycle?
        if paths_seen.contains(&link_target) {
            return Err(Error::IllegalSymlink); // FIXME: signal cycle
        }

        // If this is a masking symlink, we're done and return
        if link_target.as_path().to_str() == Some("/dev/null") {
            return Ok(ResolveSym::Masked);
        }

        // Check that target file path is legal

        // Symlinks may only point into locations under
        // legal_symlink_paths
        if !legal_symlink_paths
            .iter()
            .any(|p| link_target.starts_with(&p.path))
        {
            trace!("⤷ Illegal symlink target"); // FIXME: message should go in error
            return Err(Error::IllegalSymlink);
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
            return Ok(ResolveSym::File(link_target));
        } else if file_type.is_dir() {
            return Ok(ResolveSym::Dir(link_target));
        } else if !file_type.is_symlink() {
            warn!("Unexpected file type {file_type:?} for {link_target:?}");
            return Err(Error::IllegalSymlink);
        }

        path = link_target;
        paths_seen.insert(path.clone());
    }
}

impl VerifierSourcePath {
    /// Canonicalize a `VerifierSourcePath`, and check that its elements conform to VOA
    /// restrictions.
    ///
    /// Ensures, that the provided [`VerifierSourcePath`] represents a legal path on the local
    /// filesystem, and any involved symlinks conform to the VOA symlink restrictions.
    ///
    /// Checks the legality of symlinks (if any) in the VOA path structure, and
    /// returns the canonicalized path to the target directory.
    pub(crate) fn check_and_canonicalize(
        &self,
        legal_symlink_paths: &[&LoadPath],
    ) -> std::io::Result<PathBuf> {
        // Canonicalized base load path
        // (any potential internal symlinks of this top level "load_path" are not checked)
        let base_path = self.load_path().path.canonicalize()?;

        /// Append a segment to a path and ensure that the resulting path conforms
        /// to the VOA symlink constraints.
        fn append(
            current_path: &Path,
            segment: &Path,
            legal_symlink_paths: &[&LoadPath],
        ) -> std::io::Result<PathBuf> {
            let mut buf = current_path.join(segment);

            if buf.is_symlink() {
                match resolve_symlink(&buf, legal_symlink_paths) {
                    Ok(resolved) => {
                        match resolved {
                            ResolveSym::Dir(dir) => buf = dir,
                            ResolveSym::File(path) => {
                                warn!("⤷ Found unexpected file at {path:?}");
                                return Err(std::io::Error::other(format!(
                                    "Unexpected file at {path:?}"
                                )));
                            }
                            ResolveSym::Masked => {
                                // VOA implementations must not consider masking symlinks for
                                // directories and should raise a warning for them.

                                warn!("⤷ Found illegal masking symlink at directory {buf:?}");
                                return Err(std::io::Error::other(format!(
                                    "Illegal masking symlink at directory {buf:?}"
                                )));
                            }
                        }
                    }
                    Err(err) => {
                        warn!("⤷ Cannot resolve symlinks for {:?}: {err:?}", &buf);
                        return Err(std::io::Error::other(format!(
                            "{segment:?} is not valid following {buf:?}"
                        )));
                    }
                }
            }

            if !buf.is_dir() {
                trace!("CheckedVerifierSourcePath::new {buf:?} is not a directory");

                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    format!("{buf:?} is not a directory"),
                ));
            }

            Ok(buf)
        }

        let mut path = append(&base_path, &self.os().path_segment(), legal_symlink_paths)?;
        path = append(&path, &self.purpose().path_segment(), legal_symlink_paths)?;
        path = append(&path, &self.context().path_segment(), legal_symlink_paths)?;
        path = append(
            &path,
            &self.technology().path_segment(),
            legal_symlink_paths,
        )?;

        trace!("CheckedVerifierSourcePath::new canonicalized path: {path:?}");

        Ok(path)
    }
}
