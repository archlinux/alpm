use std::str::FromStr;

use alpm_types::{
    digests::{Blake2b512, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
    Architecture,
    Backup,
    Blake2b512Checksum,
    Changelog,
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
    SkippableChecksum,
    Source,
    Url,
};
use winnow::{
    ascii::{line_ending, newline, space0, till_line_ending},
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
    ModalResult,
    Parser,
};

/// Parses the ` = ` delimiter between keywords.
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

    // Consume the newline.
    line_ending.parse_next(input)?;

    Ok(out)
}

/// An arbitrarily typed attribute that is specific to an [alpm-architecture].
///
/// [alpm-architecture]: <https://alpm.archlinux.page/specifications/alpm-architecture.7.html>
#[derive(Debug)]
pub struct ArchProperty<T> {
    /// The optional [alpm-architecture] of the `value`.
    ///
    /// If `architecture` is [`None`] it is considered to be `"any"`.
    /// [alpm-architecture]: <https://alpm.archlinux.page/specifications/alpm-architecture.7.html>
    pub architecture: Option<Architecture>,
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
fn architecture_suffix(input: &mut &str) -> ModalResult<Option<Architecture>> {
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
        cut_err(take_till(0.., |c| c == ' ' || c == '=').try_map(Architecture::from_str))
            .context(StrContext::Label("architecture"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alpm-architecture compatible suffix (e.g. '_i386` or `_x86_64`)",
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

/// A representation of all high-level components of parsed SRCINFO data.
#[derive(Debug)]
pub struct SourceInfoContent {
    /// Empty or comment lines that occur outside of `pkgbase` or `pkgname` sections.
    pub preceding_lines: Vec<Ignored>,

    pub package_base: RawPackageBase,
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
    /// use alpm_srcinfo::parser::SourceInfoContent;
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
    ///
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

        // Afterwards one or more `pkgname` declarations are to follow.
        let (packages, _eof): (Vec<RawPackage>, _) =
            repeat_till(1.., RawPackage::parser, eof).parse_next(input)?;

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
    pub name: Name,
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
        let name =
            cut_err(terminated(take_till(1.., |c| c == '\n'), line_ending).try_map(Name::from_str))
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
    pub name: Name,
    pub properties: Vec<PackageProperty>,
}

impl RawPackage {
    /// Recognizes an entire single `pkgname` section in SRCINFO data.
    fn parser(input: &mut &str) -> ModalResult<RawPackage> {
        cut_err("pkgname")
            .context(StrContext::Label("pkgname section header"))
            .parse_next(input)?;

        cut_err(" = ")
            .context(StrContext::Label("pkgname section header delimiter"))
            .context(StrContext::Expected(StrContextValue::Description("' = '")))
            .parse_next(input)?;

        // Get the name of the base package.
        let name =
            cut_err(terminated(take_till(1.., |c| c == '\n'), line_ending).try_map(Name::from_str))
                .context(StrContext::Label("package name"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "the name of a package",
                )))
                .parse_next(input)?;

        // Go through the lines after the initial `pkgname` statement.
        //
        // We explicitly use `repeat` to allow backtracking from the inside.
        // The reason for this is that SRCINFO is no structured data format per se and we have no
        // clear indicator that the current `pkgname` section just stopped and a new `pkgname`
        // section started.
        //
        // The only way to detect this is to look for the `pkgname` keyword while parsing lines in
        // `package_line`. If that keyword is detected, we trigger a backtracking error that
        // results in this `repeat` call to wrap up and return successfully.
        let properties: Vec<PackageProperty> =
            repeat(0.., PackageProperty::parser).parse_next(input)?;

        Ok(RawPackage { name, properties })
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
    EmptyLine,
    Comment(String),
    MetaProperty(SharedMetaProperty),
    PackageVersion(PackageVersion),
    PackageRelease(PackageRelease),
    PackageEpoch(Epoch),
    ValidPgpKeys(OpenPGPIdentifier),
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
        let _ = space0.parse_next(input)?;

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
                cut_err(fail)
                    .context(StrContext::Label("package base property"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed pkgbase properties",
                    ))),
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword definitions in the `pkgbase` section in SRCINFO data.
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
                fail.context(StrContext::Label("file property type"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed pkgbase properties.",
                    ))),
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword definitions exclusive to the `pkgbase` section in SRCINFO data.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn exclusive_property_parser(input: &mut &str) -> ModalResult<PackageBaseProperty> {
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
            "'checkdepends', 'makedepends', 'pkgver', 'pkgrel', 'epoch', 'validpgpkeys'",
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
                Ok::<PackageBaseProperty, alpm_types::Error>(PackageBaseProperty::PackageEpoch(
                    epoch,
                ))
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
}

/// All possible properties of a `pkgname` section in SRCINFO data.
///
/// It's very similar to [`RawPackageBase`], but with less fields and the possibility to explicitly
/// set some fields to "empty".
#[derive(Debug)]
pub enum PackageProperty {
    EmptyLine,
    Comment(String),
    MetaProperty(SharedMetaProperty),
    RelationProperty(RelationProperty),
    Clear(ClearableProperty),
}

impl PackageProperty {
    /// Handles any line in a `pkgname` package section.
    ///
    /// This is a wrapper to separate the logic between comments/empty lines and actual package
    /// properties.
    fn parser(input: &mut &str) -> ModalResult<PackageProperty> {
        // Trim any leading spaces, which are allowed per spec.
        let _ = space0.parse_next(input)?;

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
                preceded(("#", take_until(0.., "\n")), alt((line_ending, eof)))
                    .map(|s: &str| PackageProperty::Comment(s.to_string())),
                preceded(space0, alt((line_ending, eof))).map(|_| PackageProperty::EmptyLine),
                // In case we got text, start parsing properties
                Self::property_parser,
                cut_err(fail)
                    .context(StrContext::Label("package property"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed pkgname properties.",
                    ))),
            )),
        )
        .parse_next(input)
    }

    /// Recognizes keyword definitions in a `pkgname` section in SRCINFO data.
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
        // this were to ever become an issue, it would be a good start to clone all
        // `*_property` parser functions into two functions, where one of them explicitly
        // handles clearable properties.
        trace(
            "pkgname_property",
            alt((
                ClearableProperty::parser.map(PackageProperty::Clear),
                SharedMetaProperty::parser.map(PackageProperty::MetaProperty),
                RelationProperty::parser.map(PackageProperty::RelationProperty),
                fail.context(StrContext::Label("file property type"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "one of the allowed pkgname properties.",
                    ))),
            )),
        )
        .parse_next(input)
    }
}

/// Metadata properties that may be shared between `pkgbase` and `pkgname` sections in SRCINFO data.
#[derive(Debug)]
pub enum SharedMetaProperty {
    Description(PackageDescription),
    Url(Url),
    License(License),
    Architecture(Architecture),
    Changelog(RelativePath),
    Install(RelativePath),
    Group(String),
    Option(MakepkgOption),
    Backup(RelativePath),
}

impl SharedMetaProperty {
    /// Recognizes keyword definitions that may be present in both `pkgbase` and `pkgname` sections
    /// of SRCINFO data.
    ///
    /// This function relies on [`Self::keyword_parser`] to recognize the relevant keywords.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<SharedMetaProperty> {
        // Now get the type of the property.
        let property_type = Self::keyword_parser.parse_next(input)?;

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
                Ok::<SharedMetaProperty, alpm_types::Error>(SharedMetaProperty::Changelog(
                    changelog,
                ))
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

    /// Parse the available relation property keywords.
    fn keyword_parser<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
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
}

/// Properties related to package relations.
///
/// This only handles the shared package relations that can be used in both `pkgbase` and `pkgname`
/// sections.
/// `pkgbase` specific relations are explicitly handled in the [`RawPackageBase`] enum.
/// See [alpm-package-relation] for further details on package relations.
/// [alpm-package-relation]: <https://alpm.archlinux.page/specifications/alpm-package-relation.7.html>
#[derive(Debug)]
pub enum RelationProperty {
    Dependency(ArchProperty<PackageRelation>),
    OptionalDependency(ArchProperty<OptionalDependency>),
    Provides(ArchProperty<PackageRelation>),
    Conflicts(ArchProperty<PackageRelation>),
    Replaces(ArchProperty<PackageRelation>),
}

impl RelationProperty {
    /// Recognizes package relation keyword definitions that may be present in both `pkgbase` and
    /// `pkgname` sections in SRCINFO data.
    ///
    /// This function relies on [`Self::keyword_parser`] to recognize the relevant keywords.
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<RelationProperty> {
        // First off, get the type of the property.
        let property_type = Self::keyword_parser.parse_next(input)?;

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

    /// Recognizes package relation keywords that may be present in both `pkgbase` and `pkgname`
    /// sections in SRCINFO data.
    fn keyword_parser<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
        trace(
            "relation_property_keyword",
            alt(("depends", "optdepends", "provides", "conflicts", "replaces")),
        )
        .parse_next(input)
    }

    //
    pub fn architecture(&self) -> Option<Architecture> {
        match self {
            RelationProperty::Dependency(arch_property) => arch_property.architecture,
            RelationProperty::OptionalDependency(arch_property) => arch_property.architecture,
            RelationProperty::Provides(arch_property) => arch_property.architecture,
            RelationProperty::Conflicts(arch_property) => arch_property.architecture,
            RelationProperty::Replaces(arch_property) => arch_property.architecture,
        }
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

impl SourceProperty {
    /// Recognizes package source related keyword definitions in SRCINFO data.
    ///
    /// This function relies on [`Self::keyword_parser`] to recognize the relevant keywords.
    ///
    /// This function backtracks in case no keyword in this group matches.
    fn parser(input: &mut &str) -> ModalResult<SourceProperty> {
        // First off, get the type of the property.
        let property_type = Self::keyword_parser.parse_next(input)?;

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
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "md5sums" => {
                        let checksum = Md5Checksum::from_str(s)?;
                        SourceProperty::Md5Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "sha1sums" => {
                        let checksum = Sha1Checksum::from_str(s)?;
                        SourceProperty::Sha1Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "sha224sums" => {
                        let checksum = Sha224Checksum::from_str(s)?;
                        SourceProperty::Sha224Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "sha256sums" => {
                        let checksum = Sha256Checksum::from_str(s)?;
                        SourceProperty::Sha256Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "sha384sums" => {
                        let checksum = Sha384Checksum::from_str(s)?;
                        SourceProperty::Sha384Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
                        })
                    }
                    "sha512sums" => {
                        let checksum = Sha512Checksum::from_str(s)?;
                        SourceProperty::Sha512Checksum(ArchProperty {
                            architecture,
                            value: SkippableChecksum::Checksum { digest: checksum },
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

    /// Recognizes all keywords related to package sources in SRCINFO data.
    fn keyword_parser<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
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
}

/// Properties used in `pkgname` sections that can be cleared.
///
/// Some variants of this enum are architecture specific as they might only be cleared for a
/// specific architecture, but not for others.
///
/// A clear is a keyword followed by an empty assignment, such as for example:
///
/// ```txt
/// depends =
/// ```
#[derive(Debug, Clone)]
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

impl ClearableProperty {
    /// Recognizes all keyword definitions in SRCINFO data that represent a cleared property.
    ///
    /// A cleared property is represented by a keyword that is assigned an empty value.
    /// It indicates that the keyword definition should remain empty for a given package.
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
    fn parser(input: &mut &str) -> ModalResult<ClearableProperty> {
        // First off, check if this is any of the clearable properties.
        let property_type = trace(
            "clearable_property",
            alt((
                SharedMetaProperty::keyword_parser,
                RelationProperty::keyword_parser,
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
        // This parser fails and backtracks in case there's anything but spaces and a newline after
        // the delimiter, which indicates that there's an actual value that is set for this
        // property.
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
}
