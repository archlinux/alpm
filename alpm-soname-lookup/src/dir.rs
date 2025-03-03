use std::{fmt::Display, path::PathBuf, str::FromStr};

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

impl Display for LookupDirectory {
    /// Converts the [`LookupDirectory`] to a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.directory.display())
    }
}

impl FromStr for LookupDirectory {
    type Err = Error;

    /// Creates a [`LookupDirectory`] from a string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("lib:libexample.so", LookupDirectory {
        prefix: "lib".parse().unwrap(),
        directory: PathBuf::from("libexample.so"),
    })]
    #[case("usr:libexample.so.1", LookupDirectory {
        prefix: "usr".parse().unwrap(),
        directory: PathBuf::from("libexample.so.1"),
    })]
    #[case("lib:libexample.so.1.2.3", LookupDirectory {
        prefix: "lib".parse().unwrap(),
        directory: PathBuf::from("libexample.so.1.2.3"),
    })]
    fn lookup_directory_from_string(
        #[case] input: &str,
        #[case] expected_result: LookupDirectory,
    ) -> Result<(), Error> {
        let lookup_directory = LookupDirectory::from_str(input)?;
        assert_eq!(expected_result, lookup_directory);
        assert_eq!(input, lookup_directory.to_string());
        Ok(())
    }

    #[rstest]
    #[case("libexample.so.1", "invalid shared library prefix delimiter")]
    fn invalid_lookup_directory_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = LookupDirectory::from_str(input);
        assert!(result.is_err(), "Expected LookupDirectory parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
