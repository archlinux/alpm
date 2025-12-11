use alpm_types::{
    FullVersion,
    Name,
    OptionalDependency,
    PackageInstallReason,
    PackageRelation,
    RelationOrSoname,
};

/// A trait for package metadata that provides the package name.
pub trait Named {
    /// Returns the package name.
    fn get_name(&self) -> &Name;
}

/// A trait for package metadata that provides the package version.
pub trait Versioned {
    /// Returns the package version.
    fn get_version(&self) -> &FullVersion;
}

/// A trait for package metadata that provides runtime relations.
pub trait RuntimeRelations {
    /// Returns the package dependencies.
    fn get_dependencies(&self) -> Vec<&RelationOrSoname>;

    /// Returns the package optional dependencies.
    fn get_optional_dependencies(&self) -> Vec<&OptionalDependency>;

    /// Returns the package provides.
    fn get_provides(&self) -> Vec<&RelationOrSoname>;

    /// Returns the package conflicts.
    fn get_conflicts(&self) -> Vec<&PackageRelation>;

    /// Returns the package replaces.
    fn get_replaces(&self) -> Vec<&PackageRelation>;
}

/// A trait for metadata specific to packages installed on the system.
pub trait Installed {
    /// Returns the reason why the package was installed.
    fn install_reason(&self) -> PackageInstallReason;
}

/// A trait for generic package metadata combining naming, versioning, and runtime relations.
pub trait GenericPackageMetadata: Named + Versioned + RuntimeRelations {}

/// A trait for generic installed package metadata combining generic package metadata and
/// metadata specific to installed packages.
pub trait GenericInstalledPackageMetadata: GenericPackageMetadata + Installed {}
