use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::{INSTALL_SCRIPTLET_FILENAME, MetadataFileName};

/// Collects all data files in a directory, relative to it.
///
/// Convenience wrapper around [`relative_files`] that passes in all variants of
/// [`MetadataFileName`] as well as [`INSTALL_SCRIPTLET_FILENAME`] to its `filter` option.
/// This ensures, that only the paths of data files are returned.
///
/// # Errors
///
/// Returns an error if [`relative_files`] fails.
pub fn relative_data_files(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, crate::Error> {
    relative_files(
        path,
        &[
            MetadataFileName::BuildInfo.as_ref(),
            MetadataFileName::Mtree.as_ref(),
            MetadataFileName::PackageInfo.as_ref(),
            INSTALL_SCRIPTLET_FILENAME,
        ],
    )
}

/// Collects all files contained in a directory `path` as a list of relative paths.
///
/// Iterates over and returns a list of all children of `path` (see [`read_dir`]).
/// All returned children are stripped using `path` (see [`Path::strip_prefix`]), effectively
/// providing a list of relative paths below `path`.
///
/// When providing filenames using `filter`, paths that end with those filenames will be skipped and
/// not returned in the list of paths.
///
/// # Note
///
/// This function does not follow symlinks but instead persists data about those links.
///
/// # Errors
///
/// Returns an error if
/// - calling [`read_dir`] on `path` or any of its subdirectories fails,
/// - a child in one of the (sub)directories can not be retrieved,
/// - or stripping of a file in a (sub)directory fails.
pub fn relative_files(
    path: impl AsRef<Path>,
    filter: &[&str],
) -> Result<Vec<PathBuf>, crate::Error> {
    let path = path.as_ref();
    let init_path = path;

    /// Collects all files in a `path` as a list of paths and strips `init_path` from them.
    ///
    /// When providing filenames using `filter`, paths that end in those filenames will be skipped
    /// and not returned in the list of paths.
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - calling [`read_dir`] on `path` or any of its subdirectories fails,
    /// - a child in one of the (sub)directories can not be retrieved,
    /// - or stripping of a file in a (sub)directory fails.
    fn collect_files(
        path: &Path,
        init_path: &Path,
        filter: &[&str],
    ) -> Result<Vec<PathBuf>, crate::Error> {
        let mut paths = Vec::new();
        let entries = read_dir(path).map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: "reading children of directory",
            source,
        })?;
        'entries: for entry in entries {
            let entry = entry.map_err(|source| crate::Error::IoPath {
                path: path.to_path_buf(),
                context: "reading entry in directory",
                source,
            })?;
            let meta = entry.metadata().map_err(|source| crate::Error::IoPath {
                path: entry.path(),
                context: "getting metadata of file",
                source,
            })?;

            // Ignore filtered files
            '_filter: for filter in filter.iter() {
                if entry.path().ends_with(filter) {
                    continue 'entries;
                }
            }

            paths.push(
                entry
                    .path()
                    .strip_prefix(init_path)
                    .map_err(|source| crate::Error::PathStripPrefix {
                        prefix: path.to_path_buf(),
                        path: entry.path(),
                        source,
                    })?
                    .to_path_buf(),
            );

            if meta.is_dir() {
                let mut subdir_paths = collect_files(entry.path().as_path(), init_path, filter)?;
                paths.append(&mut subdir_paths);
            }
        }
        paths.sort();
        Ok(paths)
    }

    collect_files(path, init_path, filter)
}

#[cfg(test)]
mod test {
    use std::{
        fs::{File, create_dir_all},
        io::Write,
        os::unix::fs::symlink,
    };

    use rstest::rstest;
    use tempfile::tempdir;
    use testresult::TestResult;

    use super::*;

    pub const VALID_BUILDINFO_V2_DATA: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
buildenv = envbar
format = 2
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#;

    pub const VALID_PKGINFO_V2_DATA: &str = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
conflict = conflicting-package<1.0.0
provides = some-component
backup = etc/example/config.toml
depend = glibc
optdepend = python: for special-python-script.py
makedepend = cmake
checkdepend = extra-test-tool
"#;

    const VALID_INSTALL_SCRIPTLET: &str = r#"
pre_install() {
    echo "Preparing to install package version $1"
}

post_install() {
    echo "Package version $1 installed"
}

pre_upgrade() {
    echo "Preparing to upgrade from version $2 to $1"
}

post_upgrade() {
    echo "Upgraded from version $2 to $1"
}

pre_remove() {
    echo "Preparing to remove package version $1"
}

post_remove() {
    echo "Package version $1 removed"
}
"#;

    fn create_data_files(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        // Create dummy directory structure
        create_dir_all(path.join("usr/share/foo/bar/baz"))?;
        // Create dummy text file
        let mut output = File::create(path.join("usr/share/foo/beh.txt"))?;
        write!(output, "test")?;
        // Create relative symlink to actual text file
        symlink("../../beh.txt", path.join("usr/share/foo/bar/baz/beh.txt"))?;
        Ok(())
    }

    fn create_metadata_files(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        for (input_type, input) in [
            (MetadataFileName::BuildInfo, VALID_BUILDINFO_V2_DATA),
            (MetadataFileName::PackageInfo, VALID_PKGINFO_V2_DATA),
        ] {
            let mut output = File::create(path.join(input_type.as_ref()))?;
            write!(output, "{input}")?;
        }
        Ok(())
    }

    fn create_scriptlet_file(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        let mut output = File::create(path.join(INSTALL_SCRIPTLET_FILENAME))?;
        write!(output, "{VALID_INSTALL_SCRIPTLET}")?;
        Ok(())
    }

    /// Tests the successful collection of data files relative to a directory.
    #[rstest]
    fn relative_data_files_collect_successfully() -> TestResult {
        let tempdir = tempdir()?;

        create_data_files(tempdir.path())?;
        create_metadata_files(tempdir.path())?;
        create_scriptlet_file(tempdir.path())?;

        let expected_paths = vec![
            PathBuf::from("usr"),
            PathBuf::from("usr/share"),
            PathBuf::from("usr/share/foo"),
            PathBuf::from("usr/share/foo/bar"),
            PathBuf::from("usr/share/foo/bar/baz"),
            PathBuf::from("usr/share/foo/bar/baz/beh.txt"),
            PathBuf::from("usr/share/foo/beh.txt"),
        ];

        let collected_files = relative_data_files(tempdir)?;
        assert_eq!(expected_paths.as_slice(), collected_files.as_slice());

        Ok(())
    }

    /// Tests the successful collection of all files relative to a directory.
    #[rstest]
    fn relative_files_are_collected_successfully_without_filter() -> TestResult {
        let tempdir = tempdir()?;

        create_data_files(tempdir.path())?;
        create_metadata_files(tempdir.path())?;
        create_scriptlet_file(tempdir.path())?;

        let expected_paths = vec![
            PathBuf::from(MetadataFileName::BuildInfo.as_ref()),
            PathBuf::from(INSTALL_SCRIPTLET_FILENAME),
            PathBuf::from(MetadataFileName::PackageInfo.as_ref()),
            PathBuf::from("usr"),
            PathBuf::from("usr/share"),
            PathBuf::from("usr/share/foo"),
            PathBuf::from("usr/share/foo/bar"),
            PathBuf::from("usr/share/foo/bar/baz"),
            PathBuf::from("usr/share/foo/bar/baz/beh.txt"),
            PathBuf::from("usr/share/foo/beh.txt"),
        ];

        let collected_files = relative_files(tempdir, &[])?;
        assert_eq!(expected_paths.as_slice(), collected_files.as_slice());

        Ok(())
    }
}
