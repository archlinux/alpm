//! Parser for [alpm-db-desc] files.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

use std::{fmt::Display, str::FromStr};

use alpm_parsers::iter_str_context;
use alpm_types::{
    Architecture,
    BuildDate,
    ExtraData,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageRelation,
    Packager,
    Url,
    Version,
};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    ascii::{line_ending, newline, space0, till_line_ending},
    combinator::{alt, cut_err, delimited, eof, opt, peek, preceded, repeat_till, terminated},
    error::{StrContext, StrContextValue},
    token::take_while,
};

use crate::types::{PackageInstallReason, PackageValidation};

#[derive(Debug, Display, EnumString, Eq, PartialEq, VariantNames)]
#[strum(serialize_all = "UPPERCASE")]
pub enum SectionKeyword {
    Name,
    Version,
    Base,
    Desc,
    Url,
    Arch,
    BuildDate,
    InstallDate,
    Packager,
    Size,
    Groups,
    Reason,
    License,
    Validation,
    Replaces,
    Depends,
    OptDepends,
    Conflicts,
    Provides,
    XData,
}

impl SectionKeyword {
    /// Recognizes a [`SectionKeyword`] in an input string slice.
    ///
    /// Takes the section name enclosed in `%` characters, followed by a newline.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let section = delimited("%", take_while(1.., |c| c != '%'), "%");
        terminated(
            preceded(space0, section.try_map(Self::from_str)),
            line_ending,
        )
        .parse_next(input)
    }
}

/// A single section in a database desc file.
#[derive(Debug)]
pub(crate) enum Section {
    /// The name of the package.
    Name(Name),
    /// The version of the package.
    Version(Version),
    /// The base name of the package (used in split packages).
    Base(PackageBaseName),
    /// The description of the package.
    Desc(PackageDescription),
    /// The URL for the project of the package.
    Url(Url),
    /// The architecture of the package.
    Arch(Architecture),
    /// The date at which the build of the package started.
    BuildDate(BuildDate),
    /// The date at which the package has been installed on the system.
    InstallDate(BuildDate),
    /// The User ID of the entity, that built the package.
    Packager(Packager),
    /// The optional size of the (uncompressed and unpacked) package contents in bytes.
    Size(InstalledSize),
    /// Groups the package belongs to.
    Groups(Vec<Group>),
    /// The reason why the package was installed.
    Reason(PackageInstallReason),
    /// The license(s) of the package.
    License(Vec<License>),
    /// The validation method used during installation of the package ensuring its authenticity.
    Validation(Vec<PackageValidation>),
    /// Packages that this package replaces.
    Replaces(Vec<Name>),
    /// Packages that this package depends on.
    Depends(Vec<PackageRelation>),
    /// Optional dependencies of the package.
    OptDepends(Vec<OptionalDependency>),
    /// Packages that conflict with this package.
    Conflicts(Vec<Name>),
    /// Packages that this package provides.
    Provides(Vec<Name>),
    /// Extra data associated with the package.
    XData(Vec<ExtraData>),
}

/// Parses a single value from a database desc file.
fn value<T>(input: &mut &str) -> ModalResult<T>
where
    T: FromStr + Display,
    T::Err: Display,
{
    // Parse until the end of the line.
    let out = till_line_ending.parse_to().parse_next(input)?;

    // Consume the newline or end of file.
    alt((line_ending, eof)).parse_next(input)?;

    Ok(out)
}

/// Parses a list of values from a database desc file.
// Stops when the next section starts (indicated by a `%` character)
// or the end of the file.
fn values<T>(input: &mut &str) -> ModalResult<Vec<T>>
where
    T: FromStr + Display,
    T::Err: Display,
{
    let beginning_of_section = peek(preceded(newline, SectionKeyword::parser)).map(|_| ());
    repeat_till(0.., value, alt((beginning_of_section, eof.map(|_| ()))))
        .context(StrContext::Label("values"))
        .context(StrContext::Expected(StrContextValue::Description(
            "expected a list of values in the database desc file",
        )))
        .parse_next(input)
        .map(|(outs, _)| outs)
}

/// Parses a section from a database desc file.
pub(crate) fn section(input: &mut &str) -> ModalResult<Section> {
    let section_keyword = cut_err(SectionKeyword::parser)
        .context(StrContext::Label("section name"))
        .context(StrContext::Expected(StrContextValue::Description(
            "expected a section name that is enclosed in `%` characters",
        )))
        .context_with(iter_str_context!([SectionKeyword::VARIANTS]))
        .parse_next(input)?;

    let section = match section_keyword {
        SectionKeyword::Name => Section::Name(value(input)?),
        SectionKeyword::Version => Section::Version(value(input)?),
        SectionKeyword::Base => Section::Base(value(input)?),
        SectionKeyword::Desc => Section::Desc(value(input)?),
        SectionKeyword::Url => Section::Url(value(input)?),
        SectionKeyword::Arch => Section::Arch(value(input)?),
        SectionKeyword::BuildDate => Section::BuildDate(value(input)?),
        SectionKeyword::InstallDate => Section::InstallDate(value(input)?),
        SectionKeyword::Packager => Section::Packager(value(input)?),
        SectionKeyword::Size => Section::Size(value(input)?),
        SectionKeyword::Groups => Section::Groups(values(input)?),
        SectionKeyword::Reason => Section::Reason(value(input)?),
        SectionKeyword::License => Section::License(values(input)?),
        SectionKeyword::Validation => Section::Validation(values(input)?),
        SectionKeyword::Replaces => Section::Replaces(values(input)?),
        SectionKeyword::Depends => Section::Depends(values(input)?),
        SectionKeyword::OptDepends => Section::OptDepends(values(input)?),
        SectionKeyword::Conflicts => Section::Conflicts(values(input)?),
        SectionKeyword::Provides => Section::Provides(values(input)?),
        SectionKeyword::XData => Section::XData(values(input)?),
    };

    Ok(section)
}

/// Parses all sections from a database desc file.
pub(crate) fn sections(input: &mut &str) -> ModalResult<Vec<Section>> {
    cut_err(repeat_till(0.., preceded(opt(newline), section), eof))
        .context(StrContext::Label("sections"))
        .context(StrContext::Expected(StrContextValue::Description(
            "expected a section in the database desc file",
        )))
        .parse_next(input)
        .map(|(sections, _)| sections)
}
