//! ALPM dependency solver.

use std::{collections::HashSet, fmt, fmt::Debug};

use alpm_common::{GenericInstalledPackageMetadata, GenericPackageMetadata};
use alpm_types::{Name, PackageInstallReason, Version, VersionComparison, VersionRequirement};
use resolvo::{ConditionalRequirement, Problem, Requirement, UnsolvableOrCancelled};

use crate::{
    Solution,
    error::Error,
    provider::ALPMDependencyProvider,
    types::MatchSpec,
    utils::sort_candidates,
};

/// A dependency solver for **A**rch **L**inux **P**ackage **M**anagement (ALPM).
///
/// This solver uses the [`resolvo`] library to resolve package dependencies
/// based on the state of sync databases and the current system.
pub struct Solver {
    solver: resolvo::Solver<ALPMDependencyProvider>,
}

impl From<ALPMDependencyProvider> for Solver {
    /// Creates a [`Solver`] from an [`ALPMDependencyProvider`].
    fn from(provider: ALPMDependencyProvider) -> Self {
        let solver = resolvo::Solver::new(provider);
        Self { solver }
    }
}

impl Solver {
    /// Resolves package dependencies to upgrade the `system_state` to match the highest versions
    /// available.
    ///
    /// - `require_deps` is a set of dependencies explicitly chosen by the user. Must be a subset of
    ///   `system_state`, to be considered.
    /// - If `enforce_full_upgrade` is true, will fail if any package cannot be upgraded to the
    ///   latest version. This is equivalent to the resolution behavior of [`pacman -Su`][pacman]
    ///
    /// # Errors
    ///
    /// Returns an error if the dependencies cannot be resolved.
    ///
    /// [pacman]: https://man.archlinux.org/man/pacman.8
    pub fn upgrade<S>(
        &mut self,
        system_state: Vec<S>,
        require_deps: HashSet<Name>,
        enforce_full_upgrade: bool,
    ) -> Result<Solution, Error>
    where
        S: GenericInstalledPackageMetadata + Clone,
    {
        let mut requirements = Vec::new();
        let mut soft_requirements = Vec::new();

        for installed_package in system_state.iter() {
            let name = installed_package.get_name();
            let name_id = self
                .solver
                .provider()
                .pool
                .intern_package_name(name.clone());

            if matches!(
                installed_package.install_reason(),
                PackageInstallReason::Depend
            ) && !require_deps.contains(name)
            {
                // Add deps as soft requirements,
                // so they can be replaced in case of conflicts,
                // instead of failing the whole resolution.
                let mut candidates = self
                    .solver
                    .provider()
                    .records
                    .get(&name_id)
                    .map(|c| c.candidates.clone())
                    .unwrap_or_default();

                // Note: we are only adding the highest version candidate.
                // This is not optimal, but currently resolvo does not support
                // soft requirements with version sets.
                // This needs to be introduced upstream.
                sort_candidates(&self.solver.provider().pool, &mut candidates);
                let first = candidates.first().cloned();
                if let Some(solvable_id) = first {
                    soft_requirements.push(solvable_id)
                }

                continue;
            }

            let version: Version = if enforce_full_upgrade {
                // require the latest version from available packages instead
                let latest = self
                    .solver
                    .provider()
                    .get_highest_version(&name.clone().into());
                latest.unwrap_or(installed_package.get_version().into())
            } else {
                installed_package.get_version().into()
            };

            let match_spec = MatchSpec::from_requirement(
                Some(VersionRequirement::new(
                    VersionComparison::GreaterOrEqual,
                    version,
                )),
                None,
            );
            let version_set = self
                .solver
                .provider()
                .pool
                .intern_version_set(name_id, match_spec);

            requirements.push(ConditionalRequirement {
                condition: None,
                requirement: Requirement::Single(version_set),
            })
        }

        let problem = Problem::new()
            .requirements(requirements)
            .soft_requirements(soft_requirements);
        let raw_solution = self.solver.solve(problem).map_err(|e| match e {
            UnsolvableOrCancelled::Unsolvable(unsat) => {
                Error::Unsatisfiable(unsat.display_user_friendly(&self.solver).to_string())
            }
            // todo
            UnsolvableOrCancelled::Cancelled(_) => unreachable!(),
        })?;

        Ok(Solution::new(
            self.solver.provider(),
            system_state,
            raw_solution,
        ))
    }

    /// Resolves package dependencies to downgrade the `system_state` to match the versions
    /// specified in the `downgrade_set`.
    ///
    /// Requires all packages in the `system_state` that are not part of the `downgrade_set` to not
    /// change their versions.
    ///
    /// # Errors
    ///
    /// Returns an error if the dependencies cannot be resolved.
    pub fn downgrade<S, D>(
        &mut self,
        system_state: Vec<S>,
        downgrade_set: Vec<D>,
    ) -> Result<Solution, Error>
    where
        S: GenericInstalledPackageMetadata + Clone,
        D: GenericPackageMetadata,
    {
        let mut requirements = Vec::new();

        for installed_pkg in system_state.iter() {
            if let Some(downgrade_pkg) = downgrade_set
                .iter()
                .find(|p| p.get_name() == installed_pkg.get_name())
            {
                let name_id = self
                    .solver
                    .provider()
                    .pool
                    .intern_package_name(downgrade_pkg.get_name().clone());
                let match_spec = MatchSpec::from_requirement(
                    Some(VersionRequirement::new(
                        VersionComparison::Equal,
                        downgrade_pkg.get_version().clone().into(),
                    )),
                    None,
                );
                let version_set = self
                    .solver
                    .provider()
                    .pool
                    .intern_version_set(name_id, match_spec);

                requirements.push(ConditionalRequirement {
                    condition: None,
                    requirement: Requirement::Single(version_set),
                });
            } else {
                // if not in downgrade set, try to keep the current version.
                let name_id = self
                    .solver
                    .provider()
                    .pool
                    .intern_package_name(installed_pkg.get_name().clone());
                let match_spec = MatchSpec::from_requirement(
                    Some(VersionRequirement::new(
                        VersionComparison::LessOrEqual,
                        installed_pkg.get_version().clone().into(),
                    )),
                    None,
                );
                let version_set = self
                    .solver
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
        let raw_solution = self.solver.solve(problem).map_err(|e| match e {
            UnsolvableOrCancelled::Unsolvable(unsat) => {
                Error::Unsatisfiable(unsat.display_user_friendly(&self.solver).to_string())
            }
            // todo
            UnsolvableOrCancelled::Cancelled(_) => unreachable!(),
        })?;

        Ok(Solution::new(
            self.solver.provider(),
            system_state,
            raw_solution,
        ))
    }
}

impl Debug for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}
