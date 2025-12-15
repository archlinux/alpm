use alpm_common::{Named, Versioned};
use alpm_types::{
    ElfArchitectureFormat,
    PackageRelation,
    RelationOrSoname,
    SonameV1,
    Version,
    VersionComparison,
    VersionRequirement,
};
use resolvo::{SolvableId, utils::Pool};

use crate::types::{MatchSpec, RelationName};

/// Converts a [`RelationOrSoname`] into a tuple of optional [`VersionRequirement`] and
/// [`ElfArchitectureFormat`].
pub fn into_requirement(
    relation_or_soname: RelationOrSoname,
) -> (Option<VersionRequirement>, Option<ElfArchitectureFormat>) {
    match relation_or_soname {
        RelationOrSoname::Relation(rel) => (rel.version_requirement, None),
        RelationOrSoname::SonameV1(soname) => match soname {
            SonameV1::Basic(_) => (None, None),
            SonameV1::Unversioned { architecture, .. } => (None, Some(architecture)),
            SonameV1::Explicit {
                version,
                architecture,
                ..
            } => (
                Some(VersionRequirement::new(
                    VersionComparison::Equal,
                    Version::new(version, None, None),
                )),
                Some(architecture),
            ),
        },
        RelationOrSoname::SonameV2(soname) => {
            let requirement = soname.soname.version.map(|version| {
                VersionRequirement::new(VersionComparison::Equal, Version::new(version, None, None))
            });
            (requirement, None)
        }
    }
}

/// Converts a [`RelationOrSoname`] into a tuple of optional [`Version`] and
/// [`ElfArchitectureFormat`].
///
/// This assumes that the version requirement is of type `Equal`, thus only safe to use with
/// provisions.
pub fn into_version(
    relation_or_soname: RelationOrSoname,
) -> (Option<Version>, Option<ElfArchitectureFormat>) {
    match relation_or_soname {
        RelationOrSoname::Relation(rel) => (rel.version_requirement.map(|vr| vr.version), None),
        RelationOrSoname::SonameV1(soname) => match soname {
            SonameV1::Basic(_) => (None, None),
            SonameV1::Unversioned { architecture, .. } => (None, Some(architecture)),
            SonameV1::Explicit {
                version,
                architecture,
                ..
            } => (Some(Version::new(version, None, None)), Some(architecture)),
        },
        RelationOrSoname::SonameV2(soname) => {
            let version = soname
                .soname
                .version
                .map(|version| Version::new(version, None, None));
            (version, None)
        }
    }
}

/// A generic conversion from [`Named`] and [`Versioned`] into a [`PackageRelation`].
///
/// The resulting [`PackageRelation`]'s `version_requirement` will be set to comparison type
/// [`VersionComparison::Equal`] and version returned by [`Versioned::get_version`].
///
/// This cannot be a [`From`] implementation as part of alpm-types due to circular dependencies.
pub fn into_package_relation<P: Named + Versioned>(pkg: &P) -> PackageRelation {
    PackageRelation {
        name: pkg.get_name().clone(),
        version_requirement: Some(VersionRequirement::new(
            VersionComparison::Equal,
            pkg.get_version().clone().into(),
        )),
    }
}

/// Core logic behind choosing the right candidate from a set.
/// todo: more docs
pub fn sort_candidates(pool: &Pool<MatchSpec, RelationName>, solvables: &mut [SolvableId]) {
    solvables.sort_by(|&a, &b| {
        let record_a = &pool.resolve_solvable(a).record;
        let record_b = &pool.resolve_solvable(b).record;
        // First we prioritize soft-locked packages
        // (different versions for already installed packages).
        record_b
            .is_soft_locked()
            .cmp(&record_a.is_soft_locked())
            // We always prioritize higher versions.
            .then_with(|| record_b.version().cmp(&record_a.version()))
            // We prefer real packages over virtual ones (in case of the same version).
            // We only use virtual components and sonames with no version if nothing else matches.
            .then_with(|| record_a.is_virtual().cmp(&record_b.is_virtual()))
            // At last if we are dealing with exact same versions of real packages,
            // we take "priority" into account to e.g. prefer installed or cached packages over
            // remote ones; or packages from `core` over `extra` (depending on the config).
            .then_with(|| record_b.priority().cmp(&record_a.priority()))
    });
}
