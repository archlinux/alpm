//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! A mechanism for the storage and retrieval of cryptography technology-agnostic signature
//! verifiers. For specification draft see: <https://github.com/uapi-group/specifications/pull/134>
//!
//! The VOA hierarchy acts as structured storage for files that contain "signature verifiers"
//! (such as [OpenPGP certificates], aka "public keys").
//!
//! A set of "load paths" may exist on a system, each containing different sets of verifier files.
//! This library provides an abstract, unified view of this set of signature verifiers in that set
//! of load paths.
//!
//! Libraries dealing with a specific cryptographic technology can rely on this library to collect
//! the paths of all verifier files relevant to them.
//!
//! Each load path contains a VOA hierarchy of verifier files.
//! Earlier load paths have precedence over later entries (in some technologies).
//!
//! NOTE: Depending on the technology, multiple versions of the same verifier will either "shadow"
//! one another, or get merged into one coherent view that represents the totality of available
//! information about the verifier.
//!
//! Shadowing/merging is specific to each signing technology and must be handled in the
//! technology-specific library.
//! For more details see e.g. the `voa-openpgp` implementation and the VOA specification.
//!
//! VOA expects that filenames are a strong identifier that signals whether two verifier files
//! contain variants of "the same" logical verifier.
//! Verifiers from different load paths can be identified as related via their filenames.
//!
//! For example, [OpenPGP certificates] must be stored using filenames based on their fingerprint.
//!
//!
//! [OpenPGP certificates]: https://openpgp.dev/book/certificates.html

#![warn(missing_docs)]

pub mod types;

use std::{
    fmt::Debug,
    fs::{read_dir, read_link},
    path::{Path, PathBuf},
};

use log::{debug, trace, warn};

use crate::types::{Context, Os, Purpose, Technology, Verifier};

/// Load paths for "system mode" operation of VOA.
pub const LOAD_PATHS_SYSTEM_MODE: &[&str] = &[
    "/etc/voa/",
    "/run/voa/",
    "/usr/local/share/voa/",
    "/usr/share/voa/",
];

// TODO: const LOAD_PATHS_USER_MODE
//
// $XDG_CONFIG_HOME/voa/
// the ./voa/ directory in each directory defined in $XDG_CONFIG_DIRS
// $XDG_RUNTIME_DIR/voa/
// $XDG_DATA_HOME/voa/
// the ./voa/ directory in each directory defined in $XDG_DATA_DIRS

/// A path in a VOA structure in which verifier files are stored.
#[derive(Clone, Debug, PartialEq)]
pub struct VerifierSourcePath {
    load_path: PathBuf,
    os: Os,
    purpose: Purpose,
    context: Context,
    technology: Technology,
}

/// This structure represents a [VerifierSourcePath] that has been checked for "legality":
///
/// Constructing it checks the legality of symlinks (if any) in the VOA path structure, and persists
/// the canonicalized path to the target directory.
struct CheckedVerifierSourcePath {
    verifier_source_path: VerifierSourcePath,

    // The canonicalized target path of this VerifierSourcePath (i.e. with resolved symlinks)
    canonicalized_target: PathBuf,
}

impl CheckedVerifierSourcePath {
    /// Creates a new `CheckedVerifierSourcePath` from a `VerifierSourcePath`.
    ///
    /// Ensures, that the provided [`VerifierSourcePath`] represents a legal path on the local
    /// filesystem, and any involved symlinks conform to the VOA symlink restrictions.
    fn new(verifier_source_path: VerifierSourcePath) -> std::io::Result<Self> {
        // Canonicalized base load path
        // (any potential internal symlinks of this top level "load_path" are not checked)
        let base_path = verifier_source_path.load_path.canonicalize()?;

        /// Append a segment to a path and ensure that the resulting path conforms
        /// to the VOA symlink constraints.
        fn append(
            p: &Path,
            segment: &Path,
            // FIXME: This actually needs a set of load paths that this verifier path may use
            base_path: &Path,
        ) -> std::io::Result<PathBuf> {
            let mut buf = p.join(segment);
            if buf.is_symlink() {
                // Check that the symlink-canonicalized path is acceptable, including this segment.
                let canon = buf.canonicalize()?;

                // FIXME: how does this interact with chains of symlinks? Does it process them at
                // once? If so, we still need to check legality of each intermediate hop!

                // TODO: This check should actually allow links into some of the other current load
                //       paths (but not all of them!)
                //        -> we should actually check "starts_with" against canonicalized forms of
                //           all currently legal-to-link-into load paths
                //
                // [..] However, symlinks can be used in the VOA hierarchy to point
                // to files or directories below one of the [load paths] in
                // descending priority.
                // Symlinks to files or directories below ephemeral load paths
                // (i.e. `/run/voa/` and `$XDG_RUNTIME_DIR/voa/`) are prohibited, as they
                // could lead to dangling references. [..]
                if !canon.starts_with(base_path) {
                    trace!("append: illegal path segment {segment:?} following {buf:?}");

                    return Err(std::io::Error::other(format!(
                        "{segment:?} is not valid following {buf:?} (it points to {canon:?})"
                    )));
                }

                buf = canon;
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

        let mut path = append(
            &base_path,
            &verifier_source_path.os.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.purpose.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.context.path_segment(),
            &base_path,
        )?;
        path = append(
            &path,
            &verifier_source_path.technology.path_segment(),
            &base_path,
        )?;

        trace!("CheckedVerifierSourcePath::new canonicalized path: {path:?}");

        Ok(Self {
            verifier_source_path,
            canonicalized_target: path,
        })
    }
}

/// VOA defines a list of _load paths_ with descending priority for system mode and user mode.
///
/// The following load paths are considered, depending on the load path mode:
///
/// System Mode:
/// - /etc/voa/
/// - /run/voa/
/// - /usr/local/share/voa/
/// - /usr/share/voa/
///
/// User Mode:
/// - `$XDG_CONFIG_HOME/voa/`
/// - the `./voa/` directory in each directory defined in `$XDG_CONFIG_DIRS`
/// - `$XDG_RUNTIME_DIR/voa/`
/// - `$XDG_DATA_HOME/voa/`
/// - the `./voa/` directory in each directory defined in `$XDG_DATA_DIRS`
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LoadPaths {
    /// Load paths for "system mode"
    System,

    /// Load paths for "user mode"
    User,
}

/// Access to the "File Hierarchy for the Verification of OS Artifacts (VOA)".
///
/// [`Voa`] provides file access to the verifiers stored in one or more VOA hierarchies.
/// Depending on the calling system user, a set of system-wide or user + system-wide load paths is
/// used. This access is agnostic to the cryptographic technology later using the verifiers.
pub struct Voa {
    load_paths: LoadPaths,
}

impl Voa {
    /// Initialize a Voa object, based on a set of load paths in either system mode or user mode.
    pub fn new(load_paths: LoadPaths) -> Self {
        Self { load_paths }
    }

    fn load_paths(&self) -> Vec<PathBuf> {
        match self.load_paths {
            LoadPaths::System => LOAD_PATHS_SYSTEM_MODE.iter().map(Into::into).collect(),
            LoadPaths::User => unimplemented!(),
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
    /// # fn main() -> Result<(), voa_core::types::Error> {
    /// use voa_core::{
    ///     LoadPaths,
    ///     Voa,
    ///     types::{Context, Mode, Os, Purpose, Role, Technology},
    /// };
    ///
    /// let voa = Voa::new(LoadPaths::System); // FIXME
    ///
    /// let verifiers = voa.load(
    ///     Os::new("arch".into(), None, None, None, None),
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
    pub fn load(
        &self,
        os: Os,
        purpose: Purpose,
        context: Context,
        technology: Technology,
    ) -> Vec<Verifier> {
        let mut certs = vec![];

        for load_path in self.load_paths() {
            debug!("Looking for signature verifiers in the load path {load_path:?}");

            let checked_path = match CheckedVerifierSourcePath::new(VerifierSourcePath {
                load_path: load_path.clone(),
                os: os.clone(),
                purpose,
                context: context.clone(),
                technology,
            }) {
                Ok(checked_path) => checked_path,
                Err(err) => {
                    warn!("Error while checking load path {load_path:?}: {err:?}",);
                    continue;
                }
            };

            // This has been checked to be legal according to VOA symlinking rules, and a directory
            let source_path = &checked_path.canonicalized_target;

            trace!("Loading from VOA path {source_path:?}");

            let dir = match read_dir(source_path) {
                Ok(dir) => dir,
                Err(err) => {
                    trace!("⤷ Can't read path as a directory {err:?}");
                    continue; // try next load path
                }
            };

            for res in dir {
                let entry = match res {
                    Ok(entry) => entry,
                    Err(err) => {
                        warn!("⤷ Cannot get directory entry:\n{err}");
                        continue;
                    }
                };

                let Ok(file_type) = entry.file_type() else {
                    warn!("⤷ Cannot get file type of directory entry {entry:?}");
                    continue;
                };

                let path = if file_type.is_file() {
                    &entry.path()
                } else if file_type.is_symlink() {
                    match read_link(entry.path()) {
                        // FIXME: how does this interact with chains of symlinks? Does it process
                        // them at once? If so, we still need to check legality of each intermediate
                        // hop!
                        Ok(path) => {
                            if path.as_path().to_str() == Some("/dev/null") {
                                // Individual _signature verifiers_ may be masked using
                                // a symlink to `/dev/null`, independent of
                                // [technology].

                                unimplemented!("FIXME: handle masking")
                            } else {
                                // Check that path is legal

                                // TODO: This check should actually allow links into some of the
                                // other current load paths (but not all of them!) [...]

                                let Ok(canon) = path.canonicalize() else {
                                    unimplemented!()
                                };

                                // FIXME: [..] However, symlinks can be used in the VOA
                                // hierarchy to point to files or directories below one
                                // of the [load paths] in descending priority.
                                // Symlinks to files or directories below ephemeral load
                                // paths (i.e. `/run/voa/` and `$XDG_RUNTIME_DIR/voa/`)
                                // are prohibited, as they could lead to dangling
                                // references. [..]
                                if self.load_paths().iter().any(|p| canon.starts_with(p)) {
                                    &entry.path()
                                } else {
                                    trace!("Illegal symlink target in {entry:?}");
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("⤷ Cannot get information on target of symlink {entry:?}: {e:?}");
                            continue;
                        }
                    }
                } else {
                    warn!("⤷ Unexpected file type {file_type:?} for entry {entry:?}");
                    continue;
                };

                trace!("Checking verifier file {path:?}");

                // FIXME: resolve further symlinks here?

                if path.is_file() {
                    certs.push(Verifier {
                        path: checked_path.verifier_source_path.clone(),
                        filename: entry
                            .file_name()
                            .to_str()
                            .expect("utf8 problem")
                            .to_string(), // FIXME!
                    });
                } else {
                    trace!("⤷ Verifier path is not a file {path:?}");
                }
            }
        }

        certs
    }
}
