//! Traits related to build data and build environments.

use alpm_types::{Architecture, RelationLookup};

/// Interface for gathering [build dependencies] in lookup tables.
///
/// This specific [alpm-package-relation] data is available e.g. in [BUILDINFO] or [SRCINFO] data.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use alpm_common::BuildRelationLookupData;
/// use alpm_types::{Architecture, PackageRelation, RelationLookup};
///
/// struct SimpleRelation {
///     build: Vec<PackageRelation>,
///     build_arch: BTreeMap<Architecture, Vec<PackageRelation>>,
/// }
///
/// impl BuildRelationLookupData for SimpleRelation {
///     fn add_build_dependencies_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///         architecture: Option<&Architecture>,
///     ) {
///         for package_relation in self.build.iter() {
///             lookup.insert_package_relation(package_relation, None);
///         }
///         if let Some(architecture) = architecture {
///             if let Some(arch_specific) = self.build_arch.get(architecture) {
///                 for package_relation in arch_specific.iter() {
///                     lookup.insert_package_relation(package_relation, None);
///                 }
///             }
///         }
///     }
/// }
///
/// # fn main() -> testresult::TestResult {
/// let example = SimpleRelation {
///     build: vec!["dependency>=1".parse()?],
///     build_arch: BTreeMap::from_iter(vec![(
///         Architecture::Some("x86_64".parse()?),
///         vec!["x86-dependency>=1".parse()?],
///     )]),
/// };
/// # Ok(())
/// # }
/// ```
///
/// [BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [build dependencies]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#build-dependency
pub trait BuildRelationLookupData {
    /// Adds any [build dependency] to a [`RelationLookup`].
    ///
    /// # Note
    ///
    /// If `architecture` is provided, the implementation should consider any architecture-specific
    /// [build dependency] as well and if the targeted `architecture` cannot be matched should not
    /// add any [build dependency] to the `lookup`.
    ///
    /// [build dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#build-dependency
    fn add_build_dependencies_to_lookup(
        &self,
        lookup: &mut RelationLookup,
        architecture: Option<&Architecture>,
    );
}
