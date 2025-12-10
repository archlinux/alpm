//! ...

use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use alpm_common::GenericPackageMetadata;
use alpm_types::{Version, VersionComparison, VersionRequirement};
use resolvo::{
    Candidates,
    Condition,
    ConditionId,
    ConditionalRequirement,
    Dependencies,
    DependencyProvider,
    HintDependenciesAvailable,
    Interner,
    KnownDependencies,
    NameId,
    Requirement,
    SolvableId,
    SolverCache,
    StringId,
    VersionSetId,
    VersionSetUnionId,
    utils::Pool,
};

use crate::{
    types::{MatchSpec, MetadataSource, PackageRecord, Provider, RelationName},
    utils::{into_requirement, into_version},
};

/// todo
pub struct ArchDependencyProvider {
    /// todo
    pub pool: Pool<MatchSpec, RelationName>,

    /// Cache of candidates for package names
    records: HashMap<NameId, Candidates>,

    /// Dependency lookup
    dependencies: HashMap<SolvableId, Vec<(RelationName, Option<VersionRequirement>)>>,

    /// Conflicts lookup
    conflicts: HashMap<SolvableId, Vec<(RelationName, Option<VersionRequirement>)>>,

    /// Optdepends lookup
    optdepends: HashMap<SolvableId, Vec<(RelationName, Option<VersionRequirement>)>>,

    /// Whether to enforce optional dependencies that are part of the requirement to match required
    /// versions.
    ///
    /// Not enforced by default.
    enforce_optdepends: bool,
    // Note: `replaces` is just a hint, can skip that completely
}

impl Default for ArchDependencyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchDependencyProvider {
    /// todo
    pub fn new() -> Self {
        Self {
            pool: Pool::default(),
            records: HashMap::new(),
            dependencies: HashMap::new(),
            conflicts: HashMap::new(),
            optdepends: HashMap::new(),
            enforce_optdepends: false,
        }
    }

    /// Sets whether optional dependencies should be enforced.
    ///
    /// Default is false.
    pub fn with_optdepends_enforced(mut self, enforce: bool) -> Self {
        self.enforce_optdepends = enforce;
        self
    }

    /// Adds packages/versions to the available pool.
    /// `source` describes from where the packages metadata originate (e.g. cache or sync db)
    pub fn add_available(
        &mut self,
        packages: impl Iterator<Item = impl GenericPackageMetadata>,
        source: &MetadataSource,
    ) {
        for metadata in packages {
            let name_id = self.pool.intern_package_name(metadata.get_name().clone());
            let solvable = self.pool.intern_solvable(
                name_id,
                PackageRecord::Real {
                    version: metadata.get_version().clone().into(),
                    source: source.clone(),
                },
            );
            self.records
                .entry(name_id)
                .or_default()
                .candidates
                .push(solvable);

            // This is created here and not inside the loop, so that we have an empty entry
            // if the package has no deps, and actual missing entry is a hard error.
            let dependencies_entry = self.dependencies.entry(solvable).or_default();
            for dep in metadata.get_dependencies() {
                dependencies_entry.push((dep.clone().into(), into_requirement(dep.clone())));
            }

            let conflicts_entry = self.conflicts.entry(solvable).or_default();
            for conflict in metadata.get_conflicts() {
                conflicts_entry.push((
                    RelationName::Relation(conflict.name.clone()),
                    conflict.version_requirement.clone(),
                ));
            }

            let optdepends_entry = self.optdepends.entry(solvable).or_default();
            for optdep in metadata.get_optional_dependencies() {
                optdepends_entry.push((
                    RelationName::Relation(optdep.name().clone()),
                    optdep.version_requirement().clone(),
                ));
            }

            // Now we have to also add all the provisions of the package
            // and link them back to the original package using `Provider`.
            for virtual_package in metadata.get_provides() {
                let virtual_name = RelationName::from(virtual_package.clone());
                let virtual_version = into_version(virtual_package.clone());

                let virtual_name_id = self.pool.intern_package_name(virtual_name);
                let solvable = self.pool.intern_solvable(
                    virtual_name_id,
                    PackageRecord::Virtual(
                        virtual_version,
                        Provider {
                            name: metadata.get_name().clone(),
                            version: metadata.get_version().clone().into(),
                        },
                    ),
                );
                self.records
                    .entry(virtual_name_id)
                    .or_default()
                    .candidates
                    .push(solvable);
            }
        }

        // We assume all dependencies are available,
        // because we populate everything upfront.
        for candidates in self.records.values_mut() {
            candidates.hint_dependencies_available = HintDependenciesAvailable::All;
        }
    }

    /// Gets the highest available version for a given package/soname.
    pub fn get_highest_version(&self, name: &RelationName) -> Option<Version> {
        let name_id = self.pool.intern_package_name(name.clone());
        let candidates = self.records.get(&name_id)?;

        candidates
            .candidates
            .iter()
            .filter_map(|&solvable| {
                let record = &self.pool.resolve_solvable(solvable).record;
                record.version()
            })
            .max()
            .cloned()
    }
}

impl Interner for ArchDependencyProvider {
    fn display_solvable(&self, solvable: SolvableId) -> impl Display + '_ {
        &self.pool.resolve_solvable(solvable).record
    }

    fn display_name(&self, name: NameId) -> impl Display + '_ {
        self.pool.resolve_package_name(name)
    }

    fn display_version_set(&self, version_set: VersionSetId) -> impl Display + '_ {
        self.pool.resolve_version_set(version_set)
    }

    fn display_string(&self, string_id: StringId) -> impl Display + '_ {
        self.pool.resolve_string(string_id)
    }

    fn version_set_name(&self, version_set: VersionSetId) -> NameId {
        self.pool.resolve_version_set_package_name(version_set)
    }

    fn solvable_name(&self, solvable: SolvableId) -> NameId {
        self.pool.resolve_solvable(solvable).name
    }

    fn version_sets_in_union(
        &self,
        version_set_union: VersionSetUnionId,
    ) -> impl Iterator<Item = VersionSetId> {
        self.pool.resolve_version_set_union(version_set_union)
    }

    fn resolve_condition(&self, condition: ConditionId) -> Condition {
        self.pool.resolve_condition(condition).clone()
    }
}

impl DependencyProvider for ArchDependencyProvider {
    async fn filter_candidates(
        &self,
        candidates: &[SolvableId],
        version_set: VersionSetId,
        inverse: bool,
    ) -> Vec<SolvableId> {
        let spec = self.pool.resolve_version_set(version_set);
        let requirement = &spec.requirement;
        candidates
            .iter()
            .copied()
            .filter(|&solvable| {
                let candidate = &self.pool.resolve_solvable(solvable).record;
                // Never match virtual packages if we require real ones
                if candidate.is_virtual() && spec.require_real {
                    return !inverse;
                }
                // Conflicts also invert the requirement
                let inverse = inverse ^ spec.conflict;
                match requirement {
                    // Unversioned requirements are always satisfied by any candidate
                    // e.g. `foo=1` satisfies `foo`
                    None => !inverse,
                    Some(requirement) => {
                        match candidate.version() {
                            Some(ver) => requirement.is_satisfied_by(ver) != inverse,
                            // Unversioned provides don't satisfy any version requirement
                            // (only satisfy unversioned requirements)
                            // e.g. `foo` does not satisfy `foo=1`
                            None => inverse,
                        }
                    }
                }
            })
            .collect()
    }

    async fn get_candidates(&self, name: NameId) -> Option<Candidates> {
        self.records.get(&name).cloned()
    }

    async fn sort_candidates(&self, solver: &SolverCache<Self>, solvables: &mut [SolvableId]) {
        solvables.sort_by(|&a, &b| {
            let record_a = &self.pool.resolve_solvable(a).record;
            let record_b = &self.pool.resolve_solvable(b).record;
            // Here we can later use different strategies (e.g. in case of downgrade).
            // For now we always prioritize higher versions.
            // We prefer real packages over virtual ones.
            // We only use virtual packages with no version if nothing else matches.
            record_b
                .version()
                .cmp(&record_a.version())
                .then_with(|| record_a.is_virtual().cmp(&record_b.is_virtual()))
                // At last if we are dealing with exact same versions of real packages,
                // we take "priority" into account to e.g. prefer installed or cached packages over
                // remote ones.
                .then_with(|| record_b.priority().cmp(&record_a.priority()))
        });
    }

    async fn get_dependencies(&self, solvable: SolvableId) -> Dependencies {
        let mut known_dependencies = KnownDependencies::default();

        let candidate = self.pool.resolve_solvable(solvable);

        match candidate.record.clone() {
            PackageRecord::Real { version, .. } => {
                // Regular dependencies handling
                if let Some(deps) = self.dependencies.get(&solvable) {
                    for (dep_name, dep_requirement) in deps {
                        let dep_name_id = self.pool.intern_package_name(dep_name.clone());
                        let dep_spec = MatchSpec::from_requirement(dep_requirement.clone());
                        let dep_version_set = self.pool.intern_version_set(dep_name_id, dep_spec);
                        known_dependencies
                            .requirements
                            .push(ConditionalRequirement {
                                requirement: Requirement::Single(dep_version_set),
                                condition: None,
                            });
                    }
                } else {
                    let package_name = self.pool.resolve_package_name(candidate.name);
                    let reason = self.pool.intern_string(format!(
                        "failed to find dependencies for {package_name}={version} - missing lookup entry",
                    ));
                    return Dependencies::Unknown(reason);
                }

                // Conflict handling
                // This translates to:
                // Require conflicting package with version **NOT** matching the conflict
                // requirement (MatchSpec inverted via conflict flag),
                // but **ONLY** if the conflicting package itself is in the solution
                // (handled by KnownDependencies::constrains).
                if let Some(conflicts) = self.conflicts.get(&solvable) {
                    for (conflict_name, conflict_requirement) in conflicts {
                        let conflict_name_id = self.pool.intern_package_name(conflict_name.clone());

                        let conflict_spec = MatchSpec::from_conflict(conflict_requirement.clone());
                        let conflict_version_set = self
                            .pool
                            .intern_version_set(conflict_name_id, conflict_spec);
                        known_dependencies.constrains.push(conflict_version_set);
                    }
                }

                // Optional dependencies handling
                // This works almost the same as conflicts, except we don't invert the requirement.
                if let Some(optdepends) = self.optdepends.get(&solvable) {
                    for (optdep_name, optdep_requirement) in optdepends {
                        let optdep_name_id = self.pool.intern_package_name(optdep_name.clone());

                        let optdep_spec = MatchSpec::from_requirement(optdep_requirement.clone());
                        let optdep_version_set =
                            self.pool.intern_version_set(optdep_name_id, optdep_spec);
                        known_dependencies.constrains.push(optdep_version_set);
                    }
                }
            }
            PackageRecord::Virtual(_, provider) => {
                // It's easier to make virtual dependencies "depend" on just their provider.
                // This is equivalent to adding all dependencies of the actual provider package.
                // The version of the virtual package can be completely ignored here.
                let provider_name_id = self.pool.intern_package_name(provider.name.clone());
                let provider_spec = MatchSpec::from_requirement(Some(VersionRequirement::new(
                    VersionComparison::Equal,
                    provider.version.clone(),
                )));
                let provider_version_set = self
                    .pool
                    .intern_version_set(provider_name_id, provider_spec);
                known_dependencies
                    .requirements
                    .push(ConditionalRequirement {
                        requirement: Requirement::Single(provider_version_set),
                        condition: None,
                    });
            }
        };

        Dependencies::Known(known_dependencies)
    }
}

impl Debug for ArchDependencyProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
