//! Convert parsed [`BridgeOutput::package_base`] output into a [`PackageBase`].

use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
#[cfg(doc)]
use alpm_pkgbuild::bridge::BridgeOutput;
use alpm_pkgbuild::bridge::{Keyword, Value};
use alpm_types::{
    Architectures,
    Backup,
    Changelog,
    Epoch,
    FullVersion,
    Install,
    License,
    MakepkgOption,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelationOrSoname,
    SkippableChecksum,
    Source,
    SystemArchitecture,
    Url,
};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err},
    error::StrContext,
    token::rest,
};

use crate::{
    pkgbuild_bridge::{
        ensure_keyword_exists,
        ensure_no_suffix,
        error::BridgeError,
        parse_arch_array,
        parse_optional_value,
        parse_value,
        parse_value_array,
    },
    source_info::{
        parser::{PackageBaseKeyword, RelationKeyword, SharedMetaKeyword, SourceKeyword},
        v1::package_base::{PackageBase, PackageBaseArchitecture},
    },
};

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
    ///
    /// # Errors
    ///
    /// Returns an error if `input` contains an unexpected keyword.
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

/// Handles all potentially architecture specific Vector entries in the [`handle_package_base`]
/// function.
///
/// If no architecture is encountered, it simply adds the value on the [`PackageBase`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
macro_rules! package_base_value_array {
    (
        $keyword:ident,
        $value:ident,
        $field_name:ident,
        $architecture:ident,
        $architecture_properties:ident,
        $parser:expr,
    ) => {
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
///
/// Handles parsing and type conversions of all raw input into their respective `alpm-types`.
///
/// # Errors
///
/// Returns an error if:
///
/// - values cannot be parsed,
/// - or unexpected keywords are encountered.
pub fn handle_package_base(
    name: Name,
    mut raw: HashMap<Keyword, Value>,
) -> Result<PackageBase, BridgeError> {
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
    let mut architectures = Architectures::Some(Vec::new());
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
    let mut crc_checksums = Vec::new();

    // Go through all keywords and handle them.
    for (raw_keyword, value) in &raw {
        // Parse the keyword
        let keyword = PackageBaseKeywords::parser
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
            PackageBaseKeywords::PkgBase => {
                // Explicitly handled before
                // We check for an unexpected suffix anyway in case somebody goofed up.
                ensure_no_suffix(raw_keyword, architecture)?;
                unreachable!(
                    "'pkgbase' has been handled before and should no longer exist without a suffix."
                )
            }
            PackageBaseKeywords::PackageBase(keyword) => match keyword {
                // Both PkgVer and PkgRel have been handled above.
                // We check for an unexpected suffix anyway in case somebody goofed up.
                PackageBaseKeyword::PkgVer => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                }
                PackageBaseKeyword::PkgRel => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                }
                PackageBaseKeyword::Epoch => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    epoch = parse_optional_value(raw_keyword, value, Epoch::parser)?;
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
                    package_base_value_array!(
                        raw_keyword,
                        value,
                        check_dependencies,
                        architecture,
                        architecture_properties,
                        PackageRelation::parser,
                    )
                }
                PackageBaseKeyword::MakeDepends => package_base_value_array!(
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
                    description = parse_optional_value(
                        raw_keyword,
                        value,
                        rest.try_map(PackageDescription::from_str),
                    )?;
                }
                SharedMetaKeyword::Url => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    url = parse_optional_value(raw_keyword, value, rest.try_map(Url::from_str))?;
                }
                SharedMetaKeyword::License => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    licenses =
                        parse_value_array(raw_keyword, value, rest.try_map(License::from_str))?;
                }
                SharedMetaKeyword::Arch => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    architectures = parse_arch_array(raw_keyword, value)?;
                }
                SharedMetaKeyword::Changelog => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    changelog = parse_optional_value(
                        raw_keyword,
                        value,
                        rest.try_map(Changelog::from_str),
                    )?;
                }
                SharedMetaKeyword::Install => {
                    ensure_no_suffix(raw_keyword, architecture)?;
                    install =
                        parse_optional_value(raw_keyword, value, rest.try_map(Install::from_str))?;
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
                    backups =
                        parse_value_array(raw_keyword, value, rest.try_map(Backup::from_str))?;
                }
            },
            PackageBaseKeywords::Relation(keyword) => match keyword {
                RelationKeyword::Depends => package_base_value_array!(
                    raw_keyword,
                    value,
                    dependencies,
                    architecture,
                    architecture_properties,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::OptDepends => package_base_value_array!(
                    raw_keyword,
                    value,
                    optional_dependencies,
                    architecture,
                    architecture_properties,
                    OptionalDependency::parser,
                ),
                RelationKeyword::Provides => package_base_value_array!(
                    raw_keyword,
                    value,
                    provides,
                    architecture,
                    architecture_properties,
                    RelationOrSoname::parser,
                ),
                RelationKeyword::Conflicts => package_base_value_array!(
                    raw_keyword,
                    value,
                    conflicts,
                    architecture,
                    architecture_properties,
                    PackageRelation::parser,
                ),
                RelationKeyword::Replaces => package_base_value_array!(
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
                SourceKeyword::Source => package_base_value_array!(
                    raw_keyword,
                    value,
                    sources,
                    architecture,
                    architecture_properties,
                    rest.try_map(Source::from_str),
                ),
                SourceKeyword::B2sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    b2_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Md5sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    md5_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha1sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    sha1_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha224sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    sha224_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha256sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    sha256_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha384sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    sha384_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Sha512sums => package_base_value_array!(
                    raw_keyword,
                    value,
                    sha512_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
                SourceKeyword::Cksums => package_base_value_array!(
                    raw_keyword,
                    value,
                    crc_checksums,
                    architecture,
                    architecture_properties,
                    SkippableChecksum::parser,
                ),
            },
        }
    }

    let version = FullVersion::new(package_version, package_release, epoch);

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
        version,
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
        crc_checksums,
    })
}
