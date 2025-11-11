//! Provides fully resolved package metadata derived from SRCINFO data.
use alpm_types::{
    Architecture,
    Architectures,
    FullVersion,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    RelationOrSoname,
    RelativeFilePath,
    SkippableChecksum,
    Source,
    Url,
    digests::{Blake2b512, Crc32Cksum, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::source_info::v1::package::Override;
use crate::{
    SourceInfoV1,
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
    /// The alpm-package-name for the package.
    pub name: Name,
    /// The description for the package.
    pub description: Option<PackageDescription>,
    /// The upstream URL for the package.
    pub url: Option<Url>,
    /// The list of licenses that apply to the package.
    pub licenses: Vec<License>,
    /// The alpm-architecture for the package.
    pub architecture: Architecture,
    /// The optional relative path to a changelog file for the package.
    pub changelog: Option<RelativeFilePath>,

    // Build or package management related meta fields
    /// The optional relative path to an alpm-install-scriptlet for the package.
    pub install: Option<RelativeFilePath>,
    /// The list of alpm-package-groups the package is part of.
    pub groups: Vec<String>,
    /// The list of build tool options used when builidng the package.
    pub options: Vec<MakepkgOption>,
    /// The list of relative paths to files in the package that should be backed up.
    pub backups: Vec<RelativeFilePath>,

    /// The full version of the package.
    pub version: FullVersion,
    /// The list of OpenPGP fingerprints of OpenPGP certificates used for the verification of
    /// upstream sources.
    pub pgp_fingerprints: Vec<OpenPGPIdentifier>,

    /// The list of run-time dependencies.
    pub dependencies: Vec<RelationOrSoname>,
    /// The list of optional dependencies.
    pub optional_dependencies: Vec<OptionalDependency>,
    /// The list of provisions.
    pub provides: Vec<RelationOrSoname>,
    /// The list of conflicts.
    pub conflicts: Vec<PackageRelation>,
    /// The list of replacements.
    pub replaces: Vec<PackageRelation>,
    /// The list of test dependencies.
    pub check_dependencies: Vec<PackageRelation>,
    /// The list of build dependencies.
    pub make_dependencies: Vec<PackageRelation>,

    /// The list of sources for the package.
    pub sources: Vec<MergedSource>,
    /// The list of sources for the package that are not extracted.
    pub no_extracts: Vec<String>,
}

/// An iterator over all packages of a specific architecture.
#[derive(Clone, Debug)]
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
            // If the package provides target architecture overrides, use those, otherwise
            // fallback to package base architectures.
            let architectures = match &package.architectures {
                Some(value) => value,
                None => &self.source_info.base.architectures,
            };

            match &self.architecture {
                // If the packages are filtered by `any`, make sure that the package also has `any`.
                Architecture::Any => *architectures == Architectures::Any,
                // A specific architecture has been requested.
                // The package must have that architecture in its list or be viable for `any`
                // architecture.
                Architecture::Some(iterator_arch) => match architectures {
                    Architectures::Any => true,
                    Architectures::Some(arch_vec) => arch_vec.contains(iterator_arch),
                },
            }
        })?;

        Some(MergedPackage::from_base_and_package(
            self.architecture.clone(),
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
    /// The source.
    pub source: Source,
    /// The optional Blake2 hash digest of `source`.
    pub b2_checksum: Option<SkippableChecksum<Blake2b512>>,
    /// The optional MD-5 hash digest of `source`.
    pub md5_checksum: Option<SkippableChecksum<Md5>>,
    /// The optional SHA-1 hash digest of `source`.
    pub sha1_checksum: Option<SkippableChecksum<Sha1>>,
    /// The optional SHA-224 hash digest of `source`.
    pub sha224_checksum: Option<SkippableChecksum<Sha224>>,
    /// The optional SHA-256 hash digest of `source`.
    pub sha256_checksum: Option<SkippableChecksum<Sha256>>,
    /// The optional SHA-384 hash digest of `source`.
    pub sha384_checksum: Option<SkippableChecksum<Sha384>>,
    /// The optional SHA-512 hash digest of `source`.
    pub sha512_checksum: Option<SkippableChecksum<Sha512>>,
    /// The optional CRC-32/CKSUM hash digest of `source`.
    pub crc_checksum: Option<SkippableChecksum<Crc32Cksum>>,
}

/// A convenience iterator to build a list of [`MergedSource`] from the disjoint vectors of sources
/// and digests.
///
/// The checksums and sources are by convention all in the same order, which makes this quite
/// convenient to convert into a aggregated struct representation.
#[derive(Clone, Debug)]
pub struct MergedSourceIterator<'a> {
    sources: std::slice::Iter<'a, Source>,
    b2_checksums: std::slice::Iter<'a, SkippableChecksum<Blake2b512>>,
    md5_checksums: std::slice::Iter<'a, SkippableChecksum<Md5>>,
    sha1_checksums: std::slice::Iter<'a, SkippableChecksum<Sha1>>,
    sha224_checksums: std::slice::Iter<'a, SkippableChecksum<Sha224>>,
    sha256_checksums: std::slice::Iter<'a, SkippableChecksum<Sha256>>,
    sha384_checksums: std::slice::Iter<'a, SkippableChecksum<Sha384>>,
    sha512_checksums: std::slice::Iter<'a, SkippableChecksum<Sha512>>,
    crc_checksums: std::slice::Iter<'a, SkippableChecksum<Crc32Cksum>>,
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
            crc_checksum: self.crc_checksums.next().cloned(),
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
    /// 2. All architecture-agnostic fields of the [`Package`] are merged into the
    ///    [`MergedPackage`].
    /// 3. The architecture-specific properties of the [`PackageBase`] and [`Package`] are
    ///    extracted.
    /// 4. [`PackageBaseArchitecture::merge_package_properties`] is called to merge the
    ///    architecture-specific properties of the [`Package`] into those of the [`PackageBase`].
    /// 5. The combined architecture-specific properties are merged into the [`MergedPackage`].
    pub fn from_base_and_package<A: Into<Architecture>>(
        architecture: A,
        base: &PackageBase,
        package: &Package,
    ) -> MergedPackage {
        let name = package.name.clone();
        let architecture = &architecture.into();

        // Step 1
        let mut merged_package = Self::from_base(architecture.clone(), name, base);

        // Step 2
        merged_package.merge_package(package);

        // Get the architecture specific properties from the PackageBase.
        // Use an empty default without any properties as default if none are found,
        // or when the architecture is 'any'.
        let mut architecture_properties = if let Architecture::Some(system_arch) = &architecture
            && let Some(properties) = base.architecture_properties.get(system_arch)
        {
            properties.clone()
        } else {
            PackageBaseArchitecture::default()
        };

        // Apply package specific overrides for architecture specific properties.
        if let Architecture::Some(system_arch) = architecture
            && let Some(package_properties) = package.architecture_properties.get(system_arch)
        {
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
    pub fn from_base<A: Into<Architecture>>(
        architecture: A,
        name: Name,
        base: &PackageBase,
    ) -> MergedPackage {
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
            crc_checksums: base.crc_checksums.iter(),
        };

        // If the [`PackageBase`] is compatible with any architecture, then we set the architecture
        // of the package to 'any' regardless of the requested architecture, as 'any' subsumes them
        // all.
        let architecture = match &base.architectures {
            Architectures::Any => &Architecture::Any,
            Architectures::Some(_) => &architecture.into(),
        };

        MergedPackage {
            name,
            description: base.description.clone(),
            url: base.url.clone(),
            licenses: base.licenses.clone(),
            architecture: architecture.clone(),
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

    /// Merges the non-architecture specific fields of a [`Package`] into `self`.
    ///
    /// Any field on `package` that is not [`Override::No`] overrides the pendant on `self`.
    fn merge_package(&mut self, package: &Package) {
        let package = package.clone();

        // If the [`Package`] is compatible with any architecture, then we set the architecture of
        // the package to 'any' regardless of the requested architecture, as 'any' subsumes them
        // all.
        if let Some(value) = package.architectures {
            if matches!(value, Architectures::Any) {
                self.architecture = Architecture::Any
            }
        };

        package.description.merge_option(&mut self.description);
        package.url.merge_option(&mut self.url);
        package.changelog.merge_option(&mut self.changelog);
        package.licenses.merge_vec(&mut self.licenses);
        package.install.merge_option(&mut self.install);
        package.groups.merge_vec(&mut self.groups);
        package.options.merge_vec(&mut self.options);
        package.backups.merge_vec(&mut self.backups);
        package.dependencies.merge_vec(&mut self.dependencies);
        package
            .optional_dependencies
            .merge_vec(&mut self.optional_dependencies);
        package.provides.merge_vec(&mut self.provides);
        package.conflicts.merge_vec(&mut self.conflicts);
        package.replaces.merge_vec(&mut self.replaces);
    }

    /// Merges in architecture-specific overrides for fields.
    ///
    /// Takes a [`PackageBaseArchitecture`] and extends the non-architecture specific values
    /// with the architecture specific ones.
    /// This is an accumulative and non-destructive operation.
    fn merge_architecture_properties(&mut self, base_architecture: &PackageBaseArchitecture) {
        // Merge all source related info into aggregated structs.
        let merged_sources = MergedSourceIterator {
            sources: base_architecture.sources.iter(),
            b2_checksums: base_architecture.b2_checksums.iter(),
            md5_checksums: base_architecture.md5_checksums.iter(),
            sha1_checksums: base_architecture.sha1_checksums.iter(),
            sha224_checksums: base_architecture.sha224_checksums.iter(),
            sha256_checksums: base_architecture.sha256_checksums.iter(),
            sha384_checksums: base_architecture.sha384_checksums.iter(),
            sha512_checksums: base_architecture.sha512_checksums.iter(),
            crc_checksums: base_architecture.crc_checksums.iter(),
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
    }
}
