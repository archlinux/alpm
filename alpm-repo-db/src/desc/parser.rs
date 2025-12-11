//! Parser for [alpm-repo-desc] files.
//!
//! [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html

use std::{fmt::Display, str::FromStr};

use alpm_parsers::iter_str_context;
use alpm_types::{
    Architecture,
    Base64OpenPGPSignature,
    BuildDate,
    CompressedSize,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Md5Checksum,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageFileName,
    PackageRelation,
    Packager,
    RelationOrSoname,
    Sha256Checksum,
    Url,
};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    ModalResult,
    Parser,
    ascii::{line_ending, newline, space0, till_line_ending},
    combinator::{
        alt,
        cut_err,
        delimited,
        eof,
        opt,
        peek,
        preceded,
        repeat,
        repeat_till,
        terminated,
    },
    error::{StrContext, StrContextValue},
    token::take_while,
};

/// A known section name in an [alpm-repo-desc] file.
///
/// Section names are e.g. `%NAME%` or `%VERSION%`.
///
/// [alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
#[derive(Clone, Debug, Display, EnumString, Eq, Hash, PartialEq, VariantNames)]
#[strum(serialize_all = "UPPERCASE")]
pub enum SectionKeyword {
    /// %FILENAME%
    Filename,
    /// %Name%
    Name,
    /// %BASE%
    Base,
    /// %VERSION%
    Version,
    /// %DESC%
    Desc,
    /// %GROUPS%
    Groups,
    /// %CSIZE%
    CSize,
    /// %ISIZE%
    ISize,
    /// %MD5SUM%
    Md5Sum,
    /// %SHA256SUM%
    Sha256Sum,
    /// %PGPSIG%
    PgpSig,
    /// %URL%
    Url,
    /// %LICENSE%
    License,
    /// %ARCH%
    Arch,
    /// %BUILDDATE%
    BuildDate,
    /// %PACKAGER%
    Packager,
    /// %REPLACES%
    Replaces,
    /// %CONFLICTS%
    Conflicts,
    /// %PROVIDES%
    Provides,
    /// %DEPENDS%
    Depends,
    /// %OPTDEPENDS%
    OptDepends,
    /// %MAKEDEPENDS%
    MakeDepends,
    /// %CHECKDEPENDS%
    CheckDepends,
}

impl SectionKeyword {
    /// Recognizes a [`SectionKeyword`] in an input string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_repo_db::desc::SectionKeyword;
    ///
    /// # fn main() -> winnow::ModalResult<()> {
    /// let mut input = "%NAME%\nfoo\n";
    /// let kw = SectionKeyword::parser(&mut input)?;
    /// assert_eq!(kw, SectionKeyword::Name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not start with a valid
    /// `%SECTION%` header followed by a newline.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let section = delimited("%", take_while(1.., |c| c != '%'), "%");
        terminated(
            preceded(space0, section.try_map(Self::from_str)),
            line_ending,
        )
        .parse_next(input)
    }
}

/// A single logical section from a repo desc file.
#[derive(Clone, Debug)]
pub enum Section {
    /// %FILENAME%
    Filename(PackageFileName),
    /// %NAME%
    Name(Name),
    /// %BASE%
    Base(PackageBaseName),
    /// %VERSION%
    Version(FullVersion),
    /// %DESC%
    Desc(PackageDescription),
    /// %GROUPS%
    Groups(Vec<Group>),
    /// %CSIZE%
    CSize(CompressedSize),
    /// %ISIZE%
    ISize(InstalledSize),
    /// %MD5SUM%
    Md5Sum(Md5Checksum),
    /// %SHA256SUM%
    Sha256Sum(Sha256Checksum),
    /// %PGPSIG%
    PgpSig(Base64OpenPGPSignature),
    /// %URL%
    Url(Option<Url>),
    /// %LICENSE%
    License(Vec<License>),
    /// %ARCH%
    Arch(Architecture),
    /// %BUILDDATE%
    BuildDate(BuildDate),
    /// %PACKAGER%
    Packager(Packager),
    /// %REPLACES%
    Replaces(Vec<PackageRelation>),
    /// %CONFLICTS%
    Conflicts(Vec<PackageRelation>),
    /// %PROVIDES%
    Provides(Vec<RelationOrSoname>),
    /// %DEPENDS%
    Depends(Vec<RelationOrSoname>),
    /// %OPTDEPENDS%
    OptDepends(Vec<OptionalDependency>),
    /// %MAKEDEPENDS%
    MakeDepends(Vec<PackageRelation>),
    /// %CHECKDEPENDS%
    CheckDepends(Vec<PackageRelation>),
}

/// One or multiple newlines.
///
/// This also handles the case where there might be multiple lines with spaces.
fn newlines(input: &mut &str) -> ModalResult<()> {
    repeat(0.., line_ending).parse_next(input)
}

/// Parses a single value from the input.
///
/// Consumes text until the end of the current line.
///
/// # Errors
///
/// Returns an error if the next token cannot be parsed into `T`.
fn value<T>(input: &mut &str) -> ModalResult<T>
where
    T: FromStr + Display,
    T::Err: Display,
{
    // Parse until the end of the line and attempt conversion to `T`.
    let value = till_line_ending.parse_to().parse_next(input)?;

    // Consume the newline or handle end-of-file gracefully.
    alt((line_ending, eof)).parse_next(input)?;

    Ok(value)
}

fn opt_value<T>(input: &mut &str) -> ModalResult<Option<T>>
where
    T: FromStr + Display,
    T::Err: Display,
{
    // Parse until the end of the line and attempt conversion to `Option<T>`.
    let value = opt(till_line_ending.parse_to()).parse_next(input)?;

    // Consume the newline or handle end-of-file gracefully.
    alt((line_ending, eof)).parse_next(input)?;

    Ok(value)
}

/// Parses a list of values from the input.
///
/// Repeats `value()` until the next section header (`%...%`)
/// or the end of the file.
///
/// # Errors
///
/// Returns an error if a value cannot be parsed into `T` or if the
/// section layout does not match expectations.
fn values<T>(input: &mut &str) -> ModalResult<Vec<T>>
where
    T: FromStr + Display,
    T::Err: Display,
{
    let next_section = peek(preceded(newline, SectionKeyword::parser)).map(|_| ());

    // Consume blank lines
    let blank_line = terminated(space0, newline).map(|_| ());

    repeat_till(0.., value, alt((next_section, blank_line, eof.map(|_| ()))))
        .context(StrContext::Label("values"))
        .context(StrContext::Expected(StrContextValue::Description(
            "a list of values in the database desc file",
        )))
        .parse_next(input)
        .map(|(outs, _)| outs)
}

/// Parses a single `%SECTION%` block and returns a [`Section`] variant.
///
/// # Errors
///
/// Returns an error if:
///
/// - the section name is invalid or not recognized,
/// - the section body contains malformed values,
/// - or the section does not terminate properly.
fn section(input: &mut &str) -> ModalResult<Section> {
    // Parse and validate the header keyword first.
    let section_keyword = cut_err(SectionKeyword::parser)
        .context(StrContext::Label("section name"))
        .context(StrContext::Expected(StrContextValue::Description(
            "a section name that is enclosed in `%` characters",
        )))
        .context_with(iter_str_context!([SectionKeyword::VARIANTS]))
        .parse_next(input)?;

    // Delegate to the corresponding value or values parser.
    let section = match section_keyword {
        SectionKeyword::Filename => Section::Filename(value(input)?),
        SectionKeyword::Name => Section::Name(value(input)?),
        SectionKeyword::Base => Section::Base(value(input)?),
        SectionKeyword::Version => Section::Version(value(input)?),
        SectionKeyword::Desc => Section::Desc(value(input)?),
        SectionKeyword::Groups => Section::Groups(values(input)?),
        SectionKeyword::CSize => Section::CSize(value(input)?),
        SectionKeyword::ISize => Section::ISize(value(input)?),
        SectionKeyword::Md5Sum => Section::Md5Sum(value(input)?),
        SectionKeyword::Sha256Sum => Section::Sha256Sum(value(input)?),
        SectionKeyword::PgpSig => Section::PgpSig(value(input)?),
        SectionKeyword::Url => Section::Url(opt_value(input)?),
        SectionKeyword::License => Section::License(values(input)?),
        SectionKeyword::Arch => Section::Arch(value(input)?),
        SectionKeyword::BuildDate => Section::BuildDate(value(input)?),
        SectionKeyword::Packager => Section::Packager(value(input)?),
        SectionKeyword::Replaces => Section::Replaces(values(input)?),
        SectionKeyword::Conflicts => Section::Conflicts(values(input)?),
        SectionKeyword::Provides => Section::Provides(values(input)?),
        SectionKeyword::Depends => Section::Depends(values(input)?),
        SectionKeyword::OptDepends => Section::OptDepends(values(input)?),
        SectionKeyword::MakeDepends => Section::MakeDepends(values(input)?),
        SectionKeyword::CheckDepends => Section::CheckDepends(values(input)?),
    };

    Ok(section)
}

/// Parses all `%SECTION%` blocks from the given input into a list of [`Section`]s.
///
/// This is the top-level parser used by the higher-level file constructors.
///
/// # Errors
///
/// Returns an error if:
///
/// - any section header is missing or malformed,
/// - a section value list fails to parse,
/// - or the overall structure of the file is inconsistent.
pub(crate) fn sections(input: &mut &str) -> ModalResult<Vec<Section>> {
    cut_err(repeat_till(
        0..,
        preceded(opt(newline), section),
        terminated(opt(newlines), eof),
    ))
    .context(StrContext::Label("sections"))
    .context(StrContext::Expected(StrContextValue::Description(
        "a section in the database desc file",
    )))
    .parse_next(input)
    .map(|(sections, _)| sections)
}
