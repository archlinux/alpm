use std::fmt::Display;
use std::fmt::Formatter;
use std::string::FromUtf8Error;

use alpm_types::SchemaVersion;
use thiserror::Error;

/// A line of text and a line number the line is from
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ErrorLine {
    /// The line number that the  error occurred at
    pub number: usize,
    /// The full line containing the error
    pub line: String,
}

impl Display for ErrorLine {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "Line {}: {}", self.number, self.line)
    }
}

/// The Error that can occur when using types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// ALPM type error
    #[error("ALPM type parse error: {0}")]
    AlpmType(#[from] alpm_types::Error),

    /// A generic default error
    #[error("A generic error occurred: {0}")]
    Default(String),

    /// Failed creating a directory
    #[error("Failed creating directory: {0}")]
    FailedDirCreation(String),

    /// Failed creating a file
    #[error("Failed creating file: {0}")]
    FailedFileCreation(String),

    /// Failed reading a BUILDINFO file
    #[error("Failed reading BUILDINFO file: {0}")]
    FailedReadingFile(String),

    /// Failed reading from stdin
    #[error("Failed reading from stdin: {0}")]
    FailedReadingStdin(String),

    /// Failed writing a BUILDINFO file
    #[error("Failed writing BUILDINFO file: {0}")]
    FailedWriting(String),

    /// An invalid BuildInfo version is encountered
    #[error("Invalid BUILDINFO version: {0}")]
    InvalidBuildInfoVersion(String),

    /// A mandatory key-value pair is missing in a BuildInfo
    #[error("The mandatory BUILDINFO v{0} field '{1}' is missing")]
    MissingKeyValue(String, String),

    /// A duplicate field is found in a BuildInfo
    #[error("In BUILDINFO v{0} using the field '{1}' more than once is not allowed. {2}")]
    MultipleOccurences(String, String, ErrorLine),

    /// A SchemaVersion with the wrong version is used to initialize a BuildInfo
    #[error("Wrong schema version used to create a BUILDINFO: {0}")]
    WrongSchemaVersion(SchemaVersion),

    /// An error occurred while parsing a string as UTF-8
    #[error(transparent)]
    InvalidUTF8(#[from] FromUtf8Error),

    /// No input file given
    #[error("No input file given.")]
    NoInputFile,

    // Deserialize error.
    #[error("Failed to deserialize BUILDINFO file: {0}")]
    DeserializeError(#[from] alpm_parsers::custom_ini::Error),

    /// BuildInfo file is missing the format field
    #[error("Missing format field")]
    MissingFormatField,
}
