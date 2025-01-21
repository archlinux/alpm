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
    Source,
    Url,
    Version,
};

use crate::source_info::{
    package::{Package, PackageArchitecture},
    package_base::{PackageBase, PackageBaseArchitecture},
    SkippableChecksum,
    SourceInfo,
};

/// The final merged representation of a package for a specific architecture.
/// This is most likely what you're looking for.
///
/// This struct is created by the [SourceInfo::packages_for_architecture] function.
/// This incorporates all [PackageBase] properties and the [Package] specific overrides.
#[derive(Debug, Clone)]
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

    pub sources: Vec<Source>,
    pub no_extracts: Vec<String>,
    pub b2_checksums: Vec<SkippableChecksum<Blake2b512>>,
    pub md5_checksums: Vec<SkippableChecksum<Md5>>,
    pub sha1_checksums: Vec<SkippableChecksum<Sha1>>,
    pub sha224_checksums: Vec<SkippableChecksum<Sha224>>,
    pub sha256_checksums: Vec<SkippableChecksum<Sha256>>,
    pub sha384_checksums: Vec<SkippableChecksum<Sha384>>,
    pub sha512_checksums: Vec<SkippableChecksum<Sha512>>,
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
            sources: base.sources.clone(),
            no_extracts: base.no_extracts.clone(),
            b2_checksums: base.b2_checksums.clone(),
            md5_checksums: base.md5_checksums.clone(),
            sha1_checksums: base.sha1_checksums.clone(),
            sha224_checksums: base.sha224_checksums.clone(),
            sha256_checksums: base.sha256_checksums.clone(),
            sha384_checksums: base.sha384_checksums.clone(),
            sha512_checksums: base.sha512_checksums.clone(),
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

        self.sources.extend_from_slice(&base_architecture.sources);
        self.no_extracts
            .extend_from_slice(&base_architecture.no_extracts);
        self.b2_checksums
            .extend_from_slice(&base_architecture.b2_checksums);
        self.md5_checksums
            .extend_from_slice(&base_architecture.md5_checksums);
        self.sha1_checksums
            .extend_from_slice(&base_architecture.sha1_checksums);
        self.sha224_checksums
            .extend_from_slice(&base_architecture.sha224_checksums);
        self.sha256_checksums
            .extend_from_slice(&base_architecture.sha256_checksums);
        self.sha384_checksums
            .extend_from_slice(&base_architecture.sha384_checksums);
        self.sha512_checksums
            .extend_from_slice(&base_architecture.sha512_checksums);
    }
}
