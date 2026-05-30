//! File type handling.

use std::str::FromStr;

use alpm_parsers::{iter_str_context, prelude::*};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantNames};
use winnow::{
    Parser,
    ascii::alpha1,
    error::{ErrMode, StrContext, StrContextValue},
};

/// The identifier of a file type used in ALPM.
///
/// These identifiers are used in the file names of file types such as binary packages (see
/// [alpm-package]), source packages and repository sync databases (see alpm-repo-db).
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Serialize,
    VariantNames,
)]
#[serde(untagged)]
pub enum FileTypeIdentifier {
    /// The identifier for [alpm-package] files.
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    #[serde(rename = "pkg")]
    #[strum(to_string = "pkg")]
    BinaryPackage,

    /// The identifier for alpm-repo-db files.
    #[serde(rename = "db")]
    #[strum(to_string = "db")]
    RepositorySyncDatabase,

    /// The identifier for source package files.
    #[serde(rename = "src")]
    #[strum(to_string = "src")]
    SourcePackage,
}

impl AlpmParser for FileTypeIdentifier {
    /// Recognizes a [`FileTypeIdentifier`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the immediate alphabetic `input` is not a valid variant
    /// a `FileTypeIdentifier`.
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        alpha1
            .try_map(FileTypeIdentifier::from_str)
            .context(StrContext::Label("compression algorithm file extension"))
            .context_with(iter_str_context!([FileTypeIdentifier::VARIANTS]))
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Label("FileTypeIdentifier"))
            .context(StrContext::Expected(StrContextValue::Description(
                "an alphabetic string",
            )))
    }
}
