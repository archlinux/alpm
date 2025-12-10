//! ALPM dependency solver.

use std::{fmt, fmt::Debug};

use alpm_common::{GenericInstalledPackageMetadata, GenericPackageMetadata};
use alpm_types::{
    FullVersion,
    PackageInstallReason,
    Version,
    VersionComparison,
    VersionRequirement,
};
use resolvo::{
    ConditionalRequirement,
    Problem,
    Requirement,
    SolvableId,
    Solver,
    UnsolvableOrCancelled,
};

use crate::{
    Solution,
    error::Error,
    provider::ArchDependencyProvider,
    solution::PackageResolutionAction,
    types::{
        MatchSpec,
        MetadataSource,
        MetadataSourcePriority,
        PackageRecord,
        PackageRepositoryName,
        RelationName,
    },
};

/// Default priority for package cache.
///
/// `foo=1` installed on the system > `foo=1` in package cache > `foo=1` in package repository.
const DEFAULT_CACHE_PRIORITY: MetadataSourcePriority = 50;

/// Default priority for already installed packages.
///
/// This is set to `MAX` as we don't want to needlessly reinstall packages that are already
/// installed.
///
/// `foo=1` installed on the system > `foo=1` in package cache > `foo=1` in package repository.
const DEFAULT_INSTALLED_PRIORITY: MetadataSourcePriority = MetadataSourcePriority::MAX;

/// A solver for Arch Linux package dependencies.
///
/// This solver uses the resolvo library to resolve package dependencies
/// based on the state of sync databases and the current system.
pub struct ArchSolver {
    provider: ArchDependencyProvider,
    prune_unused: bool,
}

impl Default for ArchSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchSolver {
    /// Creates a new `ArchSolver`.
    pub fn new() -> Self {
        Self {
            provider: ArchDependencyProvider::new(),
            prune_unused: false,
        }
    }

    /// Adds a package repository to the solver.
    pub fn add_package_repository(
        &mut self,
        repo_name: PackageRepositoryName,
        priority: MetadataSourcePriority,
        packages: impl Iterator<Item = impl GenericPackageMetadata>,
    ) {
        self.provider
            .add_available(packages, &MetadataSource::Sync(repo_name, priority));
    }

    /// Adds a package cache to the solver.
    pub fn add_package_cache(&mut self, cache: impl Iterator<Item = impl GenericPackageMetadata>) {
        self.provider.add_available(
            cache.into_iter(),
            &MetadataSource::Cache(DEFAULT_CACHE_PRIORITY),
        );
    }

    /// Sets whether to enforce packages that are optional dependencies to meet requirements.
    pub fn with_optdepends_enforced(mut self, enforce: bool) -> Self {
        self.provider = self.provider.with_optdepends_enforced(enforce);
        self
    }

    /// Sets whether to prune unused not-explicit packages. Defaults to false, which
    /// means that all packages are considered explicit and remain on the system,
    /// even if no other package depends on them.
    pub fn with_prune_unused(mut self, prune: bool) -> Self {
        self.prune_unused = prune;
        self
    }

    /// Resolves package dependencies to upgrade the `system_state` to match the highest versions
    /// available.
    pub fn upgrade<S, SP>(
        mut self,
        system_state: S,
        // If true, will fail if any of the packages cannot be upgraded to the latest version
        enforce_full_upgrade: bool,
    ) -> Result<Solution, Error>
    where
        S: Iterator<Item = SP>,
        SP: GenericInstalledPackageMetadata + Clone,
    {
        let system_state: Vec<SP> = system_state.collect();

        self.provider.add_available(
            system_state.iter().cloned(),
            &MetadataSource::Installed(DEFAULT_INSTALLED_PRIORITY),
        );

        let mut solver = Solver::new(self.provider);
        let mut requirements = Vec::new();

        for installed_package in system_state.iter() {
            if self.prune_unused
                && matches!(
                    installed_package.install_reason(),
                    PackageInstallReason::Depend
                )
            {
                // Skip packages that were installed as dependencies
                // if they are still needed, solver will require them anyway
                continue;
            }

            let version: Version = if enforce_full_upgrade {
                // require the latest version from available packages instead
                let latest = solver
                    .provider()
                    .get_highest_version(&installed_package.get_name().clone().into());
                latest.unwrap_or(installed_package.get_version().into())
            } else {
                installed_package.get_version().into()
            };

            let name_id = solver
                .provider()
                .pool
                .intern_package_name(installed_package.get_name().clone());
            let match_spec = MatchSpec::from_requirement(Some(VersionRequirement::new(
                VersionComparison::GreaterOrEqual,
                version,
            )));
            let version_set = solver
                .provider()
                .pool
                .intern_version_set(name_id, match_spec);

            requirements.push(ConditionalRequirement {
                condition: None,
                requirement: Requirement::Single(version_set),
            })
        }

        let problem = Problem::new().requirements(requirements);
        let raw_solution = solver.solve(problem).map_err(|e| match e {
            UnsolvableOrCancelled::Unsolvable(unsat) => {
                Error::Unsatisfiable(unsat.display_user_friendly(&solver).to_string())
            }
            // todo
            UnsolvableOrCancelled::Cancelled(_) => unreachable!(),
        })?;

        Solution::new(solver.provider(), system_state, raw_solution)
    }

    /// Resolves package dependencies to downgrade the `system_state` to match the versions
    /// specified in the `downgrade_set`.
    pub fn downgrade<S, SP, D, DP>(
        mut self,
        system_state: S,
        downgrade_set: D,
    ) -> Result<Solution, Error>
    where
        S: Iterator<Item = SP>,
        SP: GenericInstalledPackageMetadata + Clone,
        D: Iterator<Item = DP>,
        DP: GenericPackageMetadata,
    {
        let system_state: Vec<SP> = system_state.collect();
        let downgrade_set: Vec<DP> = downgrade_set.collect();

        self.provider.add_available(
            system_state.iter().cloned(),
            &MetadataSource::Installed(DEFAULT_INSTALLED_PRIORITY),
        );

        let mut solver = Solver::new(self.provider);
        let mut requirements = Vec::new();

        for installed_pkg in system_state.iter() {
            if let Some(downgrade_pkg) = downgrade_set
                .iter()
                .find(|p| p.get_name() == installed_pkg.get_name())
            {
                let name_id = solver
                    .provider()
                    .pool
                    .intern_package_name(downgrade_pkg.get_name().clone());
                let match_spec = MatchSpec::from_requirement(Some(VersionRequirement::new(
                    VersionComparison::Equal,
                    downgrade_pkg.get_version().clone().into(),
                )));
                let version_set = solver
                    .provider()
                    .pool
                    .intern_version_set(name_id, match_spec);

                requirements.push(ConditionalRequirement {
                    condition: None,
                    requirement: Requirement::Single(version_set),
                });
            } else {
                // if not in downgrade set, try to keep at most current version.
                let name_id = solver
                    .provider()
                    .pool
                    .intern_package_name(installed_pkg.get_name().clone());
                let match_spec = MatchSpec::from_requirement(Some(VersionRequirement::new(
                    VersionComparison::LessOrEqual,
                    installed_pkg.get_version().clone().into(),
                )));
                let version_set = solver
                    .provider()
                    .pool
                    .intern_version_set(name_id, match_spec);

                requirements.push(ConditionalRequirement {
                    condition: None,
                    requirement: Requirement::Single(version_set),
                });
            }
        }

        let problem = Problem::new().requirements(requirements);
        let raw_solution = solver.solve(problem).map_err(|e| match e {
            UnsolvableOrCancelled::Unsolvable(unsat) => {
                Error::Unsatisfiable(unsat.display_user_friendly(&solver).to_string())
            }
            // todo
            UnsolvableOrCancelled::Cancelled(_) => unreachable!(),
        })?;

        Solution::new(solver.provider(), system_state, raw_solution)
    }
}

impl Debug for ArchSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchSolver").finish_non_exhaustive()
    }
}
