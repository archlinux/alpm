//! Handling of metadata found in the `pkgbase` section of SRCINFO data.
use std::collections::BTreeMap;

use alpm_types::{
    Architecture,
    Epoch,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelationOrSoname,
    RelativePath,
    SkippableChecksum,
    Source,
    Url,
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};
use serde::{Deserialize, Serialize};

use super::package::PackageArchitecture;
#[cfg(doc)]
use crate::{MergedPackage, SourceInfoV1, source_info::v1::package::Package};
use crate::{
    error::{SourceInfoError, lint, unrecoverable},
    lints::{
        duplicate_architecture,
        missing_architecture_for_property,
        non_spdx_license,
        unsafe_checksum,
    },
    source_info::parser::{self, PackageBaseProperty, RawPackageBase, SharedMetaProperty},
};

/// Package base metadata based on the `pkgbase` section in SRCINFO data.
///
/// All values in this struct act as default values for all [`Package`]s in the scope of specific
/// SRCINFO data.
///
/// A [`MergedPackage`] (a full view on a package's metadata) can be created using
/// [`SourceInfoV1::packages_for_architecture`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PackageBase {
    /// The alpm-package-name of the package base.
    pub name: Name,
    /// The optional description of the package base.
    pub description: Option<PackageDescription>,
    /// The optional upstream URL of the package base.
    pub url: Option<Url>,
    /// The optional relative path to a changelog file of the package base.
    pub changelog: Option<RelativePath>,
    /// The list of licenses that apply to the package base.
    pub licenses: Vec<License>,

    // Build or package management related meta fields
    /// The optional relative path to an alpm-install-scriptlet of the package base.
    pub install: Option<RelativePath>,
    /// The optional list of alpm-package-groups the package base is part of.
    pub groups: Vec<String>,
    /// The list of build tool options used when building.
    pub options: Vec<MakepkgOption>,
    /// The list of relative paths to files in a package that should be backed up.
    pub backups: Vec<RelativePath>,

    // These metadata fields are PackageBase specific
    /// The version of the package
    pub package_version: PackageVersion,
    /// The release of the package
    pub package_release: PackageRelease,
    /// The epoch of the package
    pub epoch: Option<Epoch>,

    /// The list of OpenPGP fingerprints of OpenPGP certificates used for the verification of
    /// upstream sources.
    pub pgp_fingerprints: Vec<OpenPGPIdentifier>,

    /// Architectures and architecture specific properties
    pub architectures: Vec<Architecture>,
    /// The map of alpm-architecture specific overrides for package relations of a package base.
    pub architecture_properties: BTreeMap<Architecture, PackageBaseArchitecture>,

    /// The list of run-time dependencies of the package base.
    pub dependencies: Vec<RelationOrSoname>,
    /// The list of optional dependencies of the package base.
    pub optional_dependencies: Vec<OptionalDependency>,
    /// The list of provisions of the package base.
    pub provides: Vec<RelationOrSoname>,
    /// The list of conflicts of the package base.
    pub conflicts: Vec<PackageRelation>,
    /// The list of replacements of the package base.
    pub replaces: Vec<PackageRelation>,
    // The following dependencies are build-time specific dependencies.
    // `makepkg` expects all dependencies for all split packages to be specified in the
    // PackageBase.
    /// The list of test dependencies of the package base.
    pub check_dependencies: Vec<PackageRelation>,
    /// The list of build dependencies of the package base.
    pub make_dependencies: Vec<PackageRelation>,

    /// The list of sources of the package base.
    pub sources: Vec<Source>,
    /// The list of sources of the package base that are not extracted.
    pub no_extracts: Vec<String>,
    /// The list of Blake2 hash digests for `sources` of the package base.
    pub b2_checksums: Vec<SkippableChecksum<Blake2b512>>,
    /// The list of MD-5 hash digests for `sources` of the package base.
    pub md5_checksums: Vec<SkippableChecksum<Md5>>,
    /// The list of SHA-1 hash digests for `sources` of the package base.
    pub sha1_checksums: Vec<SkippableChecksum<Sha1>>,
    /// The list of SHA-224 hash digests for `sources` of the package base.
    pub sha224_checksums: Vec<SkippableChecksum<Sha224>>,
    /// The list of SHA-256 hash digests for `sources` of the package base.
    pub sha256_checksums: Vec<SkippableChecksum<Sha256>>,
    /// The list of SHA-384 hash digests for `sources` of the package base.
    pub sha384_checksums: Vec<SkippableChecksum<Sha384>>,
    /// The list of SHA-512 hash digests for `sources` of the package base.
    pub sha512_checksums: Vec<SkippableChecksum<Sha512>>,
}

/// Architecture specific package base properties for use in [`PackageBase`].
///
/// For each [`Architecture`] defined in [`PackageBase::architectures`] a
/// [`PackageBaseArchitecture`] is present in [`PackageBase::architecture_properties`].
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageBaseArchitecture {
    /// The list of run-time dependencies of the package base.
    pub dependencies: Vec<RelationOrSoname>,
    /// The list of optional dependencies of the package base.
    pub optional_dependencies: Vec<OptionalDependency>,
    /// The list of provisions of the package base.
    pub provides: Vec<RelationOrSoname>,
    /// The list of conflicts of the package base.
    pub conflicts: Vec<PackageRelation>,
    /// The list of replacements of the package base.
    pub replaces: Vec<PackageRelation>,
    // The following dependencies are build-time specific dependencies.
    // `makepkg` expects all dependencies for all split packages to be specified in the
    // PackageBase.
    /// The list of test dependencies of the package base.
    pub check_dependencies: Vec<PackageRelation>,
    /// The list of build dependencies of the package base.
    pub make_dependencies: Vec<PackageRelation>,

    /// The list of sources of the package base.
    pub sources: Vec<Source>,
    /// The list of sources of the package base that are not extracted.
    pub no_extracts: Vec<String>,
    /// The list of Blake2 hash digests for `sources` of the package base.
    pub b2_checksums: Vec<SkippableChecksum<Blake2b512>>,
    /// The list of MD-5 hash digests for `sources` of the package base.
    pub md5_checksums: Vec<SkippableChecksum<Md5>>,
    /// The list of SHA-1 hash digests for `sources` of the package base.
    pub sha1_checksums: Vec<SkippableChecksum<Sha1>>,
    /// The list of SHA-224 hash digests for `sources` of the package base.
    pub sha224_checksums: Vec<SkippableChecksum<Sha224>>,
    /// The list of SHA-256 hash digests for `sources` of the package base.
    pub sha256_checksums: Vec<SkippableChecksum<Sha256>>,
    /// The list of SHA-384 hash digests for `sources` of the package base.
    pub sha384_checksums: Vec<SkippableChecksum<Sha384>>,
    /// The list of SHA-512 hash digests for `sources` of the package base.
    pub sha512_checksums: Vec<SkippableChecksum<Sha512>>,
}

impl PackageBaseArchitecture {
    /// Merges in the architecture specific properties of a package.
    ///
    /// Each existing field of `properties` overrides the architecture-independent pendant on
    /// `self`.
    pub fn merge_package_properties(&mut self, properties: PackageArchitecture) {
        properties.dependencies.merge_vec(&mut self.dependencies);
        properties
            .optional_dependencies
            .merge_vec(&mut self.optional_dependencies);
        properties.provides.merge_vec(&mut self.provides);
        properties.conflicts.merge_vec(&mut self.conflicts);
        properties.replaces.merge_vec(&mut self.replaces);
    }
}

/// Handles all potentially architecture specific Vector entries in the [`PackageBase::from_parsed`]
/// function.
///
/// If no architecture is encountered, it simply adds the value on the [`PackageBase`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [`PackageBase::architectures`].
macro_rules! package_base_arch_prop {
    (
        $line:ident,
        $errors:ident,
        $architectures:ident,
        $architecture_properties:ident,
        $arch_property:ident,
        $field_name:ident,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparation
        if let Some(architecture) = $arch_property.architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $architecture_properties
                .entry(architecture)
                .or_insert(PackageBaseArchitecture::default());

            // Set the architecture specific value.
            architecture_properties
                .$field_name
                .push($arch_property.value);

            // Throw an error for all architecture specific properties that don't have
            // an explicit `arch` statement. This is considered bad style.
            // Also handle the special `Any` [Architecture], which allows all architectures.
            if !$architectures.contains(&architecture)
                && !$architectures.contains(&Architecture::Any)
            {
                missing_architecture_for_property($errors, $line, architecture);
            }
        } else {
            $field_name.push($arch_property.value)
        }
    };
}

impl PackageBase {
    /// Creates a new [`PackageBase`] instance from a [`RawPackageBase`].
    ///
    /// # Parameters
    ///
    /// - `line_start`: The number of preceding lines, so that error/lint messages can reference the
    ///   correct lines.
    /// - `parsed`: The [`RawPackageBase`] representation of the SRCINFO data. The input guarantees
    ///   that the keyword definitions have been parsed correctly, but not yet that they represent
    ///   valid SRCINFO data as a whole.
    /// - `errors`: All errors and lints encountered during the creation of the [`PackageBase`].
    ///
    /// # Errors
    ///
    /// This function does not return a [`Result`], but instead relies on aggregating all lints,
    /// warnings and errors in `errors`. This allows to keep the function call recoverable, so
    /// that all errors and lints can be returned all at once.
    pub fn from_parsed(
        line_start: usize,
        parsed: RawPackageBase,
        errors: &mut Vec<SourceInfoError>,
    ) -> Self {
        let mut description = None;
        let mut url = None;
        let mut licenses = Vec::new();
        let mut changelog = None;
        let mut architectures = Vec::new();
        let mut architecture_properties = BTreeMap::new();

        // Build or package management related meta fields
        let mut install = None;
        let mut groups = Vec::new();
        let mut options = Vec::new();
        let mut backups = Vec::new();

        // These metadata fields are PackageBase specific
        let mut epoch: Option<Epoch> = None;
        let mut package_version: Option<PackageVersion> = None;
        let mut package_release: Option<PackageRelease> = None;
        let mut pgp_fingerprints = Vec::new();

        let mut dependencies = Vec::new();
        let mut optional_dependencies = Vec::new();
        let mut provides = Vec::new();
        let mut conflicts = Vec::new();
        let mut replaces = Vec::new();
        // The following dependencies are build-time specific dependencies.
        // `makepkg` expects all dependencies for all split packages to be specified in the
        // PackageBase.
        let mut check_dependencies = Vec::new();
        let mut make_dependencies = Vec::new();

        let mut sources = Vec::new();
        let mut no_extracts = Vec::new();
        let mut b2_checksums = Vec::new();
        let mut md5_checksums = Vec::new();
        let mut sha1_checksums = Vec::new();
        let mut sha224_checksums = Vec::new();
        let mut sha256_checksums = Vec::new();
        let mut sha384_checksums = Vec::new();
        let mut sha512_checksums = Vec::new();

        // First up check all input for potential architecture declarations.
        // We need this to do proper linting when doing our actual pass through the file.
        for (index, prop) in parsed.properties.iter().enumerate() {
            // We're only interested in architecture properties.
            let PackageBaseProperty::MetaProperty(SharedMetaProperty::Architecture(architecture)) =
                prop
            else {
                continue;
            };

            // Calculate the actual line in the document based on any preceding lines.
            // We have to add one, as lines aren't 0 indexed.
            let line = index + line_start;

            // Lint to make sure there aren't duplicate architectures declarations.
            if architectures.contains(architecture) {
                duplicate_architecture(errors, line, *architecture);
            }

            // Add the architecture in case it hasn't already.
            architectures.push(*architecture);
        }

        // If no architecture is set, `makepkg` simply uses the host system as the default value.
        // In practice this translates to `any`, as this package is valid to be build on any
        // system as long as `makepkg` is executed on that system.
        if architectures.is_empty() {
            errors.push(lint(
                None,
                "No architecture has been specified. Assuming `any`.",
            ));
            architectures.push(Architecture::Any);
        }

        for (index, prop) in parsed.properties.into_iter().enumerate() {
            // Calculate the actual line in the document based on any preceding lines.
            let line = index + line_start;
            match prop {
                // Skip empty lines and comments
                PackageBaseProperty::EmptyLine | PackageBaseProperty::Comment(_) => continue,
                PackageBaseProperty::PackageVersion(inner) => package_version = Some(inner),
                PackageBaseProperty::PackageRelease(inner) => package_release = Some(inner),
                PackageBaseProperty::PackageEpoch(inner) => epoch = Some(inner),
                PackageBaseProperty::ValidPgpKeys(inner) => {
                    if let OpenPGPIdentifier::OpenPGPKeyId(_) = &inner {
                        errors.push(lint(
                            Some(line),
                            concat!(
                                "OpenPGP Key IDs are highly discouraged, as the length doesn't guarantee uniqueness.",
                                "\nUse an OpenPGP v4 fingerprint instead.",
                            )
                        ));
                    }
                    pgp_fingerprints.push(inner);
                }
                PackageBaseProperty::CheckDependency(arch_property) => {
                    package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        check_dependencies,
                    )
                }
                PackageBaseProperty::MakeDependency(arch_property) => {
                    package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        make_dependencies,
                    )
                }
                PackageBaseProperty::MetaProperty(shared_meta_property) => {
                    match shared_meta_property {
                        SharedMetaProperty::Description(inner) => description = Some(inner),
                        SharedMetaProperty::Url(inner) => url = Some(inner),
                        SharedMetaProperty::License(inner) => {
                            // Create lints for non-spdx licenses.
                            if let License::Unknown(_) = &inner {
                                non_spdx_license(errors, line, inner.to_string());
                            }

                            licenses.push(inner)
                        }
                        // We already handled those above.
                        SharedMetaProperty::Architecture(_) => continue,
                        SharedMetaProperty::Changelog(inner) => changelog = Some(inner),
                        SharedMetaProperty::Install(inner) => install = Some(inner),
                        SharedMetaProperty::Group(inner) => groups.push(inner),
                        SharedMetaProperty::Option(inner) => options.push(inner),
                        SharedMetaProperty::Backup(inner) => backups.push(inner),
                    }
                }
                PackageBaseProperty::RelationProperty(relation_property) => match relation_property
                {
                    parser::RelationProperty::Dependency(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        dependencies,
                    ),
                    parser::RelationProperty::OptionalDependency(arch_property) => {
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            optional_dependencies,
                        )
                    }
                    parser::RelationProperty::Provides(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        provides,
                    ),
                    parser::RelationProperty::Conflicts(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        conflicts,
                    ),
                    parser::RelationProperty::Replaces(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        replaces,
                    ),
                },
                PackageBaseProperty::SourceProperty(source_property) => match source_property {
                    parser::SourceProperty::Source(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        sources,
                    ),
                    parser::SourceProperty::NoExtract(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        no_extracts,
                    ),
                    parser::SourceProperty::B2Checksum(arch_property) => package_base_arch_prop!(
                        line,
                        errors,
                        architectures,
                        architecture_properties,
                        arch_property,
                        b2_checksums,
                    ),
                    parser::SourceProperty::Md5Checksum(arch_property) => {
                        unsafe_checksum(errors, line, "md5");
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            md5_checksums,
                        );
                    }
                    parser::SourceProperty::Sha1Checksum(arch_property) => {
                        unsafe_checksum(errors, line, "sha1");
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            sha1_checksums,
                        );
                    }
                    parser::SourceProperty::Sha224Checksum(arch_property) => {
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            sha224_checksums,
                        )
                    }
                    parser::SourceProperty::Sha256Checksum(arch_property) => {
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            sha256_checksums,
                        )
                    }
                    parser::SourceProperty::Sha384Checksum(arch_property) => {
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            sha384_checksums,
                        )
                    }
                    parser::SourceProperty::Sha512Checksum(arch_property) => {
                        package_base_arch_prop!(
                            line,
                            errors,
                            architectures,
                            architecture_properties,
                            arch_property,
                            sha512_checksums,
                        )
                    }
                },
            }
        }

        // Handle a missing package_version
        let package_version = match package_version {
            Some(package_version) => package_version,
            None => {
                errors.push(unrecoverable(
                    None,
                    "pkgbase section doesn't contain a 'pkgver' keyword assignment",
                ));
                // Set a package version nevertheless, so we continue parsing the rest of the file.
                PackageVersion::new("0".to_string()).unwrap()
            }
        };

        // Handle a missing package_version
        let package_release = match package_release {
            Some(package_release) => package_release,
            None => {
                errors.push(unrecoverable(
                    None,
                    "pkgbase section doesn't contain a 'pkgrel' keyword assignment",
                ));
                // Set a package version nevertheless, so we continue parsing the rest of the file.
                PackageRelease::new(1, None)
            }
        };

        PackageBase {
            name: parsed.name,
            description,
            url,
            licenses,
            changelog,
            architectures,
            architecture_properties,
            install,
            groups,
            options,
            backups,
            package_version,
            package_release,
            epoch,
            pgp_fingerprints,
            dependencies,
            optional_dependencies,
            provides,
            conflicts,
            replaces,
            check_dependencies,
            make_dependencies,
            sources,
            no_extracts,
            b2_checksums,
            md5_checksums,
            sha1_checksums,
            sha224_checksums,
            sha256_checksums,
            sha384_checksums,
            sha512_checksums,
        }
    }
}
