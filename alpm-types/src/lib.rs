#![doc = include_str!("../README.md")]

mod checksum;
pub use checksum::{
    Blake2b512Checksum,
    Checksum,
    ChecksumAlgorithm,
    Crc32CksumChecksum,
    DigestString as Digest,
    Md5Checksum,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
    SkippableChecksum,
};

mod source;
pub use source::Source;

pub mod url;
pub use url::{SourceUrl, Url};

/// Public re-exports of common hash functions, for use with [`Checksum`].
pub mod digests {
    pub use blake2::Blake2b512;
    pub use md5::Md5;
    pub use sha1::Sha1;
    pub use sha2::{Sha224, Sha256, Sha384, Sha512};

    pub use crate::checksum::{Crc32Cksum, DigestEncoding, DigestString as Digest};
}

mod compression;
pub use compression::CompressionAlgorithmFileExtension;

mod date;
pub use date::{BuildDate, FromOffsetDateTime};

mod env;
pub use env::{BuildEnvironmentOption, InstalledPackage, MakepkgOption, PackageOption};

mod file_type;
pub use file_type::FileTypeIdentifier;

mod error;
pub use error::Error;

mod license;
pub use license::License;

mod name;
pub use name::{BuildTool, Name, SharedObjectName};

mod package;
pub use package::{
    contents::{INSTALL_SCRIPTLET_FILE_NAME, MetadataFileName},
    error::Error as PackageError,
    file_name::PackageFileName,
    installation::PackageInstallReason,
    source::{PKGBUILD_FILE_NAME, SRCINFO_FILE_NAME},
    validation::PackageValidation,
};

mod path;
pub use path::{
    AbsolutePath,
    Backup,
    BuildDirectory,
    Changelog,
    Install,
    RelativeFilePath,
    RelativePath,
    SonameLookupDirectory,
    StartDirectory,
};

mod openpgp;
pub use openpgp::{
    Base64OpenPGPSignature,
    OpenPGPIdentifier,
    OpenPGPKeyId,
    OpenPGPv4Fingerprint,
    Packager,
};

mod pkg;
pub use pkg::{ExtraData, ExtraDataEntry, PackageBaseName, PackageDescription, PackageType};

mod relation;
pub use relation::{
    Group,
    OptionalDependency,
    PackageRelation,
    RelationOrSoname,
    SharedLibraryPrefix,
    Soname,
    SonameV1,
    SonameV2,
    VersionOrSoname,
};

mod size;
pub use size::{CompressedSize, InstalledSize};

mod system;
pub use system::{
    Architecture,
    Architectures,
    ElfArchitectureFormat,
    SystemArchitecture,
    UnknownArchitecture,
};

mod version;
pub use version::{
    base::{Epoch, PackageRelease, PackageVersion},
    buildtool::BuildToolVersion,
    comparison::{VersionSegment, VersionSegments},
    pkg_full::FullVersion,
    pkg_generic::Version,
    pkg_minimal::MinimalVersion,
    requirement::{VersionComparison, VersionRequirement},
    schema::SchemaVersion,
};

/// Public re-exports for use with [`SchemaVersion`].
pub mod semver_version {
    pub use semver::Version;
}

fluent_i18n::i18n!("locales");

/// This is a helper macro that is used by unit tests in the `alpm-types` crate.
///
/// Specifically, it takes care of two things:
///
/// 1. Make the test filename **somewhat** human readable. cargo-insta uses the full module name by
///    default, which is absurdly long due to our usage of rstest. Since the snapshots are placed in
///    the immediate module anyway, we only need one level of module indirection. The tests
///    filenames have the format of: `{module}::{test_function}@{test_case_name}`
/// 2. Remove the `expression` field from the snapshot, as it's of no use in parser tests.
///
/// The function returns the test name to use in `assert_snapshot`, as well as a settings guard,
/// which assures that the settings we just adjusted are local to this thread and stay up until the
/// guard goes out of scope.
///
/// # Example
///
/// ```rs,norun
/// #[rstest]
/// #[case::something_bad("oh no")]
/// #[case::something_else_bad("oh nooo")]
/// fn invalid_version_requirement(#[case] requirement: &str) {
///     let Err(Error::ParseError(err_msg)) = VersionRequirement::from_str(requirement) else {
///         panic!("'{requirement}' erroneously parsed as VersionRequirement")
///     };
///
///     let (test_name, _guard) = configure_insta();
///     assert_snapshot!(test_name, err_msg.to_string());
/// }
#[cfg(test)]
// We ignore `expect_fun_call`, as this is test code and more this makes it significantly
// more convenient/easier to read.
#[allow(clippy::expect_fun_call)]
fn configure_insta() -> (String, insta::internals::SettingsBindDropGuard) {
    // First up, disable colored output for our snapshot errors.
    colored::control::set_override(false);

    // Get the full thread name, which is pretty much a rust module string
    // e.g. `version::base::tests::invalid_pkgver::case_4`
    let thread_name = std::thread::current()
        .name()
        .expect("Couldn't determine test thread name!!")
        .to_string();

    let (mut rest, mut end) = thread_name.rsplit_once("::").expect(&format!(
        "Test thread name does not have an expected first level: {thread_name}"
    ));

    // If we're inside an rstest test case, the last section of the test will be something along the
    // line of `case_4` followed by an optional test case name.
    //
    // Otherwise, it's the name of the test function.
    let mut test_case_name: Option<String> = None;
    let function_name = if end.contains("case") {
        test_case_name = Some(end.to_string());

        // The next part will then be the actual test name
        (rest, end) = rest.rsplit_once("::").expect(&format!(
            "Test thread name does not have an expected second level: {thread_name}"
        ));

        end.to_string()
    } else {
        end.to_string()
    };

    // Now get the module name which contains the test function.
    let module_name = loop {
        (rest, end) = match rest.rsplit_once("::") {
            Some(split) => split,
            // Handle the case that we reached the topmost module.
            None => break rest.to_string(),
        };

        // Ignore any *test* modules, as those are not interesting for us.
        if !end.contains("test") {
            break end.to_string();
        };
    };

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    // If we're inside a testcase, set the case name as a suffix.
    if let Some(test_case_name) = test_case_name {
        settings.set_snapshot_suffix(test_case_name.to_string());
    }

    // Since we're test parsers, the expression is always generic and only clutters the output.
    // The expression that's needed for context is always fully visible in the actual error message.
    settings.set_omit_expression(true);
    let guard = settings.bind_to_scope();

    // Return the cargo insta test name for usage in the `assert_snapshot` function.
    (format!("{module_name}::{function_name}"), guard)
}
