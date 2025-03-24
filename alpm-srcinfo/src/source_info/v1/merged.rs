//! Provides fully resolved package metadata derived from SRCINFO data.
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
    RelativePath,
    SkippableChecksum,
    Source,
    Url,
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};
use serde::{Deserialize, Serialize};

use crate::{
    SourceInfoV1,
    relation::RelationOrSoname,
    source_info::v1::{
        package::Package,
        package_base::{PackageBase, PackageBaseArchitecture},
    },
};

/// Fully resolved metadata of a single package based on SRCINFO data.
///
/// This struct incorporates all [`PackageBase`] properties and the [`Package`] specific overrides
/// in an architecture-specific representation of a package. It can be created using
/// [`SourceInfoV1::packages_for_architecture`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MergedPackage {
    pub name: Name,
    pub description: Option<PackageDescription>,
    pub url: Option<Url>,
    pub licenses: Vec<License>,
    pub architecture: Architecture,
    pub changelog: Option<RelativePath>,

    // Build or package management related meta fields
    pub install: Option<RelativePath>,
    pub groups: Vec<String>,
    pub options: Vec<MakepkgOption>,
    pub backups: Vec<RelativePath>,

    /// The version of the package
    pub package_version: PackageVersion,
    /// The release of the package
    pub package_release: PackageRelease,
    /// The epoch of the package
    pub epoch: Option<Epoch>,
    pub pgp_fingerprints: Vec<OpenPGPIdentifier>,

    pub dependencies: Vec<RelationOrSoname>,
    pub optional_dependencies: Vec<OptionalDependency>,
    pub provides: Vec<RelationOrSoname>,
    pub conflicts: Vec<PackageRelation>,
    pub replaces: Vec<PackageRelation>,
    pub check_dependencies: Vec<PackageRelation>,
    pub make_dependencies: Vec<PackageRelation>,

    pub sources: Vec<MergedSource>,
    pub no_extracts: Vec<String>,
}

/// An iterator over all packages of a specific architecture.
pub struct MergedPackagesIterator<'a> {
    pub(crate) architecture: Architecture,
    pub(crate) source_info: &'a SourceInfoV1,
    pub(crate) package_iterator: std::slice::Iter<'a, Package>,
}

impl Iterator for MergedPackagesIterator<'_> {
    type Item = MergedPackage;

    fn next(&mut self) -> Option<MergedPackage> {
        // Search for the next package that is valid for the the architecture we're looping over.
        let package = self.package_iterator.find(|package| {
            // If the package provides target architecture overrides, use those, otherwise fallback
            // to package base architectures.
            let architectures = if let Some(architectures) = &package.architectures {
                architectures
            } else {
                &self.source_info.base.architectures
            };

            architectures.contains(&self.architecture) || architectures.contains(&Architecture::Any)
        })?;

        Some(MergedPackage::from_base_and_package(
            self.architecture,
            &self.source_info.base,
            package,
        ))
    }
}

/// A merged representation of source related information.
///
/// SRCINFO provides this info as separate lists. This struct resolves that list representation and
/// provides a convenient aggregated representation for a single source.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MergedSource {
    pub source: Source,
    pub b2_checksum: Option<SkippableChecksum<Blake2b512>>,
    pub md5_checksum: Option<SkippableChecksum<Md5>>,
    pub sha1_checksum: Option<SkippableChecksum<Sha1>>,
    pub sha224_checksum: Option<SkippableChecksum<Sha224>>,
    pub sha256_checksum: Option<SkippableChecksum<Sha256>>,
    pub sha384_checksum: Option<SkippableChecksum<Sha384>>,
    pub sha512_checksum: Option<SkippableChecksum<Sha512>>,
}

/// A convenience iterator to build a list of [`MergedSource`] from the disjoint vectors of sources
/// and digests.
///
/// The checksums and sources are by convention all in the same order, which makes this quite
/// convenient to convert into a aggregated struct representation.
pub struct MergedSourceIterator<'a> {
    sources: std::slice::Iter<'a, Source>,
    b2_checksums: std::slice::Iter<'a, SkippableChecksum<Blake2b512>>,
    md5_checksums: std::slice::Iter<'a, SkippableChecksum<Md5>>,
    sha1_checksums: std::slice::Iter<'a, SkippableChecksum<Sha1>>,
    sha224_checksums: std::slice::Iter<'a, SkippableChecksum<Sha224>>,
    sha256_checksums: std::slice::Iter<'a, SkippableChecksum<Sha256>>,
    sha384_checksums: std::slice::Iter<'a, SkippableChecksum<Sha384>>,
    sha512_checksums: std::slice::Iter<'a, SkippableChecksum<Sha512>>,
}

impl Iterator for MergedSourceIterator<'_> {
    type Item = MergedSource;

    fn next(&mut self) -> Option<MergedSource> {
        let source = self.sources.next()?;

        Some(MergedSource {
            source: source.clone(),
            b2_checksum: self.b2_checksums.next().cloned(),
            md5_checksum: self.md5_checksums.next().cloned(),
            sha1_checksum: self.sha1_checksums.next().cloned(),
            sha224_checksum: self.sha224_checksums.next().cloned(),
            sha256_checksum: self.sha256_checksums.next().cloned(),
            sha384_checksum: self.sha384_checksums.next().cloned(),
            sha512_checksum: self.sha512_checksums.next().cloned(),
        })
    }
}

impl MergedPackage {
    /// Creates the fully resolved, architecture-specific metadata representation of a package.
    ///
    /// Takes an [`Architecture`] (which defines the architecture for which to create the
    /// representation), as well as a [`PackageBase`] and a [`Package`] (from which to derive the
    /// metadata).
    ///
    /// The metadata representation is created using the following steps:
    /// 1. [`MergedPackage::from_base`] is called to create a basic representation of a
    ///    [`MergedPackage`] based on the default values in [`PackageBase`].
    /// 2. [`MergedPackage::merge_package`] is called to merge all architecture-agnostic fields of
    ///    the [`Package`] into the [`MergedPackage`].
    /// 3. The architecture-specific properties of the [`PackageBase`] and [`Package`] are
    ///    extracted.
    /// 4. [`PackageBaseArchitecture::merge_package_properties`] is called to merge the
    ///    architecture-specific properties of the [`Package`] into those of the [`PackageBase`].
    /// 5. [`MergedPackage::merge_architecture_properties`] is called to merge the combined
    ///    architecture-specific properties into the [`MergedPackage`].
    pub fn from_base_and_package(
        architecture: Architecture,
        base: &PackageBase,
        package: &Package,
    ) -> MergedPackage {
        let name = package.name.clone();
        let mut merged_package = Self::from_base(&architecture, name, base);

        merged_package.merge_package(package);

        // Get the architecture specific properties from the PackageBase.
        // Use an empty default without any properties as default.
        let mut architecture_properties =
            if let Some(properties) = base.architecture_properties.get(&architecture) {
                properties.clone()
            } else {
                PackageBaseArchitecture::default()
            };

        // Apply package specific overrides for architecture specific properties.
        if let Some(package_properties) = package.architecture_properties.get(&architecture) {
            architecture_properties.merge_package_properties(package_properties.clone());
        }

        // Merge the architecture specific properties into the final MergedPackage.
        merged_package.merge_architecture_properties(&architecture_properties);

        merged_package
    }

    /// Creates a basic, architecture-specific, but incomplete [`MergedPackage`].
    ///
    /// Takes an [`Architecture`] (which defines the architecture for which to create the
    /// representation), a [`Name`] which defines the name of the package and a [`PackageBase`]
    /// which provides the initial data.
    ///
    /// # Note
    ///
    /// The returned [`MergedPackage`] is not complete, as it neither contains package-specific nor
    /// architecture-specific overrides for its fields.
    /// Use [`from_base_and_package`](MergedPackage::from_base_and_package) to create a fully
    /// resolved representation of a package.
    pub fn from_base(architecture: &Architecture, name: Name, base: &PackageBase) -> MergedPackage {
        // Merge all source related info into aggregated structs.
        let merged_sources = MergedSourceIterator {
            sources: base.sources.iter(),
            b2_checksums: base.b2_checksums.iter(),
            md5_checksums: base.md5_checksums.iter(),
            sha1_checksums: base.sha1_checksums.iter(),
            sha224_checksums: base.sha224_checksums.iter(),
            sha256_checksums: base.sha256_checksums.iter(),
            sha384_checksums: base.sha384_checksums.iter(),
            sha512_checksums: base.sha512_checksums.iter(),
        };

        MergedPackage {
            name,
            description: base.description.clone(),
            url: base.url.clone(),
            licenses: base.licenses.clone(),
            architecture: *architecture,
            changelog: base.changelog.clone(),
            install: base.install.clone(),
            groups: base.groups.clone(),
            options: base.options.clone(),
            backups: base.backups.clone(),
            package_version: base.package_version.clone(),
            package_release: base.package_release.clone(),
            epoch: base.epoch,
            pgp_fingerprints: base.pgp_fingerprints.clone(),
            dependencies: base.dependencies.clone(),
            optional_dependencies: base.optional_dependencies.clone(),
            provides: base.provides.clone(),
            conflicts: base.conflicts.clone(),
            replaces: base.replaces.clone(),
            check_dependencies: base.check_dependencies.clone(),
            make_dependencies: base.make_dependencies.clone(),
            sources: merged_sources.collect(),
            no_extracts: base.no_extracts.clone(),
        }
    }

    /// Merges in the fields of a [`Package`].
    ///
    /// Any field on `package` that is not [`None`] overrides the pendant on `self`.
    pub fn merge_package(&mut self, package: &Package) {
        if let Some(description) = &package.description {
            self.description = description.clone();
        }
        if let Some(url) = &package.url {
            self.url = url.clone();
        }
        if let Some(changelog) = &package.changelog {
            self.changelog = changelog.clone();
        }
        if let Some(licenses) = &package.licenses {
            self.licenses = licenses.clone();
        }
        if let Some(install) = &package.install {
            self.install = install.clone();
        }
        if let Some(groups) = &package.groups {
            self.groups = groups.clone();
        }
        if let Some(options) = &package.options {
            self.options = options.clone();
        }
        if let Some(backups) = &package.backups {
            self.backups = backups.clone();
        }
        if let Some(dependencies) = &package.dependencies {
            self.dependencies = dependencies.clone();
        }
        if let Some(optional_dependencies) = &package.optional_dependencies {
            self.optional_dependencies = optional_dependencies.clone();
        }
        if let Some(provides) = &package.provides {
            self.provides = provides.clone();
        }
        if let Some(conflicts) = &package.conflicts {
            self.conflicts = conflicts.clone();
        }
        if let Some(replaces) = &package.replaces {
            self.replaces = replaces.clone();
        }
    }

    /// Merges in architecture-specific overrides for fields.
    ///
    /// Takes a [`PackageBaseArchitecture`] and uses all of its fields as overrides for the pendants
    /// on `self`.
    pub fn merge_architecture_properties(&mut self, base_architecture: &PackageBaseArchitecture) {
        // Merge all source related info into a aggregated structs.
        let merged_sources = MergedSourceIterator {
            sources: base_architecture.sources.iter(),
            b2_checksums: base_architecture.b2_checksums.iter(),
            md5_checksums: base_architecture.md5_checksums.iter(),
            sha1_checksums: base_architecture.sha1_checksums.iter(),
            sha224_checksums: base_architecture.sha224_checksums.iter(),
            sha256_checksums: base_architecture.sha256_checksums.iter(),
            sha384_checksums: base_architecture.sha384_checksums.iter(),
            sha512_checksums: base_architecture.sha512_checksums.iter(),
        };

        self.dependencies
            .extend_from_slice(&base_architecture.dependencies);
        self.optional_dependencies
            .extend_from_slice(&base_architecture.optional_dependencies);
        self.provides.extend_from_slice(&base_architecture.provides);
        self.conflicts
            .extend_from_slice(&base_architecture.conflicts);
        self.replaces.extend_from_slice(&base_architecture.replaces);
        self.check_dependencies
            .extend_from_slice(&base_architecture.check_dependencies);
        self.make_dependencies
            .extend_from_slice(&base_architecture.make_dependencies);

        self.sources
            .extend_from_slice(&merged_sources.collect::<Vec<MergedSource>>());
        self.no_extracts
            .extend_from_slice(&base_architecture.no_extracts);
    }
}
