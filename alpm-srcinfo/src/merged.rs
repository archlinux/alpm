use alpm_types::{
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
    Architecture,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    RelativePath,
    SkippableChecksum,
    Source,
    Url,
    Version,
};
use serde::Serialize;

use crate::source_info::{
    package::{Package, PackageArchitecture},
    package_base::{PackageBase, PackageBaseArchitecture},
    SourceInfo,
};

/// The final merged representation of a package for a specific architecture.
/// This is most likely what you're looking for.
///
/// This struct is created by the [SourceInfo::packages_for_architecture] function.
/// This incorporates all [PackageBase] properties and the [Package] specific overrides.
#[derive(Debug, Clone, Serialize)]
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

    pub version: Version,
    pub pgp_fingerprints: Vec<OpenPGPIdentifier>,

    pub dependencies: Vec<PackageRelation>,
    pub optional_dependencies: Vec<OptionalDependency>,
    pub provides: Vec<PackageRelation>,
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
    pub(crate) source_info: &'a SourceInfo,
    pub(crate) package_iterator: std::slice::Iter<'a, Package>,
}

impl Iterator for MergedPackagesIterator<'_> {
    type Item = MergedPackage;

    fn next(&mut self) -> Option<MergedPackage> {
        // Search for the next package that is valid for the the architecture we're looping over.
        let package = loop {
            // Get the next package in the list, return if we reached the end of the package list.
            let package = self.package_iterator.next()?;

            // If the package provides target architecture overrides, use those, otherwise fallback
            // to package_base architectures.
            let architectures = if let Some(architectures) = &package.architectures {
                architectures
            } else {
                &self.source_info.base.architectures
            };

            if architectures.contains(&self.architecture)
                || architectures.contains(&Architecture::Any)
            {
                break package;
            }
        };

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
#[derive(Debug, Clone, Serialize)]
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
        // Get the next source. If there's non left, we may return None.
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

impl PackageBaseArchitecture {
    /// Merge architecture specific properties from a package into the architecture specific
    /// properties from the [`PackageBase`]
    ///
    /// The third step in the merging process.
    pub fn merge_package_properties(&mut self, properties: PackageArchitecture) {
        if let Some(dependencies) = properties.dependencies {
            self.dependencies = dependencies;
        }
        if let Some(optional_dependencies) = properties.optional_dependencies {
            self.optional_dependencies = optional_dependencies;
        }
        if let Some(provides) = properties.provides {
            self.provides = provides;
        }
        if let Some(conflicts) = properties.conflicts {
            self.conflicts = conflicts;
        }
        if let Some(replaces) = properties.replaces {
            self.replaces = replaces;
        }
    }
}

impl MergedPackage {
    /// Create the merged representation for a given PackageBase and Package
    ///
    /// This is done in multiple steps:
    /// 1. Create the base representation of a MergedPackage based on the default values in
    ///    [`PackageBase`]. Do **not** merge the architecture specific properties yet.
    /// 2. Merge the [`Package`] specific overrides into the `MergedPackage`. At this point, the
    ///    default values are handled. Now we need to handle the architecture specific additions.
    /// 3. Merge the [`PackageArchitecture`] package specific architecture property overrides into
    ///    [`PackageBaseArchitecture`].
    /// 4. Merge the final [`PackageBaseArchitecture`] into `MergedPackage`.
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

    /// Create a incomplete, but pre-populated MergedPackage based on the values from a given
    /// [`PackageBase`].
    ///
    /// The returned value needs to be completed by merging the [`Package`] overrides into it
    /// lateron via `merge_package`.
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
            version: base.version.clone(),
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

    /// Merge a [`Package`] into a [`MergedPackage`].
    ///
    /// This is the second step after the base data from [`PackageBase`] has been set on the
    /// `MergedPackage`.
    ///
    /// Any value that's set on `Package` will now override the previously set `PackageBase` value.
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

    /// Merge a [`PackageBaseArchitecture`] into a [`MergedPackage`].
    ///
    /// This is the last step after the `MergedPackage` has been created from the
    /// [`PackageBase`].
    ///
    /// All of these values might be overridden in the next step, when the [`Package`] is merged.
    fn merge_architecture_properties(&mut self, base_architecture: &PackageBaseArchitecture) {
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
