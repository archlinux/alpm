use std::str::FromStr;

use alpm_types::{
    digests::{Blake2b512, Digest, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
    Architecture,
    Backup,
    Blake2b512Checksum,
    Changelog,
    Checksum,
    Epoch,
    Group,
    Install,
    License,
    MakepkgOption,
    Md5Checksum,
    Name,
    OpenPGPIdentifier,
    OptionalDependency,
    PackageDescription,
    PackageOption,
    PackageRelation,
    PackageRelease,
    PackageVersion,
    RelativePath,
    Sha1Checksum,
    Sha224Checksum,
    Sha256Checksum,
    Sha384Checksum,
    Sha512Checksum,
    Source,
    Url,
};
use winnow::{
    ascii::{line_ending, newline, space0, till_line_ending},
    combinator::{
        alt,
        cut_err,
        delimited,
        eof,
        fail,
        opt,
        peek,
        preceded,
        repeat,
        repeat_till,
        terminated,
        trace,
    },
    error::{ErrMode, ErrorKind, ParserError, StrContext, StrContextValue},
    token::{take_till, take_until},
    PResult,
    Parser,
};

/// Represent a checksum check that is allowed to be skipped.
/// If the `SKIP` keyword is found, a source file won't be checked for this type of checksum.
#[derive(Debug, Clone)]
pub enum SkippableChecksum<D: Digest + Clone> {
    Skip,
    Checksum(Checksum<D>),
}

/// Parse the delimiter between keywords, i.e. ` = `.
///
/// This function expects the delimiter to be there.
fn delimiter<'s>(input: &mut &'s str) -> PResult<&'s str> {
    cut_err(" = ")
        .context(StrContext::Label("delimiter"))
        .context(StrContext::Expected(StrContextValue::Description(
            "an equal sign surrounded by spaces: ' = '.",
        )))
        .parse_next(input)
}

/// Return all content until the end of line.
/// It's called after a ` = ` has been encountered.
///
/// This is a bit different than winnow's own till_line_ending in that it consumes the newline.
fn till_line_end<'s>(input: &mut &'s str) -> PResult<&'s str> {
    // Get the content til the end of line.
    let out = till_line_ending.parse_next(input)?;

    // Consume the newline.
    line_ending.parse_next(input)?;

    Ok(out)
}

/// An arbitrary typed attribute that's potentially specific to a certain architecture.
///
/// If no architecture is provided `any` (the current architecture) is assumed.
#[derive(Debug)]
pub struct ArchProperty<T> {
    pub architecture: Option<Architecture>,
    pub value: T,
}

/// Parse the architecture suffix of a keyword, if it exists.
/// If no architecture suffix is found, return `None`.
///
/// Example:
/// ```txt
/// sha256sums_i386 = 0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0
///           ^^^^^
///         This is the suffix with `i386` being the architecture.
/// ```
fn architecture_suffix(input: &mut &str) -> PResult<Option<Architecture>> {
    // First up, check if there's an underscore.
    // If there's none, there's no suffix and we can return early.
    let underscore = opt('_').parse_next(input)?;
    if underscore.is_none() {
        return Ok(None);
    }

    // There has been an underscore, so now we **expect** an architecture to be there and we have
    // to fail hard if that doesn't work.
    // We now grab all content until the expected space of the delimiter and map it to an
    // [Architecture].
    let architecture =
        cut_err(take_till(0.., |c| c == ' ' || c == '=').try_map(Architecture::from_str))
            .context(StrContext::Label("architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "a well-known architecture suffix. I.e. '_i386` or `_x86_64`",
            )))
            .parse_next(input)?;

    Ok(Some(architecture))
}

/// Track empty/comment lines
#[derive(Debug)]
pub enum Ignored {
    EmptyLine,
    Comment(String),
}

/// This enum is able to represent all high-level components of a parsed SRCINFO file.
#[derive(Debug)]
pub struct SourceInfoContent {
    /// Track empty/comment lines that aren't in the context of a pkgbuild/pkgname section.
    pub preceding_lines: Vec<Ignored>,

    pub package_base: RawPackageBase,
    pub packages: Vec<RawPackage>,
}

/// Parse the start of the file in case it contains one or more empty lines or comment lines.
///
/// This consumes the first few lines until the `pkgbase` section is hit.
/// Further comments and newlines are handled in the scope of the respective `pkgbase`/`pkgname`
/// sections.
fn preceding_lines(input: &mut &str) -> PResult<Ignored> {
    trace(
        "preceding_lines",
        alt((
            terminated(("#", take_until(0.., "\n")).take(), line_ending)
                .map(|s: &str| Ignored::Comment(s.to_string())),
            terminated(space0, line_ending).map(|_s: &str| Ignored::EmptyLine),
        )),
    )
    .parse_next(input)
}

/// Parse a given .srcinfo file input.
pub fn srcinfo(input: &mut &str) -> PResult<SourceInfoContent> {
    // Handle any comments or empty lines at the start of the line..
    let preceding_lines: Vec<Ignored> = repeat(0.., preceding_lines).parse_next(input)?;

    // At the first part of any SRCINFO file, a `pkgbase` section is expected which sets the base
    // metadata and the default values for all packages to come.
    let package_base = package_base.parse_next(input)?;

    // Afterwards one or multiple package declarations are to follow.
    let (packages, _eof): (Vec<RawPackage>, _) =
        repeat_till(1.., package, eof).parse_next(input)?;

    Ok(SourceInfoContent {
        preceding_lines,
        package_base,
        packages,
    })
}

/// Represents the parsed content of a `pkgbase` section in a SRCINFO file.
#[derive(Debug)]
pub struct RawPackageBase {
    pub name: Name,
    pub properties: Vec<PackageBaseProperty>,
}

/// Parse the full `pkgbase` section of a SRCINFO file.
fn package_base(input: &mut &str) -> PResult<RawPackageBase> {
    // Get the name of the base package.
    let name = cut_err(
        delimited("pkgbase = ", take_till(1.., |c| c == '\n'), line_ending).try_map(Name::from_str),
    )
    .context(StrContext::Label("package base name"))
    .context(StrContext::Expected(StrContextValue::Description(
        "the name of the base package",
    )))
    .parse_next(input)?;

    // Go through the lines after the initial `pkgbase` statement.
    //
    // We explicitly use `repeat` to allow backtracking from the inside.
    // The reason for this is that SRCINFO is no structured data format per se and we have no
    // clear indicator that a `pkgbase` section just stopped and a `pkgname` section started.
    //
    // The only way to detect this is to look for the `pkgname` keyword while parsing lines in
    // `package_base_line`. If that keyword is detected, we trigger a backtracking error that
    // results in this `repeat` call to wrap up and return successfully.
    let properties: Vec<PackageBaseProperty> = repeat(0.., package_base_line).parse_next(input)?;

    Ok(RawPackageBase { name, properties })
}

/// Represents the parsed content of a single `pkgname` section in a SRCINFO file.
#[derive(Debug)]
pub struct RawPackage {
    pub name: Name,
    pub properties: Vec<PackageProperty>,
}

/// Parse a single full `pkgname` section of a SRCINFO file.
fn package(input: &mut &str) -> PResult<RawPackage> {
    // Get the name of the base package.
    let name = cut_err(
        delimited("pkgname = ", take_till(1.., |c| c == '\n'), line_ending).try_map(Name::from_str),
    )
    .context(StrContext::Label("package name"))
    .context(StrContext::Expected(StrContextValue::Description(
        "the name of a package",
    )))
    .parse_next(input)?;

    // Go through the lines after the initial `pkgname` statement.
    //
    // We explicitly use `repeat` to allow backtracking from the inside.
    // The reason for this is that SRCINFO is no structured data format per se and we have no
    // clear indicator that the current `pkgname` section just stopped and a new `pkgname` section
    // started.
    //
    // The only way to detect this is to look for the `pkgname` keyword while parsing lines in
    // `package_line`. If that keyword is detected, we trigger a backtracking error that
    // results in this `repeat` call to wrap up and return successfully.
    let properties: Vec<PackageProperty> = repeat(0.., package_line).parse_next(input)?;

    Ok(RawPackage { name, properties })
}

/// This enum can represents all possible propetries of a `pkgbase` section in a SRCINFO file.
///
/// The lines have the identical order in which they appear in the SRCINFO file, which is important
/// as the file is stateful and we need to normalize the data in the next step.
///
/// Sadly we have to do it this way as the format theoretically allows comments and empty lines at
/// any given time. To produce meaningful error messages during the normalization step, we need to
/// know the line number on which the error occurred, which is why we have to encode that info into
/// the parsed data.
#[derive(Debug)]
pub enum PackageBaseProperty {
    // Track empty/comment lines.
    EmptyLine,
    Comment(String),

    // ---- Package Metadata ----
    MetaProperty(SharedMetaProperty),

    // These metadata fields are PackageBase specific
    PackageVersion(PackageVersion),
    PackageRelease(PackageRelease),
    PackageEpoch(Epoch),
    ValidPgpKeys(OpenPGPIdentifier),

    // ---- Package Relations ----
    // This wraps all package relations that can be set in both pkgbase and pkgname.
    RelationProperty(RelationProperty),
    // The following dependencies are build-time specific dependencies.
    // `makepkg` expects all dependencies for all split packages to be specified in the
    // PackageBase.
    CheckDependency(ArchProperty<PackageRelation>),
    MakeDependency(ArchProperty<PackageRelation>),

    // ---- Source file properties ----
    SourceProperty(SourceProperty),
}

/// Parse keywords that're exclusive to the `pkgbase` section.
///
/// This function backtracks in case no keyword in this group matches.
fn exclusive_package_base_property(input: &mut &str) -> PResult<PackageBaseProperty> {
    // First off, get the type of the property.
    let property_type = trace(
        "exclusive_pkgbase_property",
        alt((
            "checkdepends",
            "makedepends",
            "pkgver",
            "pkgrel",
            "epoch",
            "validpgpkeys",
        )),
    )
    .context(StrContext::Label("file property type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "'type', 'uid', 'gid', 'mode', 'size', 'link', 'md5digest', 'sha256digest' or 'time'",
    )))
    .parse_next(input)?;

    // Parse a possible architecture suffix for architecture specific fields.
    let architecture = if property_type == "checkdepends" || property_type == "makedepends" {
        architecture_suffix.parse_next(input)?
    } else {
        None
    };

    // Expect the ` = ` separator between the key-value pair
    let _ = delimiter.parse_next(input)?;

    let property = match property_type {
        "pkgver" => cut_err(till_line_end.try_map(|s| {
            let version = PackageVersion::from_str(s)?;
            Ok::<PackageBaseProperty, alpm_types::Error>(PackageBaseProperty::PackageVersion(
                version,
            ))
        }))
        .parse_next(input)?,
        "pkgrel" => cut_err(till_line_end.try_map(|s| {
            let release = PackageRelease::from_str(s)?;
            Ok::<PackageBaseProperty, alpm_types::Error>(PackageBaseProperty::PackageRelease(
                release,
            ))
        }))
        .parse_next(input)?,

        "epoch" => cut_err(till_line_end.try_map(|s| {
            let epoch = Epoch::from_str(s)?;
            Ok::<PackageBaseProperty, alpm_types::Error>(PackageBaseProperty::PackageEpoch(epoch))
        }))
        .parse_next(input)?,
        "validpgpkeys" => cut_err(till_line_end.try_map(|s| {
            let fingerprint = OpenPGPIdentifier::from_str(s)?;
            Ok::<PackageBaseProperty, alpm_types::Error>(PackageBaseProperty::ValidPgpKeys(
                fingerprint,
            ))
        }))
        .parse_next(input)?,

        // Handle `pkgbase` specific package relations.
        "checkdepends" | "makedepends" => {
            // Read and parse the generic architecture specific PackageRelation.
            let value =
                cut_err(till_line_end.try_map(PackageRelation::from_str)).parse_next(input)?;
            let arch_property = ArchProperty {
                architecture,
                value,
            };

            // Now map the generic relation to the specific relation type.
            match property_type {
                "checkdepends" => PackageBaseProperty::CheckDependency(arch_property),
                "makedepends" => PackageBaseProperty::MakeDependency(arch_property),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    };

    Ok(property)
}

/// Handle any line in a package base file.
///
/// This is a wrapper to separate the logic between comments/empty lines and actual package base
/// properties.
fn package_base_line(input: &mut &str) -> PResult<PackageBaseProperty> {
    // Trim any leading spaces, which are allowed per spec.
    let _ = space0.parse_next(input)?;

    // Look for the `pkgbase` exit condition, which is the start of a `pkgname` section.
    // Read the docs above where this function is called for more info.
    let pkgname = peek(opt("pkgname")).parse_next(input)?;
    if pkgname.is_some() {
        // If we find a `pkgname` keyword, we know that the current `pkgbase` section finished.
        // Return a backtrack so the calling parser may wrap up and we can continue with `pkgname`
        // parsing.
        return Err(ErrMode::Backtrack(ParserError::from_error_kind(
            input,
            ErrorKind::Fail,
        )));
    }

    trace(
        "package_base_line",
        alt((
            // First of handle any empty lines or comments.
            preceded(("#", take_until(0.., "\n")), line_ending)
                .map(|s: &str| PackageBaseProperty::Comment(s.to_string())),
            preceded(space0, line_ending).map(|_| PackageBaseProperty::EmptyLine),
            // In case we got text, start parsing properties
            package_base_property,
            cut_err(fail)
                .context(StrContext::Label("package base property"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "an empty line, comment or one of the allowed base package properties",
                ))),
        )),
    )
    .parse_next(input)
}

/// At this point, we encountered some text in a pkgbase section.
///
/// Since there're a lot of keywords and many of them are shared between the `pkgbase` and
/// `pkgname` section, the keywords are bundled into somewhat logical groups.
///
/// - [SourceProperty] are keywords that're related to the `source` keyword, such as checksums.
/// - [SharedMetaProperty] are keywords that're related to meta general properties of the package.
/// - [RelationProperty] are keywords that describe the relation of the package to other packages.
///   [RawPackageBase] has a two special relations that're explicitly handled in the
///   [RawPackageBase] enum.
/// - Other fields that're unique to the [RawPackageBase] are handled in
///   [`exclusive_package_base_property`].
fn package_base_property(input: &mut &str) -> PResult<PackageBaseProperty> {
    // First off, get the type of the property.
    trace("pkgbase_property", alt((
        source_property.map(PackageBaseProperty::SourceProperty),
        shared_property.map(PackageBaseProperty::MetaProperty),
        shared_package_relation_property.map(PackageBaseProperty::RelationProperty),
        exclusive_package_base_property,
        fail.context(StrContext::Label("file property type"))
            .context(StrContext::Expected(StrContextValue::Description(
            "'type', 'uid', 'gid', 'mode', 'size', 'link', 'md5digest', 'sha256digest' or 'time'",
        ))),
    )))
    .parse_next(input)
}

/// This enum can represents all possible propetries of a `pkgname` section in a SRCINFO file.
///
/// It's very similar to [`RawPackageBase`], but with less fields and the possibility to explicitly
/// set some fields to "empty".
/// Read the [`RawPackageBase`] docs for more context.
#[derive(Debug)]
pub enum PackageProperty {
    // Track empty/comment lines.
    EmptyLine,
    Comment(String),

    // ---- Package Metadata ----
    MetaProperty(SharedMetaProperty),

    // ---- Package Relations ----
    // This wraps all package relations that can be set in both pkgbase and pkgname.
    RelationProperty(RelationProperty),

    // ---- Clearable Fields ----
    // An indicator that a specific field should be explicitly cleared and be left empty.
    // This includes not falling back to any PackageBase fields.
    Clear(ClearableProperty),
}

/// At this point, we encountered some text in a pkgname section.
///
/// Since there're a lot of keywords and many of them are shared between the `pkgbase` and
/// `pkgname` section, the keywords are bundled into somewhat logical groups.
///
/// - [SourceProperty] are keywords that're related to the `source` keyword, such as checksums.
/// - [SharedMetaProperty] are keywords that're related to meta general properties of the package.
/// - [RelationProperty] are keywords that describe the relation of the package to other packages.
///   [RawPackageBase] has a two special relations that're explicitly handled in the
///   [RawPackageBase] enum.
/// - Other fields that're unique to the [RawPackageBase] are handled in
///   [`exclusive_package_base_property`].
fn package_property(input: &mut &str) -> PResult<PackageProperty> {
    // The way we handle `ClearableProperty` is a bit imperformant.
    // Since clearable properties are only allowed to occur in `pkgname` sections, I decided to not
    // handle clearable properties in the respective property parsers to keep the code as
    // reusable between `pkgbase` and `pkgname` as possible.
    //
    // Hence, we do a check for any clearable properties at the very start. If none is detected,
    // the actual property setters will be checked afterwards.
    // This means that every property is preceded by `clearable_property` pass.
    //
    // I don't expect that this will result in any significant performance issues, but **if** this
    // were to ever become an issue, it would be a good start to clone all `*_property` parser
    // functions into two functions, where one of them explicitly handles clearable properties.
    trace("pkgname_property", alt((
        clearable_property.map(PackageProperty::Clear),
        shared_property.map(PackageProperty::MetaProperty),
        shared_package_relation_property.map(PackageProperty::RelationProperty),
        fail.context(StrContext::Label("file property type"))
            .context(StrContext::Expected(StrContextValue::Description(
            "'type', 'uid', 'gid', 'mode', 'size', 'link', 'md5digest', 'sha256digest' or 'time'",
        ))),
    )))
    .parse_next(input)
}

/// Handle any line in a `pkgname` package section.
///
/// This is a wrapper to separate the logic between comments/empty lines and actual package
/// properties.
fn package_line(input: &mut &str) -> PResult<PackageProperty> {
    // Trim any leading spaces, which are allowed per spec.
    let _ = space0.parse_next(input)?;

    // Look for one of the `pkgname` exit conditions, which is the start of a nw `pkgname` section.
    // Read the docs above where this function is called for more info.
    let pkgname = peek(opt("pkgname")).parse_next(input)?;
    if pkgname.is_some() {
        // If we find a `pkgname` keyword, we know that the current `pkgname` section finished.
        // Return a backtrack so the calling parser may wrap up.
        return Err(ErrMode::Backtrack(ParserError::from_error_kind(
            input,
            ErrorKind::Fail,
        )));
    }

    // Check if we're at the end of the file.
    // If so, throw a backtrack error.
    // TODO: Check if there's a cleaner way of doing this.
    let eof_found = opt(eof).parse_next(input)?;
    if eof_found.is_some() {
        return Err(ErrMode::Backtrack(ParserError::from_error_kind(
            input,
            ErrorKind::Complete,
        )));
    }

    trace(
        "package_line",
        alt((
            // First of handle any empty lines or comments, which might also occur at the
            // end of the file.
            preceded(("#", take_until(0.., "\n")), alt((line_ending, eof)))
                .map(|s: &str| PackageProperty::Comment(s.to_string())),
            preceded(space0, alt((line_ending, eof))).map(|_| PackageProperty::EmptyLine),
            // In case we got text, start parsing properties
            package_property,
            cut_err(fail)
                .context(StrContext::Label("package property"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "an empty line, comment or one of the allowed package properties",
                ))),
        )),
    )
    .parse_next(input)
}

#[derive(Debug)]
pub enum SharedMetaProperty {
    Description(PackageDescription),
    Url(Url),
    License(License),
    Architecture(Architecture),
    Changelog(RelativePath),

    // Build or package management related meta fields
    Install(RelativePath),
    Group(String),
    Option(MakepkgOption),
    Backup(RelativePath),
}

/// Parse the available relation property keywords.
fn shared_property_keyword<'s>(input: &mut &'s str) -> PResult<&'s str> {
    trace(
        "shared_meta_property_keyword",
        alt((
            "pkgdesc",
            "url",
            "license",
            "arch",
            "changelog",
            "install",
            "groups",
            "options",
            "backup",
        )),
    )
    .parse_next(input)
}

/// Parse generic package metadata keywords that may shop up in both the `pkgbase` and `pkgname`
/// sections.
///
/// This function backtracks in case no keyword in this group matches.
fn shared_property(input: &mut &str) -> PResult<SharedMetaProperty> {
    // Now get the type of the property.
    let property_type = shared_property_keyword.parse_next(input)?;

    // Expect the ` = ` separator between the key-value pair
    let _ = delimiter.parse_next(input)?;

    let property = match property_type {
        "pkgdesc" => cut_err(
            till_line_end.map(|s| SharedMetaProperty::Description(PackageDescription::from(s))),
        )
        .parse_next(input)?,
        "url" => cut_err(till_line_end.try_map(|s| {
            let url = Url::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Url(url))
        }))
        .parse_next(input)?,
        "license" => cut_err(till_line_end.try_map(|s| {
            let license = License::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::License(license))
        }))
        .parse_next(input)?,
        "arch" => cut_err(till_line_end.try_map(|s| {
            let architecture = Architecture::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Architecture(
                architecture,
            ))
        }))
        .parse_next(input)?,
        "changelog" => cut_err(till_line_end.try_map(|s| {
            let changelog = Changelog::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Changelog(changelog))
        }))
        .parse_next(input)?,
        "install" => cut_err(till_line_end.try_map(|s| {
            let install = Install::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Install(install))
        }))
        .parse_next(input)?,
        "groups" => cut_err(till_line_end.map(|s| SharedMetaProperty::Group(Group::from(s))))
            .parse_next(input)?,
        "options" => cut_err(till_line_end.try_map(|s| {
            let option = PackageOption::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Option(option))
        }))
        .parse_next(input)?,
        "backup" => cut_err(till_line_end.try_map(|s| {
            let backup = Backup::from_str(s)?;
            Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Backup(backup))
        }))
        .parse_next(input)?,
        _ => unreachable!(),
    };

    Ok(property)
}

/// Properties related to package relations.
///
/// This only handles the shared package relations that can be set in both `pkgbase` and `pkgname`
/// sections. `pkgbase` specific relations are explicitly handled in the [`RawPackageBase`] enum.
#[derive(Debug)]
pub enum RelationProperty {
    Dependency(ArchProperty<PackageRelation>),
    OptionalDependency(ArchProperty<OptionalDependency>),
    Provides(ArchProperty<PackageRelation>),
    Conflicts(ArchProperty<PackageRelation>),
    Replaces(ArchProperty<PackageRelation>),
}

/// Parse the available relation property keywords.
fn relation_property_keyword<'s>(input: &mut &'s str) -> PResult<&'s str> {
    trace(
        "relation_property_keyword",
        alt(("depends", "optdepends", "provides", "conflicts", "replaces")),
    )
    .parse_next(input)
}

/// Parses shared properties that relate to the `source` keyword.
/// Shared properties means that these can appear in both `pkgbase` and `pkgname` sections.
///
/// This function backtracks in case no keyword in this group matches.
fn shared_package_relation_property(input: &mut &str) -> PResult<RelationProperty> {
    // First off, get the type of the property.
    let property_type = relation_property_keyword.parse_next(input)?;

    // All of these properties can be architecture specific and may have an architecture suffix.
    // Get it if there's one.
    let architecture = architecture_suffix.parse_next(input)?;

    // Expect the ` = ` separator between the key-value pair
    let _ = delimiter.parse_next(input)?;

    let property = match property_type {
        // Handle these together in a single blob as they all deserialize to the same base type.
        "depends" | "replaces" | "conflicts" | "provides" => {
            // Read and parse the generic architecture specific PackageRelation.
            let value =
                cut_err(till_line_end.try_map(PackageRelation::from_str)).parse_next(input)?;
            let arch_property = ArchProperty {
                architecture,
                value,
            };

            // Now map the generic relation to the specific relation type.
            match property_type {
                "depends" => RelationProperty::Dependency(arch_property),
                "replaces" => RelationProperty::Replaces(arch_property),
                "conflicts" => RelationProperty::Conflicts(arch_property),
                "provides" => RelationProperty::Provides(arch_property),
                _ => unreachable!(),
            }
        }
        "optdepends" => cut_err(till_line_end.try_map(|s| {
            let value = OptionalDependency::from_str(s)?;
            Ok::<RelationProperty, alpm_types::Error>(RelationProperty::OptionalDependency(
                ArchProperty {
                    architecture,
                    value,
                },
            ))
        }))
        .parse_next(input)?,
        _ => unreachable!(),
    };

    Ok(property)
}

/// Source file related properties
/// Sources and related properties can be architecture specific.
///
/// `source`s, `noextract` and checksums statements are highly correlated.
/// `noextract` and checksums are ordered in the same way as the respective sources.
/// This will be normalized into a better representation in the next step after parsing.
///
/// These properties can be set in both `pkgname` and `pkgbase`.
#[derive(Debug)]
pub enum SourceProperty {
    Source(ArchProperty<Source>),
    NoExtract(ArchProperty<String>),
    B2Checksum(ArchProperty<SkippableChecksum<Blake2b512>>),
    Md5Checksum(ArchProperty<SkippableChecksum<Md5>>),
    Sha1Checksum(ArchProperty<SkippableChecksum<Sha1>>),
    Sha256Checksum(ArchProperty<SkippableChecksum<Sha256>>),
    Sha224Checksum(ArchProperty<SkippableChecksum<Sha224>>),
    Sha384Checksum(ArchProperty<SkippableChecksum<Sha384>>),
    Sha512Checksum(ArchProperty<SkippableChecksum<Sha512>>),
}

/// Parse the available source property keywords.
fn source_property_keyword<'s>(input: &mut &'s str) -> PResult<&'s str> {
    trace(
        "source_property_keyword",
        alt((
            "source",
            "noextract",
            "b2sums",
            "md5sums",
            "sha1sums",
            "sha224sums",
            "sha256sums",
            "sha384sums",
            "sha512sums",
        )),
    )
    .parse_next(input)
}

/// Parses shared properties that relate to the `source` keyword.
/// Shared properties means that these can appear in both `pkgbase` and `pkgname` sections.
///
/// This includes checksums and `noextract` statements.
///
/// This function backtracks in case no keyword in this group matches.
fn source_property(input: &mut &str) -> PResult<SourceProperty> {
    // First off, get the type of the property.
    let property_type = source_property_keyword.parse_next(input)?;

    // All properties may be architecture specific and thereby have an architecture suffix.
    let architecture = architecture_suffix.parse_next(input)?;

    // Expect the ` = ` separator between the key-value pair
    let _ = delimiter.parse_next(input)?;

    let property = match property_type {
        "source" => cut_err(till_line_end.try_map(|s| {
            let value = Source::from_str(s)?;
            Ok::<SourceProperty, alpm_types::Error>(SourceProperty::Source(ArchProperty {
                architecture,
                value,
            }))
        }))
        .parse_next(input)?,
        "noextract" => cut_err(till_line_end.map(|s| {
            SourceProperty::NoExtract(ArchProperty {
                architecture,
                value: s.to_string(),
            })
        }))
        .parse_next(input)?,
        // Handle all checksums in one block as there's a lot of common logic.
        // Most notably, all checksums are `SKIP`pable, which means that we have to check at the
        // very first step if said checksum is to be skipped before we try to parse the input
        // as a checksum.
        "b2sums" | "md5sums" | "sha1sums" | "sha224sums" | "sha256sums" | "sha384sums"
        | "sha512sums" => cut_err(till_line_end.try_map(|s| {
            // Handle the case where we get a `SKIP` instruction for one of the checksums.
            if s == "SKIP" {
                let property: SourceProperty = match property_type {
                    "b2sums" => SourceProperty::B2Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "md5sums" => SourceProperty::Md5Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "sha1sums" => SourceProperty::Sha1Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "sha224sums" => SourceProperty::Sha224Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "sha256sums" => SourceProperty::Sha256Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "sha384sums" => SourceProperty::Sha384Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    "sha512sums" => SourceProperty::B2Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Skip,
                    }),
                    _ => unreachable!(),
                };
                return Ok::<SourceProperty, alpm_types::Error>(property);
            }

            // We seem to have gotten a real checksum
            let property: SourceProperty = match property_type {
                "b2sums" => {
                    let checksum = Blake2b512Checksum::from_str(s)?;
                    SourceProperty::B2Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "md5sums" => {
                    let checksum = Md5Checksum::from_str(s)?;
                    SourceProperty::Md5Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "sha1sums" => {
                    let checksum = Sha1Checksum::from_str(s)?;
                    SourceProperty::Sha1Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "sha224sums" => {
                    let checksum = Sha224Checksum::from_str(s)?;
                    SourceProperty::Sha224Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "sha256sums" => {
                    let checksum = Sha256Checksum::from_str(s)?;
                    SourceProperty::Sha256Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "sha384sums" => {
                    let checksum = Sha384Checksum::from_str(s)?;
                    SourceProperty::Sha384Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                "sha512sums" => {
                    let checksum = Sha512Checksum::from_str(s)?;
                    SourceProperty::Sha512Checksum(ArchProperty {
                        architecture,
                        value: SkippableChecksum::Checksum(checksum),
                    })
                }
                _ => unreachable!(),
            };

            Ok::<SourceProperty, alpm_types::Error>(property)
        }))
        .parse_next(input)?,
        _ => unreachable!(),
    };

    Ok(property)
}

/// This enum represents all list fields that can be cleared in a `pkgname` context.
///
/// Some of these fields are architecture specific as they might only be cleared for a specific
/// architecture, but not for others.
#[derive(Debug)]
pub enum ClearableProperty {
    Description,
    Url,
    Licenses,
    Changelog,

    Install,
    Groups,
    Options,
    Backups,

    Dependencies(Option<Architecture>),
    OptionalDependencies(Option<Architecture>),
    Provides(Option<Architecture>),
    Conflicts(Option<Architecture>),
    Replaces(Option<Architecture>),
}

/// Parses any property that can may be "clearable".
/// A cleared property means that it's set to an empty value and should remain empty for a given
/// package.
///
/// Example:
/// ```txt
/// source=
/// sha512sums=
/// ```
///
/// The above properties would indicate that both source and the sha512sums array are to be cleared
/// and left empty.
///
/// This function backtracks in case no keyword in this group matches or in case the property is
/// not cleared..
fn clearable_property(input: &mut &str) -> PResult<ClearableProperty> {
    // First off, check if this is any of the clearable properties.
    let property_type = trace(
        "clearable_property",
        alt((
            shared_property_keyword,
            relation_property_keyword,
            source_property_keyword,
        )),
    )
    .parse_next(input)?;

    // Some properties may be unset in the context of a specific architecture.
    // Parse the optional architecture for those properties.
    let architecture = if ["depends", "optdepends", "provides", "conflicts", "replaces"]
        .contains(&property_type)
    {
        architecture_suffix.parse_next(input)?
    } else {
        None
    };

    // Now check if it's actually a clear.
    // This parser fails and backtracks in case there's anything but spaces and a newline after the
    // delimiter, which indicates that there's an actual value that is set for this property.
    let _ = (" =", space0, newline).parse_next(input)?;

    let property = match property_type {
        "pkgdesc" => ClearableProperty::Description,
        "url" => ClearableProperty::Url,
        "license" => ClearableProperty::Licenses,
        "changelog" => ClearableProperty::Changelog,
        "install" => ClearableProperty::Install,
        "groups" => ClearableProperty::Groups,
        "options" => ClearableProperty::Options,
        "backup" => ClearableProperty::Backups,
        "depends" => ClearableProperty::Dependencies(architecture),
        "optdepends" => ClearableProperty::OptionalDependencies(architecture),
        "provides" => ClearableProperty::Provides(architecture),
        "conflicts" => ClearableProperty::Conflicts(architecture),
        "replaces" => ClearableProperty::Replaces(architecture),
        _ => unreachable!(),
    };

    Ok(property)
}
