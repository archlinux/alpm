//! Helpers for package input handling.
//!
//! Contains functions for generically deriving the files and directories contained in a package
//! input directory.
//! This functionality is used by libraries and tools that deal with files in input directories
//! (e.g. [ALPM-MTREE] and [alpm-package]).
//!
//! [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
//! [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html

use std::{
    fs::read_dir,
    path::{MAIN_SEPARATOR_STR, Path, PathBuf},
};

use alpm_types::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName};

/// An input path.
///
/// Tracks an absolute base directory and a relative path.
#[derive(Clone, Copy, Debug)]
pub struct InputPath<'a, 'b> {
    base_dir: &'a Path,
    path: &'b Path,
}

impl<'a, 'b> InputPath<'a, 'b> {
    /// Creates a new [`InputPath`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `base_dir` is not absolute,
    /// - `base_dir` is not a directory,
    /// - or `path` is not relative.
    pub fn new(base_dir: &'a Path, path: &'b Path) -> Result<Self, crate::Error> {
        if !base_dir.is_absolute() {
            return Err(crate::Error::NonAbsolutePaths {
                paths: vec![base_dir.to_path_buf()],
            });
        }
        if !base_dir.is_dir() {
            return Err(crate::Error::NotADirectory {
                path: base_dir.to_path_buf(),
            });
        }

        if !path.is_relative() {
            return Err(crate::Error::NonRelativePaths {
                paths: vec![path.to_path_buf()],
            });
        }

        Ok(Self { base_dir, path })
    }

    /// Returns a reference to the base dir.
    pub fn base_dir(&self) -> &Path {
        self.base_dir
    }

    /// Returns a reference to the relative path in base dir.
    pub fn path(&self) -> &Path {
        self.path
    }

    /// Returns an absolute path as combination of base directory and relative path.
    pub fn to_path_buf(&self) -> PathBuf {
        self.base_dir.join(self.path)
    }
}

/// A set of input paths.
///
/// Tracks a base directory and a set of relative paths.
#[derive(Clone, Copy, Debug)]
pub struct InputPaths<'a, 'b> {
    base_dir: &'a Path,
    paths: &'b [PathBuf],
}

impl<'a, 'b> InputPaths<'a, 'b> {
    /// Creates a new [`InputPaths`].
    ///
    /// # Errors
    ///
    /// Returns an error if
    /// - `base_dir` is not absolute,
    /// - `base_dir` is not a directory,
    /// - or one or more paths in `paths` are not relative.
    pub fn new(base_dir: &'a Path, paths: &'b [PathBuf]) -> Result<Self, crate::Error> {
        if !base_dir.is_absolute() {
            return Err(crate::Error::NonAbsolutePaths {
                paths: vec![base_dir.to_path_buf()],
            });
        }
        if !base_dir.is_dir() {
            return Err(crate::Error::NotADirectory {
                path: base_dir.to_path_buf(),
            });
        }

        let mut non_relative = Vec::new();
        for path in paths {
            if !path.is_relative() {
                non_relative.push(path.clone())
            }
        }
        if !non_relative.is_empty() {
            return Err(crate::Error::NonRelativePaths {
                paths: non_relative,
            });
        }

        Ok(Self { base_dir, paths })
    }

    /// Returns a reference to the base dir.
    pub fn base_dir(&self) -> &Path {
        self.base_dir
    }

    /// Returns a reference to the list of relative paths in base dir.
    pub fn paths(&self) -> &[PathBuf] {
        self.paths
    }
}

/// Collects all data files in a directory, relative to it.
///
/// Convenience wrapper around [`relative_files`] that passes in all variants of
/// [`MetadataFileName`] as well as [`INSTALL_SCRIPTLET_FILE_NAME`] to its `filter` option.
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
            INSTALL_SCRIPTLET_FILE_NAME,
        ],
    )
}

/// Collects all files contained in a directory `path` as a list of sorted relative paths.
///
/// Recursively iterates over all entries of `path` (see [`read_dir`]).
/// All returned entries are stripped using `path` (see [`Path::strip_prefix`]), effectively
/// providing a list of relative paths below `path`.
/// The list of paths is sorted (see [`slice::sort`]).
///
/// When providing file names using `filter`, any path found ending with one of the filter names
/// will be skipped and not returned in the list of paths.
///
/// # Note
///
/// This function does not follow symlinks but instead returns the path of a symlink.
///
/// # Errors
///
/// Returns an error if
///
/// - calling [`read_dir`] on `path` or any of its subdirectories fails,
/// - an entry in one of the (sub)directories can not be retrieved,
/// - or stripping the prefix of a file in a (sub)directory fails.
pub fn relative_files(
    path: impl AsRef<Path>,
    filter: &[&str],
) -> Result<Vec<PathBuf>, crate::Error> {
    let path = path.as_ref();
    let init_path = path;

    /// Collects all files in a `path` as a sorted list of paths and strips `init_path` from them.
    ///
    /// Recursively calls itself on all directories contained in `path`, retaining `init_path` and
    /// `filter` in these calls.
    /// When providing filenames using `filter`, paths that end in those filenames will be skipped
    /// and not returned in the list of paths.
    ///
    /// # Errors
    ///
    /// Returns an error if
    ///
    /// - calling [`read_dir`] on `path` or any of its subdirectories fails,
    /// - an entry in one of the (sub)directories can not be retrieved,
    /// - or stripping the prefix of a file in a (sub)directory fails.
    fn collect_files(
        path: &Path,
        init_path: &Path,
        filter: &[&str],
    ) -> Result<Vec<PathBuf>, crate::Error> {
        let mut paths = Vec::new();
        let entries = read_dir(path).map_err(|source| crate::Error::IoPath {
            path: path.to_path_buf(),
            context: "reading entries of directory",
            source,
        })?;
        for entry in entries {
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

            // Ignore filtered files or directories.
            if filter.iter().any(|filter| entry.path().ends_with(filter)) {
                continue;
            }

            paths.push({
                let mut path = entry
                    .path()
                    .strip_prefix(init_path)
                    .map_err(|source| crate::Error::PathStripPrefix {
                        prefix: path.to_path_buf(),
                        path: entry.path(),
                        source,
                    })?
                    .to_path_buf();

                // Add a trailing "/" to directory paths, if there isn't one already.
                if meta.is_dir()
                    && path
                        .as_os_str()
                        .to_str()
                        .is_some_and(|path| !path.ends_with(MAIN_SEPARATOR_STR))
                {
                    path.as_mut_os_string().push(MAIN_SEPARATOR_STR);
                }

                path
            });

            // Call `collect_files` on each directory, retaining the initial `init_path` and
            // `filter`.
            if meta.is_dir() {
                let mut subdir_paths = collect_files(entry.path().as_path(), init_path, filter)?;
                paths.append(&mut subdir_paths);
            }
        }

        // Sort paths.
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
    use tempfile::{NamedTempFile, TempDir, tempdir};
    use testresult::TestResult;

    use super::*;

    pub const VALID_BUILDINFO_V2_DATA: &str = r#"
format = 2
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = ccache
buildenv = color
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
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
        File::create(path.join("usr/share/foo/beh.txt"))?.write_all(b"test")?;
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
            File::create(path.join(input_type.as_ref()))?.write_all(input.as_bytes())?;
        }
        Ok(())
    }

    fn create_scriptlet_file(path: impl AsRef<Path>) -> TestResult {
        let path = path.as_ref();
        let mut output = File::create(path.join(INSTALL_SCRIPTLET_FILE_NAME))?;
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
            PathBuf::from(INSTALL_SCRIPTLET_FILE_NAME),
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

    /// Tests all success and failure scenarios when creating [`InputPath`].
    #[test]
    fn input_path_new() -> TestResult {
        let temp_dir = TempDir::new()?;
        let temp_dir_path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;
        let temp_file_path = temp_file.path();

        let relative_path = PathBuf::from("some_file.txt");
        let absolute_path = PathBuf::from("/some_file.txt");

        assert!(InputPath::new(temp_dir_path, &relative_path).is_ok());
        assert!(matches!(
            InputPath::new(temp_dir_path, &absolute_path),
            Err(crate::Error::NonRelativePaths { .. })
        ));
        assert!(matches!(
            InputPath::new(temp_file_path, &relative_path),
            Err(crate::Error::NotADirectory { .. })
        ));
        assert!(matches!(
            InputPath::new(&relative_path, &relative_path),
            Err(crate::Error::NonAbsolutePaths { .. })
        ));

        Ok(())
    }

    /// Tests all success and failure scenarios when creating [`InputPaths`].
    #[test]
    fn input_paths_new() -> TestResult {
        let temp_dir = TempDir::new()?;
        let temp_dir_path = temp_dir.path();
        let temp_file = NamedTempFile::new()?;
        let temp_file_path = temp_file.path();

        let relative_path_a = PathBuf::from("some_file.txt");
        let relative_path_b = PathBuf::from("some_other_file.txt");
        let absolute_path_a = PathBuf::from("/some_file.txt");
        let absolute_path_b = PathBuf::from("/some_other_file.txt");

        assert!(
            InputPaths::new(
                temp_dir_path,
                &[relative_path_a.clone(), relative_path_b.clone()]
            )
            .is_ok()
        );
        assert!(matches!(
            InputPaths::new(temp_dir_path, &[absolute_path_a, absolute_path_b]),
            Err(crate::Error::NonRelativePaths { .. })
        ));
        assert!(matches!(
            InputPaths::new(temp_file_path, std::slice::from_ref(&relative_path_a)),
            Err(crate::Error::NotADirectory { .. })
        ));
        assert!(matches!(
            InputPaths::new(&relative_path_a, std::slice::from_ref(&relative_path_a)),
            Err(crate::Error::NonAbsolutePaths { .. })
        ));

        Ok(())
    }
}
