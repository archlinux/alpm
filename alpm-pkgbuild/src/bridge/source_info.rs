//! Convert untyped and unchecked [BridgeOutput] into a well-formed [SourceInfoV1].

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
use alpm_srcinfo::{
    SourceInfoV1,
    source_info::{
        parser::{
            PackageBaseKeyword,
            RelationKeyword,
            SharedMetaKeyword,
            SourceKeyword,
            architecture as architecture_parser,
        },
        v1::{
            package::Package,
            package_base::{PackageBase, PackageBaseArchitecture},
        },
    },
};
use alpm_types::{
    Architecture,
    Epoch,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelationOrSoname,
    RelativePath,
    SkippableChecksum,
    Source,
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

use super::{
    error::BridgeError,
    parser::{BridgeOutput, ClearableValue, Keyword, RawPackageName, Value},
};

/// Make sure a given keyword exists, **remove** it from the map and return it.
///
/// This is a helper function to ensure expected values are set while throwing context-rich
/// errors if they don't.
fn ensure_keyword_exists(
    keyword: &Keyword,
    map: &mut HashMap<Keyword, Value>,
) -> Result<Value, BridgeError> {
    match map.remove(keyword) {
        Some(value) => Ok(value),
        None => Err(BridgeError::MissingRequiredKeyword {
            keyword: keyword.clone(),
        }),
    }
}

/// Make sure no architecture is provided for a keyword that doesn't allow it.
///
/// Throw an error, if an architecture is set.
fn ensure_no_suffix(
    keyword: &Keyword,
    architecture: Option<Architecture>,
) -> Result<(), BridgeError> {
    if let Some(arch) = architecture {
        return Err(BridgeError::UnexpectedArchitecture {
            keyword: keyword.clone(),
            suffix: arch,
        });
    }

    Ok(())
}

/// Make sure a given value exists and contains a [`Value::Single`].
///
/// Throw an error, if it contains an [`Value::Array`].
fn ensure_single_value<'a>(keyword: &Keyword, value: &'a Value) -> Result<&'a String, BridgeError> {
    match value {
        Value::Single(item) => Ok(item),
        Value::Array(values) => Err(BridgeError::UnexpectedArray {
            keyword: keyword.clone(),
            values: values.clone(),
        }),
    }
}

/// Takes a [`Value`], ensures it's a [`Value::Single`] and then applies the parser function to it.
///
/// Throws a error for the given keyword when the value cannot be parsed or the value is an array.
fn parse_value<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<O, BridgeError> {
    let input = ensure_single_value(keyword, value)?;
    Ok(parser.parse(input).map_err(|err| (keyword.clone(), err))?)
}

/// Takes a [`Value`], and the parser function on all elements.
/// Does not differentiate between [`Value::Single`] and [`Value::Array`] variants, as PKGBUILD
/// allows both for array values.
///
/// Throws a error for the given keyword when the value cannot be parsed.
fn parse_value_array<'a, O, P: Parser<&'a str, O, ErrMode<ContextError>>>(
    keyword: &Keyword,
    value: &'a Value,
    mut parser: P,
) -> Result<Vec<O>, BridgeError> {
    let input = value.as_vec();
    Ok(input
        .into_iter()
        .map(|item| parser.parse(item).map_err(|err| (keyword.clone(), err)))
        .collect::<Result<Vec<O>, (Keyword, ParseError<&'a str, ContextError>)>>()?)
}

pub fn source_info_from_bridge_output(
    mut output: BridgeOutput,
) -> Result<SourceInfoV1, BridgeError> {
    let mut name = None;
    // Check if there's a `pkgbase` section, which hints that this is a split package.
    let pkgbase_keyword = Keyword::simple("pkgbase");
    if let Some(value) = output.package_base.remove(&pkgbase_keyword) {
        name = Some(parse_value(&pkgbase_keyword, &value, Name::parser)?);
    }

    // Get the list of all packages that are declared.
    let pkgname_keyword = Keyword::simple("pkgname");
    let names = ensure_keyword_exists(&pkgname_keyword, &mut output.package_base)?;
    let names = parse_value_array(&pkgname_keyword, &names, Name::parser)?;

    // Use the `pkgbase` name by default, otherwise fallback to the first `pkgname` entry.
    let name = match name {
        Some(name) => name,
        None => {
            // The first package name is used as the name for the pkgbase section.
            names.first().cloned().ok_or(BridgeError::NoName)?
        }
    };

    let base = package_base(name, output.package_base)?;

    let packages = packages(output.packages)?;

    Ok(SourceInfoV1 { base, packages })
}

/// The combination of all keywords that're valid in the scope of a `pkgbase` section.
enum PackageBaseKeywords {
    // The `pkgbase` keyword.
    PkgBase,
    PackageBase(PackageBaseKeyword),
    Relation(RelationKeyword),
    SharedMeta(SharedMetaKeyword),
    Source(SourceKeyword),
}

impl PackageBaseKeywords {
    /// Recognizes any of the [`PackageBaseKeywords`] in an input string slice.
    ///
    /// Does not consume input and stops after any keyword matches.
    pub fn parser(input: &mut &str) -> ModalResult<PackageBaseKeywords> {
        cut_err(alt((
            "pkgbase".map(|_| PackageBaseKeywords::PkgBase),
            PackageBaseKeyword::parser.map(PackageBaseKeywords::PackageBase),
            RelationKeyword::parser.map(PackageBaseKeywords::Relation),
            SharedMetaKeyword::parser.map(PackageBaseKeywords::SharedMeta),
            SourceKeyword::parser.map(PackageBaseKeywords::Source),
        )))
        .context(StrContext::Label("package base property type"))
        .context_with(iter_str_context!([
            &["pkgbase"],
            PackageBaseKeyword::VARIANTS,
            RelationKeyword::VARIANTS,
            SharedMetaKeyword::VARIANTS,
            SourceKeyword::VARIANTS,
        ]))
        .parse_next(input)
    }
}

/// Handles all potentially architecture specific Vector entries in the [`PackageBase::from_parsed`]
/// function.
///
/// If no architecture is encountered, it simply adds the value on the [`PackageBase`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
///
/// Furthermore, adds linter warnings if an architecture is encountered that doesn't exist in the
/// [`PackageBase::architectures`].
macro_rules! package_base_arch_value {
    (
        $keyword:ident,
        $value:ident,
        $field_name:ident,
        $architecture:ident,
        $architecture_properties:ident,
        $parser:expr,
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparation
        if let Some(architecture) = $architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $architecture_properties
                .entry(architecture)
                .or_insert(PackageBaseArchitecture::default());

            // Set the architecture specific value.
            architecture_properties.$field_name = parse_value_array($keyword, $value, $parser)?;
        } else {
            $field_name = parse_value_array($keyword, $value, $parser)?;
        }
    };
}

/// Convert the raw keyword map from the [`BridgeOutput`] into a well-formed and typed
/// [`PackageBase`].
fn package_base(name: Name, mut raw: HashMap<Keyword, Value>) -> Result<PackageBase, BridgeError> {
    // First up, we handle keywords that're required.
    let pkgver_keyword = Keyword::simple("pkgver");
    let value = ensure_keyword_exists(&pkgver_keyword, &mut raw)?;
    let package_version: PackageVersion =
        parse_value(&pkgver_keyword, &value, PackageVersion::parser)?;

    let pkgrel_keyword = Keyword::simple("pkgrel");
    let value = ensure_keyword_exists(&pkgrel_keyword, &mut raw)?;
    let package_release: PackageRelease =
        parse_value(&pkgver_keyword, &value, PackageRelease::parser)?;

    let mut description = None;
    let mut url = None;
    let mut licenses = Vec::new();
    let mut changelog = None;
    let mut architectures = HashSet::new();
    let mut architecture_properties = BTreeMap::new();

    // Build or package management related meta fields
    let mut install = None;
    let mut groups = Vec::new();
    let mut options = Vec::new();
    let mut backups = Vec::new();

    let mut epoch: Option<Epoch> = None;
    let mut pgp_fingerprints = Vec::new();

    let mut dependencies = Vec::new();
    let mut optional_dependencies = Vec::new();
    let mut provides = Vec::new();
    let mut conflicts = Vec::new();
    let mut replaces = Vec::new();
    // The following dependencies are build-time specific dependencies.
    // `makepkg` expects all dependencies for all split packages to be specified in the
    // PackageBase.
    let mut check_dependencies = Vec::new();
    let mut make_dependencies = Vec::new();

    let mut sources = Vec::new();
    let mut no_extracts = Vec::new();
    let mut b2_checksums = Vec::new();
    let mut md5_checksums = Vec::new();
    let mut sha1_checksums = Vec::new();
    let mut sha224_checksums = Vec::new();
    let mut sha256_checksums = Vec::new();
    let mut sha384_checksums = Vec::new();
    let mut sha512_checksums = Vec::new();

    // Go through all keywords and handle them.
    for (raw_keyword, value) in &raw {
        // Parse the keyword
        let keyword = PackageBaseKeywords::parser
            .parse(&raw_keyword.keyword)
            .map_err(|err| (raw_keyword.clone(), err))?;

        // Parse the architecture suffix if it exists.
        let architecture = match &raw_keyword.suffix {
            Some(suffix) => {
                let arch = architecture_parser
                    .parse(suffix)
                    .map_err(|err| (raw_keyword.clone(), err))?;
                Some(arch)
            }
            None => None,
        };

        match keyword {
            PackageBaseKeywords::PkgBase => {
                // Explicitly handled before
                // We check for an unexpected suffix anyway in case somebody goofed up.
                ensure_no_suffix(raw_keyword, architecture)?;
                unreachable!(
                    "'pkgbase' has been handled before and should no longer exist without a suffix."
                )
            }
            PackageBaseKeywords::PackageBase(keyword) => match keyword {
                // Both PkgVer and PkgRel haven been handled above.
                // We check for an unexpected suffix anyway in case somebody goofed up.
                PackageBaseKeyword::PkgVer => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                }
                PackageBaseKeyword::PkgRel => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                }
                PackageBaseKeyword::Epoch => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    epoch = Some(parse_value(raw_keyword, value, Epoch::parser)?);
                }
                PackageBaseKeyword::ValidPGPKeys => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    pgp_fingerprints = parse_value_array(
                        raw_keyword,
                        value,
                        rest.try_map(OpenPGPIdentifier::from_str),
                    )?;
                }
                PackageBaseKeyword::CheckDepends => {
                    package_base_arch_value!(
                        raw_keyword,
                        value,
                        check_dependencies,
                        architecture,
                        architecture_properties,
                        PackageRelation::parser,
                    )
                }
                PackageBaseKeyword::MakeDepends => package_base_arch_value!(
                    raw_keyword,
                    value,
                    make_dependencies,
                    architecture,
                    architecture_properties,
                    PackageRelation::parser,
                ),
            },
            PackageBaseKeywords::SharedMeta(keyword) => match keyword {
                SharedMetaKeyword::PkgDesc => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    description = Some(ensure_single_value(raw_keyword, value)?.to_string());
                }
                SharedMetaKeyword::Url => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    url = Some(parse_value(
                        raw_keyword,
                        value,
                        rest.try_map(Url::from_str),
                    )?);
                }
                SharedMetaKeyword::License => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    licenses =
                        parse_value_array(raw_keyword, value, rest.try_map(License::from_str))?;
                }
                SharedMetaKeyword::Arch => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    let archs = parse_value_array(raw_keyword, value, architecture_parser)?;
                    // Make sure all architectures are unique.
                    for arch in archs {
                        if architectures.contains(&arch) {
                            return Err(BridgeError::LogicError {
                                keyword: raw_keyword.clone(),
                                message: format!("Found duplicate architecture {arch}"),
                            });
                        }
                        architectures.insert(arch);
                    }
                }
                SharedMetaKeyword::Changelog => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    changelog = Some(parse_value(
                        raw_keyword,
                        value,
                        rest.try_map(RelativePath::from_str),
                    )?);
                }
                SharedMetaKeyword::Install => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    install = Some(parse_value(
                        raw_keyword,
                        value,
                        rest.try_map(RelativePath::from_str),
                    )?);
                }
                SharedMetaKeyword::Groups => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    groups = value.clone().as_owned_vec();
                }
                SharedMetaKeyword::Options => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    options = parse_value_array(raw_keyword, value, MakepkgOption::parser)?;
                }
                SharedMetaKeyword::Backup => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    backups = parse_value_array(
                        raw_keyword,
                        value,
                        rest.try_map(RelativePath::from_str),
                    )?;
                }
            },
            PackageBaseKeywords::Relation(keyword) => match keyword {
                RelationKeyword::Depends => package_base_arch_value!(
                    raw_keyword,
                    value,
                    dependencies,
                    architecture,
                    architecture_properties,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::OptDepends => package_base_arch_value!(
                    raw_keyword,
                    value,
                    optional_dependencies,
                    architecture,
                    architecture_properties,
                    OptionalDependency::parser,
                ),
                RelationKeyword::Provides => package_base_arch_value!(
                    raw_keyword,
                    value,
                    provides,
                    architecture,
                    architecture_properties,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::Conflicts => package_base_arch_value!(
                    raw_keyword,
                    value,
                    conflicts,
                    architecture,
                    architecture_properties,
                    PackageRelation::parser,
                ),
                RelationKeyword::Replaces => package_base_arch_value!(
                    raw_keyword,
                    value,
                    replaces,
                    architecture,
                    architecture_properties,
                    PackageRelation::parser,
                ),
            },

            PackageBaseKeywords::Source(keyword) => match keyword {
                SourceKeyword::NoExtract => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    no_extracts = value.clone().as_owned_vec();
                }
                SourceKeyword::Source => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sources,
                    architecture,
                    architecture_properties,
                    rest.try_map(Source::from_str),
                ),
                SourceKeyword::B2sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    b2_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Md5sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    md5_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha1sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sha1_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha224sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sha224_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha256sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sha256_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha384sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sha384_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha512sums => package_base_arch_value!(
                    raw_keyword,
                    value,
                    sha512_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
            },
        }
    }

    Ok(PackageBase {
        name,
        description,
        url,
        changelog,
        licenses,
        install,
        groups,
        options,
        backups,
        package_version,
        package_release,
        epoch,
        pgp_fingerprints,
        architectures,
        architecture_properties,
        dependencies,
        optional_dependencies,
        provides,
        conflicts,
        replaces,
        check_dependencies,
        make_dependencies,
        sources,
        no_extracts,
        b2_checksums,
        md5_checksums,
        sha1_checksums,
        sha224_checksums,
        sha256_checksums,
        sha384_checksums,
        sha512_checksums,
    })
}

fn packages(
    _raw: HashMap<RawPackageName, HashMap<Keyword, ClearableValue>>,
) -> Result<Vec<Package>, BridgeError> {
    let packages = Vec::new();

    Ok(packages)
}
