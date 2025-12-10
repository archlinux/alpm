//! ALPM types specific to the dependency solver.

use std::fmt::{Display, Formatter};

use alpm_types::{
    ElfArchitectureFormat,
    Name,
    PackageRelation,
    RelationOrSoname,
    SharedLibraryPrefix,
    SharedObjectName,
    SonameV1,
    Version,
    VersionRequirement,
};
use resolvo::utils::VersionSet;

/// Describes an unversioned package, virtual package, or soname.
///
/// Together with a `Version` should uniquely identify a package/soname.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RelationName {
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

impl Display for RelationName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationName::Relation(name) => write!(f, "{}", name),
            RelationName::SonameV1 {
                name,
                soname,
                architecture,
            } => {
                write!(f, "{}", name)?;
                if let Some(soname) = soname {
                    write!(f, "-{}", soname)?;
                }
                if let Some(arch) = architecture {
                    write!(f, "-{}", arch)?;
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

impl From<PackageRelation> for RelationName {
    fn from(relation: PackageRelation) -> Self {
        RelationName::Relation(relation.name)
    }
}

impl From<RelationOrSoname> for RelationName {
    fn from(value: RelationOrSoname) -> Self {
        match value {
            RelationOrSoname::Relation(rel) => RelationName::Relation(rel.name),
            RelationOrSoname::SonameV1(soname) => match soname {
                SonameV1::Basic(name) | SonameV1::Explicit { name, .. } => RelationName::SonameV1 {
                    name,
                    soname: None,
                    architecture: None,
                },
                SonameV1::Unversioned {
                    name,
                    soname,
                    architecture,
                } => RelationName::SonameV1 {
                    name,
                    soname: Some(soname),
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

/// Package repository name
pub type PackageRepositoryName = Name;

/// The priority of metadata source in case of multiple sources providing the same package version.
///
/// Higher priority value means the source is preferred.
pub type MetadataSourcePriority = i8;

/// Origin of metadata
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum MetadataSource {
    /// Package is in sync database.
    Sync(PackageRepositoryName, MetadataSourcePriority),
    /// Package is in cache.
    Cache(MetadataSourcePriority),
    /// Package is already installed on the system.
    Installed(MetadataSourcePriority),
}

impl Display for MetadataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataSource::Sync(repo, _) => write!(f, "{}", repo),
            MetadataSource::Cache(_) => write!(f, "package cache"),
            MetadataSource::Installed(_) => write!(f, "already installed"),
        }
    }
}

impl MetadataSource {
    /// Returns the optional [`MetadataSourcePriority`].
    pub fn priority(&self) -> MetadataSourcePriority {
        match self {
            MetadataSource::Sync(_, priority) => *priority,
            MetadataSource::Cache(priority) => *priority,
            MetadataSource::Installed(priority) => *priority,
        }
    }
}

/// Represents a provider of a virtual package or soname.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Provider {
    /// Name of the package from which the virtual package originates.
    pub(crate) name: Name,
    /// Version of the package from which the virtual package originates.
    ///
    /// **NOT** the version of the virtual package itself.
    pub(crate) version: Version,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

/// todo
#[derive(Clone, Debug, Hash)]
pub enum PackageRecord {

    /// Real package
    Real {
        /// todo
        version: Version,
        /// todo
        source: MetadataSource,
    },

    /// Virtual package or a soname
    /// Only a virtual packages/sonames can have no version.
    Virtual(Option<Version>, Provider),
}

impl PackageRecord {
    /// todo
    pub fn version(&self) -> Option<&Version> {
        match self {
            PackageRecord::Real { version, .. } => Some(version),
            PackageRecord::Virtual(version, _) => version.as_ref(),
        }
    }

    /// todo
    pub fn is_virtual(&self) -> bool {
        matches!(self, PackageRecord::Virtual(_, _))
    }

    pub fn priority(&self) -> Option<MetadataSourcePriority> {
        match self {
            PackageRecord::Real { source, .. } => Some(source.priority()),
            PackageRecord::Virtual(_, _) => None,
        }
    }
}

impl Display for PackageRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageRecord::Real {
                version, source, ..
            } => write!(f, "{} ({})", version, source),
            PackageRecord::Virtual(version, provider) => {
                let version = match version {
                    Some(ver) => ver.to_string(),
                    None => "<any version>".to_string(),
                };
                write!(f, "{} (provided by {})", version, provider)
            }
        }
    }
}

/// todo
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MatchSpec {
    /// todo
    // This is option because version requirements are not mandatory.
    // In case of None, we match with all candidates.
    pub requirement: Option<VersionRequirement>,

    /// Indicates whether this spec represents a conflict.
    /// Inverts the requirement.
    pub conflict: bool,

    /// Require package to be real (not virtual/soname).
    pub require_real: bool,
}

impl MatchSpec {
    /// todo
    pub fn from_requirement(requirement: Option<VersionRequirement>) -> Self {
        Self {
            requirement,
            conflict: false,
            require_real: false,
        }
    }

    /// todo
    pub fn from_conflict(requirement: Option<VersionRequirement>) -> Self {
        Self {
            requirement,
            conflict: true,
            require_real: true,
        }
    }
}

impl VersionSet for MatchSpec {
    type V = PackageRecord;
}

impl Display for MatchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let conflicts = if self.conflict { "to not be " } else { "" };
        if let Some(req) = &self.requirement {
            write!(f, "{}{}", conflicts, req)
        } else {
            write!(f, "{}<any version>", conflicts)
        }
    }
}
