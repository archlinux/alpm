//! Traits related to package data.

use alpm_types::RelationLookup;

/// Interface for gathering package related [alpm-package-relation] data in lookup tables.
///
/// This interface concerns itself with the following package relations:
///
/// - [conflict]
/// - [optional dependency]
/// - [provision]
/// - [replacement]
/// - [run-time dependency]
///
/// These can usually be found in e.g. [PKGINFO], [alpm-db-desc] or [alpm-repo-desc] data.
///
/// # Examples
///
/// ```
/// use alpm_common::RuntimeRelationLookupData;
/// use alpm_types::{OptionalDependency, PackageRelation, RelationLookup, RelationOrSoname};
///
/// /// A simple representation of package data, that doesn't concern itself with architecture-specific distinctions.
/// struct SimpleRelation {
///     run_time: Vec<RelationOrSoname>,
///     optional: Vec<OptionalDependency>,
///     provision: Vec<RelationOrSoname>,
///     conflict: Vec<PackageRelation>,
///     replacement: Vec<PackageRelation>,
/// }
///
/// impl RuntimeRelationLookupData for SimpleRelation {
///     fn add_run_time_dependencies_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///     ) {
///         for relation in self.run_time.iter() {
///             lookup.insert_relation_or_soname(relation, None);
///         }
///     }
///
///     fn add_optional_dependencies_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///     ) {
///         for optional in self.optional.iter() {
///             lookup.insert_package_relation(optional.package_relation(), None);
///         }
///     }
///
///     fn add_provisions_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///     ) {
///         for relation in self.provision.iter() {
///             lookup.insert_relation_or_soname(relation, None);
///         }
///     }
///
///     fn add_conflicts_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///     ) {
///         for relation in self.conflict.iter() {
///             lookup.insert_package_relation(relation, None);
///         }
///     }
///
///     fn add_replacements_to_lookup(
///         &self,
///         lookup: &mut RelationLookup,
///     ) {
///         for relation in self.replacement.iter() {
///             lookup.insert_package_relation(relation, None);
///         }
///     }
/// }
///
/// # fn main() -> testresult::TestResult {
/// let example = SimpleRelation {
///     run_time: vec!["dependency>=1".parse()?],
///     optional: vec!["optional: for some additional functionality".parse()?],
///     provision: vec!["virtual>=1".parse()?],
///     conflict: vec!["conflict-dep".parse()?],
///     replacement: vec!["dependency<=1".parse()?],
/// };
/// # Ok(())
/// # }
/// ```
///
/// [PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [conflict]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#conflict
/// [optional dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#optional-dependency
/// [provision]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#provision
/// [replacement]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#conflict
/// [run-time dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
pub trait RuntimeRelationLookupData {
    /// Adds any [run-time dependency] to a [`RelationLookup`].
    ///
    /// [run-time dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#run-time-dependency
    fn add_run_time_dependencies_to_lookup(&self, lookup: &mut RelationLookup);

    /// Adds any [optional dependency] to a [`RelationLookup`].
    ///
    /// [optional dependency]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#optional-dependency
    fn add_optional_dependencies_to_lookup(&self, lookup: &mut RelationLookup);

    /// Adds any [provision] to a [`RelationLookup`].
    ///
    /// [provision]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#provision
    fn add_provisions_to_lookup(&self, lookup: &mut RelationLookup);

    /// Adds any [conflict] to a [`RelationLookup`].
    ///
    /// [conflict]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#conflict
    fn add_conflicts_to_lookup(&self, lookup: &mut RelationLookup);

    /// Adds any [replacement] to a [`RelationLookup`].
    ///
    /// [replacement]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html#conflict
    fn add_replacements_to_lookup(&self, lookup: &mut RelationLookup);
}
