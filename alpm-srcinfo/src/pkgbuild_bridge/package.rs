//! Convert parsed [`BridgeOutput::packages`] output into [`Package`]s.

use std::{collections::HashMap, str::FromStr};

use alpm_parsers::iter_str_context;
#[cfg(doc)]
use alpm_pkgbuild::bridge::BridgeOutput;
use alpm_pkgbuild::bridge::{ClearableValue, Keyword, RawPackageName};
use alpm_types::{
    Architecture,
    Backup,
    Changelog,
    Group,
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
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err},
    error::{ContextError, ErrMode, ParseError, StrContext},
    token::rest,
};

use super::ensure_no_suffix;
use crate::{
    pkgbuild_bridge::error::BridgeError,
    source_info::{
        parser::{RelationKeyword, SharedMetaKeyword},
        v1::package::{Override, Package, PackageArchitecture},
    },
};

/// Converts parsed [`BridgeOutput::packages`] output into [`Package`]s.
///
/// # Enforced Invariants
///
/// All scoped package variables must have a respective entry in `pkgbase.pkgname`.
///
/// # Errors
///
/// Returns an error if
///
/// - a `package` function without an [alpm-package-name] suffix exists in an [alpm-split-package]
///   setup,
/// - a value cannot be turned into its [`alpm_types`] equivalent,
/// - multiple values exist for a field that only accepts a singular value,
/// - an [alpm-architecture] is duplicated,
/// - an [alpm-architecture] is cleared in `package` function,
/// - or an [alpm-architecture] suffix is set on a keyword that does not support it.
///
/// [alpm-architecture]: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
/// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
/// [alpm-split-package]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
pub(crate) fn handle_packages(
    base_package: Name,
    valid_packages: Vec<Name>,
    raw_values: HashMap<RawPackageName, HashMap<Keyword, ClearableValue>>,
) -> Result<Vec<Package>, BridgeError> {
    let mut package_map: HashMap<Name, Package> = HashMap::new();

    for (name, values) in raw_values {
        // Check if the variable is assigned to a specific split package.
        // If it isn't, use the name of the base package instead, which is the default.
        let name = if let Some(name) = name.0 {
            Name::parser
                .parse(&name)
                .map_err(|err| BridgeError::InvalidPackageName {
                    name: name.clone(),
                    error: err.into(),
                })?
        } else {
            // If this is a literal `package` function we have to make sure that this isn't a split
            // package! Split package `package` functions must have a `_$name` suffix.
            if valid_packages.len() > 1 {
                return Err(BridgeError::UnusedPackageFunction(base_package));
            }

            base_package.clone()
        };

        // Make sure the package has been declared in the package base section.
        if !valid_packages.contains(&name) {
            return Err(BridgeError::UndeclaredPackageName(name.to_string()));
        }

        // Get the package on which the properties should be set.
        let package = package_map.entry(name.clone()).or_insert(name.into());

        handle_package(package, values)?;
    }

    // Convert the package map into a vector that follows the same order as the `pkgbase`
    let mut packages = Vec::new();
    for name in valid_packages {
        let Some(package) = package_map.remove(&name) else {
            // Create a empty package entry for any packages that don't have any variable set and
            // thereby haven't been initialized yet.
            packages.push(name.into());
            continue;
        };

        packages.push(package);
    }

    Ok(packages)
}

/// The combination of all keywords that're valid in the scope of a `package` section.
enum PackageKeyword {
    Relation(RelationKeyword),
    SharedMeta(SharedMetaKeyword),
}

impl PackageKeyword {
    /// Recognizes any of the [`PackageKeyword`] in an input string slice.
    ///
    /// Does not consume input and stops after any keyword matches.
    ///
    /// # Errors
    ///
    /// Returns an error, if an unknown keyword is encountered.
    pub fn parser(input: &mut &str) -> ModalResult<PackageKeyword> {
        cut_err(alt((
            RelationKeyword::parser.map(PackageKeyword::Relation),
            SharedMetaKeyword::parser.map(PackageKeyword::SharedMeta),
        )))
        .context(StrContext::Label("package base property type"))
        .context_with(iter_str_context!([
            RelationKeyword::VARIANTS,
            SharedMetaKeyword::VARIANTS,
        ]))
        .parse_next(input)
    }
}

/// Ensures that in a combination of [`Keyword`] and [`ClearableValue`], a
/// [`ClearableValue::Single`] is used and returns the value.
///
/// # Errors
///
/// Returns an error, if value is a [`ClearableValue::Array`].
fn ensure_single_clearable_value<'a>(
    keyword: &Keyword,
    value: &'a ClearableValue,
) -> Result<&'a Option<String>, BridgeError> {
    match value {
        ClearableValue::Single(value) => Ok(value),
        ClearableValue::Array(values) => Err(BridgeError::UnexpectedArray {
            keyword: keyword.clone(),
            values: values.clone().unwrap_or_default().clone(),
        }),
    }
}

/// Ensures that a combination of [`Keyword`] and [`ClearableValue`] uses a
/// [`ClearableValue::Single`] and parses the value as a specific type.
///
/// # Errors
///
/// Returns an error if
///
/// - `value` cannot be parsed as a specific type,
/// - or `value` is a [`ClearableValue::Array`].
fn parse_clearable_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a ClearableValue,
    mut parser: P,
) -> Result<Override<O>, BridgeError> {
    // Make sure we have no array
    let value = ensure_single_clearable_value(keyword, value)?;

    // If the value is `None`, it indicates a cleared value.
    let Some(value) = value else {
        return Ok(Override::Clear);
    };

    let parsed_value = parser.parse(value).map_err(|err| (keyword.clone(), err))?;

    Ok(Override::Yes {
        value: parsed_value,
    })
}

/// Parses all elements of a [`ClearableValue`] as a [`Vec`] of specific types.
///
/// # Legacy support
///
/// This does not differentiate between [`ClearableValue::Single`] and [`ClearableValue::Array`]
/// variants, as [PKGBUILD] files allow both notations for array values.
///
/// Modern versions of [makepkg] enforce that certain values **must** be arrays.
/// However, to be able to parse both historic and modern [PKGBUILD] files this function is less
/// strict.
///
/// # Errors
///
/// Returns an error if the elements of `value` cannot be parsed as a [`Vec`] of a specific type.
///
/// [PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
/// [makepkg]: https://man.archlinux.org/man/makepkg.8
fn parse_clearable_value_array<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a ClearableValue,
    mut parser: P,
) -> Result<Override<Vec<O>>, BridgeError> {
    let values = match value {
        ClearableValue::Single(value) => {
            let Some(value) = value else {
                return Ok(Override::Clear);
            };
            // An empty string is considered a clear.
            if value.is_empty() {
                return Ok(Override::Clear);
            }
            let value = parser.parse(value).map_err(|err| (keyword.clone(), err))?;

            vec![value]
        }
        ClearableValue::Array(values) => {
            let Some(values) = values else {
                return Ok(Override::Clear);
            };

            values
                .iter()
                .map(|item| parser.parse(item).map_err(|err| (keyword.clone(), err)))
                .collect::<Result<Vec<O>, (Keyword, ParseError<&'a str, ContextError>)>>()?
        }
    };

    Ok(Override::Yes { value: values })
}

/// Handles all potentially architecture specific Vector entries in the [`handle_package`] function.
///
/// If no architecture is encountered, it simply adds the value on the [`Package`] itself.
/// Otherwise, it's added to the respective [`Package::architecture_properties`].
macro_rules! package_value_array {
    (
        $keyword:expr,
        $value:expr,
        $package:ident,
        $field_name:ident,
        $architecture:ident,
        $parser:expr,
    ) => {
        if let Some(architecture) = $architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $package
                .architecture_properties
                .entry(architecture)
                .or_insert(PackageArchitecture::default());

            // Set the architecture specific value.
            architecture_properties.$field_name =
                parse_clearable_value_array($keyword, $value, $parser)?;
        } else {
            $package.$field_name = parse_clearable_value_array($keyword, $value, $parser)?;
        }
    };
}

/// Adds a map of [`Keyword`] and [`ClearableValue`] to a [`Package`].
///
/// Handles parsing and type conversions of all raw input into their respective [`alpm_types`]
/// types.
///
/// # Errors
///
/// Returns an error if
///
/// - one of the values in `values` cannot be converted into its respective [`alpm_types`] type,
/// - or keywords incompatible with [`Package`] are encountered.
fn handle_package(
    package: &mut Package,
    values: HashMap<Keyword, ClearableValue>,
) -> Result<(), BridgeError> {
    for (raw_keyword, value) in values {
        // Parse the keyword
        let keyword = PackageKeyword::parser
            .parse(&raw_keyword.keyword)
            .map_err(|err| (raw_keyword.clone(), err))?;

        // Parse the architecture suffix if it exists.
        let architecture = match &raw_keyword.suffix {
            Some(suffix) => {
                // SystemArchitecture::parser forbids "any"
                let arch = SystemArchitecture::parser
                    .parse(suffix)
                    .map_err(|err| (raw_keyword.clone(), err))?;
                Some(arch)
            }
            None => None,
        };

        // Parse and set the value based on which keyword it is.
        match keyword {
            PackageKeyword::Relation(keyword) => match keyword {
                RelationKeyword::Depends => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    dependencies,
                    architecture,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::OptDepends => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    optional_dependencies,
                    architecture,
                    OptionalDependency::parser,
                ),
                RelationKeyword::Provides => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    provides,
                    architecture,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::Conflicts => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    conflicts,
                    architecture,
                    PackageRelation::parser,
                ),
                RelationKeyword::Replaces => package_value_array!(
                    &raw_keyword,
                    &value,
                    package,
                    replaces,
                    architecture,
                    PackageRelation::parser,
                ),
            },
            PackageKeyword::SharedMeta(keyword) => match keyword {
                SharedMetaKeyword::PkgDesc => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.description = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(PackageDescription::from_str),
                    )?;
                }
                SharedMetaKeyword::Url => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.url =
                        parse_clearable_value(&raw_keyword, &value, rest.try_map(Url::from_str))?;
                }
                SharedMetaKeyword::License => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.licenses = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(License::from_str),
                    )?;
                }
                SharedMetaKeyword::Arch => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    let archs = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Architecture::from_str),
                    )?;

                    // Architectures are a bit special as they **are not** allowed to be cleared.
                    package.architectures = match archs {
                        Override::No => None,
                        Override::Clear => {
                            return Err(BridgeError::UnclearableValue {
                                keyword: raw_keyword,
                            });
                        }
                        Override::Yes { value } => Some(value.try_into()?),
                    };
                }
                SharedMetaKeyword::Changelog => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.changelog = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(Changelog::from_str),
                    )?;
                }
                SharedMetaKeyword::Install => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.install = parse_clearable_value(
                        &raw_keyword,
                        &value,
                        rest.try_map(Install::from_str),
                    )?;
                }
                SharedMetaKeyword::Groups => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.groups = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Group::from_str),
                    )?;
                }
                SharedMetaKeyword::Options => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.options = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(MakepkgOption::from_str),
                    )?;
                }
                SharedMetaKeyword::Backup => {
                    ensure_no_suffix(&raw_keyword, architecture)?;
                    package.backups = parse_clearable_value_array(
                        &raw_keyword,
                        &value,
                        rest.try_map(Backup::from_str),
                    )?;
                }
            },
        }
    }

    Ok(())
}
