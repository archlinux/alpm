//! Checks for [alpm-install-scriptlet] files.
//!
//! [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html

use std::{fs::File, io::Read, path::Path};

use crate::Error;

/// Function signatures of which at least one must be present in an [alpm-install-scriptlet]
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
const REQUIRED_FUNCTION_SIGNATURES: &[&str] = &[
    "pre_install",
    "post_install",
    "pre_upgrade",
    "post_upgrade",
    "pre_remove",
    "post_remove",
];

/// Validates an [alpm-install-scriptlet] at `path`.
///
/// Naively checks whether at least one of the required function signatures is present in the file.
///
/// # Note
///
/// The file at `path` is _neither sourced nor fully evaluated_.
/// This function only provides a _very limited_ validity check!
///
/// # Errors
///
/// Returns an error, if
/// - `path` can not be opened for reading,
/// - `path` can not be read to String,
/// - none of the required function signatures is present in the file.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
pub fn check_scriptlet(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    let mut file = File::open(path).map_err(|source| Error::IoPath {
        path: path.to_path_buf(),
        context: "opening an alpm-install-scriptlet file for reading",
        source,
    })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|source| Error::IoPath {
            path: path.to_path_buf(),
            context: "reading the contents to string",
            source,
        })?;

    for line in buf.lines() {
        for function_name in REQUIRED_FUNCTION_SIGNATURES {
            if line.starts_with(&format!("{function_name}()"))
                || line.starts_with(&format!("{function_name}() {{"))
                || line.starts_with(&format!("function {function_name}()"))
                || line.starts_with(&format!("function {function_name}() {{"))
            {
                return Ok(());
            }
        }
    }

    Err(Error::InstallScriptlet {
        path: path.to_path_buf(),
        context: format!(
            "it must implement at least one of the functions: {}",
            REQUIRED_FUNCTION_SIGNATURES.join(", ")
        ),
    })
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use rstest::rstest;
    use tempfile::NamedTempFile;
    use testresult::TestResult;

    use super::*;

    const INSTALL_SCRIPTLET_FULL: &str = r#"pre_install() {
  true
}

post_install() {
  true
}

pre_upgrade() {
  true
}

post_upgrade() {
  true
}

pre_remove() {
  true
}

post_remove() {
  true
}"#;
    const INSTALL_SCRIPTLET_FULL_FUNCTION_PREFIX: &str = r#"function pre_install() {
  true
}

function post_install() {
  true
}

function pre_upgrade() {
  true
}

function post_upgrade() {
  true
}

function pre_remove() {
  true
}

function post_remove() {
  true
}"#;
    const INSTALL_SCRIPTLET_EMPTY: &str = "";
    const INSTALL_SCRIPTLET_INVALID: &str = r#"pre_install
post_install
pre_upgrade
post_upgrade
pre_remove
post_remove"#;
    const INSTALL_SCRIPTLET_INVALID_FUNCTION_PREFIX: &str = r#"function pre_install
function post_install
function pre_upgrade
function post_upgrade
function pre_remove
function post_remove"#;

    #[rstest]
    #[case(INSTALL_SCRIPTLET_FULL)]
    #[case(INSTALL_SCRIPTLET_FULL_FUNCTION_PREFIX)]
    fn valid_scriptlet(#[case] scriptlet: &str) -> TestResult {
        let mut file = NamedTempFile::new()?;
        write!(file, "{scriptlet}")?;

        check_scriptlet(file.path())?;

        Ok(())
    }

    #[rstest]
    #[case(INSTALL_SCRIPTLET_EMPTY)]
    #[case(INSTALL_SCRIPTLET_INVALID)]
    #[case(INSTALL_SCRIPTLET_INVALID_FUNCTION_PREFIX)]
    fn invalid_scriptlet(#[case] scriptlet: &str) -> TestResult {
        let mut file = NamedTempFile::new()?;
        write!(file, "{scriptlet}")?;

        assert!(check_scriptlet(file.path()).is_err());

        Ok(())
    }
}
