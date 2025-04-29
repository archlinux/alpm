use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

use alpm_parsers::iter_str_context;
use alpm_srcinfo::source_info::{
    parser::{
        PackageBaseKeyword,
        RelationKeyword,
        SharedMetaKeyword,
        SourceKeyword,
        architecture_suffix,
    },
    v1::package_base::{PackageBase, PackageBaseArchitecture},
};
use alpm_types::{
    Architecture,
    Epoch,
    License,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageOption,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelationOrSoname,
    RelativePath,
    SkippableChecksum,
    Source,
    Url,
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};
use strum::VariantNames;
use winnow::{
    ModalResult,
    Parser,
    ascii::space1,
    combinator::{alt, cut_err},
    error::StrContext,
};

use super::{
    string_value_list_till_newline,
    string_value_till_newline,
    value_list_till_newline,
    value_till_newline,
};

enum PackageBaseKeywords {
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
            PackageBaseKeyword::parser.map(PackageBaseKeywords::PackageBase),
            RelationKeyword::parser.map(PackageBaseKeywords::Relation),
            SharedMetaKeyword::parser.map(PackageBaseKeywords::SharedMeta),
            SourceKeyword::parser.map(PackageBaseKeywords::Source),
        )))
        .context(StrContext::Label("package base property type"))
        .context_with(iter_str_context!([
            PackageBaseKeyword::VARIANTS,
            RelationKeyword::VARIANTS,
            SharedMetaKeyword::VARIANTS,
            SourceKeyword::VARIANTS,
        ]))
        .parse_next(input)
    }
}

/// Handles all potentially architecture specific Vector entries in the [`parse_package_base`]
/// function.
///
/// If no architecture is encountered, it simply sets the value on the [`PackageBase`] itself.
/// Otherwise, it's added to the respective [`PackageBase::architecture_properties`].
macro_rules! package_base_arch_prop {
    (
        $architecture:ident,
        $architecture_properties:ident,
        $field_name:ident,
        $value:expr
    ) => {
        // Check if the property is architecture specific.
        // If so, we have to perform some checks and preparation
        if let Some(architecture) = $architecture {
            // Make sure the architecture specific properties are initialized.
            let architecture_properties = $architecture_properties
                .entry(architecture)
                .or_insert(PackageBaseArchitecture::default());

            // Set the architecture specific value.
            architecture_properties.$field_name = $value
        } else {
            $field_name = $value
        }
    };
}

/// Parse all input until we hit the next section
pub fn parse_package_base(input: &mut &str) -> ModalResult<(PackageBase, Vec<PackageBaseName>)> {
    let mut package_names = Vec::new();
    let mut description: Option<PackageDescription> = None;
    let mut url: Option<Url> = None;
    let mut changelog: Option<RelativePath> = None;
    let mut licenses: Vec<License> = Vec::new();

    let mut install: Option<RelativePath> = None;
    let mut groups: Vec<String> = Vec::new();
    let mut options: Vec<PackageOption> = Vec::new();
    let mut backups: Vec<RelativePath> = Vec::new();

    let mut package_version: Option<PackageVersion> = None;
    let mut package_release: Option<PackageRelease> = None;
    let mut epoch: Option<Epoch> = None;

    let mut pgp_fingerprints: Vec<OpenPGPIdentifier> = Vec::new();

    let mut architectures: HashSet<Architecture> = HashSet::new();
    let mut architecture_properties: BTreeMap<Architecture, PackageBaseArchitecture> =
        BTreeMap::new();

    let mut dependencies: Vec<RelationOrSoname> = Vec::new();
    let mut optional_dependencies: Vec<OptionalDependency> = Vec::new();
    let mut provides: Vec<RelationOrSoname> = Vec::new();
    let mut conflicts: Vec<PackageRelation> = Vec::new();
    let mut replaces: Vec<PackageRelation> = Vec::new();
    let mut check_dependencies: Vec<PackageRelation> = Vec::new();
    let mut make_dependencies: Vec<PackageRelation> = Vec::new();

    let mut sources: Vec<Source> = Vec::new();
    let mut no_extracts: Vec<String> = Vec::new();
    let mut b2_checksums: Vec<SkippableChecksum<Blake2b512>> = Vec::new();
    let mut md5_checksums: Vec<SkippableChecksum<Md5>> = Vec::new();
    let mut sha1_checksums: Vec<SkippableChecksum<Sha1>> = Vec::new();
    let mut sha224_checksums: Vec<SkippableChecksum<Sha224>> = Vec::new();
    let mut sha256_checksums: Vec<SkippableChecksum<Sha256>> = Vec::new();
    let mut sha384_checksums: Vec<SkippableChecksum<Sha384>> = Vec::new();
    let mut sha512_checksums: Vec<SkippableChecksum<Sha512>> = Vec::new();

    loop {
        // Parse the start of the line. We already know what types the values should have, so
        // we simply ignore the `ARRAY/STRING` definition. It doesn't make a difference in the
        // formatting anyway and we have full control over the bridge script.
        //
        // If the parser succeeds, we're good to go and may continue.
        // If this is a backtracking error, we know that we hit the end of the pkgbase section.
        let result: ModalResult<(&str, &str, &str, &str)> =
            ("VAR GLOBAL", space1, alt(("ARRAY", "STRING")), space1).parse_next(input);
        if result.is_err() {
            break;
        }

        let keyword = PackageBaseKeywords::parser.parse_next(input)?;

        match keyword {
            PackageBaseKeywords::PackageBase(keyword) => match keyword {
                PackageBaseKeyword::PkgVer => {
                    package_version = Some(value_till_newline(input, PackageVersion::from_str)?);
                }
                PackageBaseKeyword::PkgRel => {
                    package_release = Some(value_till_newline(input, PackageRelease::from_str)?);
                }
                PackageBaseKeyword::Epoch => {
                    epoch = Some(value_till_newline(input, Epoch::from_str)?);
                }
                PackageBaseKeyword::ValidPGPKeys => {
                    pgp_fingerprints = value_list_till_newline(input, OpenPGPIdentifier::from_str)?;
                }
                PackageBaseKeyword::CheckDepends | PackageBaseKeyword::MakeDepends => {
                    // Parse an architecture_suffix if it exists.
                    let architecture = architecture_suffix.parse_next(input)?;

                    match keyword {
                        PackageBaseKeyword::CheckDepends => package_base_arch_prop!(
                            architecture,
                            architecture_properties,
                            check_dependencies,
                            value_list_till_newline(input, PackageRelation::from_str)?
                        ),
                        PackageBaseKeyword::MakeDepends => package_base_arch_prop!(
                            architecture,
                            architecture_properties,
                            make_dependencies,
                            value_list_till_newline(input, PackageRelation::from_str)?
                        ),
                        _ => unreachable!(),
                    }
                }
            },
            PackageBaseKeywords::Relation(keyword) => {
                let architecture = architecture_suffix.parse_next(input)?;
                match keyword {
                    RelationKeyword::Depends => package_base_arch_prop!(
                        architecture,
                        architecture_properties,
                        dependencies,
                        value_list_till_newline(input, RelationOrSoname::from_str)?
                    ),
                    RelationKeyword::OptDepends => package_base_arch_prop!(
                        architecture,
                        architecture_properties,
                        optional_dependencies,
                        value_list_till_newline(input, OptionalDependency::from_str)?
                    ),
                    RelationKeyword::Provides => package_base_arch_prop!(
                        architecture,
                        architecture_properties,
                        provides,
                        value_list_till_newline(input, RelationOrSoname::from_str)?
                    ),
                    RelationKeyword::Conflicts => package_base_arch_prop!(
                        architecture,
                        architecture_properties,
                        conflicts,
                        value_list_till_newline(input, PackageRelation::from_str)?
                    ),
                    RelationKeyword::Replaces => package_base_arch_prop!(
                        architecture,
                        architecture_properties,
                        replaces,
                        value_list_till_newline(input, PackageRelation::from_str)?
                    ),
                }
            }
            PackageBaseKeywords::SharedMeta(keyword) => match keyword {
                SharedMetaKeyword::PkgDesc => description = Some(string_value_till_newline(input)?),
                SharedMetaKeyword::Url => url = Some(value_till_newline(input, Url::from_str)?),
                SharedMetaKeyword::License => {
                    licenses = value_list_till_newline(input, License::from_str)?
                }
                SharedMetaKeyword::Arch => {
                    let arch_vec = value_list_till_newline(input, Architecture::from_str)?;
                    architectures = HashSet::from_iter(arch_vec);
                }
                SharedMetaKeyword::Changelog => {
                    changelog = Some(value_till_newline(input, RelativePath::from_str)?)
                }
                SharedMetaKeyword::Install => {
                    install = Some(value_till_newline(input, RelativePath::from_str)?)
                }
                SharedMetaKeyword::Groups => groups = string_value_list_till_newline(input)?,
                SharedMetaKeyword::Options => {
                    options = value_list_till_newline(input, PackageOption::from_str)?
                }
                SharedMetaKeyword::Backup => {
                    backups = value_list_till_newline(input, RelativePath::from_str)?
                }
            },
            PackageBaseKeywords::Source(keyword) => {
                match keyword {
                    // NoExtract is the only one field without an architecture suffix.
                    SourceKeyword::NoExtract => {
                        no_extracts = string_value_list_till_newline(input)?
                    }
                    SourceKeyword::Source
                    | SourceKeyword::B2sums
                    | SourceKeyword::Md5sums
                    | SourceKeyword::Sha1sums
                    | SourceKeyword::Sha224sums
                    | SourceKeyword::Sha256sums
                    | SourceKeyword::Sha384sums
                    | SourceKeyword::Sha512sums => {
                        // Parse an architecture_suffix if it exists.
                        let architecture = architecture_suffix.parse_next(input)?;
                        match keyword {
                            SourceKeyword::Source => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sources,
                                value_list_till_newline(input, Source::from_str)?
                            ),

                            SourceKeyword::B2sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                b2_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Md5sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                md5_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Sha1sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sha1_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Sha224sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sha224_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Sha256sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sha256_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Sha384sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sha384_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::Sha512sums => package_base_arch_prop!(
                                architecture,
                                architecture_properties,
                                sha512_checksums,
                                value_list_till_newline(input, SkippableChecksum::from_str)?
                            ),
                            SourceKeyword::NoExtract => unreachable!(),
                        }
                    }
                }
            }
        }
    }

    Ok((
        PackageBase {
            name: name.unwrap(),
            description,
            url,
            licenses,
            changelog,
            architectures,
            architecture_properties,
            install,
            groups,
            options,
            backups,
            package_version: package_version.unwrap(),
            package_release: package_release.unwrap(),
            epoch,
            pgp_fingerprints,
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
        },
        package_names,
    ))
}
