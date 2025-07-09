use std::{ffi::OsStr, fs::read_dir};

use log::{debug, trace, warn};

use crate::{
    identifiers::{Context, Os, Purpose, Technology},
    load_path::LoadPathList,
    util::symlinks::{ResolvedSymlink, resolve_symlink},
    verifier::{Verifier, VoaLocation},
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
    /// The VOA instance is initialized with a set of load paths, either in system mode or
    /// user mode, based on the user id of the current process.
    ///
    /// - For user ids < 1000, the VOA instance is initialized in system mode. See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#system-mode>
    /// - For user ids >= 1000, the VOA instance is initialized in user mode. See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#user-mode>
    pub fn init() -> Self {
        warn!("Initializing VOA instance");

        let uid = uzers::get_current_uid();
        trace!("LoadPathList::init called with process user id {uid}");

        let load_paths = if uid < 1000 {
            debug!("⤷ Using system mode load paths");
            LoadPathList::load_path_list_system()
        } else {
            debug!("⤷ Using user mode load paths");
            LoadPathList::load_path_list_user()
        };

        Self { load_paths }
    }

    /// Find applicable signature verifiers for a set of identifiers in all VOA load paths.
    ///
    /// Verifiers are found based on the provided [`Os`], [`Purpose`], [`Context`] and
    /// [`Technology`] identifiers.
    ///
    /// Warnings are emitted (via the Rust `log` mechanism) for all unusable files and directories
    /// in the subset of the VOA hierarchy specified by the set of identifiers.
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
    /// TODO: Do we want a convenience lookup function that searches for both `Mode`s at once?
    ///
    /// TODO: Should the returned data have more structure? (e.g. grouped by common filename)
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

        // Search in each load path
        for load_path in self.load_paths.paths() {
            debug!("Looking for signature verifiers in the load path {load_path:?}");

            // Load paths that symlinks from this load path may link into (or traverse through)
            let legal_symlink_paths = self.load_paths.legal_symlink_load_paths(load_path);

            let voa_location = VoaLocation::new(
                load_path.clone(),
                os.clone(),
                purpose.clone(),
                context.clone(),
                technology.clone(),
            );

            // Get the validated and canonicalized filesystem path for this VOA location
            let canonicalized = match voa_location.check_and_canonicalize(&legal_symlink_paths) {
                Ok(canonicalized) => canonicalized,
                Err(err) => {
                    warn!("Error while checking load path {load_path:?}: {err:?}",);
                    continue;
                }
            };

            // TODO: Store a list of canonicalized filesystem paths that we've already processed,
            // and don't process canonicalized paths multiple times?
            // (When symlinks point to the same canonicalized path from multiple load paths)
            //
            // (-> is it ever useful for callers to know that a verifier was found via multiple
            // load paths?)

            // Get the entries of this verifier directory
            trace!("Scanning verifiers in canonicalized VOA path {canonicalized:?}");
            let dir = match read_dir(canonicalized) {
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

                // Get the checked and canonicalized path for the verifier file behind this
                // directory entry
                let verifier = if file_type.is_file() {
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
                        ResolvedSymlink::File(path) => path,
                        ResolvedSymlink::Masked => {
                            // Masking symlinks are only expected in writable load paths
                            if !load_path.writable() {
                                warn!(
                                    "Masked file name {entry:?} is illegal in writable load path {load_path:?}, ignoring"
                                );
                                continue;
                            }

                            // Store name of the masked verifier for an output filtering step
                            match entry.file_name().to_str() {
                                None => warn!("Masked file name {entry:?} contains invalid UTF-8"),
                                Some(name) => {
                                    masked_names.push(name.to_string());
                                }
                            }

                            continue;
                        }
                        ResolvedSymlink::Dir(d) => {
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

                trace!("Checking verifier file {verifier:?}");

                if verifier.is_file() {
                    verifiers.push(Verifier::new(voa_location.clone(), verifier));
                } else {
                    trace!("⤷ Verifier path is not a file {verifier:?}");
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
