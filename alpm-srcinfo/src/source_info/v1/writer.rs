use alpm_types::Architecture;

use super::{
    SourceInfoV1,
    package::{Override, Package, PackageArchitecture},
    package_base::{PackageBase, PackageBaseArchitecture},
};

/// Take a [SourceInfoV1] and return the [alpm-srcinfo] representation, aka `.SRCINFO` file.
///
/// ```
/// use std::{env::var, path::PathBuf};
///
/// use alpm_srcinfo::{SourceInfoV1, source_info::v1::writer::source_info_as_srcinfo};
///
/// const TEST_FILE: &str = include_str!("../../../tests/unit_test_files/SRCINFO");
///
/// # use testresult::TestResult;
/// # fn main() -> TestResult {
/// // Read a .SRCINFO file and bring it into the `SourceInfoV1` representation.
/// let source_info = SourceInfoV1::from_string(TEST_FILE)?.source_info()?;
/// // Convert the `SourceInfoV1` back into it's alpm `.SRCINFO` format.
/// let output = source_info_as_srcinfo(source_info);
///
/// println!("{output}");
/// # Ok(())
/// # }
/// ```
///
/// [alpm-srcinfo]: https://alpm.archlinux.page/specifications/alpm-srcinfo.7.html
pub fn source_info_as_srcinfo(source_info: &SourceInfoV1) -> String {
    let mut srcinfo = String::new();

    pkgbase_section(&source_info.base, &mut srcinfo);
    for package in &source_info.packages {
        srcinfo.push('\n');
        pkgname_section(package, &mut srcinfo);
    }

    // Pop the trailing newline as makepkg doesn't have that one.
    if srcinfo.ends_with('\n') {
        srcinfo.pop();
    }

    srcinfo
}

/// Adds a new section header.
/// Section headers are either `pkgname` or `pkgbase` and are **not** indented.
fn push_section(section: &str, value: &str, output: &mut String) {
    output.push_str(section);
    output.push_str(" = ");
    output.push_str(value);
    output.push('\n');
}

/// Adds a new value.
///
/// Values are fields scoped to a section.
/// To make this easier to visually distinquish, the values are indented by 4 spaces.
fn push_value(key: &str, value: &str, output: &mut String) {
    output.push('\t');
    output.push_str(key);
    output.push_str(" = ");
    output.push_str(value);
    output.push('\n');
}

/// Adds an optional value, **if** it is set.
///
/// Values are fields scoped to a section.
/// To make this easier to visually distinquish, the values are indented by 4 spaces.
fn push_optional_value<T: ToString>(key: &str, value: &Option<T>, output: &mut String) {
    let Some(value) = value else {
        return;
    };

    push_value(key, &value.to_string(), output);
}

/// Adds a list of values, where each value is added as a new line.
/// If the list is empty, nothing is added.
///
/// Values are fields scoped to a section.
/// To make this easier to visually distinquish, the values are indented by 4 spaces.
fn push_value_list<T: ToString>(key: &str, values: &Vec<T>, output: &mut String) {
    for value in values {
        push_value(key, &value.to_string(), output);
    }
}

/// Appends the `pkgbase` section of a `.SRCINFO` file to `output`.
///
/// The order in here has been specifically choosen to mirror that used in `makepkg`, which differs
/// one depending on the type of section.
fn pkgbase_section(base: &PackageBase, output: &mut String) {
    push_section("pkgbase", base.name.inner(), output);

    if let Some(description) = &base.description {
        push_value("pkgdesc", &description, output);
    }
    push_value("pkgver", &base.package_version.to_string(), output);
    push_value("pkgrel", &base.package_release.to_string(), output);
    push_optional_value("epoch", &base.epoch, output);
    push_optional_value("url", &base.url, output);
    push_optional_value("install", &base.install, output);
    push_optional_value("changelog", &base.changelog, output);

    // Return the architectures in sorted order.
    // We loose some information about the order as architectures are stored as a Hashset and we
    // sort it to be idempotent.
    let mut architectures: Vec<Architecture> = base.architectures.clone().into_iter().collect();
    architectures.sort_unstable();
    push_value_list("arch", &architectures, output);

    push_value_list("groups", &base.groups, output);
    push_value_list("license", &base.licenses, output);
    push_value_list("checkdepends", &base.check_dependencies, output);
    push_value_list("makedepends", &base.make_dependencies, output);
    push_value_list("depends", &base.dependencies, output);
    push_value_list("optdepends", &base.optional_dependencies, output);
    push_value_list("provides", &base.provides, output);
    push_value_list("conflicts", &base.conflicts, output);
    push_value_list("replaces", &base.replaces, output);
    push_value_list("noextract", &base.no_extracts, output);
    push_value_list("options", &base.options, output);
    push_value_list("backup", &base.backups, output);
    push_value_list("source", &base.sources, output);
    push_value_list("validpgpkeys", &base.pgp_fingerprints, output);
    push_value_list("md5sums", &base.md5_checksums, output);
    push_value_list("sha1sums", &base.sha1_checksums, output);
    push_value_list("sha2sums24sums", &base.sha224_checksums, output);
    push_value_list("sha2sums56sums", &base.sha256_checksums, output);
    push_value_list("sha3sums84sums", &base.sha384_checksums, output);
    push_value_list("sha512sums", &base.sha512_checksums, output);
    push_value_list("b2sums", &base.b2_checksums, output);

    for (architecture, properties) in &base.architecture_properties {
        pkgbase_architecture_properties(*architecture, properties, output);
    }
}

/// Appends the `pkgbase` section of a `.SRCINFO` file to `output`.
///
/// The order in here has been specifically choosen to mirror that used in `makepkg`, which differs
/// one depending on the type of section.
fn pkgbase_architecture_properties(
    architecture: Architecture,
    properties: &PackageBaseArchitecture,
    output: &mut String,
) {
    push_value_list(
        &format!("source_{architecture}"),
        &properties.sources,
        output,
    );
    push_value_list(
        &format!("provides_{architecture}"),
        &properties.provides,
        output,
    );
    push_value_list(
        &format!("conflicts_{architecture}"),
        &properties.conflicts,
        output,
    );
    push_value_list(
        &format!("depends_{architecture}"),
        &properties.dependencies,
        output,
    );
    push_value_list(
        &format!("replaces_{architecture}"),
        &properties.replaces,
        output,
    );
    push_value_list(
        &format!("optdepends_{architecture}"),
        &properties.optional_dependencies,
        output,
    );
    push_value_list(
        &format!("makedepends_{architecture}"),
        &properties.make_dependencies,
        output,
    );
    push_value_list(
        &format!("checkdepends_{architecture}"),
        &properties.check_dependencies,
        output,
    );
    // `noextract` is currently not written by `makepkg`, which really doesn't make any sense
    // though.
    push_value_list(
        &format!("noextract_{architecture}"),
        &properties.no_extracts,
        output,
    );
    push_value_list(
        &format!("md5sums_{architecture}"),
        &properties.md5_checksums,
        output,
    );
    push_value_list(
        &format!("sha1sums_{architecture}"),
        &properties.sha1_checksums,
        output,
    );
    push_value_list(
        &format!("sha224sums_{architecture}"),
        &properties.sha224_checksums,
        output,
    );
    push_value_list(
        &format!("sha256sums_{architecture}"),
        &properties.sha256_checksums,
        output,
    );
    push_value_list(
        &format!("sha384sums_{architecture}"),
        &properties.sha384_checksums,
        output,
    );
    push_value_list(
        &format!("sha512sums_{architecture}"),
        &properties.sha512_checksums,
        output,
    );
    push_value_list(
        &format!("b2sums_{architecture}"),
        &properties.b2_checksums,
        output,
    );
}

/// Adds a new value.
///
/// Values are fields scoped to a section.
/// To make this easier to visually distinquish, the values are indented by 4 spaces.
fn push_override_value<T: ToString>(key: &str, value: &Override<T>, output: &mut String) {
    match value {
        Override::No => (),
        Override::Clear => {
            // Clear the value
            output.push('\t');
            output.push_str(key);
            output.push_str(" =");
        }
        Override::Yes { value } => {
            push_value(key, &value.to_string(), output);
        }
    }
}

/// Adds a list of values, where each value is added as a new line.
/// If the list is empty, nothing is added.
///
/// Values are fields scoped to a section.
/// To make this easier to visually distinquish, the values are indented by 4 spaces.
fn push_override_value_list<T: ToString>(
    key: &str,
    values: &Override<Vec<T>>,
    output: &mut String,
) {
    match values {
        Override::No => (),
        Override::Clear => {
            // Clear the value
            output.push('\t');
            output.push_str(key);
            output.push_str(" =");
        }
        Override::Yes { value } => {
            for inner_value in value {
                push_value(key, &inner_value.to_string(), output);
            }
        }
    }
}

/// Appends a `pkgname` section of a `.SRCINFO` file to `output`.
///
/// The order in here has been specifically choosen to mirror that used in `makepkg`, which differs
/// one depending on the type of section.
fn pkgname_section(package: &Package, output: &mut String) {
    push_section("pkgname", package.name.inner(), output);

    push_override_value("pkgdesc", &package.description, output);
    push_override_value("url", &package.url, output);
    push_override_value("install", &package.install, output);
    push_override_value("changelog", &package.changelog, output);

    // Return the architectures in sorted order.
    // We loose some information about the order as architectures are stored as a Hashset and we
    // sort it to be idempotent.
    if let Some(architectures) = &package.architectures {
        let mut architectures: Vec<Architecture> = architectures.clone().into_iter().collect();
        architectures.sort_unstable();
        push_value_list("arch", &architectures, output);
    }

    push_override_value_list("groups", &package.groups, output);
    push_override_value_list("license", &package.licenses, output);
    push_override_value_list("depends", &package.dependencies, output);
    push_override_value_list("optdepends", &package.optional_dependencies, output);
    push_override_value_list("provides", &package.provides, output);
    push_override_value_list("conflicts", &package.conflicts, output);
    push_override_value_list("replaces", &package.replaces, output);
    push_override_value_list("options", &package.options, output);
    push_override_value_list("backup", &package.backups, output);

    for (architecture, properties) in &package.architecture_properties {
        pkgname_architecture_properties(*architecture, properties, output);
    }
}

/// Appends the `pkgbase` section of a `.SRCINFO` file to `output`.
///
/// The order in here has been specifically choosen to mirror that used in `makepkg`, which differs
/// one depending on the type of section.
fn pkgname_architecture_properties(
    architecture: Architecture,
    properties: &PackageArchitecture,
    output: &mut String,
) {
    push_override_value_list(
        &format!("provides_{architecture}"),
        &properties.provides,
        output,
    );
    push_override_value_list(
        &format!("conflicts_{architecture}"),
        &properties.conflicts,
        output,
    );
    push_override_value_list(
        &format!("depends_{architecture}"),
        &properties.dependencies,
        output,
    );
    push_override_value_list(
        &format!("replaces_{architecture}"),
        &properties.replaces,
        output,
    );
    push_override_value_list(
        &format!("optdepends_{architecture}"),
        &properties.optional_dependencies,
        output,
    );
}
