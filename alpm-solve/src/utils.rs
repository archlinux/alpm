use alpm_types::{RelationOrSoname, SonameV1, Version, VersionComparison, VersionRequirement};

/// Converts a [`RelationOrSoname`] into an optional [`VersionRequirement`].
pub fn into_requirement(relation_or_soname: RelationOrSoname) -> Option<VersionRequirement> {
    match relation_or_soname {
        RelationOrSoname::Relation(rel) => rel.version_requirement,
        RelationOrSoname::SonameV1(soname) => match soname {
            SonameV1::Basic(_) | SonameV1::Unversioned { .. } => None,
            SonameV1::Explicit { version, .. } => Some(VersionRequirement::new(
                VersionComparison::Equal,
                Version::new(version, None, None),
            )),
        },
        RelationOrSoname::SonameV2(soname) => soname.soname.version.map(|version| {
            VersionRequirement::new(VersionComparison::Equal, Version::new(version, None, None))
        }),
    }
}

/// Converts a [`RelationOrSoname`] into an optional [`Version`].
///
/// This assumes that the version requirement is of type `Equal`, thus only safe to use with
/// provisions.
pub fn into_version(relation_or_soname: RelationOrSoname) -> Option<Version> {
    match relation_or_soname {
        RelationOrSoname::Relation(rel) => rel.version_requirement.map(|vr| vr.version),
        RelationOrSoname::SonameV1(soname) => match soname {
            SonameV1::Basic(_) | SonameV1::Unversioned { .. } => None,
            SonameV1::Explicit { version, .. } => Some(Version::new(version, None, None)),
        },
        RelationOrSoname::SonameV2(soname) => soname
            .soname
            .version
            .map(|version| Version::new(version, None, None)),
    }
}
