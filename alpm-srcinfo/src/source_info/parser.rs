//! The parser for SRCINFO data.
//!
//! It returns a rather raw line-based, but already typed representation of the contents.
//! The representation is not useful for end-users as it provides data that is not yet validated.
use std::str::FromStr;

use alpm_parsers::iter_str_context;
use alpm_types::{
    Architecture,
    Backup,
    Changelog,
    Epoch,
    Group,
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
    RelativeFilePath,
    SkippableChecksum,
    Source,
    Url,
    digests::{Blake2b512, Crc32Cksum, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
};
use strum::{EnumString, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    ascii::{alpha1, alphanumeric1, line_ending, multispace0, newline, space0, till_line_ending},
    combinator::{
        alt,
        cut_err,
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
    error::{ErrMode, ParserError, StrContext, StrContextValue},
    token::{take_till, take_until},
};

/// Recognizes the ` = ` delimiter between keywords.
///
/// This function expects the delimiter to exist.
fn delimiter<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    cut_err(" = ")
        .context(StrContext::Label("delimiter"))
        .context(StrContext::Expected(StrContextValue::Description(
            "an equal sign surrounded by spaces: ' = '.",
        )))
        .parse_next(input)
}

/// Recognizes all content until the end of line.
///
/// This function is called after a ` = ` has been recognized using [`delimiter`].
/// It extends upon winnow's [`till_line_ending`] by also consuming the newline character.
/// [`till_line_ending`]: <https://docs.rs/winnow/latest/winnow/ascii/fn.till_line_ending.html>
fn till_line_end<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    // Get the content til the end of line.
    let out = till_line_ending.parse_next(input)?;

    // Consume the newline. This is `opt` in case we hit `eof`, which is also handled by winnow's
    // `till_line_ending`
    opt(line_ending).parse_next(input)?;

    Ok(out)
}

/// An arbitrarily typed attribute that is specific to an [alpm-architecture].
///
/// This type is designed to wrap **any** type that is architecture specific.
/// For example, all checksums may be architecture specific.
///
/// # Example
///
/// ```text
/// # Without architecture
/// sha256 = 0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0
///
/// # With architecture
/// sha256_x86_64 = 0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0
/// ```
///
/// The above would be reflected by the following code.
/// ```
/// use std::str::FromStr;
///
/// use alpm_srcinfo::source_info::parser::ArchProperty;
/// use alpm_types::{Sha256Checksum, SystemArchitecture};
///
/// # fn main() -> Result<(), alpm_srcinfo::Error> {
/// let without_architecture = ArchProperty {
///     architecture: None,
///     value: Sha256Checksum::from_str(
///         "0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0",
///     )?,
/// };
///
/// let with_architecture = ArchProperty {
///     architecture: Some(SystemArchitecture::X86_64.into()),
///     value: Sha256Checksum::from_str(
///         "0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0",
///     )?,
/// };
///
/// # Ok(())
/// # }
/// ```
///
/// [alpm-architecture]: <https://alpm.archlinux.page/specifications/alpm-architecture.7.html>
#[derive(Debug)]
pub struct ArchProperty<T> {
    /// The optional [alpm-architecture] of the `value`.
    ///
    /// If `architecture` is [`None`] it is considered to be `"any"`.
    /// [alpm-architecture]: <https://alpm.archlinux.page/specifications/alpm-architecture.7.html>
    pub architecture: Option<Architecture>,
    /// The architecture specific type.
    pub value: T,
}

/// Recognizes and returns the architecture suffix of a keyword, if it exists.
///
/// Returns [`None`] if no architecture suffix is found.
///
/// ## Examples
/// ```txt
/// sha256sums_i386 = 0db1b39fd70097c6733cdcce56b1559ece5521ec1aad9ee1d520dda73eff03d0
///           ^^^^^
///         This is the suffix with `i386` being the architecture.
/// ```
pub fn architecture_suffix(input: &mut &str) -> ModalResult<Option<Architecture>> {
    // First up, check if there's an underscore.
    // If there's none, there's no suffix and we can return early.
    let underscore = opt('_').parse_next(input)?;
    if underscore.is_none() {
        return Ok(None);
    }

    // There has been an underscore, so now we **expect** an architecture to be there and we have
    // to fail hard if that doesn't work.
    // We now grab all content until the expected space of the delimiter and map it to an
    // alpm_types::Architecture.
    let architecture =
        cut_err(take_till(0.., |c| c == ' ' || c == '=').and_then(Architecture::parser))
            .parse_next(input)?;

    Ok(Some(architecture))
}

/// Track empty/comment lines
#[derive(Debug)]
pub enum Ignored {
    /// An empty line
    EmptyLine,

    /// A commented line.
    Comment(String),
}

/// A representation of all high-level components of parsed SRCINFO data.
#[derive(Debug)]
pub struct SourceInfoContent {
    /// Empty or comment lines that occur outside of `pkgbase` or `pkgname` sections.
    pub preceding_lines: Vec<Ignored>,
    /// The raw package base data.
    pub package_base: RawPackageBase,
    /// The list of raw package data.
    pub packages: Vec<RawPackage>,
}

impl SourceInfoContent {
    /// Parses the start of the file in case it contains one or more empty lines or comment lines.
    ///
    /// This consumes the first few lines until the `pkgbase` section is hit.
    /// Further comments and newlines are handled in the scope of the respective `pkgbase`/`pkgname`
    /// sections.
    fn preceding_lines_parser(input: &mut &str) -> ModalResult<Ignored> {
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

    /// Recognizes a complete SRCINFO file from a string slice.
    ///
    /// ```rust
    /// use alpm_srcinfo::source_info::parser::SourceInfoContent;
    /// use winnow::Parser;
    ///
    /// # fn main() -> Result<(), alpm_srcinfo::Error> {
    /// let source_info_data = r#"
    /// pkgbase = example
    ///     pkgver = 1.0.0
    ///     epoch = 1
    ///     pkgrel = 1
    ///     pkgdesc = A project that does something
    ///     url = https://example.org/
    ///     arch = x86_64
    ///     depends = glibc
    ///     optdepends = python: for special-python-script.py
    ///     makedepends = cmake
    ///     checkdepends = extra-test-tool
    ///
    /// pkgname = example
    ///     depends = glibc
    ///     depends = gcc-libs
    /// "#;
    ///
    /// // Parse the given srcinfo content.
    /// let parsed = SourceInfoContent::parser
    ///     .parse(source_info_data)
    ///     .map_err(|err| alpm_srcinfo::Error::ParseError(format!("{err}")))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parser(input: &mut &str) -> ModalResult<SourceInfoContent> {
        // Handle any comments or empty lines at the start of the line..
        let preceding_lines: Vec<Ignored> =
            repeat(0.., Self::preceding_lines_parser).parse_next(input)?;

        // At the first part of any SRCINFO file, a `pkgbase` section is expected which sets the
        // base metadata and the default values for all packages to come.
        let package_base = RawPackageBase::parser.parse_next(input)?;

        // Trim newlines or spaces between the pkgbase section and the following pkgname section.
        let _ = multispace0.parse_next(input)?;

        // Afterwards one or more `pkgname` declarations are to follow.
        //
        // `RawPackage::parser` expects all newlines and leading whitespaces to be trimmed.
        // This is explicitly done once at the start (see above) and implicitly via `terminated` in
        // between the repeats.
        multispace0.parse_next(input)?;
        let (packages, _eof): (Vec<RawPackage>, _) =
            repeat_till(0.., terminated(RawPackage::parser, multispace0), eof).parse_next(input)?;

        // Fail with a special error if there's no package section.
        if packages.is_empty() {
            fail.context(StrContext::Expected(StrContextValue::Description(
                "a pkgname section",
            )))
            .parse_next(input)?;
        }

        Ok(SourceInfoContent {
            preceding_lines,
            package_base,
            packages,
        })
    }
}

/// The parsed contents of a `pkgbase` section in SRCINFO data.
#[derive(Debug)]
pub struct RawPackageBase {
    /// The name of the `pkgbase` section.
    pub name: Name,
    /// The properties of the `pkbase` section.
    pub properties: Vec<PackageBaseProperty>,
}

impl RawPackageBase {
    /// Recognizes the entire `pkgbase` section in SRCINFO data.
    fn parser(input: &mut &str) -> ModalResult<RawPackageBase> {
        cut_err("pkgbase")
            .context(StrContext::Label("pkgbase section header"))
            .parse_next(input)?;

        cut_err(" = ")
            .context(StrContext::Label("pkgbase section header delimiter"))
            .context(StrContext::Expected(StrContextValue::Description("' = '")))
            .parse_next(input)?;

        // Get the name of the base package.
        // Don't use `till_line_ending`, as we want the name to have a length of at least one.
        let name = till_line_end
            .and_then(Name::parser)
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
        let properties: Vec<PackageBaseProperty> =
            repeat(0.., PackageBaseProperty::parser).parse_next(input)?;

        Ok(RawPackageBase { name, properties })
    }
}

/// The parsed contents of a `pkgname` section in SRCINFO data.
#[derive(Debug)]
pub struct RawPackage {
    /// The name of the `pkgname` section.
    pub name: Name,
    /// The properties of the `pkgname` section.
    pub properties: Vec<PackageProperty>,
}

impl RawPackage {
    /// Recognizes an entire single `pkgname` section in SRCINFO data.
    ///
    /// # Note
    ///
    /// This parser expects the cursor to directly start at the `pkgname` keyword.
    /// This means that the caller must trim any leading newlines or whitespaces.
    fn parser(input: &mut &str) -> ModalResult<RawPackage> {
        cut_err("pkgname")
            .context(StrContext::Label("pkgname section header"))
            .parse_next(input)?;

        cut_err(" = ")
            .context(StrContext::Label("pkgname section header delimiter"))
            .context(StrContext::Expected(StrContextValue::Description("' = '")))
            .parse_next(input)?;

        // Get the name of the base package.
        let name = till_line_end
            .and_then(Name::parser)
            .context(StrContext::Label("package name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "the name of a package",
            )))
            .parse_next(input)?;

        // Trim any leading whitespaces before the first pass of the `PackageProperty::parser`.
        space0.parse_next(input)?;

        // Go through the lines after the initial `pkgname` statement.
        //
        // # Usage of Backtracking
        //
        // We explicitly use `repeat` to allow backtracking from the inside.
        // The reason for this is that SRCINFO is no structured data format per se and we have no
        // clear indicator that the current `pkgname` section just stopped and a new `pkgname`
        // section started.
        //
        // The only way to detect this is to look for the `pkgname` keyword while parsing lines in
        // `package_line`. If that keyword is detected, we trigger a backtracking error that
        // results in this `repeat` call to wrap up and return successfully.
        //
        // # Whitespace handling
        //
        // `PackageProperty::parser` expects leading whitespaces of a line to be trimmed.
        // This is explicitly done once at the start (see above) and implicitly done via
        // `terminated` in between the repeats.
        let properties: Vec<PackageProperty> =
            repeat(0.., terminated(PackageProperty::parser, space0)).parse_next(input)?;

        Ok(RawPackage { name, properties })
    }
}

/// Keywords that are exclusive to the `pkgbase` section in SRCINFO data.
#[derive(Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum PackageBaseKeyword {
    /// Test dependencies.
    CheckDepends,
    /// Build dependencies.
    MakeDepends,
    /// An alpm-pkgver.
    PkgVer,
    /// An alpm-pkgrel.
    PkgRel,
    /// An alpm-epoch
    Epoch,
    /// Valid Openpgp keys.
    ValidPGPKeys,
}

impl PackageBaseKeyword {
    /// Recognizes a [`PackageBaseKeyword`] in an input string slice.
    pub fn parser(input: &mut &str) -> ModalResult<PackageBaseKeyword> {
        trace(
            "package_base_keyword",
            // Read until we hit something non alphabetical.
            // This could be either a space or a `_` in case there's an architecture specifier.
            alpha1.try_map(PackageBaseKeyword::from_str),
        )
        .parse_next(input)
    }
}

/// All possible properties of a `pkgbase` section in SRCINFO data.
///
/// The ordering of the variants represents the order in which keywords would appear in a SRCINFO
/// file. This is important as the file format represents stateful data which needs normalization.
///
/// The SRCINFO format allows comments and empty lines anywhere in the file.
/// To produce meaningful error messages for the consumer during data normalization, the line number
/// on which an error occurred is encoded in the parsed data.
#[derive(Debug)]
pub enum PackageBaseProperty {
    /// An empty line.
    EmptyLine,
    /// A commented line.
    Comment(String),
    /// A [`SharedMetaProperty`].
    MetaProperty(SharedMetaProperty),
    /// A [`PackageVersion`].
    PackageVersion(PackageVersion),
    /// A [`PackageRelease`].
    PackageRelease(PackageRelease),
    /// An [`Epoch`].
    PackageEpoch(Epoch),
    /// An [`OpenPGPIdentifier`].
    ValidPgpKeys(OpenPGPIdentifier),
    /// A [`RelationProperty`]
    RelationProperty(RelationProperty),
    /// Build-time specific check dependencies.
    CheckDependency(ArchProperty<PackageRelation>),
    /// Build-time specific make dependencies.
    MakeDependency(ArchProperty<PackageRelation>),
    /// Source file properties
    SourceProperty(SourceProperty),
}

impl PackageBaseProperty {
    /// Recognizes any line in the `pkgbase` section of SRCINFO data.
    ///
    /// This is a wrapper to separate the logic between comments/empty lines and actual `pkgbase`
    /// properties.
    fn parser(input: &mut &str) -> ModalResult<PackageBaseProperty> {
        // Trim any leading spaces, which are allowed per spec.
        let _ = multispace0.parse_next(input)?;

        // Look for the `pkgbase` exit condition, which is the start of a `pkgname` section or the
        // EOL if the pkgname section is missing.
        // Read the docs above where this function is called for more info.
        let pkgname = peek(opt(alt(("pkgname", eof)))).parse_next(input)?;
        if pkgname.is_some() {
            // If we find a `pkgname` keyword, we know that the current `pkgbase` section finished.
            // Return a backtrack so the calling parser may wrap up and we can continue with
            // `pkgname` parsing.
            return Err(ErrMode::Backtrack(ParserError::from_input(input)));
        }

        trace(
            "package_base_line",
            alt((
                // First of handle any empty lines or comments.
                preceded(("#", take_until(0.., "\n")), line_ending)
                    .map(|s: &str| PackageBaseProperty::Comment(s.to_string())),
                preceded(space0, line_ending).map(|_| PackageBaseProperty::EmptyLine),
                // In case we got text, start parsing properties
                Self::property_parser,
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword assignments in the `pkgbase` section in SRCINFO data.
    ///
    /// Since there're a lot of keywords and many of them are shared between the `pkgbase` and
    /// `pkgname` section, the keywords are bundled into somewhat logical groups.
    ///
    /// - [`SourceProperty`] are keywords that are related to the `source` keyword, such as
    ///   checksums.
    /// - [`SharedMetaProperty`] are keywords that are related to general meta properties of the
    ///   package.
    /// - [`RelationProperty`] are keywords that describe the relation of the package to other
    ///   packages. [`RawPackageBase`] has two special relations that are explicitly handled in
    ///   [`Self::exclusive_property_parser`].
    /// - Other fields that're unique to the [`RawPackageBase`] are handled in
    ///   [`Self::exclusive_property_parser`].
    fn property_parser(input: &mut &str) -> ModalResult<PackageBaseProperty> {
        // First off, get the type of the property.
        trace(
            "pkgbase_property",
            alt((
                SourceProperty::parser.map(PackageBaseProperty::SourceProperty),
                SharedMetaProperty::parser.map(PackageBaseProperty::MetaProperty),
                RelationProperty::parser.map(PackageBaseProperty::RelationProperty),
                PackageBaseProperty::exclusive_property_parser,
                cut_err(fail)
                    .context(StrContext::Label("package base property type"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed pkgbase section properties:",
                    )))
                    .context_with(iter_str_context!([
                        PackageBaseKeyword::VARIANTS,
                        RelationKeyword::VARIANTS,
                        SharedMetaKeyword::VARIANTS,
                        SourceKeyword::VARIANTS,
                    ])),
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword assignments exclusive to the `pkgbase` section in SRCINFO data.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn exclusive_property_parser(input: &mut &str) -> ModalResult<PackageBaseProperty> {
        // First off, get the type of the property.
        let keyword =
            trace("exclusive_pkgbase_property", PackageBaseKeyword::parser).parse_next(input)?;

        // Parse a possible architecture suffix for architecture specific fields.
        let architecture = match keyword {
            PackageBaseKeyword::MakeDepends | PackageBaseKeyword::CheckDepends => {
                architecture_suffix.parse_next(input)?
            }
            _ => None,
        };

        // Expect the ` = ` separator between the key-value pair
        let _ = delimiter.parse_next(input)?;

        let property = match keyword {
            PackageBaseKeyword::PkgVer => cut_err(
                till_line_end
                    .and_then(PackageVersion::parser)
                    .map(PackageBaseProperty::PackageVersion),
            )
            .parse_next(input)?,
            PackageBaseKeyword::PkgRel => cut_err(
                till_line_end
                    .and_then(PackageRelease::parser)
                    .map(PackageBaseProperty::PackageRelease),
            )
            .parse_next(input)?,

            PackageBaseKeyword::Epoch => cut_err(
                till_line_end
                    .and_then(Epoch::parser)
                    .map(PackageBaseProperty::PackageEpoch),
            )
            .parse_next(input)?,
            PackageBaseKeyword::ValidPGPKeys => cut_err(
                till_line_end
                    .try_map(OpenPGPIdentifier::from_str)
                    .map(PackageBaseProperty::ValidPgpKeys),
            )
            .parse_next(input)?,

            // Handle `pkgbase` specific package relations.
            PackageBaseKeyword::MakeDepends | PackageBaseKeyword::CheckDepends => {
                // Read and parse the generic architecture specific PackageRelation.
                let value =
                    cut_err(till_line_end.and_then(PackageRelation::parser)).parse_next(input)?;
                let arch_property = ArchProperty {
                    architecture,
                    value,
                };

                // Now map the generic relation to the specific relation type.
                match keyword {
                    PackageBaseKeyword::CheckDepends => {
                        PackageBaseProperty::CheckDependency(arch_property)
                    }
                    PackageBaseKeyword::MakeDepends => {
                        PackageBaseProperty::MakeDependency(arch_property)
                    }
                    _ => unreachable!(),
                }
            }
        };

        Ok(property)
    }
}

/// All possible properties of a `pkgname` section in SRCINFO data.
///
/// It's very similar to [`RawPackageBase`], but with less fields and the possibility to explicitly
/// set some fields to "empty".
#[derive(Debug)]
pub enum PackageProperty {
    /// An empty line.
    EmptyLine,
    /// A commented line.
    Comment(String),
    /// A [`SharedMetaProperty`].
    MetaProperty(SharedMetaProperty),
    /// A [`RelationProperty`].
    RelationProperty(RelationProperty),
    /// A [`ClearableProperty`].
    Clear(ClearableProperty),
}

impl PackageProperty {
    /// Handles any line in a `pkgname` package section.
    ///
    /// This is a wrapper to separate the logic between comments/empty lines and actual package
    /// properties.
    fn parser(input: &mut &str) -> ModalResult<PackageProperty> {
        // Look for one of the `pkgname` exit conditions, which is the start of a new `pkgname`
        // section. Read the docs above where this function is called for more info.
        let pkgname = peek(opt("pkgname")).parse_next(input)?;
        if pkgname.is_some() {
            // If we find a `pkgname` keyword, we know that the current `pkgname` section finished.
            // Return a backtrack so the calling parser may wrap up.
            return Err(ErrMode::Backtrack(ParserError::from_input(input)));
        }

        // Check if we're at the end of the file.
        // If so, throw a backtrack error.
        let eof_found = opt(eof).parse_next(input)?;
        if eof_found.is_some() {
            return Err(ErrMode::Backtrack(ParserError::from_input(input)));
        }

        trace(
            "package_line",
            alt((
                // First of handle any empty lines or comments, which might also occur at the
                // end of the file.
                preceded("#", till_line_end).map(|s: &str| PackageProperty::Comment(s.to_string())),
                line_ending.map(|_| PackageProperty::EmptyLine),
                // In case we got text, start parsing properties
                Self::property_parser,
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword assignments in a `pkgname` section in SRCINFO data.
    ///
    /// Since there're a lot of keywords and many of them are shared between the `pkgbase` and
    /// `pkgname` section, the keywords are bundled into somewhat logical groups.
    ///
    /// - [`SourceProperty`] are keywords that are related to the `source` keyword, such as
    ///   checksums.
    /// - [`SharedMetaProperty`] are keywords that are related to general meta properties of the
    ///   package.
    /// - [`RelationProperty`] are keywords that describe the relation of the package to other
    ///   packages. [`RawPackageBase`] has two special relations that are explicitly handled in that
    ///   enum.
    fn property_parser(input: &mut &str) -> ModalResult<PackageProperty> {
        // The way we handle `ClearableProperty` is a bit imperformant.
        // Since clearable properties are only allowed to occur in `pkgname` sections, I decided to
        // not handle clearable properties in the respective property parsers to keep the
        // code as reusable between `pkgbase` and `pkgname` as possible.
        //
        // Hence, we do a check for any clearable properties at the very start. If none is detected,
        // the actual property setters will be checked afterwards.
        // This means that every property is preceded by `clearable_property` pass.
        //
        // I don't expect that this will result in any significant performance issues, but **if**
        // this were to ever become an issue, it would be a good start to duplicate all
        // `*_property` parser functions, where one of them explicitly handles clearable properties.
        trace(
            "pkgname_property",
            alt((
                ClearableProperty::relation_parser.map(PackageProperty::Clear),
                ClearableProperty::shared_meta_parser.map(PackageProperty::Clear),
                SharedMetaProperty::parser.map(PackageProperty::MetaProperty),
                RelationProperty::parser.map(PackageProperty::RelationProperty),
                cut_err(fail)
                    .context(StrContext::Label("package property type"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed package section properties:",
                    )))
                    .context_with(iter_str_context!([
                        RelationKeyword::VARIANTS,
                        SharedMetaKeyword::VARIANTS
                    ])),
            )),
        )
        .parse_next(input)
    }
}

/// Keywords that may exist both in `pkgbase` and `pkgname` sections in SRCINFO data.
#[derive(Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum SharedMetaKeyword {
    /// The description of a package.
    PkgDesc,
    /// The upstream URL of a package.
    Url,
    /// The license of a package.
    License,
    /// The alpm-architecture of a package.
    Arch,
    /// The path to a changelog file of a package.
    Changelog,
    /// The path to an alpm-install-scriptlet of a package.
    Install,
    /// The alpm-package-groups a package is part of.
    Groups,
    /// The build tool options used when building a package.
    Options,
    /// The path of a file in a package that should be backed up.
    Backup,
}

impl SharedMetaKeyword {
    /// Recognizes a [`SharedMetaKeyword`] in a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<SharedMetaKeyword> {
        // Read until we hit something non alphabetical.
        // This could be either a space or a `_` in case there's an architecture specifier.
        trace(
            "shared_meta_keyword",
            alpha1.try_map(SharedMetaKeyword::from_str),
        )
        .parse_next(input)
    }
}

/// Metadata properties that may be shared between `pkgbase` and `pkgname` sections in SRCINFO data.
#[derive(Debug)]
pub enum SharedMetaProperty {
    /// A [`PackageDescription`].
    Description(PackageDescription),
    /// A [`Url`].
    Url(Url),
    /// A [`License`].
    License(License),
    /// An [`Architecture`].
    Architecture(Architecture),
    /// A [`RelativeFilePath`] for a changelog of a package.
    Changelog(RelativeFilePath),
    /// A [`RelativeFilePath`] for an alpm-install-scriptlet of a package.
    Install(RelativeFilePath),
    /// An alpm-package-group of a package.
    Group(String),
    /// A [`MakepkgOption`] used for building a package.
    Option(MakepkgOption),
    /// A [`RelativeFilePath`] for file in a package that should be backed up.
    Backup(RelativeFilePath),
}

impl SharedMetaProperty {
    /// Recognizes keyword assignments that may be present in both `pkgbase` and `pkgname` sections
    /// of SRCINFO data.
    ///
    /// This function relies on [`SharedMetaKeyword::parser`] to recognize the relevant keywords.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<SharedMetaProperty> {
        // Now get the type of the property.
        let keyword = SharedMetaKeyword::parser.parse_next(input)?;

        // Expect the ` = ` separator between the key-value pair
        let _ = delimiter.parse_next(input)?;

        let property = match keyword {
            SharedMetaKeyword::PkgDesc => cut_err(
                till_line_end.map(|s| SharedMetaProperty::Description(PackageDescription::from(s))),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Url => cut_err(
                till_line_end
                    .try_map(Url::from_str)
                    .map(SharedMetaProperty::Url),
            )
            .parse_next(input)?,
            SharedMetaKeyword::License => cut_err(
                till_line_end
                    .try_map(License::from_str)
                    .map(SharedMetaProperty::License),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Arch => cut_err(
                till_line_end
                    .and_then(Architecture::parser)
                    .map(SharedMetaProperty::Architecture),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Changelog => cut_err(
                till_line_end
                    .try_map(Changelog::from_str)
                    .map(SharedMetaProperty::Changelog),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Install => cut_err(
                till_line_end
                    .try_map(Install::from_str)
                    .map(SharedMetaProperty::Install),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Groups => {
                cut_err(till_line_end.map(|s| SharedMetaProperty::Group(Group::from(s))))
                    .parse_next(input)?
            }
            SharedMetaKeyword::Options => cut_err(
                till_line_end
                    .try_map(MakepkgOption::from_str)
                    .map(SharedMetaProperty::Option),
            )
            .parse_next(input)?,
            SharedMetaKeyword::Backup => cut_err(
                till_line_end
                    .try_map(Backup::from_str)
                    .map(SharedMetaProperty::Backup),
            )
            .parse_next(input)?,
        };

        Ok(property)
    }
}

/// Keywords that describe [alpm-package-relations].
///
/// [alpm-package-relations]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum RelationKeyword {
    /// A run-time dependency.
    Depends,
    /// An optional dependency.
    OptDepends,
    /// A provision.
    Provides,
    /// A conflict.
    Conflicts,
    /// A replacement.
    Replaces,
}

impl RelationKeyword {
    /// Recognizes a [`RelationKeyword`] in a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<RelationKeyword> {
        // Read until we hit something non alphabetical.
        // This could be either a space or a `_` in case there's an architecture specifier.
        trace(
            "relation_keyword",
            alpha1.try_map(RelationKeyword::from_str),
        )
        .parse_next(input)
    }
}

/// Properties related to package relations.
///
/// This only handles the shared package relations that can be used in both `pkgbase` and `pkgname`
/// sections.
/// `pkgbase` specific relations are explicitly handled in the [`RawPackageBase`] enum.
/// See [alpm-package-relation] for further details on package relations and [alpm-sonamev1] for
/// information on _soname_ handling.
/// [alpm-package-relation]: <https://alpm.archlinux.page/specifications/alpm-package-relation.7.html>
/// [alpm-sonamev1]: <https://alpm.archlinux.page/specifications/alpm-sonamev1.7.html>
#[derive(Debug)]
pub enum RelationProperty {
    /// An [`ArchProperty<RelationOrSoname>`] for a run-time dependency.
    Dependency(ArchProperty<RelationOrSoname>),
    /// An [`ArchProperty<OptionalDependency>`] for an optional dependency.
    OptionalDependency(ArchProperty<OptionalDependency>),
    /// An [`ArchProperty<RelationOrSoname>`] for a provision.
    Provides(ArchProperty<RelationOrSoname>),
    /// An [`ArchProperty<PackageRelation>`] for a conflict.
    Conflicts(ArchProperty<PackageRelation>),
    /// An [`ArchProperty<PackageRelation>`] for a replacement.
    Replaces(ArchProperty<PackageRelation>),
}

impl RelationProperty {
    /// Recognizes package relation keyword assignments that may be present in both `pkgbase` and
    /// `pkgname` sections in SRCINFO data.
    ///
    /// This function relies on [`RelationKeyword::parser`] to recognize the relevant keywords.
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<RelationProperty> {
        // First off, get the type of the property.
        let keyword = RelationKeyword::parser.parse_next(input)?;

        // All of these properties can be architecture specific and may have an architecture suffix.
        // Get it if there's one.
        let architecture = architecture_suffix.parse_next(input)?;

        // Expect the ` = ` separator between the key-value pair
        let _ = delimiter.parse_next(input)?;

        let property = match keyword {
            // Handle these together in a single blob as they all deserialize to the same base type.
            RelationKeyword::Conflicts | RelationKeyword::Replaces => {
                // Read and parse the generic architecture specific PackageRelation.
                let value =
                    cut_err(till_line_end.and_then(PackageRelation::parser)).parse_next(input)?;
                let arch_property = ArchProperty {
                    architecture,
                    value,
                };

                // Now map the generic relation to the specific relation type.
                match keyword {
                    RelationKeyword::Replaces => RelationProperty::Replaces(arch_property),
                    RelationKeyword::Conflicts => RelationProperty::Conflicts(arch_property),
                    _ => unreachable!(),
                }
            }
            RelationKeyword::Depends | RelationKeyword::Provides => {
                // Read and parse the generic architecture specific RelationOrSoname.
                let value =
                    cut_err(till_line_end.try_map(RelationOrSoname::from_str)).parse_next(input)?;
                let arch_property = ArchProperty {
                    architecture,
                    value,
                };

                // Now map the generic relation to the specific relation type.
                match keyword {
                    RelationKeyword::Depends => RelationProperty::Dependency(arch_property),
                    RelationKeyword::Provides => RelationProperty::Provides(arch_property),
                    _ => unreachable!(),
                }
            }
            RelationKeyword::OptDepends => cut_err(
                till_line_end
                    .and_then(OptionalDependency::parser)
                    .map(|value| {
                        RelationProperty::OptionalDependency(ArchProperty {
                            architecture: architecture.clone(),
                            value,
                        })
                    }),
            )
            .parse_next(input)?,
        };

        Ok(property)
    }

    /// Returns the [`Architecture`] of the current variant.
    ///
    /// Can be used to extract the architecture without knowing which variant this is.
    pub fn architecture(&self) -> Option<&Architecture> {
        match self {
            RelationProperty::Dependency(arch_property) => &arch_property.architecture,
            RelationProperty::OptionalDependency(arch_property) => &arch_property.architecture,
            RelationProperty::Provides(arch_property) => &arch_property.architecture,
            RelationProperty::Conflicts(arch_property) => &arch_property.architecture,
            RelationProperty::Replaces(arch_property) => &arch_property.architecture,
        }
        .as_ref()
    }
}

/// Package source keywords that are exclusive to the `pkgbase` section in SRCINFO data.
#[derive(Debug, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum SourceKeyword {
    /// A source entry.
    Source,
    /// A noextract entry.
    NoExtract,
    /// A blake2 hash digest.
    B2sums,
    /// An MD-5 hash digest.
    Md5sums,
    /// An SHA-1 hash digest.
    Sha1sums,
    /// An SHA-224 hash digest.
    Sha224sums,
    /// An SHA-256 hash digest.
    Sha256sums,
    /// An SHA-384 hash digest.
    Sha384sums,
    /// An SHA-512 hash digest.
    Sha512sums,
    /// An CRC-32/CKSUM hash digest.
    Cksums,
}

impl SourceKeyword {
    /// Parse a [`SourceKeyword`].
    pub fn parser(input: &mut &str) -> ModalResult<SourceKeyword> {
        // Read until we hit something non alphabetical.
        // This could be either a space or a `_` in case there's an architecture specifier.
        trace(
            "source_keyword",
            alphanumeric1.try_map(SourceKeyword::from_str),
        )
        .parse_next(input)
    }
}

/// Properties related to package sources.
///
/// Sources and related properties can be architecture specific.
///
/// The `source`, `noextract` and checksum related keywords in SRCINFO data correlate in ordering:
/// `noextract` and any checksum entries are ordered in the same way as the respective `source`
/// entry they relate to. The representation of this correlation is normalized after initial
/// parsing.
#[derive(Debug)]
pub enum SourceProperty {
    /// An [`ArchProperty<Source>`] for a source entry.
    Source(ArchProperty<Source>),
    /// An [`ArchProperty<String>`] for a noextract entry.
    NoExtract(String),
    /// An [`ArchProperty<SkippableChecksum<Blake2b512>>`] for a blake2 hash digest.
    B2Checksum(ArchProperty<SkippableChecksum<Blake2b512>>),
    /// An [`ArchProperty<SkippableChecksum<Md5>>`] for an MD-5 hash digest.
    Md5Checksum(ArchProperty<SkippableChecksum<Md5>>),
    /// An [`ArchProperty<SkippableChecksum<Sha1>>`] for a SHA-1 hash digest.
    Sha1Checksum(ArchProperty<SkippableChecksum<Sha1>>),
    /// An [`ArchProperty<SkippableChecksum<Sha256>>`] for a SHA-256 hash digest.
    Sha256Checksum(ArchProperty<SkippableChecksum<Sha256>>),
    /// An [`ArchProperty<SkippableChecksum<Sha224>>`] for a SHA-224 hash digest.
    Sha224Checksum(ArchProperty<SkippableChecksum<Sha224>>),
    /// An [`ArchProperty<SkippableChecksum<Sha384>>`] for a SHA-384 hash digest.
    Sha384Checksum(ArchProperty<SkippableChecksum<Sha384>>),
    /// An [`ArchProperty<SkippableChecksum<Sha512>>`] for a SHA-512 hash digest.
    Sha512Checksum(ArchProperty<SkippableChecksum<Sha512>>),
    /// An [`ArchProperty<SkippableChecksum<Crc32Cksum>>`] for a CRC-32/CKSUM hash digest.
    CrcChecksum(ArchProperty<SkippableChecksum<Crc32Cksum>>),
}

impl SourceProperty {
    /// Recognizes package source related keyword assignments in SRCINFO data.
    ///
    /// This function relies on [`SourceKeyword::parser`] to recognize the relevant keywords.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<SourceProperty> {
        // First off, get the type of the property.
        let keyword = SourceKeyword::parser.parse_next(input)?;

        let property = match keyword {
            SourceKeyword::NoExtract => {
                // Expect the ` = ` separator between the key-value pair
                let _ = delimiter.parse_next(input)?;

                cut_err(till_line_end.map(|s| SourceProperty::NoExtract(s.to_string())))
                    .parse_next(input)?
            }
            SourceKeyword::Source
            | SourceKeyword::B2sums
            | SourceKeyword::Md5sums
            | SourceKeyword::Sha1sums
            | SourceKeyword::Sha224sums
            | SourceKeyword::Sha256sums
            | SourceKeyword::Sha384sums
            | SourceKeyword::Sha512sums
            | SourceKeyword::Cksums => {
                // All other properties may be architecture specific and thereby have an
                // architecture suffix.
                let architecture = architecture_suffix.parse_next(input)?;

                // Expect the ` = ` separator between the key-value pair
                let _ = delimiter.parse_next(input)?;

                match keyword {
                    SourceKeyword::Source => {
                        cut_err(till_line_end.try_map(Source::from_str).map(|value| {
                            SourceProperty::Source(ArchProperty {
                                architecture: architecture.clone(),
                                value,
                            })
                        }))
                        .parse_next(input)?
                    }
                    // all checksum properties are parsed the same way.
                    SourceKeyword::B2sums => SourceProperty::B2Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Md5sums => SourceProperty::Md5Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Sha1sums => SourceProperty::Sha1Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Sha224sums => SourceProperty::Sha224Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Sha256sums => SourceProperty::Sha256Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Sha384sums => SourceProperty::Sha384Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Sha512sums => SourceProperty::Sha512Checksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::Cksums => SourceProperty::CrcChecksum(ArchProperty {
                        architecture,
                        value: till_line_end
                            .and_then(SkippableChecksum::parser)
                            .parse_next(input)?,
                    }),
                    SourceKeyword::NoExtract => unreachable!(),
                }
            }
        };

        Ok(property)
    }
}

/// Properties used in `pkgname` sections that can be cleared.
///
/// Some variants of this enum are architecture-specific, as they might only be cleared for a
/// specific architecture, but not for another.
///
/// Clearing a keyword in SRCINFO data is achieved by an empty keyword assignment, e.g.:
///
/// ```txt
/// depends =
/// ```
#[derive(Clone, Debug)]
pub enum ClearableProperty {
    /// The description for a package.
    Description,
    /// The upstream URL for a package.
    Url,
    /// The licenses that apply to a package.
    Licenses,
    /// The changelog for a package.
    Changelog,
    /// The alpm-install-scriptlet for a package.
    Install,
    /// The alpm-package-groups a package is part of.
    Groups,
    /// The build tool options used for building a package.
    Options,
    /// The path to a file in a package that should be backed up.
    Backups,
    /// The alpm-architecture of run-time dependencies.
    Dependencies(Option<Architecture>),
    /// The alpm-architecture of optional dependencies.
    OptionalDependencies(Option<Architecture>),
    /// The alpm-architecture of provisions.
    Provides(Option<Architecture>),
    /// The alpm-architecture of conflicts.
    Conflicts(Option<Architecture>),
    /// The alpm-architecture of replacements.
    Replaces(Option<Architecture>),
}

impl ClearableProperty {
    /// Recognizes all keyword assignments in SRCINFO data that represent a cleared
    /// [`SharedMetaProperty`].
    ///
    /// A cleared property is represented by a keyword that is assigned an empty value.
    /// It indicates that the keyword assignment should remain empty for a given package.
    ///
    /// Example:
    /// ```txt
    /// pkgdesc =
    /// depends =
    /// ```
    ///
    /// The above properties would indicate that both `pkgdesc` and the `depends` array are to be
    /// cleared and left empty for a given package.
    ///
    /// This function backtracks in case no keyword in this group matches or in case the property is
    /// not cleared.
    fn shared_meta_parser(input: &mut &str) -> ModalResult<ClearableProperty> {
        // First off, check if this is any of the clearable properties.
        let keyword =
            trace("clearable_shared_meta_property", SharedMetaKeyword::parser).parse_next(input)?;

        // Now check if it's actually a clear.
        // This parser fails and backtracks in case there's anything but spaces and a newline after
        // the delimiter, which indicates that there's an actual value that is set for this
        // property.
        let _ = (" =", space0, newline).parse_next(input)?;

        let property = match keyword {
            // The `Arch` property matches the keyword, but isn't clearable.
            SharedMetaKeyword::Arch => {
                return Err(ErrMode::Backtrack(ParserError::from_input(input)));
            }
            SharedMetaKeyword::PkgDesc => ClearableProperty::Description,
            SharedMetaKeyword::Url => ClearableProperty::Url,
            SharedMetaKeyword::License => ClearableProperty::Licenses,
            SharedMetaKeyword::Changelog => ClearableProperty::Changelog,
            SharedMetaKeyword::Install => ClearableProperty::Install,
            SharedMetaKeyword::Groups => ClearableProperty::Groups,
            SharedMetaKeyword::Options => ClearableProperty::Options,
            SharedMetaKeyword::Backup => ClearableProperty::Backups,
        };

        Ok(property)
    }

    /// Same as [`Self::shared_meta_parser`], but for clearable [RelationProperty].
    fn relation_parser(input: &mut &str) -> ModalResult<ClearableProperty> {
        // First off, check if this is any of the clearable properties.
        let keyword = trace("clearable_property", RelationKeyword::parser).parse_next(input)?;

        // All relations may be architecture specific.
        let architecture = architecture_suffix.parse_next(input)?;

        // Now check if it's actually a clear.
        // This parser fails and backtracks in case there's anything but spaces and a newline after
        // the delimiter, which indicates that there's an actual value that is set for this
        // property.
        let _ = (" =", space0, newline).parse_next(input)?;

        let property = match keyword {
            RelationKeyword::Depends => ClearableProperty::Dependencies(architecture),
            RelationKeyword::OptDepends => ClearableProperty::OptionalDependencies(architecture),
            RelationKeyword::Provides => ClearableProperty::Provides(architecture),
            RelationKeyword::Conflicts => ClearableProperty::Conflicts(architecture),
            RelationKeyword::Replaces => ClearableProperty::Replaces(architecture),
        };

        Ok(property)
    }
}
