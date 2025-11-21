//! Lookup table to identify compatible versions and version requirements of package relations.

use std::collections::HashMap;

use log::trace;

use crate::{Name, PackageRelation, Version, VersionRequirement};

/// Data on where a specific [`VersionRequirement`] originates from.
///
/// Tracks an optional [`VersionRequirement`] and an optional [`Name`].
///
/// # Note
///
/// This struct is used in a [`RelationLookup`] to encode the optional version requirement of an
/// [alpm-package-relation] and its context (i.e. which specific package it belongs to in the case
/// of a _provision_).
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Debug)]
struct OriginRequirement {
    pub(crate) requirement: Option<VersionRequirement>,
    pub(crate) origin: Option<Name>,
}

/// A lookup table for [alpm-package-relation] data.
///
/// The lookup table tracks a _virtual component_ or package name (see [alpm-package-name]) and one
/// or more sets of information on what is the origin of the _virtual component_ or package name and
/// whether it imposes a version requirement.
///
/// A [`RelationLookup`] is used when checking whether a [`PackageRelation`] or a set of [`Name`]
/// and [`Version`] is compatible with one [alpm-package-relation] in a list of relations.
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Debug, Default)]
pub struct RelationLookup {
    package_relations: HashMap<Name, Vec<OriginRequirement>>,
}

impl RelationLookup {
    /// Inserts a [`PackageRelation`] and its optional origin.
    ///
    /// The `origin` tracks the name of the package that the `package_relation` originates from.
    pub fn insert_package_relation(
        &mut self,
        package_relation: PackageRelation,
        origin: Option<Name>,
    ) {
        if let Some(requirements) = self.package_relations.get_mut(&package_relation.name) {
            requirements.push(OriginRequirement {
                requirement: package_relation.version_requirement,
                origin,
            });
        } else {
            self.package_relations.insert(
                package_relation.name,
                vec![OriginRequirement {
                    requirement: package_relation.version_requirement,
                    origin,
                }],
            );
        }
    }

    /// Checks whether a relation in this [`RelationLookup`] can satisfy a [`PackageRelation`].
    ///
    /// # Note
    ///
    /// When comparing, the assumption is that if either `self` or `relation` do not specify a
    /// concrete [`VersionRequirement`], this means that any version can satisfy this requirement.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use alpm_types::{PackageRelation, RelationLookup};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // Lookup with a single entry per key.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation("name>=1".parse()?, Some("name".parse()?));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "name".parse()?,
    ///     version_requirement: Some("=1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "name".parse()?,
    ///     version_requirement: Some(">1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "name".parse()?,
    ///     version_requirement: Some(">=0.1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "name".parse()?,
    ///     version_requirement: None,
    /// }));
    /// assert!(!lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "name".parse()?,
    ///     version_requirement: Some("<1".parse()?),
    /// }));
    /// assert!(!lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "other-name".parse()?,
    ///     version_requirement: Some("=1".parse()?),
    /// }));
    ///
    /// // Lookup with multiple entries per key.
    /// // This may be the case, if e.g. multiple packages provide the same _virtual component_ (but in different versions).
    /// // Lookup with a single entry per key.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation("virtual-name>1".parse()?, Some("name".parse()?));
    /// lookup.insert_package_relation("virtual-name<=1".parse()?, Some("name".parse()?));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "virtual-name".parse()?,
    ///     version_requirement: Some("=1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "virtual-name".parse()?,
    ///     version_requirement: Some(">1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "virtual-name".parse()?,
    ///     version_requirement: Some(">=0.1".parse()?),
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "virtual-name".parse()?,
    ///     version_requirement: None,
    /// }));
    /// assert!(lookup.satisfies_package_relation(&PackageRelation {
    ///     name: "virtual-name".parse()?,
    ///     version_requirement: Some("<1".parse()?),
    /// }));
    /// # Ok(())
    /// # }
    /// ```
    pub fn satisfies_package_relation(&self, relation: &PackageRelation) -> bool {
        let Some(origin_requirements) = self.package_relations.get(&relation.name) else {
            trace!(
                "Found no matching relation lookup for relation {}",
                relation.name
            );
            return false;
        };

        origin_requirements.iter().any(|origin_requirement| {
            match (
                origin_requirement.requirement.as_ref(),
                relation.version_requirement.as_ref(),
            ) {
                (Some(requirement), Some(other)) => {
                    if other.is_satisfied_by_requirement(requirement) {
                        trace!(
                            "The requirement {} for {} is satisfied by the requirement {} for {}{}",
                            other,
                            relation.name,
                            requirement,
                            relation.name,
                            if let Some(origin) = origin_requirement.origin.as_ref() {
                                format!(" (provided by {origin})")
                            } else {
                                String::new()
                            }
                        );
                        true
                    } else {
                        false
                    }
                }
                _ => {
                    trace!(
                        "The{} relation {} is satisfied by the{} relation {}{}",
                        if let Some(requirement) = relation.version_requirement.as_ref() {
                            format!(" requirement {requirement} for")
                        } else {
                            String::new()
                        },
                        relation.name,
                        if let Some(requirement) = origin_requirement.requirement.as_ref() {
                            format!(" requirement {requirement} for")
                        } else {
                            String::new()
                        },
                        relation.name,
                        if let Some(origin) = origin_requirement.origin.as_ref() {
                            format!(" (provided by {origin})")
                        } else {
                            String::new()
                        }
                    );
                    true
                }
            }
        })
    }

    /// Checks whether a relation in this [`RelationLookup`] satisfies a [`Name`] and [`Version`].
    ///
    /// # Note
    ///
    /// When comparing, the assumption is that if `self` does not specify a concrete
    /// [`VersionRequirement`], this means that any `version` can satisfy this requirement.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use alpm_types::{RelationLookup};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // Lookup with a single entry per key.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation("name>=1".parse()?, Some("name".parse()?));
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"1".parse()?,));
    /// assert!(!lookup.satisfies_name_and_version(&"name".parse()?, &"0.1".parse()?,));
    ///
    /// // Lookup with a single entry per key and no version requirement.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation("name".parse()?, None);
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"1".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"0.1".parse()?,));
    ///
    /// // Lookup with multiple entries per key.
    /// // This may be the case, if e.g. multiple packages provide the same _virtual component_ (but in different versions).
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation("virtual-name>1".parse()?, Some("name".parse()?));
    /// lookup.insert_package_relation("virtual-name<=1".parse()?, Some("name1".parse()?));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"2".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"1".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"0.1".parse()?,));
    /// # Ok(())
    /// # }
    /// ```
    pub fn satisfies_name_and_version(&self, name: &Name, version: &Version) -> bool {
        let Some(origin_requirements) = self.package_relations.get(name) else {
            return false;
        };

        origin_requirements.iter().any(|origin_requirement| {
            if let Some(requirement) = origin_requirement.requirement.as_ref() {
                if requirement.is_satisfied_by_version(version) {
                    trace!(
                        "The version {version} of {name} is compatible with the requirement {requirement} for {name}{}",
                        if let Some(origin) = origin_requirement.origin.as_ref() {
                            format!(" (provided by {origin})")
                        } else {
                            String::new()
                        }
                    );
                    true
                } else {
                    false
                }
            } else {
                trace!(
                    "The version {version} of {name} is compatible with {name}{}",
                    if let Some(origin) = origin_requirement.origin.as_ref() {
                        format!(" (provided by {origin})")
                    } else {
                        String::new()
                    }
                );
                true
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use rstest::rstest;
    use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
    use testresult::TestResult;

    use super::*;

    const NAME: &str = "virtual-name";
    const ORIGIN: &str = "package-name";

    /// Initialize a logger that shows trace messages on stderr.
    fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        )
        .is_err()
        {
            eprintln!("Not initializing another logger, as one is initialized already.");
        }
    }

    fn relation_lookup_none() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(NAME.parse()?, None);

        Ok(lookup)
    }

    fn relation_lookup_less() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(format!("{NAME}<1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_less_or_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(format!("{NAME}<=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(format!("{NAME}=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_greater_or_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(format!("{NAME}>=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_greater() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(format!("{NAME}>1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    #[rstest]
    #[case::lookup_none_relation_none(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_none_relation_less(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_none_relation_less_or_equal(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_equal(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_greater_or_equal(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_greater(
        relation_lookup_none(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_less_relation_none(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_less_relation_less(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_less_relation_less_or_equal(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_none(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_less_or_equal_relation_less(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_less_or_equal(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_equal(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_greater_or_equal(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_none(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_equal_relation_less_or_equal(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_equal(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_greater_or_equal(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_none(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_greater_or_equal_relation_less_or_equal(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_equal(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_greater_or_equal(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_greater(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_none(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_greater_relation_greater_or_equal(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_greater(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    fn relation_lookup_satisfies_package_relation_true(
        #[case] lookup: TestResult<RelationLookup>,
        #[case] relation: PackageRelation,
    ) -> TestResult {
        init_logger();

        assert!(lookup?.satisfies_package_relation(&relation));

        Ok(())
    }

    #[rstest]
    #[case::lookup_less_relation_equal(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_less_relation_greater_or_equal(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_less_relation_greater(
        relation_lookup_less(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_less_or_equal_large(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=2".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_greater(
        relation_lookup_less_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_less(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_greater(
        relation_lookup_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_less(
        relation_lookup_greater_or_equal(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_less(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_less_or_equal(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_equal(
        relation_lookup_greater(),
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_none_mismatching_relation_name(
        relation_lookup_none(),
        PackageRelation {
            name: "something-else".parse()?,
            version_requirement: None,
        }
    )]
    fn relation_lookup_satisfies_package_relation_false(
        #[case] lookup: TestResult<RelationLookup>,
        #[case] relation: PackageRelation,
    ) -> TestResult {
        init_logger();

        assert!(!lookup?.satisfies_package_relation(&relation));

        Ok(())
    }

    #[rstest]
    #[case::lookup_none_version_less(relation_lookup_none(), "0.1".parse()?)]
    #[case::lookup_none_version_equal(relation_lookup_none(), "1".parse()?)]
    #[case::lookup_none_version_greater(relation_lookup_none(), "2".parse()?)]
    #[case::lookup_less_version_less(relation_lookup_less(), "0.1".parse()?)]
    #[case::lookup_less_or_equal_version_less(relation_lookup_less_or_equal(), "0.1".parse()?)]
    #[case::lookup_less_or_equal_version_equal(relation_lookup_less_or_equal(), "1".parse()?)]
    #[case::lookup_equal_version_equal(relation_lookup_equal(), "1".parse()?)]
    #[case::lookup_greater_or_equal_version_equal(relation_lookup_greater_or_equal(), "1".parse()?)]
    #[case::lookup_greater_or_equal_version_greater(relation_lookup_greater_or_equal(), "2".parse()?)]
    #[case::lookup_greater_version_greater(relation_lookup_greater(), "2".parse()?)]
    fn relation_lookup_satisfies_name_and_version_true(
        #[case] lookup: TestResult<RelationLookup>,
        #[case] version: Version,
    ) -> TestResult {
        init_logger();

        assert!(lookup?.satisfies_name_and_version(&NAME.parse()?, &version));

        Ok(())
    }

    #[rstest]
    #[case::lookup_less_version_greater(
        relation_lookup_less(),
        NAME.parse()?,
        "1".parse()?,
    )]
    #[case::lookup_less_or_equal_version_greater(
        relation_lookup_less_or_equal(),
        NAME.parse()?,
        "2".parse()?,
    )]
    #[case::lookup_equal_version_less(
        relation_lookup_equal(),
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_equal_version_greater(
        relation_lookup_equal(),
        NAME.parse()?,
        "2".parse()?,
    )]
    #[case::lookup_greater_or_equal_version_less(
        relation_lookup_greater_or_equal(),
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_greater_version_less(
        relation_lookup_greater(),
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_greater_version_equal(
        relation_lookup_greater(),
        NAME.parse()?,
        "1".parse()?,
    )]
    #[case::lookup_none_mismatching_name(
        relation_lookup_none(),
        "other-name".parse()?,
        "1".parse()?,
    )]
    fn relation_lookup_satisfies_name_and_version_false(
        #[case] lookup: TestResult<RelationLookup>,
        #[case] name: Name,
        #[case] version: Version,
    ) -> TestResult {
        init_logger();

        assert!(!lookup?.satisfies_name_and_version(&name, &version));

        Ok(())
    }
}
