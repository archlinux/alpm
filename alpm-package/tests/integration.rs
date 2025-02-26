//! Integration tests for `alpm-package`.

use std::{
    fs::{File, FileTimes, create_dir, create_dir_all, read},
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    time::SystemTime,
};

use alpm_mtree::create_mtree_v2_from_input_dir;
use alpm_package::{
    CompressionSettings,
    InputDir,
    OutputDir,
    Package,
    PackageCreationConfig,
    PackageInput,
    compression::{
        Bzip2CompressionLevel,
        GzipCompressionLevel,
        XzCompressionLevel,
        ZstdCompressionLevel,
        ZstdThreads,
    },
};
use alpm_types::{Blake2b512Checksum, INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName};
use filetime::{FileTime, set_symlink_file_times};
use log::{LevelFilter, debug};
use rstest::rstest;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use tempfile::TempDir;
use testresult::TestResult;

const VALID_BUILDINFO_V2_DATA: &str = r#"
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = ccache
buildenv = color
format = 2
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = lto
options = !strip
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#;

const VALID_PKGINFO_V2_DATA: &str = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
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

/// Initializes a global [`TermLogger`].
fn init_logger() {
    if TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
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

/// Creates data files and directories below `path`.
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

/// Creates a [BUILDINFO] file in `path`.
///
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
fn create_build_info_file(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let mut file = File::create(path.join(MetadataFileName::BuildInfo.as_ref()))?;
    write!(file, "{VALID_BUILDINFO_V2_DATA}")?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// Creates a [PKGINFO] file in `path`.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
fn create_package_info_file(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let mut file = File::create(path.join(MetadataFileName::PackageInfo.as_ref()))?;
    write!(file, "{VALID_PKGINFO_V2_DATA}")?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// Creates an [alpm-install-scriptlet] file in `path`.
///
/// [alpm-install-scriptlet]: https://alpm.archlinux.page/specifications/alpm-install-scriptlet.5.html
fn create_install_scriptlet(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let mut file = File::create(path.join(INSTALL_SCRIPTLET_FILE_NAME))?;
    write!(file, "{VALID_INSTALL_SCRIPTLET}")?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// Creates a [PKGINFO] file in `path`.
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
fn create_mtree_file(path: impl AsRef<Path>) -> TestResult {
    let path = path.as_ref();
    let _mtree_file = create_mtree_v2_from_input_dir(path)?;
    let file = File::open(path.join(MetadataFileName::Mtree.as_ref()))?;
    file.set_times(default_filetimes())?;

    Ok(())
}

/// A config for creating an input directory.
struct InputDirConfig {
    build_info: bool,
    data_files: bool,
    mtree: bool,
    package_info: bool,
    scriptlet: bool,
}

/// Prepares the input directory with all necessary files for package creation.
fn prepare_input_dir(path: impl AsRef<Path>, config: &InputDirConfig) -> TestResult {
    let path = path.as_ref();

    // Create package data files.
    if config.data_files {
        create_data_files(path)?;
    }
    if config.scriptlet {
        create_install_scriptlet(path)?;
    }
    // Create metadata files.
    if config.build_info {
        create_build_info_file(path)?;
    }
    if config.package_info {
        create_package_info_file(path)?;
    }
    if config.mtree {
        create_mtree_file(path)?;
    }

    Ok(())
}

/// Creates a package from an input directory and returns it and its hash digest.
///
/// Creates an `input` and `output` directory in `path` (the caller has to ensure that those are
/// unique!).
/// Uses `input_dir_config` to configure the input directory and `compression` during the creation
/// of the package.
fn package_digest(
    path: impl AsRef<Path>,
    input: &str,
    output: &str,
    compression: Option<CompressionSettings>,
    input_dir_config: &InputDirConfig,
) -> TestResult<(PathBuf, Blake2b512Checksum)> {
    let path = path.as_ref();
    debug!("Creating package digest while using test dir {path:?}");

    // Create the input and output directories.
    let input_dir_path = path.join(input);
    create_dir(&input_dir_path)?;
    let input_dir = InputDir::new(input_dir_path)?;
    let output_dir = OutputDir::new(path.join(output))?;

    // Prepare the input directory based on InputDirConfig.
    prepare_input_dir(&input_dir, input_dir_config)?;

    // Create PackageInput
    let package_input: PackageInput = input_dir.try_into()?;
    let config = PackageCreationConfig::new(package_input, output_dir, compression)?;

    // Create package file.
    let package = Package::try_from(&config)?;
    let buf = read(package.to_path_buf())?;

    Ok((
        package.to_path_buf(),
        Blake2b512Checksum::calculate_from(buf),
    ))
}

/// Check that [alpm-package] files can be created reproducibly from input directories.
///
/// This test assumes, that all files in the input directory have the same timestamps and contain
/// the same data when re-creating a package.
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[rstest]
#[case::bzip2_compression_all_files(
    Some(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::bzip2_compression_no_scriptlet(
    Some(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::bzip2_compression_no_data_files(
    Some(CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
#[case::gzip_compression_all_files(
    Some(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::gzip_compression_no_scriptlet(
    Some(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::gzip_compression_no_data_files(
    Some(CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
#[case::xz_compression_all_files(
    Some(CompressionSettings::Xz { compression_level: XzCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::xz_compression_no_scriptlet(
    Some(CompressionSettings::Xz { compression_level: XzCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::xz_compression_no_data_files(
    Some(CompressionSettings::Xz { compression_level: XzCompressionLevel::default() }),
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
#[case::zstd_compression_all_files(
    Some(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::all() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::zstd_compression_no_scriptlet(
    Some(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::all() }),
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::zstd_compression_no_data_files(
    Some(CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::all() }),
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
#[case::no_compression_all_files(
    None,
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::no_compression_no_scriptlet(
    None,
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::no_compression_no_data_files(
    None,
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
fn create_package_from_input(
    #[case] compression: Option<CompressionSettings>,
    #[case] input_dir_config: InputDirConfig,
) -> TestResult {
    init_logger();

    // Create a common temporary dir.
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path();

    let (package_a, package_digest_a) = package_digest(
        test_dir,
        "input1",
        "output1",
        compression.clone(),
        &input_dir_config,
    )?;
    debug!("Created package A: {package_a:?}");

    let (package_b, package_digest_b) = package_digest(
        test_dir,
        "input2",
        "output2",
        compression.clone(),
        &input_dir_config,
    )?;
    debug!("Created package B: {package_b:?}");

    assert_eq!(package_digest_a, package_digest_b);
    Ok(())
}

/// Ensures that [`PackageInput::from_input_dir`] fails on missing metadata files.
#[rstest]
#[case::no_build_info(
    InputDirConfig { build_info: false, data_files: false, mtree: true, package_info: true, scriptlet: false },
    PathBuf::from(MetadataFileName::BuildInfo.as_ref()),
)]
#[case::no_package_info(
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: false, scriptlet: false },
    PathBuf::from(MetadataFileName::PackageInfo.as_ref()),
)]
#[case::no_mtree(
    InputDirConfig { build_info: true, data_files: false, mtree: false, package_info: true, scriptlet: false },
    PathBuf::from(MetadataFileName::Mtree.as_ref()),
)]
fn package_input_fails_on_missing_metadata(
    #[case] config: InputDirConfig,
    #[case] expected: PathBuf,
) -> TestResult {
    init_logger();

    // Create a common temporary dir.
    let temp_dir = TempDir::new()?;
    let input_dir = InputDir::new(temp_dir.path().to_path_buf())?;

    prepare_input_dir(&input_dir, &config)?;

    if let Err(error) = PackageInput::try_from(input_dir) {
        match error {
            alpm_package::Error::Input(alpm_package::input::Error::FileIsMissing {
                path,
                input_dir: _,
            }) => assert_eq!(path, expected),
            _ => return Err("Did not return the correct error variant".into()),
        }
    } else {
        return Err("Should have returned an error but succeeded".into());
    }

    Ok(())
}

/// Ensures that [`PackageInput`] has the correct amount of files depending on whether data files
/// and scriptlet are present.
#[rstest]
#[case::all_files(
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: true },
)]
#[case::no_scriptlet(
    InputDirConfig { build_info: true, data_files: true, mtree: true, package_info: true, scriptlet: false },
)]
#[case::no_data_files_no_scriptlet(
    InputDirConfig { build_info: true, data_files: false, mtree: true, package_info: true, scriptlet: false },
)]
fn test_package_input_methods(#[case] config: InputDirConfig) -> TestResult {
    init_logger();

    // Create a common temporary dir.
    let temp_dir = TempDir::new()?;
    let input_dir = InputDir::new(temp_dir.path().to_path_buf())?;

    prepare_input_dir(&input_dir, &config)?;
    let package_input: PackageInput = input_dir.try_into()?;

    // Ensure that all metadata can be retrieved.
    let _mtree = package_input.mtree()?;
    let _build_info = package_input.build_info();
    let _package_info = package_input.package_info();

    if config.scriptlet {
        assert!(package_input.install_scriptlet().is_some());
        if !config.data_files {
            // Only metadata files and the scriptlet are present
            assert!(package_input.relative_paths().len() == 4)
        } else {
            assert!(package_input.relative_paths().len() > 4)
        }
    } else {
        assert!(package_input.install_scriptlet().is_none());
        if !config.data_files {
            // Only metadata files are present
            assert!(package_input.relative_paths().len() == 3)
        } else {
            assert!(package_input.relative_paths().len() > 3)
        }
    }

    Ok(())
}

/// Ensures that [`PackageCreationConfig::new`] fails on overlapping input and output directories.
///
/// This includes that the output directory may not be a subdirectory of the input directory and
/// vice versa.
#[test]
fn package_creation_config_new_fails() -> TestResult {
    init_logger();

    let config = InputDirConfig {
        build_info: true,
        data_files: false,
        mtree: true,
        package_info: true,
        scriptlet: false,
    };

    // Create a common temporary dir.
    let temp_dir = TempDir::new()?;
    let path_equal = temp_dir.path().join("equal-input");
    create_dir_all(&path_equal)?;
    let input_dir = InputDir::new(path_equal)?;

    prepare_input_dir(&input_dir, &config)?;
    let package_input: PackageInput = input_dir.clone().try_into()?;

    // Create an input and output at the same location.
    let output_dir = OutputDir::new(input_dir.to_path_buf())?;
    match PackageCreationConfig::new(package_input.clone(), output_dir, None) {
        Err(error) => assert!(matches!(
            error,
            alpm_package::Error::InputDirIsOutputDir { path: _ }
        )),
        Ok(_) => return Err("Succeeded, but should have failed".into()),
    }

    // Set the output to a sudirectory of the input directory.
    let output_dir = OutputDir::new(input_dir.join("output-in-input"))?;
    match PackageCreationConfig::new(package_input.clone(), output_dir, None) {
        Err(error) => assert!(matches!(
            error,
            alpm_package::Error::OutputDirInInputDir {
                input_path: _,
                output_path: _
            }
        )),
        Ok(_) => return Err("Succeeded, but should have failed".into()),
    }

    // Set the input to a subdirectory of the output directory.
    let output_dir = OutputDir::new(temp_dir.path().to_path_buf())?;
    match PackageCreationConfig::new(package_input, output_dir, None) {
        Err(error) => assert!(matches!(
            error,
            alpm_package::Error::InputDirInOutputDir {
                input_path: _,
                output_path: _
            }
        )),
        Ok(_) => return Err("Succeeded, but should have failed".into()),
    }

    Ok(())
}
