//! Write implementation for [`SourceInfo`].

use alpm_types::{Architecture, Architectures};

use super::{
    package::{Override, Package, PackageArchitecture},
    package_base::{PackageBase, PackageBaseArchitecture},
};
#[cfg(doc)]
use crate::SourceInfo;

/// Pushes a section header to a [`String`].
///
/// Section headers are either `pkgname` or `pkgbase` and are **not** indented.
fn push_section(section: &str, value: &str, output: &mut String) {
    output.push_str(section);
    output.push_str(" = ");
    output.push_str(value);
    output.push('\n');
}

/// Pushes a key-value pair to a [`String`].
///
/// Key-value pairs are scoped to a section.
/// To make this visually distinguishable, the key-value pair is indented by a tab.
fn push_key_value(key: &str, value: &str, output: &mut String) {
    output.push('\t');
    output.push_str(key);
    output.push_str(" = ");
    output.push_str(value);
    output.push('\n');
}

/// Pushes a key-value pair to a [`String`], if it is set.
///
/// Key-value pairs are scoped to a section.
/// To make this visually distinguishable, the key-value pair is indented by a tab.
fn push_optional_value<T: ToString>(key: &str, value: &Option<T>, output: &mut String) {
    let Some(value) = value else {
        return;
    };

    push_key_value(key, &value.to_string(), output);
}

/// Pushes a list of key-value pairs in [SRCINFO] format to a [`String`].
///
/// Each value in `values` is added as a new line.
/// If `values` is empty, nothing is added.
///
/// The Key-value pairs are fields scoped to a section.
/// To make this visually distinguishable, each key-value pair is indented by a tab.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
fn push_value_list<T: ToString>(key: &str, values: &Vec<T>, output: &mut String) {
    for value in values {
        push_key_value(key, &value.to_string(), output);
    }
}

/// Appends a [`PackageBase`] in [SRCINFO] format to a [`String`].
///
/// The items in the `pkgbase` section are written to `output` in an order compatible with
/// [makepkg].
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
pub(crate) fn pkgbase_section(base: &PackageBase, output: &mut String) {
    push_section("pkgbase", base.name.inner(), output);

    if let Some(description) = &base.description {
        push_key_value("pkgdesc", description.as_ref(), output);
    }
    push_key_value("pkgver", &base.version.pkgver.to_string(), output);
    push_key_value("pkgrel", &base.version.pkgrel.to_string(), output);
    push_optional_value("epoch", &base.version.epoch, output);
    push_optional_value("url", &base.url, output);
    push_optional_value("install", &base.install, output);
    push_optional_value("changelog", &base.changelog, output);
    let architectures: Vec<Architecture> = (&base.architectures).into();
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
    push_value_list("sha224sums", &base.sha224_checksums, output);
    push_value_list("sha256sums", &base.sha256_checksums, output);
    push_value_list("sha384sums", &base.sha384_checksums, output);
    push_value_list("sha512sums", &base.sha512_checksums, output);
    push_value_list("b2sums", &base.b2_checksums, output);
    push_value_list("cksums", &base.crc_checksums, output);

    // Go through architecture specific values **in the same order** as in `pkgbase.arch`.
    // That's how `makepkg` does it.
    for architecture in &base.architectures {
        if let Architecture::Some(system_arch) = &architecture
            && let Some(properties) = base.architecture_properties.get(system_arch)
        {
            pkgbase_architecture_properties(architecture, properties, output);
        }
    }
}

/// Appends a [`PackageBaseArchitecture`] based on an [`Architecture`] in [SRCINFO] format to a
/// [`String`].
///
/// The architecture-specific `properties` are written to `output` in an order compatible with
/// [makepkg].
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
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
    push_value_list(
        &format!("cksums_{architecture}"),
        &properties.crc_checksums,
        output,
    )
}

/// Pushes an override key-value pair in [SRCINFO] format to a [`String`].
///
/// Key-value pairs are scoped to a section.
/// To make this visually distinguishable, the key-value pair is indented by a tab.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
fn push_override_value<T: ToString>(key: &str, value: &Override<T>, output: &mut String) {
    match value {
        Override::No => (),
        Override::Clear => {
            // Clear the value
            output.push('\t');
            output.push_str(key);
            output.push_str(" = \n");
        }
        Override::Yes { value } => {
            push_key_value(key, &value.to_string(), output);
        }
    }
}

/// Pushes a list of override key-value pairs in [SRCINFO] format to a [`String`].
///
/// Each value in `values` is added as a new line.
/// If `values` is empty, nothing is added.
///
/// The Key-value pairs are fields scoped to a section.
/// To make this visually distinguishable, each key-value pair is indented by a tab.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
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
            output.push_str(" = \n");
        }
        Override::Yes { value } => {
            for inner_value in value {
                push_key_value(key, &inner_value.to_string(), output);
            }
        }
    }
}

/// Appends a [`Package`] with an [`Architecture`] in [SRCINFO] format to a [`String`].
///
/// The items in the `pkgname` section are written to `output` in an order compatible with
/// [makepkg].
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
pub(crate) fn pkgname_section(
    package: &Package,
    base_architectures: &Architectures,
    output: &mut String,
) {
    push_section("pkgname", package.name.inner(), output);

    push_override_value("pkgdesc", &package.description, output);
    push_override_value("url", &package.url, output);
    push_override_value("install", &package.install, output);
    push_override_value("changelog", &package.changelog, output);

    if let Some(architectures) = &package.architectures {
        let arch_vec: Vec<Architecture> = architectures.into();
        push_value_list("arch", &arch_vec, output);
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

    // Go through architecture specific values **in the same order** as in `pkgbase.arch`.
    for architecture in base_architectures {
        if let Architecture::Some(system_arch) = &architecture
            && let Some(properties) = package.architecture_properties.get(system_arch)
        {
            pkgname_architecture_properties(architecture, properties, output);
        }
    }
}

/// Appends a [`PackageArchitecture`] based on an [`Architecture`] in [SRCINFO] format to a
/// [`String`].
///
/// The architecture-specific `properties` are written to `output` in an order compatible with
/// [makepkg].
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
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
