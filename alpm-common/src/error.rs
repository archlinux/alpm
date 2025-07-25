use std::path::{PathBuf, StripPrefixError};

use unic_langid::LanguageIdentifierError;

use crate::t;

/// An error that can occur when dealing with package inputs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error occurred at a path.
    #[error("{msg}", msg = t!("error-io-path", { "context" => context, "path" => path }))]
    IoPath {
        /// The path at which the error occurred.
        path: PathBuf,
        /// The context in which the error occurred.
        ///
        /// This is meant to complete the sentence "I/O error at path while ".
        context: &'static str,
        /// The source error.
        source: std::io::Error,
    },

    /// A path is not a directory.
    #[error("{msg}", msg = t!("error-not-a-directory", { "path" => path }))]
    NotADirectory {
        /// The path that is not a directory.
        path: PathBuf,
    },

    /// One or more paths are not absolute.
    #[error("{msg}", msg = t!("error-non-absolute-paths", {
        "paths" => paths.iter().fold(
            String::new(),
            |mut output, path| {
                output.push_str(&format!("{path:?}\n"));
                output
            }
        )
    }))]
    NonAbsolutePaths {
        /// The list of non-absolute paths.
        paths: Vec<PathBuf>,
    },

    /// One or more paths are not relative.
    #[error("{msg}", msg = t!("error-non-relative-paths", {
        "paths" => paths.iter().fold(
            String::new(),
            |mut output, path| {
                output.push_str(&format!("{path:?}\n"));
                output
            }
        )
    }))]
    NonRelativePaths {
        /// The list of non-relative paths.
        paths: Vec<PathBuf>,
    },

    /// A path's prefix cannot be stripped.
    #[error("{msg}\n{source}", msg = t!("error-path-strip-prefix", {
        "prefix" => prefix,
        "path" => path
    }))]
    PathStripPrefix {
        /// The prefix that is supposed to be stripped from `path`.
        prefix: PathBuf,
        /// The path that is supposed to stripped.
        path: PathBuf,
        /// The source error.
        source: StripPrefixError,
    },

    /// An error occurred while parsing a locale.
    #[error("{msg}\n{source}", msg = t!("error-locale-parse", {
        "locale" => locale
    }))]
    LocaleParseError {
        /// The locale string that could not be parsed.
        locale: String,
        /// The source error.
        source: LanguageIdentifierError,
    },
}

#[cfg(test)]
mod tests {
    use crate::i18n;

    i18n!("locales", fallback = "en-US");

    #[test]
    fn test_localized_io_path_error() -> testresult::TestResult {
        use std::{io, path::PathBuf};

        use crate::Error;

        // Initialize localization
        crate::locale::set_locale(Some("en-US"))?;

        // Construct the error
        let err = Error::IoPath {
            path: PathBuf::from("/etc/foo"),
            context: "reading",
            source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
        };

        let msg = err.to_string();

        // Check translation was applied
        assert!(
            msg.contains("I/O error while reading: /etc/foo"),
            "Unexpected translation: {msg}"
        );

        Ok(())
    }
}
