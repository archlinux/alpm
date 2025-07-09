//! Verifier path handling and canonicalization

use std::{
    fmt::{Debug, Formatter},
    fs::File,
    path::{Path, PathBuf},
};

use log::{trace, warn};

use crate::{
    identifiers::{Context, Os, Purpose, Technology},
    load_path::LoadPath,
    util::symlinks::{ResolvedSymlink, resolve_symlink},
};

/// A [`Verifier`] points to a signature verifier in the file system.
///
/// It consists of the [`VoaLocation`] via which the verifier was obtained, and a canonicalized
/// path to the actual verifier file.
///
/// Depending on the verifier [`Technology`], a [`Verifier`] instance may represent, e.g.:
///
/// - an individual, standalone signature verifier,
/// - an individual verifier that acts as a trust anchor,
/// - a certificate complete with its trust chain,
/// - a set of individual verifiers in one shared data structure.
pub struct Verifier {
    /// The logical VOA location via which the verifier was found
    voa_location: VoaLocation,

    /// Canonicalized path of the verifier file
    canonicalized: PathBuf,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "VOA location: {:#?}", self.voa_location)?;
        writeln!(f, "Canonicalized verifier path: {:?}", self.canonicalized)?;

        Ok(())
    }
}

impl Verifier {
    /// Constructor for [Verifier]
    ///
    /// Callers of this constructor must ensure that `canonicalized` contains a fully canonicalized
    /// filename, that the file exists, and that the naming and any potential symlinks involved
    /// conform to the constraints defined in the VOA specification.
    pub(crate) fn new(voa_location: VoaLocation, canonicalized: PathBuf) -> Self {
        Self {
            voa_location,
            canonicalized,
        }
    }

    /// The verifier source path definition that this verifier file was found through
    pub fn voa_location(&self) -> &VoaLocation {
        &self.voa_location
    }

    /// The canonicalized [`Path`] representation of this [`Verifier`]
    pub fn canonicalized(&self) -> &Path {
        &self.canonicalized
    }
    /// Just the filename part of the canonicalized path of this verifier
    pub(crate) fn filename(&self) -> Option<&std::ffi::OsStr> {
        self.canonicalized.file_name()
    }

    /// Open this verifier as a file in read-only mode
    pub fn open(&self) -> std::io::Result<File> {
        File::open(&self.canonicalized)
    }
}

/// A leaf directory location in a VOA filesystem hierarchy (not canonicalized).
/// Signature verifier files are situated in a [`VoaLocation`].
///
/// A [`VoaLocation`] combines a load path and a set of identifier parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct VoaLocation {
    load_path: LoadPath,
    os: Os,
    purpose: Purpose,
    context: Context,
    technology: Technology,
}

impl VoaLocation {
    pub(crate) fn new(
        load_path: LoadPath,
        os: Os,
        purpose: Purpose,
        context: Context,
        technology: Technology,
    ) -> Self {
        Self {
            load_path,
            os,
            purpose,
            context,
            technology,
        }
    }

    /// The load path of the [`VoaLocation`].
    pub fn load_path(&self) -> &Path {
        // Note: the `LoadPath` type is handled as an internal implementation detail,
        // we're returning this as a &Path

        &self.load_path.path
    }

    /// The [`Os`] of the [`VoaLocation`].
    pub fn os(&self) -> &Os {
        &self.os
    }

    /// The [`Purpose`] of the [`VoaLocation`].
    pub fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    /// The [`Context`] of the [`VoaLocation`].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The [`Technology`] of the [`VoaLocation`].
    pub fn technology(&self) -> &Technology {
        &self.technology
    }

    /// Canonicalize a [`VoaLocation`] and check that its identifiers conform to VOA
    /// restrictions.
    ///
    /// Ensures that the provided [`VoaLocation`] points to a legal path in the local
    /// filesystem, and that any involved symlinks conform to the VOA symlink restrictions.
    ///
    /// Checks the legality of symlinks (if any) in the VOA path structure, and
    /// returns the canonicalized path to the target directory.
    pub(crate) fn check_and_canonicalize(
        &self,
        legal_symlink_paths: &[&LoadPath],
    ) -> std::io::Result<PathBuf> {
        // Canonicalized base load path
        // (any potential internal symlinks of this top level "load_path" are not checked)
        let base_path = self.load_path().canonicalize()?;

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
                            ResolvedSymlink::Dir(dir) => buf = dir,
                            ResolvedSymlink::File(path) => {
                                warn!("⤷ Found unexpected file at {path:?}");
                                return Err(std::io::Error::other(format!(
                                    "Unexpected file at {path:?}"
                                )));
                            }
                            ResolvedSymlink::Masked => {
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
                trace!("VoaLocation::check_and_canonicalize {buf:?} is not a directory");

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

        trace!("VoaLocation::check_and_canonicalize canonicalized path: {path:?}");

        Ok(path)
    }
}
