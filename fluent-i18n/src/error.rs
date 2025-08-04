//! Error handling.

use unic_langid::LanguageIdentifierError;

use crate::t;

/// The error that may occur while using the crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while parsing a locale.
    #[error("{msg}\n{source}", msg = t!("error-locale-parse", { "locale" => locale }))]
    LocaleParseError {
        /// The locale string that could not be parsed.
        locale: String,
        /// The source error.
        source: LanguageIdentifierError,
    },
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    /// Make sure that the message of the error is properly translated
    #[rstest]
    #[case("en-US", r#"Could not parse locale "en-US""#)]
    #[case("fr-FR", r#"Impossible d’analyser la langue « fr-FR »"#)]
    #[case("de-DE", r#"Konnte die Spracheinstellung "de-DE" nicht analysieren"#)]
    #[case("tr-TR", r#""tr-TR" yerel ayarı tanınmadı"#)]
    #[case("ja-JP", r#"ロケール "ja-JP" を解析できませんでした"#)]
    #[case("ar-SA", r#"تعذر تحليل اللغة "ar-SA""#)]
    fn test_error_message_localization(
        #[case] locale: &str,
        #[case] expected: &str,
    ) -> testresult::TestResult {
        // Set the locale for the test
        crate::locale::set_locale(Some(locale))?;

        // Construct the error
        let err = Error::LocaleParseError {
            locale: locale.to_string(),
            source: LanguageIdentifierError::Unknown,
        };
        let msg = err.to_string();

        // Check translation was applied
        assert_eq!(
            msg.lines().next(),
            Some(expected),
            "Error message did not match expected translation"
        );

        Ok(())
    }
}
