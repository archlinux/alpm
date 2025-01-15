use lints::{duplicate_architecture, missing_architecture_for_property, non_spdx_license};
use parser::PackageProperty;

use super::*;

/// A "raw" and not-yet merged representation of a package (`pkgname`) section in a SRCINFO file.
///
/// All values and nested structs inside this struct, except the `name`, are either nested
/// [Option]s, e.g. `Option<Option<String>>` or optional collections, e.g. `Option<Vec>`.
/// This is due to the fact that all fields are overrides for the defaults set by the
/// [PackageBase] struct.
/// - If a value is `None`, this indicates that the [PackageBase]'s value should be used.
/// - If a value is `Some<None>`, this means that the value should be empty and the [PackageBase]
///   should be ignored. The same goes for collections in the sense of `Some(Vec::new())`.
/// - If a value is `Some(Some(value))` or `Some(vec![values])`, these values should then be used.
///
/// This struct merely contains the overrides that should be applied on top of the
/// [PackageBase] to get the final definition of this package.
#[derive(Debug)]
pub struct Package {
    pub name: Name,
    pub description: Option<Option<PackageDescription>>,
    pub url: Option<Option<Url>>,
    pub changelog: Option<Option<RelativePath>>,
    pub licenses: Option<Vec<License>>,

    // Build or package management related meta fields
    pub install: Option<Option<RelativePath>>,
    pub groups: Option<Vec<String>>,
    pub options: Option<Vec<MakepkgOption>>,
    pub backups: Option<Vec<RelativePath>>,

    /// These are all override fields that may be architecture specific.
    pub architectures: Option<HashSet<Architecture>>,
    pub architecture_properties: HashMap<Architecture, PackageArchitecture>,

    pub dependencies: Option<Vec<PackageRelation>>,
    pub optional_dependencies: Option<Vec<OptionalDependency>>,
    pub provides: Option<Vec<PackageRelation>>,
    pub conflicts: Option<Vec<PackageRelation>>,
    pub replaces: Option<Vec<PackageRelation>>,
}

/// Represents architecture specific fields for a specific architecture on a [Package] struct.
///
/// Multiple of these may co-exist on a [Package::architectures] field, one for each
/// architecture.
#[derive(Default, Debug, Clone)]
pub struct PackageArchitecture {
    pub dependencies: Option<Vec<PackageRelation>>,
    pub optional_dependencies: Option<Vec<OptionalDependency>>,
    pub provides: Option<Vec<PackageRelation>>,
    pub conflicts: Option<Vec<PackageRelation>>,
    pub replaces: Option<Vec<PackageRelation>>,
}

/// Handle all potentially architecture specific clearable entries in the [Package::from_parsed]
/// function.
///
/// If no architecture is encountered, it simply clears the value on the [Package] itself.
/// Otherwise, it's added to the respective [PackageBase::architecture_properties].
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [PackageBase::architectures] or [Package::architectures] if overridden.
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
        // If so, we have to do perform some checks and preparation
        if let Some(architecture) = $architecture {
            let properties = $architecture_properties.entry(*architecture).or_default();
            properties.dependencies = Some(Vec::new());

            // Throw an error for all architecture specific properties that don't have
            // an explicit `arch` statement. This is considered bad style.
            // Also handle the special `Any` [Architecture], which allows all architectures.
            if !$lint_architectures.contains(&architecture)
                && !$lint_architectures.contains(&Architecture::Any)
            {
                missing_architecture_for_property($errors, $line, *architecture);
            }
        } else {
            $field_name = Some(Vec::new());
        }
    };
}

/// Handle all potentially architecture specific Vector entries in the [Package::from_parsed]
/// function.
///
/// If no architecture is encountered, it simply adds the value on the [Package] itself.
/// Otherwise, it clears the value on the respective [Package::architecture_properties] entry.
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [PackageBase::architectures] or [Package::architectures] if overridden.
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
        // If so, we have to do perform some checks and preparation
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
    /// Take the parsed `pkgname` content and convert it into a properly typed structural
    /// representation.
    ///
    /// # Errors
    ///
    /// This function doesn't throw any errors, instead it aggregates all lints, warnings and errors
    /// into the `errors` array. The idea is to keep this recoverable so that all errors/lints can
    /// be returned in a single go.
    ///
    /// # Parameters
    ///
    /// - `line_start`: The number of preceding lines. Needed so that our error/lint messages can
    ///   reference the correct lines.
    /// - `parsed`: The already typed parsed entries from the SRCINFO file. At this point, we know
    ///   that the types are correct, but the SRCINFO file in its whole might still be borked.
    /// - `errors`: All errors (including lints) will be written into this to be passed back.
    pub fn from_parsed(
        line_start: usize,
        package_base_architectures: &HashSet<Architecture>,
        parsed: parser::RawPackage,
        errors: &mut Vec<SourceInfoError>,
    ) -> Self {
        let mut description = None;
        let mut url = None;
        let mut licenses = None;
        let mut changelog = None;
        let mut architectures = None;
        let mut architecture_properties: HashMap<Architecture, PackageArchitecture> =
            HashMap::new();

        // Build or package management related meta fields
        let mut install = None;
        let mut groups = None;
        let mut options = None;
        let mut backups = None;

        let mut dependencies = None;
        let mut optional_dependencies = None;
        let mut provides = None;
        let mut conflicts = None;
        let mut replaces = None;

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
        // architectures, which're then used instead.
        let architectures_for_lint = if let Some(architectures) = &architectures {
            architectures
        } else {
            package_base_architectures
        };

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

            match clearable_property {
                parser::ClearableProperty::Description => description = None,
                parser::ClearableProperty::Url => url = None,
                parser::ClearableProperty::Licenses => licenses = Some(Vec::new()),
                parser::ClearableProperty::Changelog => changelog = None,
                parser::ClearableProperty::Install => install = None,
                parser::ClearableProperty::Groups => groups = Some(Vec::new()),
                parser::ClearableProperty::Options => options = Some(Vec::new()),
                parser::ClearableProperty::Backups => backups = Some(Vec::new()),
                parser::ClearableProperty::Dependencies(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    dependencies,
                ),
                parser::ClearableProperty::OptionalDependencies(architecture) => {
                    clearable_arch_vec!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        architecture,
                        dependencies,
                    )
                }
                parser::ClearableProperty::Provides(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    provides,
                ),
                parser::ClearableProperty::Conflicts(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    conflicts,
                ),
                parser::ClearableProperty::Replaces(architecture) => clearable_arch_vec!(
                    line,
                    errors,
                    architectures_for_lint,
                    architecture_properties,
                    architecture,
                    replaces,
                ),
            }
        }

        // TODO: Idea: Should we add a lint that ensures that cleared property doesn't get
        // overwritten again in the same scope of a package?
        //
        // E.g.
        // ```
        // depends = depends = vim
        // ```

        for (line, prop) in parsed.properties.into_iter().enumerate() {
            // Calculate the actual line in the document based on any preceding lines.
            let line = line + line_start;
            match prop {
                // Skip empty lines and comments
                PackageProperty::EmptyLine | PackageProperty::Comment(_) => continue,
                PackageProperty::MetaProperty(shared_meta_property) => {
                    match shared_meta_property {
                        SharedMetaProperty::Description(inner) => description = Some(Some(inner)),
                        SharedMetaProperty::Url(inner) => url = Some(Some(inner)),
                        SharedMetaProperty::License(inner) => {
                            // Create lints for non-spdx licenses.
                            if let License::Unknown(_) = &inner {
                                non_spdx_license(errors, line, inner.to_string());
                            }
                            licenses.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Changelog(inner) => changelog = Some(Some(inner)),
                        SharedMetaProperty::Install(inner) => install = Some(Some(inner)),
                        SharedMetaProperty::Group(inner) => {
                            groups.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Option(inner) => {
                            options.get_or_insert(Vec::new()).push(inner)
                        }
                        SharedMetaProperty::Backup(inner) => {
                            backups.get_or_insert(Vec::new()).push(inner)
                        }
                        // We already handled at the start in a separate pass.
                        SharedMetaProperty::Architecture(_) => continue,
                    }
                }
                PackageProperty::RelationProperty(relation_property) => match relation_property {
                    parser::RelationProperty::Dependency(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        dependencies,
                    ),
                    parser::RelationProperty::OptionalDependency(arch_property) => {
                        package_arch_prop!(
                            line,
                            errors,
                            architectures_for_lint,
                            architecture_properties,
                            arch_property,
                            optional_dependencies,
                        )
                    }
                    parser::RelationProperty::Provides(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        provides,
                    ),
                    parser::RelationProperty::Conflicts(arch_property) => package_arch_prop!(
                        line,
                        errors,
                        architectures_for_lint,
                        architecture_properties,
                        arch_property,
                        conflicts,
                    ),
                    parser::RelationProperty::Replaces(arch_property) => package_arch_prop!(
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
