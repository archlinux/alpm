//! ALPM types specific to the dependency solver.

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{
    ElfArchitectureFormat,
    FullVersion,
    Name,
    OptionalDependency,
    PackageRelation,
    RelationOrSoname,
    RepositoryName,
    SharedLibraryPrefix,
    SharedObjectName,
    SonameV1,
    Version,
    VersionComparison,
    VersionRequirement,
};
use resolvo::utils::VersionSet;

/// Indicates whether a conflicting dependency replaces another.
///
/// # Note
///
/// Indicates the package is a replacement **AND** a forward or reverse conflict. Replacements that
/// are not conflicts are **not considered by the solver**.
pub type Replaces = bool;

/// An unversioned name of a package, [virtual component], or an [alpm-soname].
///
/// Together with a [`Version`] should uniquely identify a package, virtual component or a soname.
///
/// [alpm-soname]: https://alpm.archlinux.page/specifications/alpm-soname.7.html
/// [virtual component]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#packages-and-virtual-components
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RelationName {
    /// This can also be a soname, please watch my 14h video essay if you want to know why.
    Relation(Name),
    SonameV1 {
        name: SharedObjectName,
        soname: Option<SharedObjectName>,
        architecture: Option<ElfArchitectureFormat>,
    },
    SonameV2 {
        prefix: SharedLibraryPrefix,
        soname: SharedObjectName,
    },
}

impl RelationName {
    /// Strips architecture information from a [`RelationName::SonameV1`], if any.
    ///
    /// Returns [`None`] if there was no architecture to strip.
    pub fn strip_architecture(&self) -> Option<Self> {
        match self {
            RelationName::Relation(..) => None,
            RelationName::SonameV1 {
                name,
                soname,
                architecture,
            } => {
                match architecture {
                    Some(_) => {
                        match soname {
                            Some(soname) => Some(RelationName::SonameV1 {
                                name: name.clone(),
                                soname: Some(soname.clone()),
                                architecture: None,
                            }),
                            // More on why it's `RelationName::Relation` on
                            // `From<RelationOrSoname> for RelationName` impl comments.
                            None => Some(RelationName::Relation(Name::from_str(&name.to_string())
                                .expect("This can't panic as `Name` is a superset of `SharedObjectName`"))),
                        }
                    }
                    None => None,
                }
            }
            RelationName::SonameV2 { .. } => None,
        }
    }

    /// Converts this [`RelationName`] into a [`PackageRelation`], if possible.
    // TODO: this probably should be fallible and check if RelationName and PackageRecord are
    // compatible
    pub fn package_relation(&self, record: &PackageRecord) -> Option<PackageRelation> {
        match self {
            RelationName::Relation(name) => Some(PackageRelation {
                name: name.clone(),
                version_requirement: record
                    .version()
                    .map(|v| VersionRequirement::new(VersionComparison::Equal, v)),
            }),
            RelationName::SonameV1 { .. } => None,
            RelationName::SonameV2 { .. } => None,
        }
    }
}

impl Display for RelationName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationName::Relation(name) => write!(f, "{}", name),
            RelationName::SonameV1 { name, soname, .. } => {
                write!(f, "{}", name)?;
                if let Some(soname) = soname {
                    write!(f, "-{}", soname)?;
                }
                Ok(())
            }
            RelationName::SonameV2 { prefix, soname } => {
                write!(f, "{}:{}", prefix, soname)
            }
        }
    }
}

impl From<Name> for RelationName {
    fn from(name: Name) -> Self {
        RelationName::Relation(name)
    }
}

impl From<&Name> for RelationName {
    fn from(name: &Name) -> Self {
        RelationName::Relation(name.clone())
    }
}

impl From<PackageRelation> for RelationName {
    fn from(relation: PackageRelation) -> Self {
        RelationName::Relation(relation.name)
    }
}

impl From<&PackageRelation> for RelationName {
    fn from(relation: &PackageRelation) -> Self {
        relation.clone().into()
    }
}

impl From<RelationOrSoname> for RelationName {
    fn from(value: RelationOrSoname) -> Self {
        match value {
            RelationOrSoname::Relation(rel) => RelationName::Relation(rel.name),
            RelationOrSoname::SonameV1(soname) => match soname {
                // We treat basic sonames as relations, because we can have things like
                // "libexample.so>=10" that would be parsed as relation, and
                // "libexample.so" wouldn't satisfy it otherwise...
                SonameV1::Basic(name) => RelationName::Relation(
                    Name::from_str(&name.to_string())
                        .expect("This can't panic as `Name` is a superset of `SharedObjectName`"),
                ),
                SonameV1::Unversioned {
                    name,
                    soname,
                    architecture,
                } => RelationName::SonameV1 {
                    name,
                    soname: Some(soname),
                    architecture: Some(architecture),
                },
                SonameV1::Explicit {
                    name, architecture, ..
                } => RelationName::SonameV1 {
                    name,
                    soname: None,
                    architecture: Some(architecture),
                },
            },
            RelationOrSoname::SonameV2(soname) => RelationName::SonameV2 {
                prefix: soname.prefix,
                soname: soname.soname.name,
            },
        }
    }
}

impl From<&RelationOrSoname> for RelationName {
    fn from(value: &RelationOrSoname) -> Self {
        value.clone().into()
    }
}

impl From<&OptionalDependency> for RelationName {
    fn from(value: &OptionalDependency) -> Self {
        value.name().into()
    }
}

/// The priority of metadata source in case of multiple sources providing the same package version.
///
/// Higher priority value means the source is preferred.
pub type MetadataSourcePriority = i8;

/// The origin of package metadata.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PackageMetadataOrigin {
    /// An [alpm-repo-db].
    ///
    /// [alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html
    Sync(RepositoryName, MetadataSourcePriority),

    /// Local package cache.
    Cache(MetadataSourcePriority),

    /// The local [alpm-db].
    ///
    /// [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
    Db(MetadataSourcePriority),
}

impl Display for PackageMetadataOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageMetadataOrigin::Sync(repo, _) => write!(f, "{}", repo),
            PackageMetadataOrigin::Cache(_) => write!(f, "package cache"),
            PackageMetadataOrigin::Db(_) => write!(f, "already installed"),
        }
    }
}

impl PackageMetadataOrigin {
    /// Returns the optional [`MetadataSourcePriority`].
    pub fn priority(&self) -> MetadataSourcePriority {
        match self {
            PackageMetadataOrigin::Sync(_, priority) => *priority,
            PackageMetadataOrigin::Cache(priority) => *priority,
            PackageMetadataOrigin::Db(priority) => *priority,
        }
    }
}

/// A provider of a [virtual component] or an [alpm-soname].
///
/// [alpm-soname]: https://alpm.archlinux.page/specifications/alpm-soname.7.html
/// [virtual component]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#packages-and-virtual-components
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Provider {
    /// Name of the package from which the virtual component or soname originates.
    pub(crate) name: Name,
    /// Version of the package from which the virtual component or soname originates.
    ///
    /// **NOT** the version of the virtual component or soname itself.
    pub(crate) version: FullVersion,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

/// A package record.
///
/// A package record tracks the data of a package, or a virtual components or alpm-soname that a
/// package may provide. Package records are useful, as they allow to distinguish packages and
/// virtual components or alpm-sonames of the same name.
#[derive(Clone, Debug, Hash)]
pub enum PackageRecord {
    /// A package.
    Real {
        /// Version of the package.
        version: FullVersion,
        /// Origin of the package metadata.
        source: PackageMetadataOrigin,
        /// Indicates whether this package is currently installed on the system (but not
        /// necessarily the same version of it).
        ///
        /// This increases the priority of this record to prevent unnecessary replacements
        /// of alternative dependencies chosen previously by the user.
        soft_lock: bool,
    },

    /// A virtual component or alpm-soname.
    Virtual {
        /// [`Version`] of the virtual component or alpm-soname, if any.
        version: Option<Version>,
        /// [`ElfArchitectureFormat`] of soname, if any.
        architecture: Option<ElfArchitectureFormat>,
        /// The [`Provider`] of the virtual component or alpm-soname.
        ///
        /// This tracks the reverse provision relation.
        provider: Provider,
        /// Indicates whether this virtual component/soname originates from the same package
        /// as the one currently installed on the system (but not necessarily the same version of
        /// it).
        ///
        /// This increases the priority of this record to prevent unnecessary replacements
        /// of alternative dependencies chosen previously by the user.
        soft_lock: bool,
    },
}

impl PackageRecord {
    /// Returns the version of the package/virtual component, if any.
    pub fn version(&self) -> Option<Version> {
        match self {
            PackageRecord::Real { version, .. } => Some(version.into()),
            PackageRecord::Virtual { version, .. } => version.clone(),
        }
    }

    /// Returns the architecture format of the soname, if any.
    pub fn architecture_format(&self) -> Option<&ElfArchitectureFormat> {
        match self {
            PackageRecord::Real { .. } => None,
            PackageRecord::Virtual { architecture, .. } => architecture.as_ref(),
        }
    }

    /// Returns true if the package record represents a virtual component or soname.
    pub fn is_virtual(&self) -> bool {
        matches!(self, PackageRecord::Virtual { .. })
    }

    /// Returns the optional [`MetadataSourcePriority`] if the package is real.
    pub fn priority(&self) -> Option<MetadataSourcePriority> {
        match self {
            PackageRecord::Real { source, .. } => Some(source.priority()),
            PackageRecord::Virtual { .. } => None,
        }
    }

    /// Returns whether this package record is soft-locked.
    ///
    /// Soft-locked candidates have the highest priority when choosing between multiple candidates.
    pub fn is_soft_locked(&self) -> bool {
        match self {
            PackageRecord::Real { soft_lock, .. } => *soft_lock,
            PackageRecord::Virtual { soft_lock, .. } => *soft_lock,
        }
    }
}

impl Display for PackageRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageRecord::Real {
                version, source, ..
            } => write!(f, "{} ({})", version, source),
            PackageRecord::Virtual {
                version,
                provider,
                architecture,
                ..
            } => {
                let architecture = match architecture {
                    Some(arch) => format!(" ({}-bit)", arch),
                    None => "".to_string(),
                };
                let version = match version {
                    Some(ver) => ver.to_string(),
                    None => "<any version>".to_string(),
                };
                write!(f, "{}{} (provided by {})", version, architecture, provider)
            }
        }
    }
}

/// Specification for [`PackageRecord`] match criteria.
///
/// Match criteria provides requirements and rules for finding matches for a [`PackageRecord`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MatchSpec {
    /// [`VersionRequirement`] to match against.
    ///
    /// [`None`] matches all candidates.
    pub requirement: Option<VersionRequirement>,

    /// If specified, restricts matching to packages built for a specific ELF architecture format.
    pub architecture: Option<ElfArchitectureFormat>,

    /// Indicates whether this spec represents a conflict.
    ///
    /// Inverts the imposed requirements and only matches [`PackageRecord::Real`].
    pub conflict: bool,
}

impl MatchSpec {
    /// Creates a new [`MatchSpec`] from a version requirement and optional architecture format.
    pub fn from_requirement(
        requirement: Option<VersionRequirement>,
        architecture: Option<ElfArchitectureFormat>,
    ) -> Self {
        Self {
            requirement,
            architecture,
            conflict: false,
        }
    }

    /// Creates a new conflicting [`MatchSpec`] from a version requirement.
    pub fn from_conflict(requirement: Option<VersionRequirement>) -> Self {
        Self {
            requirement,
            architecture: None,
            conflict: true,
        }
    }

    /// Returns `true` if the given [`PackageRecord`] satisfies this match specification.
    ///
    /// Following must be true for a [`PackageRecord`] to satisfy the match specification:
    /// 1. [`PackageRecord::version`] can't be [`None`] and must satisfy a version requirement of
    ///    [`MatchSpec`] **OR** the version requirement of [`MatchSpec`] must be [`None`].
    /// 2. [`PackageRecord::architecture_format`] can't be [`None`] and must be equal to
    ///    [`MatchSpec`] architecture **OR** the architecture of [`MatchSpec`] must be [`None`].
    ///
    /// Version requirement check delegates to [`VersionRequirement::is_satisfied_by`].
    ///
    /// If [`MatchSpec`] is a conflict (created using [`MatchSpec::from_conflict`]):
    /// 1. The output of the method is inverted.
    /// 2. [`PackageRecord::Virtual`] always returns `true`.
    pub fn matches(&self, package: &PackageRecord) -> bool {
        // We only consider conflicts with _real_ packages.
        // Otherwise, e.g. `rustup` conflicts with itself as it provides `cargo`
        // and also conflicts with `cargo`.
        if self.conflict && package.is_virtual() {
            // `false ^ self.conflict`
            return true;
        }

        // Only consider architecture if specified in the match spec so that e.g.:
        // - soname `libfoo.so=15-64` satisfies requirement `libfoo.so>=14`
        // - soname `libfoo.so=14-32` doesn't satisfy requirement `libfoo.so=14-64`
        if self.architecture.is_some()
            && package.architecture_format() != self.architecture.as_ref()
        {
            return self.conflict;
        }

        let matches = match &self.requirement {
            Some(requirement) => match &package.version() {
                Some(version) => requirement.is_satisfied_by(version),
                None => {
                    // If the package has no version (virtual/soname without version),
                    // we can only satisfy the requirement if there is no requirement.
                    // e.g.:
                    // - virtual component `foo` does not satisfy requirement `foo=1`
                    // - virtual component `foo` satisfies requirement `foo`
                    return self.requirement.is_none() ^ self.conflict;
                }
            },
            // Unversioned requirements are always satisfied by any candidate
            // e.g. package `foo=1` satisfies requirement `foo`
            None => true,
        };

        matches ^ self.conflict
    }
}

impl VersionSet for MatchSpec {
    type V = PackageRecord;
}

impl Display for MatchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let conflicts = if self.conflict { "to not be " } else { "" };
        let architecture = if let Some(arch) = &self.architecture {
            format!(" {}-bit", arch)
        } else {
            "".to_string()
        };
        if let Some(req) = &self.requirement {
            write!(f, "{}{}{}", conflicts, req, architecture)
        } else {
            write!(f, "{}<any version>{}", conflicts, architecture)
        }
    }
}
