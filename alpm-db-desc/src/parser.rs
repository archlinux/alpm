//! Parser for database desc files

use std::{fmt::Display, str::FromStr};

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
use winnow::{
    ModalResult,
    Parser,
    ascii::{line_ending, newline, space0, till_line_ending},
    combinator::{alt, cut_err, delimited, eof, opt, peek, preceded, repeat_till, terminated},
    error::{StrContext, StrContextValue},
    token::take_while,
};

use crate::types::{PackageInstallReason, PackageValidation};

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
    /// An unknown section in the database desc file.
    ///
    /// Using this section will result in an error.
    Unknown(String),
}

/// Parses the name of a section in a database desc file.
///
/// Takes the section name enclosed in `%` characters, followed by a newline.
fn section_name<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    let section = delimited("%", take_while(1.., |c| c != '%'), "%");
    terminated(preceded(space0, section), line_ending).parse_next(input)
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
    let beginning_of_section = peek(preceded(newline, section_name));
    repeat_till(0.., value, alt((beginning_of_section, eof)))
        .context(StrContext::Label("values"))
        .context(StrContext::Expected(StrContextValue::Description(
            "expected a list of values in the database desc file",
        )))
        .parse_next(input)
        .map(|(outs, _)| outs)
}

/// Parses a section from a database desc file.
pub(crate) fn section(input: &mut &str) -> ModalResult<Section> {
    let section_name = cut_err(section_name)
        .context(StrContext::Label("section name"))
        .context(StrContext::Expected(StrContextValue::Description(
            "expected a section name that is enclosed in `%` characters",
        )))
        .parse_next(input)?;

    let section = match section_name {
        "NAME" => Section::Name(value(input)?),
        "VERSION" => Section::Version(value(input)?),
        "BASE" => Section::Base(value(input)?),
        "DESC" => Section::Desc(value(input)?),
        "URL" => Section::Url(value(input)?),
        "ARCH" => Section::Arch(value(input)?),
        "BUILDDATE" => Section::BuildDate(value(input)?),
        "INSTALLDATE" => Section::InstallDate(value(input)?),
        "PACKAGER" => Section::Packager(value(input)?),
        "SIZE" => Section::Size(value(input)?),
        "GROUPS" => Section::Groups(values(input)?),
        "REASON" => Section::Reason(value(input)?),
        "LICENSE" => Section::License(values(input)?),
        "VALIDATION" => Section::Validation(values(input)?),
        "REPLACES" => Section::Replaces(values(input)?),
        "DEPENDS" => Section::Depends(values(input)?),
        "OPTDEPENDS" => Section::OptDepends(values(input)?),
        "CONFLICTS" => Section::Conflicts(values(input)?),
        "PROVIDES" => Section::Provides(values(input)?),
        "XDATA" => Section::XData(values(input)?),
        _ => Section::Unknown(section_name.to_string()),
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
