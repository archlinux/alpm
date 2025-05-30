//! Handling of metadata found in a `pkgname` section of SRCINFO data.
use std::collections::{BTreeMap, HashSet};

use alpm_types::{
    Architecture,
    License,
    MakepkgOption,
    Name,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    RelationOrSoname,
    RelativePath,
    Url,
};
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::{MergedPackage, SourceInfoV1, source_info::v1::package_base::PackageBase};
use crate::{
    error::SourceInfoError,
    lints::{
        duplicate_architecture,
        missing_architecture_for_property,
        non_spdx_license,
        reassigned_cleared_property,
    },
    source_info::{
        helper::ordered_optional_hashset,
        parser::{
            ClearableProperty,
            PackageProperty,
            RawPackage,
            RelationProperty,
            SharedMetaProperty,
        },
    },
};

/// A [`Package`] property that can override its respective defaults in [`PackageBase`].
///
/// This type is similar to [`Option`], which has special serialization behavior.
/// However, in some file formats (e.g. JSON) it is not possible to represent data such as
/// `Option<Option<T>>`, as serialization would flatten the structure. This type enables
/// representation of this type of data.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "override")]
pub enum Override<T> {
    #[default]
    No,
    Clear,
    Yes {
        value: T,
    },
}

impl<T> Override<T> {
    /// Applies `self` onto an `Option<T>`.
    ///
    /// - `Override::No` -> `other` stays untouched.
    /// - `Override::Clear` -> `other` is set to `None`.
    /// - `Override::Yes { value }` -> `other` is set to `Some(value)`.
    #[inline]
    pub fn merge_option(self, other: &mut Option<T>) {
        match self {
            Override::No => (),
            Override::Clear => *other = None,
            Override::Yes { value } => *other = Some(value),
        }
    }

    /// If `Override::Yes`, its value will be returned.
    /// If `self` is something else, `self` will be set to a `Override::Yes { value: default }`.
    ///
    /// Similar to as [Option::get_or_insert].
    #[inline]
    pub fn get_or_insert(&mut self, default: T) -> &mut T {
        if let Override::Yes { value } = self {
            return value;
        }

        *self = Override::Yes { value: default };

        // This is infallible.
        if let Override::Yes { value } = self {
            return value;
        }
        unreachable!()
    }
}

impl<T> Override<Vec<T>> {
    /// Applies `self` onto an `Vec<T>`.
    ///
    /// - `Override::No` -> `other` stays untouched.
    /// - `Override::Clear` -> `other` is set to `Vec::new()`.
    /// - `Override::Yes { value }` -> `other` is set to `value`.
    #[inline]
    pub fn merge_vec(self, other: &mut Vec<T>) {
        match self {
            Override::No => (),
            Override::Clear => *other = Vec::new(),
            Override::Yes { value } => *other = value,
        }
    }
}

/// Package metadata based on a `pkgname` section in SRCINFO data.
///
/// This struct only contains package specific overrides.
/// Only in combination with [`PackageBase`] data a full view on a package's metadata is possible.
///
/// All values and nested structs inside this struct, except the `name` field, are either nested
/// [`Option`]s (e.g. `Override<Option<String>>`) or optional collections (e.g. `Option<Vec>`).
/// This is due to the fact that all fields are overrides for the defaults set by the
/// [`PackageBase`] struct.
/// - If a value is `Override::No`, this indicates that the [`PackageBase`]'s value should be used.
/// - If a value is `Override::Yes<None>`, this means that the value should be empty and the
///   [`PackageBase`] should be ignored. The same goes for collections in the sense of
///   `Override::Yes(Vec::new())`.
/// - If a value is `Override::Yes(Some(value))` or `Override::Yes(vec![values])`, these values
///   should then be used.
///
/// This struct merely contains the overrides that should be applied on top of the
/// [PackageBase] to get the final definition of this package.
//
/// Take a look at [SourceInfoV1::packages_for_architecture] on how to get the merged representation
/// [MergedPackage] of a package.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: Name,
    pub description: Override<PackageDescription>,
    pub url: Override<Url>,
    pub changelog: Override<RelativePath>,
    pub licenses: Override<Vec<License>>,

    // Build or package management related meta fields
    pub install: Override<RelativePath>,
    pub groups: Override<Vec<String>>,
    pub options: Override<Vec<MakepkgOption>>,
    pub backups: Override<Vec<RelativePath>>,

    /// These are all override fields that may be architecture specific.
    /// Despite being overridable, `architectures` field isn't of the `Override` type, as it
    /// **cannot** be cleared.
    #[serde(serialize_with = "ordered_optional_hashset")]
    pub architectures: Option<HashSet<Architecture>>,
    pub architecture_properties: BTreeMap<Architecture, PackageArchitecture>,

    pub dependencies: Override<Vec<RelationOrSoname>>,
    pub optional_dependencies: Override<Vec<OptionalDependency>>,
    pub provides: Override<Vec<RelationOrSoname>>,
    pub conflicts: Override<Vec<PackageRelation>>,
    pub replaces: Override<Vec<PackageRelation>>,
}

/// Architecture specific package properties for use in [`Package`].
///
/// For each [`Architecture`] defined in [`Package::architectures`] a [`PackageArchitecture`] is
/// present in [`Package::architecture_properties`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PackageArchitecture {
    pub dependencies: Override<Vec<RelationOrSoname>>,
    pub optional_dependencies: Override<Vec<OptionalDependency>>,
    pub provides: Override<Vec<RelationOrSoname>>,
    pub conflicts: Override<Vec<PackageRelation>>,
    pub replaces: Override<Vec<PackageRelation>>,
}

/// Handles all potentially architecture specific, clearable entries in the [`Package::from_parsed`]
/// function.
///
/// If no architecture is encountered, it simply clears the value on the [`Package`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [`PackageBase::architectures`] or [`Package::architectures`] if overridden.
macro_rules! clearable_arch_vec {
    (
        $line:ident,
        $errors:ident,
        $lint_architectures:ident,
        $architecture_properties:ident,
        $architecture:ident,
        $field_name:ident,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparations
        if let Some(architecture) = $architecture {
            let properties = $architecture_properties.entry(*architecture).or_default();
            properties.$field_name = Override::Clear;

            // Throw an error for all architecture specific properties that don't have
            // an explicit `arch` statement. This is considered bad style.
            // Also handle the special `Any` [Architecture], which allows all architectures.
            if !$lint_architectures.contains(&architecture)
                && !$lint_architectures.contains(&Architecture::Any)
            {
                missing_architecture_for_property($errors, $line, *architecture);
            }
        } else {
            $field_name = Override::Clear;
        }
    };
}

/// Handles all potentially architecture specific Vector entries in the [`Package::from_parsed`]
/// function.
///
/// If no architecture is encountered, it simply adds the value on the [`Package`] itself.
/// Otherwise, it clears the value on the respective [`Package::architecture_properties`] entry.
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [`PackageBase::architectures`] or [`Package::architectures`] if overridden.
macro_rules! package_arch_prop {
    (
        $line:ident,
        $errors:ident,
        $lint_architectures:ident,
        $architecture_properties:ident,
        $arch_property:ident,
        $field_name:ident,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparations
        if let Some(architecture) = $arch_property.architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $architecture_properties
                .entry(architecture)
                .or_insert(PackageArchitecture::default());

            // Set the architecture specific value.
            architecture_properties
                .$field_name
                .get_or_insert(Vec::new())
                .push($arch_property.value);

            // Throw an error for all architecture specific properties that don't have
            // an explicit `arch` statement. This is considered bad style.
            // Also handle the special `Any` [Architecture], which allows all architectures.
            if !$lint_architectures.contains(&architecture)
                && !$lint_architectures.contains(&Architecture::Any)
            {
                missing_architecture_for_property($errors, $line, architecture);
            }
        } else {
            $field_name
                .get_or_insert(Vec::new())
                .push($arch_property.value)
        }
    };
}

impl Package {
    /// Creates a new [`Package`] instance from a [`RawPackage`].
    ///
    /// # Parameters
    ///
    /// - `line_start`: The number of preceding lines, so that error/lint messages can reference the
    ///   correct lines.
    /// - `parsed`: The [`RawPackage`] representation of the SRCINFO data. The input guarantees that
    ///   the keyword assignments have been parsed correctly, but not yet that they represent valid
    ///   SRCINFO data as a whole.
    /// - `errors`: All errors and lints encountered during the creation of the [`Package`].
    ///
    /// # Errors
    ///
    /// This function does not return a [`Result`], but instead relies on aggregating all lints,
    /// warnings and errors in `errors`.
    /// This allows to keep the function call recoverable, so that all errors and lints can
    /// be returned all at once.
    pub fn from_parsed(
        line_start: usize,
        package_base_architectures: &HashSet<Architecture>,
        parsed: RawPackage,
        errors: &mut Vec<SourceInfoError>,
    ) -> Self {
        let mut description = Override::No;
        let mut url = Override::No;
        let mut licenses = Override::No;
        let mut changelog = Override::No;
        let mut architectures = None;
        let mut architecture_properties: BTreeMap<Architecture, PackageArchitecture> =
            BTreeMap::new();

        // Build or package management related meta fields
        let mut install = Override::No;
        let mut groups = Override::No;
        let mut options = Override::No;
        let mut backups = Override::No;

        let mut dependencies = Override::No;
        let mut optional_dependencies = Override::No;
        let mut provides = Override::No;
        let mut conflicts = Override::No;
        let mut replaces = Override::No;

        // First up, check all input for potential architecture overrides.
        // We need this to do proper linting when doing our actual pass through the file.
        for (index, prop) in parsed.properties.iter().enumerate() {
            // We're only interested in architecture properties.
            let PackageProperty::MetaProperty(SharedMetaProperty::Architecture(architecture)) =
                prop
            else {
                continue;
            };

            // Calculate the actual line in the document based on any preceding lines.
            // We have to add one, as lines aren't 0 indexed.
            let line = index + line_start;

            // Make sure to set the value of the HashSet to
            let architectures = architectures.get_or_insert(HashSet::new());

            // Lint to make sure there aren't duplicate architectures declarations.
            if architectures.contains(architecture) {
                duplicate_architecture(errors, line, *architecture);
            }

            // Add the architecture in case it hasn't already.
            architectures.insert(*architecture);
            architecture_properties.entry(*architecture).or_default();
        }

        // If there's an overrides for architectures of this package, we need to use those
        // architectures for linting. If there isn't, we have to fall back to the PackageBase
        // architectures, which are then used instead.
        let architectures_for_lint = match &architectures {
            Some(architectures) => architectures,
            None => package_base_architectures,
        };

        // Save all ClearableProperties so that we may use them for linting lateron.
        let mut cleared_properties = Vec::new();

        // Next, check if there're any [ClearableProperty] overrides.
        // These indicate that a value or a vector should be overridden and set to None or an empty
        // vector, based on the property.
        for (index, prop) in parsed.properties.iter().enumerate() {
            // Calculate the actual line in the document based on any preceding lines.
            // We have to add one, as lines aren't 0 indexed.
            let line = index + line_start;

            // We're only interested in clearable properties.
            let PackageProperty::Clear(clearable_property) = prop else {
                continue;
            };

            cleared_properties.push(clearable_property.clone());

            match clearable_property {
                ClearableProperty::Description => description = Override::Clear,
                ClearableProperty::Url => url = Override::Clear,
                ClearableProperty::Licenses => licenses = Override::Clear,
                ClearableProperty::Changelog => changelog = Override::Clear,
                ClearableProperty::Install => install = Override::Clear,
                ClearableProperty::Groups => groups = Override::Clear,
                ClearableProperty::Options => options = Override::Clear,
                ClearableProperty::Backups => backups = Override::Clear,
                ClearableProperty::Dependencies(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    dependencies,
                ),
                ClearableProperty::OptionalDependencies(architecture) => {
                    clearable_arch_vec!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        architecture,
                        optional_dependencies,
                    )
                }
                ClearableProperty::Provides(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    provides,
                ),
                ClearableProperty::Conflicts(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    conflicts,
                ),
                ClearableProperty::Replaces(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    replaces,
                ),
            }
        }

        /// Mini helper macro that crates a filter closure to filter a specific SharedMetaProperty.
        /// Needed in the following ClearableProperty lint check.
        /// The function must be boxed as we mix this with closures from
        /// `relation_property_filter`.
        macro_rules! meta_property_filter {
            ($pattern:pat) => {
                Box::new(|(_, property): &(usize, &PackageProperty)| {
                    matches!(property, PackageProperty::MetaProperty($pattern))
                })
            };
        }

        /// Mini helper macro that crates a filter closure to filter a specific RelationProperty.
        /// Needed in the following ClearableProperty lint check.
        macro_rules! relation_property_filter {
            ($architecture:ident, $pattern:pat) => {{
                // Clone the cleared architecture so that it may be copied into the closure
                let cleared_architecture = $architecture.clone();
                Box::new(move |(_, property): &(usize, &PackageProperty)| {
                    // Make sure we have a relation
                    let PackageProperty::RelationProperty(relation) = property else {
                        return false;
                    };
                    // Make sure we match the pattern
                    if !matches!(relation, $pattern) {
                        return false;
                    }

                    // Check whether the architecture matches
                    cleared_architecture == relation.architecture()
                })
            }};
        }

        // Ensures that cleared properties don't get overwritten again in the same scope of a
        // package. E.g.
        // ```txt
        // depends =
        // depends = vim
        // ```
        for clearable in cleared_properties {
            #[allow(clippy::type_complexity)]
            // Return a filter closure/function that's used to search all properties for a certain
            // enum variant. In the case of architecture specific properties, the closure also
            // looks for properties that use the same architecture as the cleared property.
            //
            // This needs to be boxed as we're working with closures in the context of architecture
            // specific properties. They capture the cleared property's architecture for comparison.
            let filter: Box<dyn Fn(&(usize, &PackageProperty)) -> bool> = match clearable {
                ClearableProperty::Description => {
                    meta_property_filter!(SharedMetaProperty::Description(_))
                }
                ClearableProperty::Url => {
                    meta_property_filter!(SharedMetaProperty::Url(_))
                }
                ClearableProperty::Licenses => {
                    meta_property_filter!(SharedMetaProperty::License(_))
                }
                ClearableProperty::Changelog => {
                    meta_property_filter!(SharedMetaProperty::Changelog(_))
                }
                ClearableProperty::Install => {
                    meta_property_filter!(SharedMetaProperty::Install(_))
                }
                ClearableProperty::Groups => {
                    meta_property_filter!(SharedMetaProperty::Group(_))
                }
                ClearableProperty::Options => {
                    meta_property_filter!(SharedMetaProperty::Option(_))
                }
                ClearableProperty::Backups => {
                    meta_property_filter!(SharedMetaProperty::Backup(_))
                }
                ClearableProperty::Dependencies(architecture) => {
                    relation_property_filter!(architecture, RelationProperty::Dependency(_))
                }
                ClearableProperty::OptionalDependencies(architecture) => {
                    relation_property_filter!(architecture, RelationProperty::OptionalDependency(_))
                }
                ClearableProperty::Provides(architecture) => {
                    relation_property_filter!(architecture, RelationProperty::Provides(_))
                }
                ClearableProperty::Conflicts(architecture) => {
                    relation_property_filter!(architecture, RelationProperty::Conflicts(_))
                }
                ClearableProperty::Replaces(architecture) => {
                    relation_property_filter!(architecture, RelationProperty::Replaces(_))
                }
            };

            // Check if we found a declaration even though the field is also being cleared.
            let Some((index, _)) = parsed.properties.iter().enumerate().find(filter) else {
                continue;
            };

            // Calculate the actual line in the document based on any preceding lines.
            let line = index + line_start;

            // Create the lint error
            reassigned_cleared_property(errors, line);
        }

        // Set all of the package's properties.
        for (line, prop) in parsed.properties.into_iter().enumerate() {
            // Calculate the actual line in the document based on any preceding lines.
            let line = line + line_start;
            match prop {
                // Skip empty lines and comments
                PackageProperty::EmptyLine | PackageProperty::Comment(_) => continue,
                PackageProperty::MetaProperty(shared_meta_property) => {
                    match shared_meta_property {
                        SharedMetaProperty::Description(inner) => {
                            description = Override::Yes { value: inner }
                        }
                        SharedMetaProperty::Url(inner) => url = Override::Yes { value: inner },
                        SharedMetaProperty::License(inner) => {
                            // Create lints for non-spdx licenses.
                            if let License::Unknown(_) = &inner {
                                non_spdx_license(errors, line, inner.to_string());
                            }
                            licenses.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Changelog(inner) => {
                            changelog = Override::Yes { value: inner }
                        }
                        SharedMetaProperty::Install(inner) => {
                            install = Override::Yes { value: inner }
                        }
                        SharedMetaProperty::Group(inner) => {
                            groups.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Option(inner) => {
                            options.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Backup(inner) => {
                            backups.get_or_insert(Vec::new()).push(inner)
                        }
                        // We already handled these at the start of the function in a previous pass.
                        SharedMetaProperty::Architecture(_) => continue,
                    }
                }
                PackageProperty::RelationProperty(relation_property) => match relation_property {
                    RelationProperty::Dependency(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        dependencies,
                    ),
                    RelationProperty::OptionalDependency(arch_property) => {
                        package_arch_prop!(
                            line,
                            errors,
                            architectures_for_lint,
                            architecture_properties,
                            arch_property,
                            optional_dependencies,
                        )
                    }
                    RelationProperty::Provides(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        provides,
                    ),
                    RelationProperty::Conflicts(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        conflicts,
                    ),
                    RelationProperty::Replaces(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        replaces,
                    ),
                },
                // We already handled at the start in a separate pass.
                PackageProperty::Clear(_) => continue,
            }
        }

        Package {
            name: parsed.name,
            description,
            url,
            changelog,
            licenses,
            architectures,
            architecture_properties,
            install,
            groups,
            options,
            backups,
            dependencies,
            optional_dependencies,
            provides,
            conflicts,
            replaces,
        }
    }
}
