//! File type handling.

use std::str::FromStr;

use alpm_parsers::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr, VariantNames};
use winnow::{ascii::alpha1, error::ErrMode};

/// The identifier of a file type used in ALPM.
///
/// These identifiers are used in the file names of file types such as binary packages (see
/// [alpm-package]), source packages and repository sync databases (see alpm-repo-db).
///
/// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
#[derive(
    AsRefStr, Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq, VariantNames,
)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum FileTypeIdentifier {
    /// The identifier for [alpm-package] files.
    ///
    /// [alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
    #[cfg_attr(feature = "serde", serde(rename = "pkg"))]
    #[strum(to_string = "pkg")]
    BinaryPackage,

    /// The identifier for alpm-repo-db files.
    #[cfg_attr(feature = "serde", serde(rename = "db"))]
    #[strum(to_string = "db")]
    RepositorySyncDatabase,

    /// The identifier for source package files.
    #[cfg_attr(feature = "serde", serde(rename = "src"))]
    #[strum(to_string = "src")]
    SourcePackage,
}

impl AlpmParser for FileTypeIdentifier {
    /// Recognizes a [`FileTypeIdentifier`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` does not begin with a valid variant
    /// of a [`FileTypeIdentifier`].
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        alpha1
            .try_map(FileTypeIdentifier::from_str)
            .expected_strings(FileTypeIdentifier::VARIANTS)
            .layer("file type identifier")
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .expected_text("a string consisting of alphabetic characters")
            .layer("file type identifier")
    }
}
