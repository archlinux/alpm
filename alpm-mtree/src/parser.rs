use std::path::PathBuf;

use alpm_parsers::iter_str_context;
use alpm_types::{Md5Checksum, Sha256Checksum};
use winnow::{
    ModalResult,
    Parser as WinnowParser,
    ascii::{digit1, line_ending, space0},
    combinator::{
        alt,
        cut_err,
        eof,
        fail,
        preceded,
        repeat_till,
        separated,
        separated_pair,
        terminated,
    },
    error::{StrContext, StrContextValue},
    stream::AsChar,
    token::{take_until, take_while},
};

use crate::path_decoder::decode_utf8_chars;

/// Each line represents a line in a .MTREE file.
#[derive(Debug, Clone)]
pub enum Statement<'a> {
    /// All lines that're irrelevant and don't contribute anything to the actual mtree file.
    ///
    /// Includes the following:
    /// - Empty lines
    /// - Lines that start with `#` (e.g. `#mtree` line and comments)
    Ignored,
    /// A `/set` command followed by some properties.
    Set(Vec<SetProperty<'a>>),
    /// A `/unset` command followed by some properties.
    Unset(Vec<UnsetProperty>),
    /// Any path statement followed by some properties.
    Path {
        path: PathBuf,
        properties: Vec<PathProperty<'a>>,
    },
}

/// Represents the properties that may be set in `/set` lines.
#[derive(Debug, Clone)]
pub enum SetProperty<'a> {
    Uid(u32),
    Gid(u32),
    Mode(&'a str),
    Type(PathType),
}

/// Represents the properties that can be unset by `/unset` lines.
#[derive(Debug, Clone)]
pub enum UnsetProperty {
    Uid,
    Gid,
    Mode,
    Type,
}

/// This type is used in a path line to define properties for that path.
#[derive(Debug, Clone)]
pub enum PathProperty<'a> {
    Uid(u32),
    Gid(u32),
    Mode(&'a str),
    Type(PathType),
    Size(u64),
    Link(PathBuf),
    Md5Digest(Md5Checksum),
    Sha256Digest(Sha256Checksum),
    Time(i64),
}

/// All allowed kinds of path types.
#[derive(Debug, Clone, Copy)]
pub enum PathType {
    Dir,
    File,
    Link,
}

/// Parse a single `/set` property.
fn set_property<'s>(input: &mut &'s str) -> ModalResult<SetProperty<'s>> {
    // First off, get the type of the property.
    let keywords = ["uid", "gid", "type", "mode"];
    let property_type = cut_err(alt(keywords))
        .context(StrContext::Label("property"))
        .context_with(iter_str_context!([keywords]))
        .parse_next(input)?;

    // Expect the `=` separator between the key-value pair
    let _ = "=".parse_next(input)?;

    // Now we continue parsing based on the type of the property.
    let property = match property_type {
        "type" => {
            let path_types = ["dir", "file", "link"];
            alt(path_types)
                .map(|value| match value {
                    "dir" => SetProperty::Type(PathType::Dir),
                    "file" => SetProperty::Type(PathType::File),
                    "link" => SetProperty::Type(PathType::Link),
                    _ => unreachable!(),
                })
                .context(StrContext::Label("property file type"))
                .context_with(iter_str_context!([path_types]))
                .parse_next(input)?
        }
        "uid" => SetProperty::Uid(system_id("user id", input)?),
        "gid" => SetProperty::Gid(system_id("group id", input)?),
        "mode" => SetProperty::Mode(mode(input)?),
        _ => unreachable!(),
    };

    Ok(property)
}

/// Parse a single `/unset` property.
fn unset_property(input: &mut &str) -> ModalResult<UnsetProperty> {
    // First off, get the type of the property.
    let keywords = ["uid", "gid", "type", "mode"];
    let property_type = cut_err(alt(keywords))
        .context(StrContext::Label("property"))
        .context_with(iter_str_context!([keywords]))
        .parse_next(input)?;

    // Map the parsed property type to the correct enum variant.
    let property = match property_type {
        "type" => UnsetProperty::Type,
        "uid" => UnsetProperty::Uid,
        "gid" => UnsetProperty::Gid,
        "mode" => UnsetProperty::Mode,
        _ => unreachable!(),
    };

    Ok(property)
}

/// Parse a simple system id as usize.
fn system_id(id_type: &'static str, input: &mut &str) -> ModalResult<u32> {
    cut_err(digit1.parse_to())
        .context(StrContext::Label(id_type))
        .context(StrContext::Expected(StrContextValue::Description(
            "a system id.",
        )))
        .parse_next(input)
}

/// Parse a Unix timestamp.
///
/// In mtree, this is a float for some reason, even though the decimal place is always a `0`.
fn timestamp(input: &mut &str) -> ModalResult<i64> {
    let (timestamp, _) = cut_err(separated_pair(digit1.parse_to(), '.', digit1))
        .context(StrContext::Label("unix epoch"))
        .context(StrContext::Expected(StrContextValue::Description(
            "A unix epoch in float notation.",
        )))
        .parse_next(input)?;

    Ok(timestamp)
}

/// Parse a filesystem mode.
///
/// Should be between 3-5 octal numbers **without** a `0o` prefix.
fn mode<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    cut_err(take_while(3..5, AsChar::is_oct_digit))
        .context(StrContext::Label("file mode"))
        .context(StrContext::Expected(StrContextValue::Description(
            "octal string of length 3-5.",
        )))
        .parse_next(input)
}

/// Parse a SHA-256 hash.
fn sha256(input: &mut &str) -> ModalResult<Sha256Checksum> {
    cut_err(take_while(64.., AsChar::is_hex_digit).parse_to())
        .context(StrContext::Label("sha256 hash"))
        .context(StrContext::Expected(StrContextValue::Description(
            "64 char long hexadecimal string",
        )))
        .parse_next(input)
}

/// Parse an MD5 hash.
fn md5(input: &mut &str) -> ModalResult<Md5Checksum> {
    cut_err(take_while(32.., AsChar::is_hex_digit).parse_to())
        .context(StrContext::Label("md5 hash"))
        .context(StrContext::Expected(StrContextValue::Description(
            "32 char long hexadecimal string",
        )))
        .parse_next(input)
}

/// Consume all chars of a link until a newline or space is hit.
///
/// Check [`decode_utf8_chars`] for more info on how special chars in paths are escaped.
fn link(input: &mut &str) -> ModalResult<String> {
    take_while(0.., |c| c != ' ' && c != '\n')
        .and_then(decode_utf8_chars)
        .parse_next(input)
}

/// Get a string representing a size by consuming all integers.
fn size(input: &mut &str) -> ModalResult<u64> {
    cut_err(take_while(0.., |c| c != ' ' && c != '\n').parse_to())
        .context(StrContext::Label("file size"))
        .context(StrContext::Expected(StrContextValue::Description(
            "a positive integer representing the file's size.",
        )))
        .parse_next(input)
}

/// Parse a single property.
fn property<'s>(input: &mut &'s str) -> ModalResult<PathProperty<'s>> {
    // First off, get the type of the property.
    let keywords = [
        "type",
        "uid",
        "gid",
        "mode",
        "size",
        "link",
        "md5digest",
        "sha256digest",
        "time",
    ];
    let property_type = cut_err(alt(keywords))
        .context(StrContext::Label("file property type"))
        .context_with(iter_str_context!([keywords]))
        .parse_next(input)?;

    // Expect the `=` separator between the key-value pair
    let _ = "=".parse_next(input)?;

    // Now we continue parsing based on the type of the property.
    let property = match property_type {
        "type" => alt(("dir", "file", "link"))
            .map(|value| match value {
                "dir" => PathProperty::Type(PathType::Dir),
                "file" => PathProperty::Type(PathType::File),
                "link" => PathProperty::Type(PathType::Link),
                _ => unreachable!(),
            })
            .context(StrContext::Label("property file type"))
            .context(StrContext::Expected(StrContextValue::Description(
                "'dir', 'file' or 'link'",
            )))
            .parse_next(input)?,
        "uid" => PathProperty::Uid(system_id("user id", input)?),
        "gid" => PathProperty::Gid(system_id("group id", input)?),
        "mode" => PathProperty::Mode(mode(input)?),
        "size" => PathProperty::Size(size.parse_next(input)?),
        "link" => PathProperty::Link(PathBuf::from(link.parse_next(input)?)),
        "md5digest" => PathProperty::Md5Digest(md5(input)?),
        "sha256digest" => PathProperty::Sha256Digest(sha256(input)?),
        "time" => PathProperty::Time(timestamp(input)?),
        _ => unreachable!(),
    };

    Ok(property)
}

/// Parse all path related properties that follow after a path declaration.
///
/// An example without all possible properties:
/// E.g. `./some_path uid=0 gid=0 type=file`
///                   ↑                   ↑
///                         This part
fn properties<'s>(input: &mut &'s str) -> ModalResult<Vec<PathProperty<'s>>> {
    cut_err(terminated(separated(0.., property, " "), line_ending)).parse_next(input)
}

/// Parse all properties that follow a `/set` command.
///
/// E.g. `/set uid=0 gid=0`
///            ↑         ↑
///             This part
fn set_properties<'s>(input: &mut &'s str) -> ModalResult<Vec<SetProperty<'s>>> {
    cut_err(terminated(separated(0.., set_property, " "), line_ending)).parse_next(input)
}

/// Parse all properties that follow an `/unset` command.
//////
/// E.g. `/unset uid gid`
///              ↑     ↑
///             This part
fn unset_properties(input: &mut &str) -> ModalResult<Vec<UnsetProperty>> {
    cut_err(terminated(separated(0.., unset_property, " "), line_ending)).parse_next(input)
}

/// Parse the next statement in the file.
fn statement<'s>(input: &mut &'s str) -> ModalResult<Statement<'s>> {
    // First, we figure out what kind of line we're looking at.
    let statement_type: String = alt((
        // A Path statement line
        //
        // Path statements may be preceded with whitespaces.
        // Otherwise read the line until terminated by the first space or newline.
        // Whitespace characters are encoded as `\s' (space), `\t' (tab), and `\n' (new line)
        // which is why we can simply ignore those while parsing the path.
        preceded(
            space0,
            terminated((".", take_until(0.., " ")).take(), alt((' ', '\n'))),
        ).and_then(decode_utf8_chars),
        terminated("/set", " ").map(|s: &str| s.to_string()),
        terminated("/unset", " ").map(|s: &str| s.to_string()),
        // A comment line that starts with `#`.
        preceded(("#", take_until(0.., "\n")), line_ending).map(|s: &str| s.to_string()),
        // An empty line that possibly contains spaces.
        preceded(space0, line_ending).map(|s: &str| s.to_string()),
        // If none of the above match, fail hard with a correct error message.
        fail.context(StrContext::Label("statement"))
        .context(StrContext::Expected(StrContextValue::Description(
            "'/set', '/unset', or a relative local path (./some/path) followed by their respective properties.",
        )))
    ))
    .parse_next(input)?;

    // Ignore comments and empty lines.
    if statement_type.trim().is_empty() {
        return Ok(Statement::Ignored);
    }

    // Now parse the properties based on the statement type until the end of line.
    let statement = match statement_type.as_str() {
        "/set" => Statement::Set(set_properties.parse_next(input)?),
        "/unset" => Statement::Unset(unset_properties.parse_next(input)?),
        path => Statement::Path {
            path: PathBuf::from(path),
            properties: properties.parse_next(input)?,
        },
    };

    Ok(statement)
}

/// Parse a given .MTREE file.
///
/// Empty lines and comment lines are returned as `Statement::Ignored`.
/// This is to provide a proper line-based representation of the file, so we can later on provide
/// proper context in error messages during the interpretation step.
///
/// # Errors
///
/// - `Error::ParseError` if a malformed MTREE file is encountered.
pub fn mtree<'s>(input: &mut &'s str) -> ModalResult<Vec<Statement<'s>>> {
    let (statements, _eof): (Vec<Statement<'s>>, _) =
        repeat_till(0.., statement, eof).parse_next(input)?;

    Ok(statements)
}
