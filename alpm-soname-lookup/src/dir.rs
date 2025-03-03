use std::{path::PathBuf, str::FromStr};

use alpm_types::SharedLibraryPrefix;
use winnow::{
    ModalResult,
    Parser,
    combinator::{alt, cut_err, eof, peek, repeat_till},
    error::{StrContext, StrContextValue},
    token::{any, rest},
};

use crate::Error;

/// A directory to look for shared objects in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LookupDirectory {
    /// The lookup prefix for shared objects.
    pub prefix: SharedLibraryPrefix,
    /// The directory to look for shared objects in.
    pub directory: PathBuf,
}

impl LookupDirectory {
    /// Creates a new lookup directory with a prefix and a directory.
    pub fn new(prefix: SharedLibraryPrefix, directory: PathBuf) -> Self {
        Self { prefix, directory }
    }

    /// Parses a [`LookupDirectory`] from a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Parse until the first `:`, which separates the prefix from the directory.
        let prefix = cut_err(
            repeat_till(1.., any, peek(alt((":", eof))))
                .try_map(|(name, _): (String, &str)| SharedLibraryPrefix::from_str(&name)),
        )
        .context(StrContext::Label("prefix for a shared object lookup path"))
        .parse_next(input)?;

        // Take the delimiter.
        cut_err(":")
            .context(StrContext::Label("shared library prefix delimiter"))
            .context(StrContext::Expected(StrContextValue::Description(
                "shared library prefix `:`",
            )))
            .parse_next(input)?;

        // Parse the rest as a directory.
        let directory = rest.map(PathBuf::from).parse_next(input)?;

        Ok(Self { prefix, directory })
    }
}

impl FromStr for LookupDirectory {
    type Err = Error;

    /// Creates a [`LookupDirectory`] from a string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}
