//! Package relation handling

mod base;
mod composite;
mod lookup;
mod soname;

pub use base::{Group, OptionalDependency, PackageRelation};
pub use composite::RelationOrSoname;
pub use lookup::RelationLookup;
pub use soname::{SharedLibraryPrefix, Soname, SonameV1, SonameV2, VersionOrSoname};
