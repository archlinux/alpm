//! Handling of metadata found in a `pkgname` section of SRCINFO data.
use std::collections::BTreeMap;

use alpm_types::{
    Architecture,
    Architectures,
    Backup,
    Changelog,
    Install,
    License,
    MakepkgOption,
    Name,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    RelationOrSoname,
    SystemArchitecture,
    Url,
};
use serde::{Deserialize, Serialize};

use crate::{
    Error,
    source_info::parser::{
        ClearableProperty,
        PackageProperty,
        RawPackage,
        RelationProperty,
        SharedMetaProperty,
    },
};
#[cfg(doc)]
use crate::{MergedPackage, SourceInfoV1, source_info::v1::package_base::PackageBase};

/// A [`Package`] property that can override its respective defaults in [`PackageBase`].
///
/// This type is similar to [`Option`], which has special serialization behavior.
/// However, in some file formats (e.g. JSON) it is not possible to represent data such as
/// `Option<Option<T>>`, as serialization would flatten the structure. This type enables
/// representation of this type of data.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(tag = "override")]
pub enum Override<T> {
    /// The property is not overridden.
    #[default]
    No,
    /// The property is cleared.
    Clear,
    /// The property is overridden.
    Yes {
        /// The value with which the property is overridden.
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Package {
    /// The alpm-package-name of the package.
    pub name: Name,
    /// The (potentially overridden) description of the package.
    pub description: Override<PackageDescription>,
    /// The (potentially overridden) upstream URL of the package.
    pub url: Override<Url>,
    /// The (potentially overridden) relative path to a changelog file of the package.
    pub changelog: Override<Changelog>,
    /// The (potentially overridden) list of licenses that apply to the package.
    pub licenses: Override<Vec<License>>,

    // Build or package management related meta fields
    /// The (potentially overridden) relative path to an alpm-install-scriptlet of the package.
    pub install: Override<Install>,
    /// The (potentially overridden) list of alpm-package-groups the package is part of.
    pub groups: Override<Vec<String>>,
    /// The (potentially overridden) list of build tool options used when building the package.
    pub options: Override<Vec<MakepkgOption>>,
    /// The (potentially overridden) list of relative paths to files in the package that should be
    /// backed up.
    pub backups: Override<Vec<Backup>>,

    /// The architectures that are supported by this package.
    // Despite being overridable, `architectures` field isn't of the `Override` type, as it
    // **cannot** be cleared.
    pub architectures: Option<Architectures>,
    /// The map of alpm-architecture specific overrides for package relations of a package.
    pub architecture_properties: BTreeMap<SystemArchitecture, PackageArchitecture>,

    /// The (potentially overridden) list of run-time dependencies of the package.
    pub dependencies: Override<Vec<RelationOrSoname>>,
    /// The (potentially overridden) list of optional dependencies of the package.
    pub optional_dependencies: Override<Vec<OptionalDependency>>,
    /// The (potentially overridden) list of provisions of the package.
    pub provides: Override<Vec<RelationOrSoname>>,
    /// The (potentially overridden) list of conflicts of the package.
    pub conflicts: Override<Vec<PackageRelation>>,
    /// The (potentially overridden) list of replacements of the package.
    pub replaces: Override<Vec<PackageRelation>>,
}

impl From<Name> for Package {
    /// Creates a new [`Package`] from a [`Name`] by calling [`Package::new_with_defaults`].
    fn from(value: Name) -> Self {
        Package::new_with_defaults(value)
    }
}

/// Architecture specific package properties for use in [`Package`].
///
/// For each [`Architecture`] defined in [`Package::architectures`] a [`PackageArchitecture`] is
/// present in [`Package::architecture_properties`].
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageArchitecture {
    /// The (potentially overridden) list of run-time dependencies of the package.
    pub dependencies: Override<Vec<RelationOrSoname>>,
    /// The (potentially overridden) list of optional dependencies of the package.
    pub optional_dependencies: Override<Vec<OptionalDependency>>,
    /// The (potentially overridden) list of provisions of the package.
    pub provides: Override<Vec<RelationOrSoname>>,
    /// The (potentially overridden) list of conflicts of the package.
    pub conflicts: Override<Vec<PackageRelation>>,
    /// The (potentially overridden) list of replacements of the package.
    pub replaces: Override<Vec<PackageRelation>>,
}

/// Handles all potentially architecture specific, clearable entries in the [`Package::from_parsed`]
/// function.
///
/// If no architecture is encountered, it simply clears the value on the [`Package`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
macro_rules! clearable_arch_vec {
    (
        $architecture_properties:ident,
        $architecture:ident,
        $field_name:ident,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparations
        if let Some(architecture) = $architecture
            && let Architecture::Some(system_arch) = architecture
        {
            let properties = $architecture_properties
                .entry(system_arch.clone())
                .or_default();
            properties.$field_name = Override::Clear;
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
macro_rules! package_arch_prop {
    (
        $architecture_properties:ident,
        $arch_property:ident,
        $field_name:ident,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparations
        if let Some(architecture) = $arch_property.architecture
            && let Architecture::Some(system_arch) = architecture
        {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $architecture_properties
                .entry(system_arch)
                .or_insert(PackageArchitecture::default());

            // Set the architecture specific value.
            architecture_properties
                .$field_name
                .get_or_insert(Vec::new())
                .push($arch_property.value);
        } else {
            $field_name
                .get_or_insert(Vec::new())
                .push($arch_property.value)
        }
    };
}

impl Package {
    /// Creates a new [`Package`] from a [`Name`].
    ///
    /// Uses `name` and initializes all remaining fields of [`Package`] with default values.
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use alpm_srcinfo::source_info::v1::package::Package;
    /// use alpm_types::Name;
    ///
    /// # fn main() -> testresult::TestResult {
    ///
    /// let package = Package::new_with_defaults(Name::from_str("example_package")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_defaults(value: Name) -> Self {
        Package {
            name: value,
            description: Default::default(),
            url: Default::default(),
            changelog: Default::default(),
            licenses: Default::default(),
            install: Default::default(),
            groups: Default::default(),
            options: Default::default(),
            backups: Default::default(),
            architectures: Default::default(),
            architecture_properties: Default::default(),
            dependencies: Default::default(),
            optional_dependencies: Default::default(),
            provides: Default::default(),
            conflicts: Default::default(),
            replaces: Default::default(),
        }
    }

    /// Creates a new [`Package`] instance from a [`RawPackage`].
    ///
    /// # Parameters
    ///
    /// - `parsed`: The [`RawPackage`] representation of the SRCINFO data. The input guarantees that
    ///   the keyword assignments have been parsed correctly, but not yet that they represent valid
    ///   SRCINFO data as a whole.
    pub fn from_parsed(parsed: RawPackage) -> Result<Self, Error> {
        let mut description = Override::No;
        let mut url = Override::No;
        let mut licenses = Override::No;
        let mut changelog = Override::No;
        let mut architectures = None;
        let mut architecture_properties: BTreeMap<SystemArchitecture, PackageArchitecture> =
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
        for prop in parsed.properties.iter() {
            // We're only interested in architecture properties.
            let PackageProperty::MetaProperty(SharedMetaProperty::Architecture(architecture)) =
                prop
            else {
                continue;
            };
            let architectures = architectures.get_or_insert(Vec::new());
            architectures.push(architecture);
        }

        let architectures = if let Some(arch_vec) = architectures {
            // Try to convert the list of architectures into an `Architectures` instance.
            // This will fail if "any" is combined with any specific system architecture.
            let architectures: Architectures = arch_vec.try_into()?;
            Some(architectures)
        } else {
            None
        };

        // Next, check if there are any ClearableProperty overrides.
        // These indicate that a value or a vector should be overridden and set to None or an empty
        // vector, based on the property.
        for prop in parsed.properties.iter() {
            // We're only interested in clearable properties.
            let PackageProperty::Clear(clearable_property) = prop else {
                continue;
            };

            match clearable_property {
                ClearableProperty::Description => description = Override::Clear,
                ClearableProperty::Url => url = Override::Clear,
                ClearableProperty::Licenses => licenses = Override::Clear,
                ClearableProperty::Changelog => changelog = Override::Clear,
                ClearableProperty::Install => install = Override::Clear,
                ClearableProperty::Groups => groups = Override::Clear,
                ClearableProperty::Options => options = Override::Clear,
                ClearableProperty::Backups => backups = Override::Clear,
                ClearableProperty::Dependencies(architecture) => {
                    clearable_arch_vec!(architecture_properties, architecture, dependencies,)
                }
                ClearableProperty::OptionalDependencies(architecture) => {
                    clearable_arch_vec!(
                        architecture_properties,
                        architecture,
                        optional_dependencies,
                    )
                }
                ClearableProperty::Provides(architecture) => {
                    clearable_arch_vec!(architecture_properties, architecture, provides,)
                }
                ClearableProperty::Conflicts(architecture) => {
                    clearable_arch_vec!(architecture_properties, architecture, conflicts,)
                }
                ClearableProperty::Replaces(architecture) => {
                    clearable_arch_vec!(architecture_properties, architecture, replaces,)
                }
            }
        }

        // Set all of the package's properties.
        for prop in parsed.properties.into_iter() {
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
                    RelationProperty::Dependency(arch_property) => {
                        package_arch_prop!(architecture_properties, arch_property, dependencies,)
                    }
                    RelationProperty::OptionalDependency(arch_property) => {
                        package_arch_prop!(
                            architecture_properties,
                            arch_property,
                            optional_dependencies,
                        )
                    }
                    RelationProperty::Provides(arch_property) => {
                        package_arch_prop!(architecture_properties, arch_property, provides,)
                    }
                    RelationProperty::Conflicts(arch_property) => {
                        package_arch_prop!(architecture_properties, arch_property, conflicts,)
                    }
                    RelationProperty::Replaces(arch_property) => {
                        package_arch_prop!(architecture_properties, arch_property, replaces,)
                    }
                },
                // We already handled at the start in a separate pass.
                PackageProperty::Clear(_) => continue,
            }
        }

        Ok(Package {
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
        })
    }
}
