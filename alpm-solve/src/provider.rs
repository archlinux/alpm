//! Dependency provider for Arch Linux packages.

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
};

use alpm_common::{GenericPackageMetadata, Named};
use alpm_types::{
    Name,
    OptionalDependency,
    PackageRelation,
    RelationOrSoname,
    RepositoryName,
    Version,
    VersionComparison,
    VersionRequirement,
};
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
    MetadataSourcePriority,
    types::{MatchSpec, PackageMetadataOrigin, PackageRecord, Provider, RelationName, Replaces},
    utils::{into_package_relation, into_requirement, into_version},
};

/// Default priority for package cache.
///
/// `foo=1` installed on the system > `foo=1` in package cache > `foo=1` in package repository.
const DEFAULT_CACHE_PRIORITY: MetadataSourcePriority = 50;

/// Default priority for already installed packages.
///
/// This is set to [`i8::MAX`] as we don't want to needlessly reinstall packages that are already
/// installed.
///
/// `foo=1` installed on the system > `foo=1` in package cache > `foo=1` in package repository.
const DEFAULT_INSTALLED_PRIORITY: MetadataSourcePriority = MetadataSourcePriority::MAX;

/// Dependency provider for **A**rch **L**inux **P**ackage **M**anagement (ALPM).
///
/// Implements the [`DependencyProvider`] trait for use with the [`resolvo`] solver.
pub struct ALPMDependencyProvider {
    /// Internalized data about available packages.
    pub(crate) pool: Pool<MatchSpec, RelationName>,

    /// Cache of candidates for package names
    pub(crate) records: HashMap<NameId, Candidates>,

    /// Dependency lookup
    dependencies: HashMap<SolvableId, Vec<RelationOrSoname>>,

    /// Conflicts lookup
    conflicts: HashMap<SolvableId, Vec<PackageRelation>>,

    /// Optdepends lookup
    optdepends: HashMap<SolvableId, Vec<OptionalDependency>>,

    // Note: `replaces` is just a hint, that is taken into account after SAT solving.
    // that's why we don't need a separate lookup for it.
    /// Two-way conflicts lookup used for detecting which dependencies must be removed due to
    /// conflict as opposed to being simply not required anymore.
    ///
    /// Additionally tracks if one conflicting dependency can be replaced by another.
    pub(crate) conflicts_map: HashMap<PackageRelation, HashMap<PackageRelation, Replaces>>,

    /// Whether to enforce optional dependencies that are part of the requirement to match required
    /// versions.
    ///
    /// Not enforced by default.
    enforce_optdepends: bool,

    /// Names of packages that should be left on the system if possible.
    soft_locks: HashSet<Name>,
}

impl ALPMDependencyProvider {
    /// Creates a new [`ALPMDependencyProvider`] from the system state.
    pub fn new<S: GenericPackageMetadata + Clone>(system_state: &[S]) -> Self {
        Self {
            pool: Pool::default(),
            records: HashMap::new(),
            dependencies: HashMap::new(),
            conflicts: HashMap::new(),
            optdepends: HashMap::new(),
            conflicts_map: HashMap::new(),
            enforce_optdepends: false,
            soft_locks: HashSet::from_iter(system_state.iter().map(Named::get_name).cloned()),
        }
    }

    /// Sets whether optional dependencies should be enforced.
    ///
    /// Default is false.
    pub fn with_optdepends_enforced(mut self, enforce: bool) -> Self {
        self.enforce_optdepends = enforce;
        self
    }

    /// Adds a package repository to the solver's available packages.
    pub fn add_package_repository(
        &mut self,
        repo_name: RepositoryName,
        priority: MetadataSourcePriority,
        packages: impl IntoIterator<Item = impl GenericPackageMetadata>,
    ) {
        self.add_available(packages, &PackageMetadataOrigin::Sync(repo_name, priority));
    }

    /// Adds a package cache to the solver's available packages.
    pub fn add_package_cache(
        &mut self,
        cache: impl IntoIterator<Item = impl GenericPackageMetadata>,
    ) {
        self.add_available(cache, &PackageMetadataOrigin::Cache(DEFAULT_CACHE_PRIORITY));
    }

    /// Adds already installed packages to the solver's available packages.
    pub fn add_installed(
        &mut self,
        installed: impl IntoIterator<Item = impl GenericPackageMetadata>,
    ) {
        self.add_available(
            installed,
            &PackageMetadataOrigin::Db(DEFAULT_INSTALLED_PRIORITY),
        );
    }

    /// Adds packages/versions to the available pool.
    ///
    /// `source` describes from where the packages metadata originate (e.g. cache or sync db).
    pub fn add_available(
        &mut self,
        packages: impl IntoIterator<Item = impl GenericPackageMetadata>,
        source: &PackageMetadataOrigin,
    ) {
        // micro-optimization
        let soft_lock_all = matches!(source, PackageMetadataOrigin::Db(_));
        for metadata in packages {
            // Check if the real package with the same name (not version!) is currently installed.
            // This is used to soft-lock dependencies.
            // So that resolver won't needlessly switch e.g. `pipewire-jack` to `jack2`
            // if the user has explicitly chosen `pipewire-jack`.
            let is_soft_locked = soft_lock_all || self.soft_locks.contains(metadata.get_name());

            let name_id = self.pool.intern_package_name(metadata.get_name().clone());
            let solvable = self.pool.intern_solvable(
                name_id,
                PackageRecord::Real {
                    version: metadata.get_version().clone(),
                    source: source.clone(),
                    soft_lock: is_soft_locked,
                },
            );
            self.records
                .entry(name_id)
                .or_default()
                .candidates
                .push(solvable);

            self.dependencies
                .insert(solvable, metadata.get_run_time_dependencies().to_vec());
            self.conflicts
                .insert(solvable, metadata.get_conflicts().to_vec());
            self.optdepends
                .insert(solvable, metadata.get_optional_dependencies().to_vec());

            // Forward conflicts + replaces
            {
                let entry = self
                    .conflicts_map
                    .entry(into_package_relation(&metadata))
                    .or_default();
                for conflict in metadata.get_conflicts() {
                    let inner_entry = entry.entry(conflict.clone()).or_default();
                    // Note: replaces are only considered if they are also a conflict.
                    *inner_entry |= metadata.get_replacements().contains(conflict);
                }
            }
            // Reverse conflicts
            for conflict in metadata.get_conflicts() {
                self.conflicts_map
                    .entry(conflict.clone())
                    .or_default()
                    .entry(into_package_relation(&metadata))
                    .or_default();
            }
            // Reverse replaces
            for replace in metadata.get_replacements() {
                *self
                    .conflicts_map
                    .entry(replace.clone())
                    .or_default()
                    .entry(into_package_relation(&metadata))
                    .or_default() |= true;
            }

            // Now we have to also add all the provisions of the package
            // and link them back to the original package using `Provider`.
            for provide in metadata.get_provisions() {
                let virtual_name = RelationName::from(provide.clone());
                let (virtual_version, virtual_arch) = into_version(provide.clone());

                let virtual_name_id = self.pool.intern_package_name(virtual_name.clone());
                let solvable = self.pool.intern_solvable(
                    virtual_name_id,
                    PackageRecord::Virtual {
                        version: virtual_version.clone(),
                        architecture: virtual_arch,
                        provider: Provider {
                            name: metadata.get_name().clone(),
                            version: metadata.get_version().clone(),
                        },
                        soft_lock: is_soft_locked,
                    },
                );
                self.records
                    .entry(virtual_name_id)
                    .or_default()
                    .candidates
                    .push(solvable);

                // 32-bit and 64-bit sonames can be installed at the same time.
                // The simplest solution to this is to append the architecture to the name.
                // _But_ this causes another issue - dependencies/relations on sonames without
                // architecture will not match provides with architectures, as solver will see these
                // as different virtual components.
                // To hack around it, we add each soname provide that includes an architecture
                // twice: once with the architecture in the name, and once without.
                if let Some(virtual_name) = virtual_name.strip_architecture() {
                    let virtual_name_id = self.pool.intern_package_name(virtual_name.clone());
                    let solvable = self.pool.intern_solvable(
                        virtual_name_id,
                        PackageRecord::Virtual {
                            version: virtual_version,
                            architecture: virtual_arch,
                            provider: Provider {
                                name: metadata.get_name().clone(),
                                version: metadata.get_version().clone(),
                            },
                            soft_lock: is_soft_locked,
                        },
                    );
                    self.records
                        .entry(virtual_name_id)
                        .or_default()
                        .candidates
                        .push(solvable);
                }
            }
        }

        // We assume all dependencies are available,
        // because we populate everything upfront.
        for candidates in self.records.values_mut() {
            candidates.hint_dependencies_available = HintDependenciesAvailable::All;
        }
    }

    /// Gets the highest available version for a given package, virtual component or soname.
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
    }
}

impl Interner for ALPMDependencyProvider {
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

impl DependencyProvider for ALPMDependencyProvider {
    async fn filter_candidates(
        &self,
        candidates: &[SolvableId],
        version_set: VersionSetId,
        inverse: bool,
    ) -> Vec<SolvableId> {
        let spec = self.pool.resolve_version_set(version_set);
        candidates
            .iter()
            .copied()
            .filter(|&solvable| {
                spec.matches(&self.pool.resolve_solvable(solvable).record) ^ inverse
            })
            .collect()
    }

    async fn get_candidates(&self, name: NameId) -> Option<Candidates> {
        self.records.get(&name).cloned()
    }

    async fn sort_candidates(&self, _solver: &SolverCache<Self>, solvables: &mut [SolvableId]) {
        // Sorting logic is moved out, so it can be used in sync code.
        crate::utils::sort_candidates(&self.pool, solvables);
    }

    async fn get_dependencies(&self, solvable: SolvableId) -> Dependencies {
        let mut known_dependencies = KnownDependencies::default();

        let candidate = self.pool.resolve_solvable(solvable);

        match candidate.record.clone() {
            PackageRecord::Real { version, .. } => {
                // Regular dependencies handling
                if let Some(deps) = self.dependencies.get(&solvable) {
                    for dep in deps {
                        let dep_name_id = self.pool.intern_package_name::<RelationName>(dep.into());
                        let (requirement, arch) = into_requirement(dep.clone());
                        let dep_spec = MatchSpec::from_requirement(requirement, arch);
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
                    for conflict in conflicts {
                        let conflict_name_id = self
                            .pool
                            .intern_package_name::<RelationName>(conflict.into());
                        let conflict_spec =
                            MatchSpec::from_conflict(conflict.version_requirement.clone());
                        let conflict_version_set = self
                            .pool
                            .intern_version_set(conflict_name_id, conflict_spec);
                        known_dependencies.constrains.push(conflict_version_set);
                    }
                }

                // Optional dependencies handling
                // This works almost the same as conflicts, except we don't invert the requirement.
                if let Some(optdepends) = self.optdepends.get(&solvable) {
                    for optdep in optdepends {
                        let optdep_name_id =
                            self.pool.intern_package_name::<RelationName>(optdep.into());
                        let optdep_spec =
                            MatchSpec::from_requirement(optdep.version_requirement().clone(), None);
                        let optdep_version_set =
                            self.pool.intern_version_set(optdep_name_id, optdep_spec);
                        known_dependencies.constrains.push(optdep_version_set);
                    }
                }
            }
            PackageRecord::Virtual { provider, .. } => {
                // It's easier to make virtual dependencies "depend" on just their provider.
                // This is equivalent to adding all dependencies of the actual provider package.
                // The version of the virtual component / soname can be completely ignored here.
                let provider_name_id = self.pool.intern_package_name(provider.name.clone());
                let provider_spec = MatchSpec::from_requirement(
                    Some(VersionRequirement::new(
                        VersionComparison::Equal,
                        provider.version.clone().into(),
                    )),
                    None,
                );
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

impl Debug for ALPMDependencyProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ALPMDependencyProvider").finish()
    }
}
