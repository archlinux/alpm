//! File Hierarchy for the Verification of OS Artifacts (VOA)
//!
//! Types for voa-core

use std::{
    fmt::{Debug, Formatter},
    fs::File,
    path::{Path, PathBuf},
};

use crate::load_path::LoadPath;
pub use crate::{
    error::Error,
    identifiers::{
        Context,
        CustomContext,
        CustomRole,
        CustomTechnology,
        Mode,
        Os,
        Purpose,
        Role,
        Technology,
    },
};

/// Specifies a logical location in a VOA directory structure.
#[derive(Clone, Debug, PartialEq)]
pub struct VerifierSourcePath {
    load_path: LoadPath,
    os: Os,
    purpose: Purpose,
    context: Context,
    technology: Technology,
}

impl VerifierSourcePath {
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

    /// The filesystem path that this [VerifierSourcePath] represents.
    /// This representation of the path doesn't canonicalize symlinks, if any.
    #[allow(dead_code)]
    pub(crate) fn to_path_buf(&self) -> PathBuf {
        self.load_path
            .path
            .join(self.os.path_segment())
            .join(self.purpose.path_segment())
            .join(self.context.path_segment())
            .join(self.technology.path_segment())
    }

    /// The load path of the [`VerifierSourcePath`].
    pub fn load_path(&self) -> &Path {
        // Note: the LoadPath type is handled as an internal implementation detail,
        // we're just returning a &Path

        &self.load_path.path
    }

    /// The [`Os`] of the [`VerifierSourcePath`].
    pub fn os(&self) -> &Os {
        &self.os
    }

    /// The [`Purpose`] of the [`VerifierSourcePath`].
    pub fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    /// The [`Context`] of the [`VerifierSourcePath`].
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The [`Technology`] of the [`VerifierSourcePath`].
    pub fn technology(&self) -> &Technology {
        &self.technology
    }
}

/// Points to a signature verifier in the file system.
///
/// Depending on the technology, this may represent, e.g.:
/// - an individual, loose verifier
/// - a certificate complete with its trust chain
/// - a set of individual verifiers in one shared data structure
pub struct Verifier {
    /// Specification of the path from which the verifier was loaded
    pub(crate) verifier_path: VerifierSourcePath,

    /// Canonicalized path of the verifier file, in [`Verifier::canonicalized`]
    pub(crate) canonicalized: PathBuf,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Verifier source path: {:#?}", self.verifier_path)?;
        writeln!(f, "Canonicalized verifier path: {:?}", self.canonicalized)?;

        Ok(())
    }
}

impl Verifier {
    /// The verifier source path definition that this verifier file was found through
    pub fn verifier_path(&self) -> &VerifierSourcePath {
        &self.verifier_path
    }

    /// The canonicalized filename (excluding the path)
    pub(crate) fn filename(&self) -> Option<&std::ffi::OsStr> {
        self.canonicalized.file_name()
    }

    /// The canonicalized [`Path`] representation of this [`Verifier`]
    pub fn canonicalized(&self) -> &Path {
        &self.canonicalized
    }

    /// Open this verifier as a file in read-only mode
    pub fn open(&self) -> std::io::Result<File> {
        File::open(&self.canonicalized)
    }
}

#[test]
fn role_to_string() {
    assert_eq!(Role::Image.to_string(), "image");

    let foo = CustomRole::new("foo".to_string()).unwrap();
    let custom_role = Role::Custom(foo);
    assert_eq!(custom_role.to_string(), "foo");
}
