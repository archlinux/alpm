use alpm_types::{
    FullVersion,
    Name,
    OptionalDependency,
    PackageInstallReason,
    PackageRelation,
    RelationOrSoname,
};

/// An interface for retrieving the name of a package.
///
/// This trait can be implemented for package metadata that provides the name of a package.
pub trait Named {
    /// Returns the package name.
    fn get_name(&self) -> &Name;
}

/// An interface for retrieving the full version of a package.
///
/// This trait can be implemented for package metadata that provides the full version of a package.
pub trait Versioned {
    /// Returns the package version.
    fn get_version(&self) -> &FullVersion;
}

/// An interface for retrieving run-time package relations.
///
/// This trait can be implemented for package metadata that provides run-time relations.
pub trait RuntimeRelations {
    /// Returns the run-time dependencies of a package.
    fn get_run_time_dependencies(&self) -> &[RelationOrSoname];

    /// Returns the optional dependencies of a package.
    fn get_optional_dependencies(&self) -> &[OptionalDependency];

    /// Returns the provisions of a package.
    fn get_provisions(&self) -> &[RelationOrSoname];

    /// Returns the conflicts of a package.
    fn get_conflicts(&self) -> &[PackageRelation];

    /// Returns the replacements of a package.
    fn get_replacements(&self) -> &[PackageRelation];
}

/// An interface to retrieve the installation reason for a package.
///
/// This trait can be implemented for package metadata specific to the packages installed on an ALPM
/// based system.
pub trait Installed {
    /// Returns the installation reason of a package.
    fn install_reason(&self) -> PackageInstallReason;
}

/// Generic interface for package metadata offering access to name, version, and run-time relations.
pub trait GenericPackageMetadata: Named + Versioned + RuntimeRelations {}

impl<T> GenericPackageMetadata for T where T: Named + Versioned + RuntimeRelations {}

/// Generic interface for metadata of packages installed on an ALPM based system.
///
/// This trait can be implemented for metadata facilities specific to packages installed on an ALPM
/// based system. It combines access as described in [`GenericPackageMetadata`] and the installation
/// reason of a package.
pub trait GenericInstalledPackageMetadata: GenericPackageMetadata + Installed {}

impl<T> GenericInstalledPackageMetadata for T where T: GenericPackageMetadata + Installed {}
