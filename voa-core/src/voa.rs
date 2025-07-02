use std::{ffi::OsStr, fs::read_dir};

use log::{debug, trace, warn};

use crate::{
    identifiers::{Context, Os, Purpose, Technology},
    load_path::{LoadPathList, LoadPathMode},
    symlinks::{ResolveSym, resolve_symlink},
    verifier::{Verifier, VerifierSourcePath},
};

/// Access to the "File Hierarchy for the Verification of OS Artifacts (VOA)".
///
/// [`Voa`] provides lookup facilities for signature verifiers that are stored in a VOA hierarchy.
/// Lookup of verifiers is agnostic to the cryptographic technology later using the verifiers.
#[derive(Debug)]
pub struct Voa {
    load_paths: LoadPathList,
}

impl Voa {
    /// Initialize a VOA instance.
    ///
    /// The VOA instance will be initialized to use a set of load paths, either in system mode or
    /// user mode, based on the user id for the current process.
    ///
    /// For user ids < 1000, the VOA instance initialize in system mode,
    /// for user ids >= 1000, the VOA instance initialize in user mode.
    pub fn init() -> Self {
        let load_path_mode = LoadPathMode::init();
        warn!("Initializing VOA instance for {load_path_mode:?}");

        Self {
            load_paths: load_path_mode.load_path_list(),
        }
    }

    /// Find verifiers in all available VOA hierarchies.
    ///
    /// Detection of verifiers is based on the provided [`Os`], [`Purpose`], [`Context`] and
    /// [`Technology`]. Finds all available verifier files and emits warnings for all files and
    /// directories that cannot be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), voa_core::Error> {
    /// use voa_core::{
    ///     Voa,
    ///     identifiers::{Context, Mode, Os, Purpose, Role, Technology},
    /// };
    ///
    /// let voa = Voa::init(); // Auto-detects System or User mode
    ///
    /// let verifiers = voa.lookup(
    ///     Os::new("arch".into(), None, None, None, None)?,
    ///     Purpose::new(Role::Packages, Mode::ArtifactVerifier),
    ///     Context::Default,
    ///     Technology::OpenPGP,
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// TODO: How should the `trust_anchor` parameter work in this API?
    ///  (At least for some callers it would probably be convenient not to pass it at all, and
    ///  get the combination of basic verifiers and trust anchors back.)
    ///
    /// TODO: Should the return type have more specialized lookup functionality?
    ///   (e.g. grouped by common filename, or organized by `trust_anchor` value?)
    pub fn lookup(
        &self,
        os: Os,
        purpose: Purpose,
        context: Context,
        technology: Technology,
    ) -> Vec<Verifier> {
        // Collects all verifiers that we find for this set of search parameters
        let mut verifiers = Vec::new();

        // A set of filenames that we will mask out of `verifiers` in the end
        let mut masked_names = Vec::new();

        for load_path in &self.load_paths.paths {
            debug!("Looking for signature verifiers in the load path {load_path:?}");

            // Load paths that files under this load path may legally link through, or into
            let legal_symlink_paths = self.load_paths.legal_symlink_load_paths(load_path);

            let verifier_path = VerifierSourcePath::new(
                load_path.clone(),
                os.clone(),
                purpose.clone(),
                context.clone(),
                technology.clone(),
            );

            // Get the validated and canonicalized filesystem path for this verifier path
            let checked_path = match verifier_path.check_and_canonicalize(&legal_symlink_paths) {
                Ok(checked_path) => checked_path,
                Err(err) => {
                    warn!("Error while checking load path {load_path:?}: {err:?}",);
                    continue;
                }
            };

            // Get the entries of this verifier directory
            trace!("Loading verifiers from VOA path {checked_path:?}");

            let dir = match read_dir(checked_path) {
                Ok(dir) => dir,
                Err(err) => {
                    trace!("⤷ Can't read path as a directory {err:?}");
                    continue; // try next load path
                }
            };

            // Loop through (potential) verifier files
            for res in dir {
                let entry = match res {
                    Ok(entry) => entry,
                    Err(err) => {
                        warn!("⤷ Invalid directory entry:\n{err}");
                        continue;
                    }
                };

                let Ok(file_type) = entry.file_type() else {
                    warn!("⤷ Cannot get file type of directory entry {entry:?}");
                    continue;
                };

                // Get the checked and canonicalized path for the file behind this directory entry
                let path = if file_type.is_file() {
                    entry.path()
                } else if file_type.is_symlink() {
                    let resolved = match resolve_symlink(&entry.path(), &legal_symlink_paths) {
                        Ok(rs) => rs,
                        Err(err) => {
                            warn!(
                                "⤷ Symlink {:?} is invalid for use with VOA ({err:?})",
                                &entry.path()
                            );
                            continue;
                        }
                    };

                    match resolved {
                        ResolveSym::File(path) => path,
                        ResolveSym::Masked => {
                            // Store name of the masked verifier for an output filtering step

                            // TODO: Masking symlinks are only expected in writable load paths:
                            // - system mode: `/etc/voa/` or `/run/voa/`
                            // - user mode: `$XDG_CONFIG_HOME/voa/` or `$XDG_RUNTIME_DIR/voa/`
                            //
                            //  -> check if the current load path is writable, otherwise ... warn?

                            match entry.file_name().to_str() {
                                None => warn!("Masked file name {entry:?} contains invalid UTF-8"),
                                Some(name) => {
                                    masked_names.push(name.to_string());
                                }
                            }

                            continue;
                        }
                        ResolveSym::Dir(d) => {
                            warn!(
                                "⤷ Ignoring symlink that points to a directory {:?}: {d:?}",
                                &entry.path()
                            );
                            continue;
                        }
                    }
                } else {
                    warn!("⤷ Unexpected file type {file_type:?} for entry {entry:?}");
                    continue;
                };

                trace!("Checking verifier file {path:?}");

                if path.is_file() {
                    verifiers.push(Verifier {
                        verifier_path: verifier_path.clone(),
                        canonicalized: path,
                    });
                } else {
                    trace!("⤷ Verifier path is not a file {path:?}");
                }
            }
        }

        verifiers
            .into_iter()
            .filter(|verifier| {
                if let Some(filename) = verifier.filename().and_then(OsStr::to_str) {
                    // Filter out masked verifiers
                    !masked_names.contains(&filename.to_string())
                } else {
                    // Filename is not valid UTF-8 -> filter out
                    false
                }
            })
            .collect()
    }
}
