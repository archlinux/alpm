/// The error that can occur when working with the ALPM database desc files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A winnow parser for a type didn't work and produced an error.
    #[error("Parser failed with the following error:\n{0}")]
    ParseError(String),

    /// A section is missing in the parsed data.
    #[error("Missing field: {0}")]
    MissingSection(String),

    /// An unknown section was encountered in the parsed data.
    #[error("Unknown keyword: {0}")]
    UnknownSection(String),
}

impl<'a> From<winnow::error::ParseError<&'a str, winnow::error::ContextError>> for Error {
    /// Converts a [`winnow::error::ParseError`] into an [`Error::ParseError`].
    fn from(value: winnow::error::ParseError<&'a str, winnow::error::ContextError>) -> Self {
        Self::ParseError(value.to_string())
    }
}
