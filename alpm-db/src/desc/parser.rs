//! Parser for [alpm-db-desc] files.
//!
//! [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html

use std::str::FromStr;

use alpm_parsers::{iter_str_context, prelude::*};
use alpm_types::{
    Architecture,
    BuildDate,
    ExtraData,
    ExtraDataEntry,
    FullVersion,
    Group,
    InstalledSize,
    License,
    Name,
    OptionalDependency,
    PackageBaseName,
    PackageDescription,
    PackageInstallReason,
    PackageRelation,
    PackageValidation,
    Packager,
    RelationOrSoname,
    Url,
};
use strum::{Display, EnumString, VariantNames};
use winnow::{
    ascii::{line_ending, newline, space0, till_line_ending},
    combinator::{
        alt,
        cut_err,
        delimited,
        eof,
        not,
        opt,
        peek,
        preceded,
        repeat,
        repeat_till,
        terminated,
    },
    error::ErrMode,
    token::take_while,
};

/// A known section name in an [alpm-db-desc] file.
///
/// Section names are e.g. `%NAME%` or `%VERSION%`.
///
/// [alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
#[derive(Clone, Debug, Display, EnumString, Eq, Hash, PartialEq, VariantNames)]
#[strum(serialize_all = "UPPERCASE")]
pub enum SectionKeyword {
    /// %NAME%
    Name,
    /// %VERSION%
    Version,
    /// %BASE%
    Base,
    /// %DESC%
    Desc,
    /// %URL%
    Url,
    /// %ARCH%
    Arch,
    /// %BUILDDATE%
    BuildDate,
    /// %INSTALLDATE%
    InstallDate,
    /// %PACKAGER%
    Packager,
    /// %SIZE%
    Size,
    /// %GROUPS%
    Groups,
    /// %REASON%
    Reason,
    /// %LICENSE%
    License,
    /// %VALIDATION%
    Validation,
    /// %REPLACES%
    Replaces,
    /// %DEPENDS%
    Depends,
    /// %OPTDEPENDS%
    OptDepends,
    /// %CONFLICTS%
    Conflicts,
    /// %PROVIDES%
    Provides,
    /// %XDATA%
    XData,
}

impl SectionKeyword {
    /// Recognizes a [`SectionKeyword`] in an input string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_db::desc::SectionKeyword;
    /// use alpm_parsers::prelude::*;
    ///
    /// # fn main() -> testresult::TestResult {
    /// let (remaining, kw) = SectionKeyword::parser.parse_peek(Input::new("%NAME%\nfoo\n"))?;
    /// assert_eq!(kw, SectionKeyword::Name);
    /// assert_eq!(*remaining, "foo\n");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not start with a valid
    /// `%SECTION%` header followed by a newline.
    pub fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        let section = delimited("%", take_while(1.., |c| c != '%'), "%");
        terminated(
            preceded(space0, section.try_map(Self::from_str)),
            line_ending,
        )
        .parse_next(input)
    }
}

/// A single logical section from a database desc file.
#[derive(Clone, Debug)]
pub enum Section {
    /// %NAME%
    Name(Name),
    /// %VERSION%
    Version(FullVersion),
    /// %BASE%
    Base(PackageBaseName),
    /// %DESC%
    Desc(PackageDescription),
    /// %URL%
    Url(Option<Url>),
    /// %ARCH%
    Arch(Architecture),
    /// %BUILDDATE%
    BuildDate(BuildDate),
    /// %INSTALLDATE%
    InstallDate(BuildDate),
    /// %PACKAGER%
    Packager(Packager),
    /// %SIZE%
    Size(InstalledSize),
    /// %GROUPS%
    Groups(Vec<Group>),
    /// %REASON%
    Reason(PackageInstallReason),
    /// %LICENSE%
    License(Vec<License>),
    /// %VALIDATION%
    Validation(Vec<PackageValidation>),
    /// %REPLACES%
    Replaces(Vec<PackageRelation>),
    /// %DEPENDS%
    Depends(Vec<RelationOrSoname>),
    /// %OPTDEPENDS%
    OptDepends(Vec<OptionalDependency>),
    /// %CONFLICTS%
    Conflicts(Vec<PackageRelation>),
    /// %PROVIDES%
    Provides(Vec<RelationOrSoname>),
    /// %XDATA%
    XData(ExtraData),
}

/// One or multiple newlines.
///
/// This also handles the case where there might be multiple lines with spaces.
fn newlines<'a>(input: &mut Input<'a>) -> PResult<'a, ()> {
    repeat(0.., line_ending).parse_next(input)
}

/// Recognizes the end of a list of values, without consuming anything.
///
/// A list of values ends at a blank line, at the next section header or at the end of the file.
fn end_of_values<'a>(input: &mut Input<'a>) -> PResult<'a, ()> {
    peek(alt((
        line_ending.map(|_| ()),
        SectionKeyword::parser.map(|_| ()),
        eof.map(|_| ()),
    )))
    .parse_next(input)
}

/// A parser helper, which returns a [`FromStr`]-style parser for types that don't implement their
/// own parser.
///
/// Parses until the end of the current line. The line ending itself is not consumed.
/// Wraps the returned parser in a named layer.
///
/// # Errors
///
/// Returns an error if the next token cannot be parsed into `T`.
/// The error message of `T::from_str` is preserved as external error.
fn try_value_from<'a, T>(layer: &str) -> impl Parser<Input<'a>, T, ErrMode<ParseStack<'a>>>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    till_line_ending.try_map(T::from_str).layer(layer)
}

/// A parser helper for multi-value types, which returns a [`FromStr`]-style parser for types that
/// don't implement their own parser.
///
/// Parses a list of values, where each value is in its own line.
/// Repeats until a blank line, the next section header (`%...%`) or the end of the file.
/// The line ending of the last value is not consumed.
///
/// # Errors
///
/// Returns an error if a value cannot be parsed into `T` or if the
/// section layout does not match expectations.
fn try_values_from<'a, T>(layer: &str) -> impl Parser<Input<'a>, Vec<T>, ErrMode<ParseStack<'a>>>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    move |input: &mut Input<'a>| -> PResult<'a, Vec<T>> {
        repeat_till(
            0..,
            // A value, followed by the line ending that separates it from the next value.
            terminated(
                try_value_from(layer),
                opt(terminated(line_ending, not(end_of_values))),
            ),
            end_of_values,
        )
        .map(|(outs, _)| outs)
        .parse_next(input)
    }
}

/// Parses a list of values, where each value is in its own line.
///
/// The Parser `P` is used to parse the value of each line.
///
/// Repeats until a blank line, the next section header (`%...%`) or the end of the file.
/// The line ending of the last value is not consumed.
///
/// # Errors
///
/// Returns an error if a value cannot be parsed into `T` or if the
/// section layout does not match expectations.
fn values<'a, T, P>(
    type_parser: P,
    layer: &str,
) -> impl Parser<Input<'a>, Vec<T>, ErrMode<ParseStack<'a>>>
where
    P: Parser<Input<'a>, T, ErrMode<ParseStack<'a>>>,
{
    let mut type_parser = type_parser;
    let parser = move |input: &mut Input<'a>| -> PResult<'a, Vec<T>> {
        repeat_till(
            0..,
            // A value, followed by the line ending that separates it from the next value.
            terminated(
                type_parser.by_ref(),
                opt(terminated(line_ending, not(end_of_values))),
            ),
            end_of_values,
        )
        .map(|(outs, _)| outs)
        .parse_next(input)
    };

    parser.layer(layer)
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
fn section<'a>(input: &mut Input<'a>) -> PResult<'a, Section> {
    // Parse and validate the header keyword first.
    let section_keyword = cut_err(SectionKeyword::parser)
        .description("expected a valid section name that is enclosed in `%` characters.")
        .context_with(iter_str_context!([SectionKeyword::VARIANTS]))
        .layer("section header")
        .parse_next(input)?;

    // Delegate to the corresponding value or values parser.
    // Every value parser stops before the line ending of its last line.
    let section = match section_keyword {
        SectionKeyword::Name => Section::Name(Name::parser_until_line_ending(input)?),
        SectionKeyword::Version => Section::Version(FullVersion::parser_until_line_ending(input)?),
        SectionKeyword::Base => Section::Base(PackageBaseName::parser_until_line_ending(input)?),
        SectionKeyword::Desc => Section::Desc(
            till_line_ending
                .map(PackageDescription::new)
                .parse_next(input)?,
        ),
        SectionKeyword::Url => Section::Url(
            alt((
                // Handle the case of an empty URL
                till_line_ending
                    .verify(|s: &str| s.trim().is_empty())
                    .map(|_s: &str| None),
                // Handle the case of an existing URL
                till_line_ending
                    .try_map(Url::from_str)
                    .map(Some)
                    .layer("url field"),
            ))
            .parse_next(input)?,
        ),
        SectionKeyword::Arch => Section::Arch(Architecture::parser_until_line_ending(input)?),
        SectionKeyword::BuildDate => {
            Section::BuildDate(try_value_from("build date").parse_next(input)?)
        }
        SectionKeyword::InstallDate => {
            Section::InstallDate(try_value_from("install date").parse_next(input)?)
        }
        SectionKeyword::Packager => Section::Packager(Packager::parser_until_line_ending(input)?),
        SectionKeyword::Size => Section::Size(try_value_from("size").parse_next(input)?),
        SectionKeyword::Groups => Section::Groups(try_values_from("group").parse_next(input)?),
        SectionKeyword::Reason => {
            Section::Reason(try_value_from("install reason").parse_next(input)?)
        }
        SectionKeyword::License => Section::License(try_values_from("license").parse_next(input)?),
        SectionKeyword::Validation => Section::Validation(
            values(PackageValidation::parser_until_line_ending, "validation").parse_next(input)?,
        ),
        SectionKeyword::Replaces => Section::Replaces(
            values(PackageRelation::parser_until_line_ending, "replaces").parse_next(input)?,
        ),
        SectionKeyword::Depends => Section::Depends(
            values(RelationOrSoname::parser_until_line_ending, "depends").parse_next(input)?,
        ),
        SectionKeyword::OptDepends => Section::OptDepends(
            values(OptionalDependency::parser_until_line_ending, "optdepends").parse_next(input)?,
        ),
        SectionKeyword::Conflicts => Section::Conflicts(
            values(PackageRelation::parser_until_line_ending, "conflicts").parse_next(input)?,
        ),
        SectionKeyword::Provides => Section::Provides(
            values(RelationOrSoname::parser_until_line_ending, "provides").parse_next(input)?,
        ),
        SectionKeyword::XData => Section::XData(
            values(ExtraDataEntry::parser_until_line_ending, "xdata")
                .try_map(ExtraData::try_from)
                .parse_next(input)?,
        ),
    };

    // Consume the newline or handle end-of-file gracefully.
    alt((line_ending, eof)).parse_next(input)?;

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
pub(crate) fn sections<'a>(input: &mut Input<'a>) -> PResult<'a, Vec<Section>> {
    cut_err(repeat_till(
        0..,
        preceded(
            opt(newline),
            section.layer("alpm-repo-desc file value section"),
        ),
        terminated(opt(newlines), eof),
    ))
    .parse_next(input)
    .map(|(sections, _)| sections)
}
