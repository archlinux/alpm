//! Tests for [`Mtree::validate_paths`].
//!
//! Covers all cases, apart from mismatching UID/GID for files (as this would require root).
//!
//! Additionally, this does not cover the test case of the [ALPM-MTREE] file being part of the
//! [ALPM-MTREE] data, as [`mtree_v2_from_input_dir`] prevents this from happening.
//! Arguably, covering this would only satisfy the potential behavior of misbehaving
//! implementations.
//!
//! [ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
#![cfg(feature = "creation")]

use std::{
    fs::{
        File,
        FileTimes,
        Permissions,
        create_dir_all,
        remove_dir,
        remove_file,
        rename,
        set_permissions,
    },
    io::Write,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    thread::current,
    time::{Duration, SystemTime},
};

use alpm_common::{InputPaths, MetadataFile, relative_files};
use alpm_mtree::{Mtree, create_mtree_v2_from_input_dir};
use alpm_types::MetadataFileName;
use filetime::{FileTime, set_symlink_file_times};
use insta::{Settings, assert_snapshot, with_settings};
use log::debug;
use rstest::rstest;
use simplelog::{Config, TermLogger};
use tempfile::TempDir;
use testresult::TestResult;

/// A string slice representing valid [BUILDINFOv2] data.
///
/// [BUILDINFOv2]: https://alpm.archlinux.page/specifications/BUILIDNFOv2.5.html
pub const VALID_BUILDINFO_V2_DATA: &str = r#"
format = 2
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = envfoo
buildenv = envbar
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

/// A string slice representing valid [PKGINFOv2] data.
///
/// [PKGINFOv2]: https://alpm.archlinux.page/specifications/PKGINFOv2.5.html
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

macro_rules! apply_common_filters {
    {} => {
        let mut settings = Settings::clone_current();
        // Linux temp dir in runtime directory.
        settings.add_filter(r"/run/user/\d+/\.tmp[A-Za-z0-9]+", "[TEMP_FILE]");
        // Linux temp dir in /tmp directory.
        settings.add_filter(r"/tmp/\.tmp[A-Za-z0-9]+", "[TEMP_FILE]");
        let _bound = settings.bind_to_scope();
    }
}

/// Returns a default [`FileTimes`] struct with all times set to UNIX_EPOCH.
fn default_filetimes() -> FileTimes {
    FileTimes::new()
        .set_accessed(SystemTime::UNIX_EPOCH)
        .set_modified(SystemTime::UNIX_EPOCH)
}

/// Returns a default [`FileTime`] struct with all times set to UNIX_EPOCH.
fn default_filetime() -> FileTime {
    FileTime::from_unix_time(0, 0)
}

/// Returns a [`FileTimes`] struct with custom seconds since the UNIX_EPOCH.
fn new_filetimes(seconds: u64) -> TestResult<FileTimes> {
    let epoch = SystemTime::UNIX_EPOCH;
    let duration = Duration::new(seconds, 0);
    let Some(new) = epoch.checked_add(duration) else {
        panic!("Can not add duration to unix epoch");
    };
    Ok(FileTimes::new().set_accessed(new).set_modified(new))
}

/// Returns a [`FileTime`] with custom seconds since the UNIX_EPOCH.
fn new_filetime(seconds: i64) -> FileTime {
    FileTime::from_unix_time(seconds, 0)
}

/// Creates test files and directories below `path`.
fn create_data_files(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    // Create dummy directory structure
    create_dir_all(path.join("foo/bar/baz"))?;
    create_dir_all(path.join("foo/bar/buh"))?;

    // Create dummy text file
    let mut file = File::create(path.join("foo/beh.txt"))?;
    write!(file, "test")?;
    file.set_times(default_filetimes())?;

    // Create relative symlink to actual text file
    let existing_target_link = path.join("foo/bar/baz/beh.txt");
    symlink("../../beh.txt", &existing_target_link)?;
    set_symlink_file_times(
        &existing_target_link,
        default_filetime(),
        default_filetime(),
    )?;

    // Create symlink to absolute, non-existing file.
    let non_existing_target_link = path.join("foo/bar/baz/buh.txt");
    symlink(
        "/tmp/something/very/unlikely/to/ever/exist/hopefully.txt",
        &non_existing_target_link,
    )?;
    set_symlink_file_times(
        &non_existing_target_link,
        default_filetime(),
        default_filetime(),
    )?;

    // Adjust file times of directory structure.
    for path in [
        path.join("foo/bar/baz"),
        path.join("foo/bar/buh"),
        path.join("foo/bar"),
        path.join("foo"),
    ] {
        let file = File::open(path)?;
        file.set_times(default_filetimes())?;
    }

    Ok(())
}

/// Creates ALPM package metadata files in `path`.
fn create_build_info_file(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let mut file = File::create(path.join(MetadataFileName::BuildInfo.as_ref()))?;
    write!(file, "{VALID_BUILDINFO_V2_DATA}")?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// Creates ALPM package metadata files in `path`.
fn create_package_info_file(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let mut file = File::create(path.join(MetadataFileName::PackageInfo.as_ref()))?;
    write!(file, "{VALID_PKGINFO_V2_DATA}")?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// Prepares the input directory by creating (meta)data files and ALPM-MTREE file.
fn prepare_input_dir() -> TestResult<(Mtree, TempDir)> {
    let test_dir = TempDir::new()?;
    let path = test_dir.path();

    // Create dummy package data files.
    create_data_files(path)?;

    // Create metadata files.
    create_build_info_file(path)?;
    create_package_info_file(path)?;

    // Create .MTREE file (as ALPM-MTREEv2) and derive an Mtree from it.
    let mtree_file = create_mtree_v2_from_input_dir(path)?;

    Ok((Mtree::from_file(mtree_file)?, test_dir))
}

/// Initializes a global logger once.
fn init_logger() -> TestResult {
    if TermLogger::init(
        log::LevelFilter::Trace,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }

    Ok(())
}

/// Creates a package input directory and validates it using an ALPM-MTREE file.
#[rstest]
fn validate_paths_success() -> TestResult {
    init_logger()?;

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    mtree.validate_paths(&InputPaths::new(path, &relative_files)?)?;

    Ok(())
}

/// Creates a package input directory and fails to validates it because duplicate paths are provided
/// as input.
#[rstest]
fn validate_paths_fails_on_duplicate_input_paths() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    if let Err(error) = mtree.validate_paths(&InputPaths::new(
        path,
        &[PathBuf::from("duplicate"), PathBuf::from("duplicate")],
    )?) {
        with_settings!({
                    description => "duplicate path".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should have failed");
    };

    Ok(())
}

/// Creates a package input directory and fails to validates it because input dir and ALPM-MTREE
/// have different amounts of files.
#[rstest]
fn validate_paths_fails_on_paths_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Retrieve all files relative to input dir, creating a mismatch by also including .MTREE.
    let relative_files = relative_files(path, &[])?;
    remove_dir(path.join("foo/bar/buh"))?;
    remove_file(path.join("foo/beh.txt"))?;
    remove_file(path.join("foo/bar/baz/beh.txt"))?;
    let file = File::open(path.join("foo/bar/baz"))?;
    file.set_times(default_filetimes())?;
    let file = File::open(path.join("foo/bar"))?;
    file.set_times(default_filetimes())?;
    let file = File::open(path.join("foo"))?;
    file.set_times(default_filetimes())?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "paths mismatch".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should have failed");
    };

    Ok(())
}

/// Creates a package input directory and fails to validate it because a file in the input dir does
/// not match the ALPM-MTREE data.
#[rstest]
fn validate_paths_fails_on_data_path_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify a data file to have a mismatching name (but matching modification time).
    debug!(
        "Rename {:?} to {:?}",
        path.join("foo/beh.txt"),
        path.join("foo/bar.txt")
    );
    rename(path.join("foo/beh.txt"), path.join("foo/bar.txt"))?;
    let file = File::open(path.join("foo/bar.txt"))?;
    file.set_times(default_filetimes())?;
    let file = File::open(path.join("foo"))?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "Path names mismatch".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should have failed");
    };

    Ok(())
}

/// Creates a package input directory and fails to validate it because a file in the input dir
/// should be a directory.
#[rstest]
fn validate_paths_fails_on_not_a_dir() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by removing a directory and replacing it with a file (with
    // matching modification time).
    remove_dir(path.join("foo/bar/buh"))?;
    let mut file = File::create(path.join("foo/bar/buh"))?;
    write!(file, "test")?;
    file.set_times(default_filetimes())?;

    let file = File::open(path.join("foo/bar"))?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file is not a directory".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file should be a symlink.
#[rstest]
fn validate_paths_fails_on_not_a_symlink() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by removing a symlink and replacing it with a file (with matching
    // modification time).
    remove_file(path.join("foo/bar/baz/buh.txt"))?;
    let mut file = File::create(path.join("foo/bar/baz/buh.txt"))?;
    write!(file, "test")?;
    file.set_times(default_filetimes())?;
    let file = File::open(path.join("foo/bar/baz"))?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file is not a symlink".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a symlink points at a wrong
/// file.
#[rstest]
fn validate_paths_fails_on_symlink_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by replacing the destination of a symlink.
    remove_file(path.join("foo/bar/baz/buh.txt"))?;
    symlink(
        "/tmp/something/other/very/unlikely/to/ever/exist/hopefully.txt",
        path.join("foo/bar/baz/buh.txt"),
    )?;
    set_symlink_file_times(
        path.join("foo/bar/baz/buh.txt"),
        default_filetime(),
        default_filetime(),
    )?;

    let file = File::open(path.join("foo/bar/baz"))?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A symlink does not point at the correct file".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file is not a file.
#[rstest]
fn validate_paths_fails_on_not_a_file() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by replacing a file with a directory.
    remove_file(path.join("foo/beh.txt"))?;
    create_dir_all(path.join("foo/beh.txt"))?;
    let file = File::open(path.join("foo"))?;
    file.set_times(default_filetimes())?;
    let file = File::open(path.join("foo/beh.txt"))?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file is not a file".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file has the wrong size.
#[rstest]
fn validate_paths_fails_on_size_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by adding different content to a file (but retain initial
    // modification time of file).
    let mut file = File::create(path.join("foo/beh.txt"))?;
    write!(file, "test23")?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file is not of the correct size".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file has the wrong hash
/// digest.
#[rstest]
fn validate_paths_fails_on_digest_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by adding new content (of the same length as the previous) to a
    // file (but retain the original file modification time).
    let mut file = File::create(path.join("foo/beh.txt"))?;
    write!(file, "TEST")?;
    file.set_times(default_filetimes())?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file digest does not match".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file has a different
/// creation time.
#[rstest]
fn validate_paths_fails_on_time_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by changing the creation time of a file, directory and symlink.
    let file = File::open(path.join("foo/beh.txt"))?;
    file.set_times(new_filetimes(1)?)?;
    let dir = File::open(path.join("foo"))?;
    dir.set_times(new_filetimes(1)?)?;
    set_symlink_file_times(
        path.join("foo/bar/baz/beh.txt"),
        new_filetime(1),
        new_filetime(1),
    )?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file creation time does not match".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}

/// Fail to validate an input dir with [`Mtree::validate_paths`] because a file has the wrong mode.
#[rstest]
fn validate_paths_fails_on_mode_mismatch() -> TestResult {
    init_logger()?;
    apply_common_filters!();

    // Prepare the input dir and create an Mtree object.
    let (mtree, test_dir) = prepare_input_dir()?;
    let path = test_dir.path();

    // Modify the input directory by changing the mode of a file.
    set_permissions(path.join("foo/beh.txt"), Permissions::from_mode(0o640))?;

    // Retrieve all files relative to input dir (excluding the ALPM-MTREE file).
    let relative_files = relative_files(path, &[".MTREE"])?;

    if let Err(error) = mtree.validate_paths(&InputPaths::new(path, &relative_files)?) {
        with_settings!({
                    description => "A file mode does not match".to_string(),
                    snapshot_path => "validate_snapshots",
                    prepend_module_to_snapshot => false,
                }, {
                    assert_snapshot!(current()
                    .name()
                    .unwrap()
                    .to_string()
                    .replace("::", "__")
        , format!("{error}"));
                });
    } else {
        panic!("The validation succeeded even though it should've failed");
    };

    Ok(())
}
