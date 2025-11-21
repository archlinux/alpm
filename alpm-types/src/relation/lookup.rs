//! Lookup table to identify compatible versions and version requirements of package relations.

use std::collections::HashMap;

use log::{debug, trace};

use crate::{
    Name,
    PackageRelation,
    PackageVersion,
    RelationOrSoname,
    SharedLibraryPrefix,
    SharedObjectName,
    SonameV1,
    SonameV2,
    Version,
    VersionRequirement,
};

/// Data on a [`VersionRequirement`] and where it originates from.
///
/// # Note
///
/// This struct is used in a [`RelationLookup`] to encode the optional version requirement of an
/// [alpm-package-relation] and its context (i.e. which specific package it belongs to in the case
/// of a _provision_).
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Debug)]
struct VersionRequirementOrigin {
    pub(crate) requirement: Option<VersionRequirement>,
    pub(crate) origin: Option<Name>,
}

/// Data on a [`SonameV1`] and where it originates from.
///
/// # Note
///
/// This struct is used in a [`RelationLookup`] to encode the optional version requirement of an
/// [alpm-sonamev1] used as [alpm-package-relation], as well as its context (i.e. which specific
/// package it belongs to in the case of a _provision_).
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [alpm-sonamev1]: https://alpm.archlinux.page/specifications/alpm-sonamev1.7.html
#[derive(Debug)]
struct SonameV1Origin {
    pub(crate) soname: SonameV1,
    pub(crate) origin: Option<Name>,
}

/// Data on a [`SonameV2`] and where it originates from.
///
/// Tracks the optional version of an [alpm-sonamev2] and its [`SharedLibraryPrefix`].
///
/// # Note
///
/// This struct is used in a [`RelationLookup`] to encode the optional version requirement and
/// prefix of an [alpm-sonamev2] used as [alpm-package-relation], as well as its context (i.e. which
/// specific package it belongs to in the case of a _provision_).
///
/// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
/// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
#[derive(Debug)]
struct SonameV2Origin {
    pub(crate) version: Option<PackageVersion>,
    pub(crate) prefix: SharedLibraryPrefix,
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
    package_relations: HashMap<Name, Vec<VersionRequirementOrigin>>,
    sonamev1s: HashMap<SharedObjectName, Vec<SonameV1Origin>>,
    sonamev2s: HashMap<SharedObjectName, Vec<SonameV2Origin>>,
}

impl RelationLookup {
    /// Returns the amount of all items tracked by this [`RelationLookup`].
    pub fn len(&self) -> usize {
        self.package_relations.len() + self.sonamev1s.len() + self.sonamev2s.len()
    }

    /// Returns whether there are no items tracked by this [`RelationLookup`].
    pub fn is_empty(&self) -> bool {
        self.package_relations.is_empty() && self.sonamev1s.is_empty() && self.sonamev2s.is_empty()
    }

    /// Inserts a [`PackageRelation`] and its optional origin.
    ///
    /// The `origin` tracks the name of the package that the `package_relation` originates from.
    pub fn insert_package_relation(
        &mut self,
        package_relation: &PackageRelation,
        origin: Option<Name>,
    ) {
        if let Some(requirements) = self.package_relations.get_mut(&package_relation.name) {
            requirements.push(VersionRequirementOrigin {
                requirement: package_relation.version_requirement.clone(),
                origin,
            });
        } else {
            self.package_relations.insert(
                package_relation.name.clone(),
                vec![VersionRequirementOrigin {
                    requirement: package_relation.version_requirement.clone(),
                    origin,
                }],
            );
        }
    }

    /// Inserts a [`SonameV1`] and its optional origin.
    ///
    /// The `origin` tracks the name of the package that `sonamev1` originates from.
    pub fn insert_sonamev1(&mut self, sonamev1: &SonameV1, origin: Option<Name>) {
        let name = sonamev1.shared_object_name();

        if let Some(requirements) = self.sonamev1s.get_mut(name) {
            requirements.push(SonameV1Origin {
                soname: sonamev1.clone(),
                origin,
            });
        } else {
            self.sonamev1s.insert(
                name.clone(),
                vec![SonameV1Origin {
                    soname: sonamev1.clone(),
                    origin,
                }],
            );
        }
    }

    /// Inserts a [`SonameV2`] and its optional origin.
    ///
    /// The `origin` tracks the name of the package that `sonamev2` originates from.
    pub fn insert_sonamev2(&mut self, sonamev2: &SonameV2, origin: Option<Name>) {
        let name = &sonamev2.soname.name;
        let prefix = &sonamev2.prefix;
        let version = &sonamev2.soname.version;

        if let Some(requirements) = self.sonamev2s.get_mut(name) {
            requirements.push(SonameV2Origin {
                version: version.clone(),
                prefix: prefix.clone(),
                origin,
            });
        } else {
            self.sonamev2s.insert(
                name.clone(),
                vec![SonameV2Origin {
                    version: version.clone(),
                    prefix: prefix.clone(),
                    origin,
                }],
            );
        }
    }

    /// Inserts a [`RelationOrSoname`] and its optional origin.
    ///
    /// The `origin` tracks the name of the package that `relation` originates from.
    /// This is a convenience wrapper, which delegates to
    /// [`RelationLookup::insert_package_relation`], [`RelationLookup::insert_sonamev1`], or
    /// [`RelationLookup::insert_sonamev2`] as needed.
    pub fn insert_relation_or_soname(&mut self, relation: &RelationOrSoname, origin: Option<Name>) {
        match relation {
            RelationOrSoname::Relation(package_relation) => {
                self.insert_package_relation(package_relation, origin)
            }
            RelationOrSoname::SonameV1(sonamev1) => self.insert_sonamev1(sonamev1, origin),
            RelationOrSoname::SonameV2(sonamev2) => self.insert_sonamev2(sonamev2, origin),
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
    /// lookup.insert_package_relation(&"name>=1".parse()?, Some("name".parse()?));
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
    /// lookup.insert_package_relation(&"virtual-name>1".parse()?, Some("name".parse()?));
    /// lookup.insert_package_relation(&"virtual-name<=1".parse()?, Some("name".parse()?));
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
                    if other.is_intersection(requirement) {
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
    /// lookup.insert_package_relation(&"name>=1".parse()?, Some("name".parse()?));
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"1".parse()?,));
    /// assert!(!lookup.satisfies_name_and_version(&"name".parse()?, &"0.1".parse()?,));
    ///
    /// // Lookup with a single entry per key and no version requirement.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation(&"name".parse()?, None);
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"1".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"name".parse()?, &"0.1".parse()?,));
    ///
    /// // Lookup with multiple entries per key.
    /// // This may be the case, if e.g. multiple packages provide the same _virtual component_ (but in different versions).
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_package_relation(&"virtual-name>1".parse()?, Some("name".parse()?));
    /// lookup.insert_package_relation(&"virtual-name<=1".parse()?, Some("name1".parse()?));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"2".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"1".parse()?,));
    /// assert!(lookup.satisfies_name_and_version(&"virtual-name".parse()?, &"0.1".parse()?,));
    /// # Ok(())
    /// # }
    /// ```
    pub fn satisfies_name_and_version(&self, name: &Name, version: &Version) -> bool {
        let Some(origin_requirements) = self.package_relations.get(name) else {
            trace!("No matching package relation found for {name} and {version}");
            return false;
        };

        origin_requirements.iter().any(|origin_requirement| {
            if let Some(requirement) = origin_requirement.requirement.as_ref() {
                if requirement.is_satisfied_by(version) {
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

    /// Checks whether a [`SonameV1`] in this [`RelationLookup`] satisfies another [`SonameV1`].
    ///
    /// # Note
    ///
    /// One [`SonameV1`] satisfies another, if it can be matched fully.
    /// Due to the complexity of the type and to be able to check for equality, full copies are
    /// retained in a [`RelationLookup`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use alpm_types::{ElfArchitectureFormat, RelationLookup, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // A "basic" SonameV1 variant.
    /// let basic = SonameV1::new("example.so".parse()?, None, None)?;
    /// // An "unversioned" SonameV1 variant.
    /// let unversioned = SonameV1::new(
    ///     "example.so".parse()?,
    ///     Some("example.so".parse()?),
    ///     Some(ElfArchitectureFormat::Bit64),
    /// )?;
    /// // An "explicit" SonameV1 variant.
    /// let explicit = SonameV1::new(
    ///     "example.so".parse()?,
    ///     Some("1.0.0".parse()?),
    ///     Some(ElfArchitectureFormat::Bit64),
    /// )?;
    ///
    /// // Lookup with a "basic" SonameV1 variant.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_sonamev1(&basic.clone(), Some("name".parse()?));
    /// assert!(lookup.satisfies_sonamev1(&basic));
    /// assert!(!lookup.satisfies_sonamev1(&unversioned));
    /// assert!(!lookup.satisfies_sonamev1(&explicit));
    ///
    /// // Lookup with an "unversioned" SonameV1 variant.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_sonamev1(&unversioned.clone(), Some("name".parse()?));
    /// assert!(lookup.satisfies_sonamev1(&unversioned));
    /// assert!(!lookup.satisfies_sonamev1(&basic));
    /// assert!(!lookup.satisfies_sonamev1(&explicit));
    ///
    /// // Lookup with an "explicit" SonameV1 variant.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_sonamev1(&explicit.clone(), Some("name".parse()?));
    /// assert!(lookup.satisfies_sonamev1(&explicit));
    /// assert!(!lookup.satisfies_sonamev1(&basic));
    /// assert!(!lookup.satisfies_sonamev1(&unversioned));
    /// # Ok(())
    /// # }
    /// ```
    pub fn satisfies_sonamev1(&self, sonamev1: &SonameV1) -> bool {
        let Some(soname_origins) = self.sonamev1s.get(sonamev1.shared_object_name()) else {
            trace!("No matching alpm-sonamev1 found for {sonamev1}");
            return false;
        };

        soname_origins.iter().any(|soname_origin| {
            if &soname_origin.soname == sonamev1 {
                debug!(
                    "The alpm-sonamev1 {sonamev1} is compatible with the alpm-sonamev1 {}{}",
                    soname_origin.soname,
                    if let Some(origin) = soname_origin.origin.as_ref() {
                        format!(" (provided by {origin})")
                    } else {
                        String::new()
                    }
                );
                true
            } else {
                trace!(
                    "The alpm-sonamev1 {sonamev1} is NOT compatible with the alpm-sonamev1 {}{}",
                    soname_origin.soname,
                    if let Some(origin) = soname_origin.origin.as_ref() {
                        format!(" (provided by {origin})")
                    } else {
                        String::new()
                    }
                );
                false
            }
        })
    }

    /// Checks whether a [`SonameV2`] in this [`RelationLookup`] satisfies another [`SonameV2`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use alpm_types::{ElfArchitectureFormat, RelationLookup, SonameV2};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// // A SonameV2.
    /// let default = SonameV2::new("lib".parse()?, "libexample.so.1".parse()?);
    /// let lib32 = SonameV2::new("lib32".parse()?, "libexample.so.1".parse()?);
    ///
    /// // Lookup with a SonameV2.
    /// let mut lookup = RelationLookup::default();
    /// lookup.insert_sonamev2(&default.clone(), Some("name".parse()?));
    /// assert!(lookup.satisfies_sonamev2(&default));
    /// assert!(!lookup.satisfies_sonamev2(&lib32));
    ///
    /// lookup.insert_sonamev2(&lib32.clone(), Some("lib32-name".parse()?));
    /// assert!(lookup.satisfies_sonamev2(&lib32));
    /// # Ok(())
    /// # }
    /// ```
    pub fn satisfies_sonamev2(&self, sonamev2: &SonameV2) -> bool {
        let name = &sonamev2.soname.name;
        let Some(soname_origins) = self.sonamev2s.get(name) else {
            trace!("No matching alpm-sonamev1 found for {sonamev2}");
            return false;
        };

        soname_origins.iter().any(|soname_origin| {
            if soname_origin.version == sonamev2.soname.version
                && soname_origin.prefix == sonamev2.prefix
            {
                debug!(
                    "The alpm-sonamev2 {sonamev2} is compatible with the alpm-sonamev2 {}:{name}{}{}",
                    soname_origin.prefix,
                    if let Some(version) = soname_origin.version.as_ref() {
                        format!(".{version}")
                    } else {
                        String::new()
                    },
                    if let Some(origin) = soname_origin.origin.as_ref() {
                        format!(" (provided by {origin})")
                    } else {
                        String::new()
                    }
                );
                true
            } else {
                debug!(
                    "The alpm-sonamev2 {sonamev2} is NOT compatible with the alpm-sonamev2 {}:{name}{}{}",
                    soname_origin.prefix,
                    if let Some(version) = soname_origin.version.as_ref() {
                        format!(".{version}")
                    } else {
                        String::new()
                    },
                    if let Some(origin) = soname_origin.origin.as_ref() {
                        format!(" (provided by {origin})")
                    } else {
                        String::new()
                    }
                );
                false
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

    fn relation_lookup_package_relation_none() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&NAME.parse()?, None);

        Ok(lookup)
    }

    fn relation_lookup_package_relation_multiple_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(
            &format!("{NAME}=1").parse()?,
            Some(format!("{ORIGIN}1").parse()?),
        );
        lookup.insert_package_relation(&format!("{NAME}=2").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_package_relation_less() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&format!("{NAME}<1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_package_relation_less_or_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&format!("{NAME}<=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_package_relation_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&format!("{NAME}=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_package_relation_greater_or_equal() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&format!("{NAME}>=1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_package_relation_greater() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_package_relation(&format!("{NAME}>1").parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_basic_multiple() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(&"libexample.so".parse()?, Some(ORIGIN.parse()?));
        lookup.insert_sonamev1(
            &"libexample.so".parse()?,
            Some(format!("lib32-{ORIGIN}").parse()?),
        );

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_unversioned_multiple() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(
            &"libexample.so=libexample.so-64".parse()?,
            Some(ORIGIN.parse()?),
        );
        lookup.insert_sonamev1(
            &"libexample.so=libexample.so-32".parse()?,
            Some(format!("lib32-{ORIGIN}").parse()?),
        );

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_explicit_multiple() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(&"libexample.so=1.0.0-64".parse()?, Some(ORIGIN.parse()?));
        lookup.insert_sonamev1(
            &"libexample.so=1.0.0-32".parse()?,
            Some(format!("lib32-{ORIGIN}").parse()?),
        );

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_basic() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(&"libexample.so".parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_unversioned() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(
            &"libexample.so=libexample.so-64".parse()?,
            Some(ORIGIN.parse()?),
        );

        Ok(lookup)
    }

    fn relation_lookup_sonamev1_explicit() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev1(&"libexample.so=1.0.0-64".parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_sonamev2() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev2(&"lib:libexample.so.1".parse()?, Some(ORIGIN.parse()?));

        Ok(lookup)
    }

    fn relation_lookup_sonamev2_multiple() -> TestResult<RelationLookup> {
        let mut lookup = RelationLookup::default();
        lookup.insert_sonamev2(&"lib:libexample.so.1".parse()?, Some(ORIGIN.parse()?));
        lookup.insert_sonamev2(
            &"lib32:libexample.so.1".parse()?,
            Some(format!("lib32-{ORIGIN}").parse()?),
        );

        Ok(lookup)
    }

    #[rstest]
    #[case::lookup_none_relation_none(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_none_relation_less(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_none_relation_less_or_equal(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_equal(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_greater_or_equal(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_none_relation_greater(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_multiple_relation_equal_one(
        relation_lookup_package_relation_multiple_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_multiple_relation_equal_two(
        relation_lookup_package_relation_multiple_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=2".parse()?),
        }
    )]
    #[case::lookup_less_relation_none(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_less_relation_less(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_less_relation_less_or_equal(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_none(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_less_or_equal_relation_less(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_less_or_equal(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_less_or_equal_large(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=2".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_equal(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_greater_or_equal(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_none(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_equal_relation_less_or_equal(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_equal(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_greater_or_equal(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_none(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_greater_or_equal_relation_less_or_equal(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_equal(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_greater_or_equal(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_greater(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_none(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: None,
        }
    )]
    #[case::lookup_greater_relation_greater_or_equal(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_greater(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    fn relation_lookup_satisfies_package_relation_true(
        #[case] lookup: RelationLookup,
        #[case] relation: PackageRelation,
    ) -> TestResult {
        init_logger();

        assert!(lookup.satisfies_package_relation(&relation));

        Ok(())
    }

    #[rstest]
    #[case::lookup_empty_relation_less(RelationLookup::default(), format!("{NAME}<1").parse()?)]
    #[case::lookup_empty_relation_less_or_equal(RelationLookup::default(), format!("{NAME}<=1").parse()?)]
    #[case::lookup_empty_relation_equal(RelationLookup::default(), format!("{NAME}=1").parse()?)]
    #[case::lookup_empty_relation_greater_or_equal(RelationLookup::default(), format!("{NAME}>=1").parse()?)]
    #[case::lookup_empty_relation_greater(RelationLookup::default(), format!("{NAME}>1").parse()?)]
    #[case::lookup_less_relation_equal(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_less_relation_greater_or_equal(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">=1".parse()?),
        }
    )]
    #[case::lookup_less_relation_greater(
        relation_lookup_package_relation_less()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_less_or_equal_relation_greater(
        relation_lookup_package_relation_less_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_less(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_equal_relation_greater(
        relation_lookup_package_relation_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some(">1".parse()?),
        }
    )]
    #[case::lookup_greater_or_equal_relation_less(
        relation_lookup_package_relation_greater_or_equal()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_less(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_less_or_equal(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("<=1".parse()?),
        }
    )]
    #[case::lookup_greater_relation_equal(
        relation_lookup_package_relation_greater()?,
        PackageRelation {
            name: NAME.parse()?,
            version_requirement: Some("=1".parse()?),
        }
    )]
    #[case::lookup_none_mismatching_relation_name(
        relation_lookup_package_relation_none()?,
        PackageRelation {
            name: "something-else".parse()?,
            version_requirement: None,
        }
    )]
    fn relation_lookup_satisfies_package_relation_false(
        #[case] lookup: RelationLookup,
        #[case] relation: PackageRelation,
    ) -> TestResult {
        init_logger();

        assert!(!lookup.satisfies_package_relation(&relation));

        Ok(())
    }

    #[rstest]
    #[case::lookup_none_version_less(
        relation_lookup_package_relation_none()?,
        "0.1".parse()?,
    )]
    #[case::lookup_none_version_equal(
        relation_lookup_package_relation_none()?,
        "1".parse()?,
    )]
    #[case::lookup_none_version_greater(
        relation_lookup_package_relation_none()?,
        "2".parse()?,
    )]
    #[case::lookup_less_version_less(
        relation_lookup_package_relation_less()?,
        "0.1".parse()?,
    )]
    #[case::lookup_less_or_equal_version_less(
        relation_lookup_package_relation_less_or_equal()?,
        "0.1".parse()?,
    )]
    #[case::lookup_less_or_equal_version_equal(
        relation_lookup_package_relation_less_or_equal()?,
        "1".parse()?,
    )]
    #[case::lookup_equal_version_equal(
        relation_lookup_package_relation_equal()?,
        "1".parse()?,
    )]
    #[case::lookup_greater_or_equal_version_equal(
        relation_lookup_package_relation_greater_or_equal()?,
        "1".parse()?,
    )]
    #[case::lookup_greater_or_equal_version_greater(
        relation_lookup_package_relation_greater_or_equal()?,
        "2".parse()?,
    )]
    #[case::lookup_greater_version_greater(
        relation_lookup_package_relation_greater()?,
        "2".parse()?,
    )]
    fn relation_lookup_satisfies_name_and_version_true(
        #[case] lookup: RelationLookup,
        #[case] version: Version,
    ) -> TestResult {
        init_logger();

        assert!(lookup.satisfies_name_and_version(&NAME.parse()?, &version));

        Ok(())
    }

    #[rstest]
    #[case::lookup_less_version_greater(
        relation_lookup_package_relation_less()?,
        NAME.parse()?,
        "1".parse()?,
    )]
    #[case::lookup_less_or_equal_version_greater(
        relation_lookup_package_relation_less_or_equal()?,
        NAME.parse()?,
        "2".parse()?,
    )]
    #[case::lookup_equal_version_less(
        relation_lookup_package_relation_equal()?,
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_equal_version_greater(
        relation_lookup_package_relation_equal()?,
        NAME.parse()?,
        "2".parse()?,
    )]
    #[case::lookup_greater_or_equal_version_less(
        relation_lookup_package_relation_greater_or_equal()?,
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_greater_version_less(
        relation_lookup_package_relation_greater()?,
        NAME.parse()?,
        "0.1".parse()?,
    )]
    #[case::lookup_greater_version_equal(
        relation_lookup_package_relation_greater()?,
        NAME.parse()?,
        "1".parse()?,
    )]
    #[case::lookup_none_mismatching_name(
        relation_lookup_package_relation_none()?,
        "other-name".parse()?,
        "1".parse()?,
    )]
    fn relation_lookup_satisfies_name_and_version_false(
        #[case] lookup: RelationLookup,
        #[case] name: Name,
        #[case] version: Version,
    ) -> TestResult {
        init_logger();

        assert!(!lookup.satisfies_name_and_version(&name, &version));

        Ok(())
    }

    #[rstest]
    #[case::lookup_basic_sonamev1_basic_multiple(
        relation_lookup_sonamev1_basic_multiple()?,
        "libexample.so".parse()?
    )]
    #[case::lookup_basic_sonamev1_unversioned_multiple_64bit(
        relation_lookup_sonamev1_unversioned_multiple()?,
        "libexample.so=libexample.so-64".parse()?
    )]
    #[case::lookup_basic_sonamev1_unversioned_multiple_32bit(
        relation_lookup_sonamev1_unversioned_multiple()?,
        "libexample.so=libexample.so-32".parse()?
    )]
    #[case::lookup_basic_sonamev1_explicit_multiple_64bit(
        relation_lookup_sonamev1_explicit_multiple()?,
        "libexample.so=1.0.0-64".parse()?
    )]
    #[case::lookup_basic_sonamev1_explicit_multiple_32bit(
        relation_lookup_sonamev1_explicit_multiple()?,
        "libexample.so=1.0.0-32".parse()?
    )]
    #[case::lookup_basic_sonamev1_basic(
        relation_lookup_sonamev1_basic()?,
        "libexample.so".parse()?
    )]
    #[case::lookup_unversioned_sonamev1_unversioned(
        relation_lookup_sonamev1_unversioned()?,
        "libexample.so=libexample.so-64".parse()?
    )]
    #[case::lookup_explicit_sonamev1_explicit(
        relation_lookup_sonamev1_explicit()?,
        "libexample.so=1.0.0-64".parse()?
    )]
    fn relation_lookup_satisfies_sonamev1_true(
        #[case] lookup: RelationLookup,
        #[case] sonamev1: SonameV1,
    ) -> TestResult {
        init_logger();

        assert!(lookup.satisfies_sonamev1(&sonamev1));

        Ok(())
    }

    #[rstest]
    #[case::lookup_empty_sonamev1_basic(
        RelationLookup::default(),
        "libexample.so".parse()?
    )]
    #[case::lookup_empty_sonamev1_unversioned(
        RelationLookup::default(),
        "libexample.so=libexample.so-64".parse()?
    )]
    #[case::lookup_empty_sonamev1_explicit(
        RelationLookup::default(),
        "libexample.so=1.0.0-64".parse()?
    )]
    #[case::lookup_basic_sonamev1_unversioned(
        relation_lookup_sonamev1_basic()?,
        "libexample.so=libexample.so-64".parse()?
    )]
    #[case::lookup_basic_sonamev1_explicit(
        relation_lookup_sonamev1_basic()?,
        "libexample.so=1.0.0-64".parse()?
    )]
    #[case::lookup_unversioned_sonamev1_basic(
        relation_lookup_sonamev1_unversioned()?,
        "libexample.so".parse()?
    )]
    #[case::lookup_unversioned_sonamev1_explicit(
        relation_lookup_sonamev1_unversioned()?,
        "libexample.so=1.0.0-64".parse()?
    )]
    #[case::lookup_explicit_sonamev1_basic(
        relation_lookup_sonamev1_explicit()?,
        "libexample.so".parse()?
    )]
    #[case::lookup_explicit_sonamev1_unversioned(
        relation_lookup_sonamev1_explicit()?,
        "libexample.so=libexample.so-64".parse()?
    )]
    fn relation_lookup_satisfies_sonamev1_false(
        #[case] lookup: RelationLookup,
        #[case] sonamev1: SonameV1,
    ) -> TestResult {
        init_logger();

        assert!(!lookup.satisfies_sonamev1(&sonamev1));

        Ok(())
    }

    #[rstest]
    #[case::lookup_sonamev2_single(
        relation_lookup_sonamev2()?,
        "lib:libexample.so.1".parse()?
    )]
    #[case::lookup_sonamev2_multiple(
        relation_lookup_sonamev2_multiple()?,
        "lib:libexample.so.1".parse()?
    )]
    #[case::lookup_sonamev2_multiple(
        relation_lookup_sonamev2_multiple()?,
        "lib32:libexample.so.1".parse()?
    )]
    fn relation_lookup_satisfies_sonamev2_true(
        #[case] lookup: RelationLookup,
        #[case] sonamev2: SonameV2,
    ) -> TestResult {
        init_logger();

        assert!(lookup.satisfies_sonamev2(&sonamev2));

        Ok(())
    }

    #[rstest]
    #[case::lookup_empty_sonamev2(
        RelationLookup::default(),
        "lib:libexample.so.1".parse()?
    )]
    #[case::lookup_sonamev2_single_mismatching_prefix(
        relation_lookup_sonamev2()?,
        "lib32:libexample.so.1".parse()?
    )]
    #[case::lookup_sonamev2_single_no_version(
        relation_lookup_sonamev2()?,
        "lib:libexample.so".parse()?
    )]
    #[case::lookup_sonamev2_single_mismatching_version(
        relation_lookup_sonamev2()?,
        "lib:libexample.so.2".parse()?
    )]
    fn relation_lookup_satisfies_sonamev2_false(
        #[case] lookup: RelationLookup,
        #[case] sonamev2: SonameV2,
    ) -> TestResult {
        init_logger();

        assert!(!lookup.satisfies_sonamev2(&sonamev2));

        Ok(())
    }
}
