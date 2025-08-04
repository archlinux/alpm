//! Locale management methods and macros.

use std::{
    cell::RefCell,
    sync::{LazyLock, OnceLock},
};

use fluent_templates::LanguageIdentifier;
use unic_langid::langid;

use crate::Error;

/// The default locale used when no other locale is set or detected (`"en-US"`).
///
/// This is lazily initialized to avoid unnecessary overhead if not used.
static DEFAULT_LOCALE: LazyLock<LanguageIdentifier> = LazyLock::new(|| langid!("en-US"));

thread_local! {
    /// The current runtime locale.
    ///
    /// This is set using the [`set_locale`] function and can be accessed via [`get_locale`].
    ///
    /// # Thread Safety
    ///
    /// This is thread-local storage, meaning each thread has its own instance of this variable.
    static CURRENT_LOCALE: RefCell<Option<LanguageIdentifier>> = const { RefCell::new(None) };

    /// Indicates whether the raw mode is enabled.
    ///
    /// When raw mode is enabled, translations will return the key itself instead of
    /// looking up the translation. This is useful for debugging purposes to see which keys are
    /// being requested.
    ///
    /// # Thread Safety
    ///
    /// This is thread-local storage, meaning each thread has its own instance of this variable.
    pub static RAW_MODE_ENABLED: RefCell<bool> = const { RefCell::new(false) };
}

/// The fallback locale used when no other locale is set or detected.
///
/// This is set during the static loader initialization and used when
/// the current locale is not available.
///
/// # Note
///
/// This is public to allow access in the [`i18n!`] macro for setting the fallback locale.
///
/// [`i18n!`]: crate::i18n
pub static FALLBACK_LOCALE: OnceLock<LanguageIdentifier> = OnceLock::new();

/// Sets the current runtime locale.
///
/// It attempts to set the locale in the following order:
///
/// 1. Use the given `locale` string if valid.
/// 2. Try to detect the system locale (via [`sys_locale`]).
/// 3. Fallback to the statically configured fallback locale.
/// 4. If all else fails, use the default locale (`"en-US"`).
///
/// If `locale` is [`None`], this function will try to detect the system locale.
///
/// # Errors
///
/// Returns an error if:
///
/// - the provided `locale` string cannot be parsed into a [`LanguageIdentifier`],
/// - or the system locale cannot be parsed into a [`LanguageIdentifier`].
pub fn set_locale(locale: Option<&str>) -> Result<(), Error> {
    let langid = if let Some(loc) = locale {
        loc.parse::<LanguageIdentifier>()
            .map_err(|source| Error::LocaleParseError {
                locale: loc.to_string(),
                source,
            })?
    } else if let Some(sys_loc) = sys_locale::get_locale() {
        sys_loc
            .parse::<LanguageIdentifier>()
            .map_err(|source| Error::LocaleParseError {
                locale: sys_loc.to_string(),
                source,
            })?
    } else if let Some(fallback) = FALLBACK_LOCALE.get() {
        fallback.clone()
    } else {
        DEFAULT_LOCALE.clone()
    };

    CURRENT_LOCALE.with(|cell| {
        *cell.borrow_mut() = Some(langid);
    });

    Ok(())
}

/// Returns the current locale.
///
/// This function retrieves the current locale by:
///
/// 1. Checking if a current locale is set using [`set_locale`].
/// 2. If current locale is not set, the fallback locale is returned instead.
/// 3. If neither current nor fallback locale are set, the default locale (`"en-US"`) is returned.
///
/// # Thread Safety
///
/// This function is thread-safe and uses thread-local storage to manage the current locale.
/// In other words, each thread has its own instance of the current locale.
pub fn get_locale() -> LanguageIdentifier {
    CURRENT_LOCALE.with(|cell| {
        cell.borrow()
            .clone()
            .or_else(|| FALLBACK_LOCALE.get().cloned())
            .unwrap_or_else(|| DEFAULT_LOCALE.clone())
    })
}

/// Enables or disables the raw mode.
///
/// When raw mode is enabled, translations will return the key itself instead of
/// looking up the translation. This is useful for debugging purposes to see which keys are
/// being requested.
pub fn set_raw_mode(enabled: bool) {
    RAW_MODE_ENABLED.with(|cell| {
        *cell.borrow_mut() = enabled;
    });
}

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr};

    use testresult::TestResult;
    use unic_langid::subtags::Language;

    use super::*;
    use crate::t;

    /// Ensures that the missing keys fallback to the English locale.
    #[test]
    fn test_localization_fallback_to_english() -> TestResult<()> {
        set_locale(Some("fr-FR"))?;

        assert_eq!(t!("greeting"), "Bonjour, le monde!");
        assert_eq!(
            t!("english-only-translation"),
            "This message only exists in English"
        );

        Ok(())
    }

    /// Asserts the erroneous lookups return the expected value.
    #[test]
    fn test_localization_message_not_found() -> TestResult<()> {
        set_locale(Some("en-US"))?;
        let result = t!("nonexistent-key");
        assert_eq!(result, "Unknown localization key: \"nonexistent-key\"");

        set_locale(Some("fr-FR"))?;
        let result = t!("nonexistent-key");
        assert_eq!(result, "Unknown localization key: \"nonexistent-key\"");

        Ok(())
    }

    /// Ensures that the locale can be set to a valid language code
    /// and that the translations are correctly applied with the
    /// default fallback locale.
    #[test]
    fn test_localization_invalid_locale_english_fallback() -> TestResult<()> {
        set_locale(Some("invalid-locale"))?;

        assert_eq!(get_locale().language, Language::from_str("invalid")?);
        assert_eq!(t!("greeting"), "Hello, world!");
        Ok(())
    }

    /// Ensure that setting the locale with an environment variable
    /// works as expected.
    #[test]
    fn test_localization_via_lang_env() -> TestResult<()> {
        unsafe {
            env::set_var("LANGUAGE", "ja-JP");
        }
        set_locale(None)?;
        assert_eq!(get_locale().language, Language::from_str("ja")?);

        assert_eq!(t!("greeting"), "こんにちは、世界！");
        assert_eq!(
            t!("welcome", { "name" => "Orhun" }),
            "ようこそ、Orhunさん！"
        );

        // Test the environment variable fallback
        unsafe {
            env::remove_var("LANGUAGE");
            env::set_var("LC_ALL", "ja-JP");
        }
        set_locale(None)?;

        assert_eq!(get_locale().language, Language::from_str("ja")?);
        assert_eq!(
            t!("count-items", { "count" => 1 }),
            "1個のアイテムがあります"
        );
        assert_eq!(
            t!("count-items", { "count" => 5 }),
            "5個のアイテムがあります"
        );

        Ok(())
    }

    // Ensures that setting an unknown locale
    // falls back to the default locale and
    // the translations are still available.
    #[test]
    fn test_unknown_locale() -> TestResult<()> {
        assert!(set_locale(Some("???")).is_err());

        set_locale(Some("unknown-locale"))?;
        assert_eq!(
            get_locale(),
            LanguageIdentifier::from_str("unknown-locale")?,
        );
        assert_eq!(t!("greeting"), "Hello, world!");
        Ok(())
    }

    /// Ensures that the raw mode works as expected.
    /// When raw mode is enabled, the translations
    /// should return the key itself.
    #[test]
    fn test_raw_mode() -> TestResult<()> {
        set_locale(Some("en-US"))?;
        set_raw_mode(true);
        assert_eq!(t!("greeting"), "greeting");
        assert_eq!(t!("welcome", { "name" => "Orhun" }), "welcome");

        set_locale(Some("ja-JP"))?;
        assert_eq!(t!("whatever"), "whatever");

        Ok(())
    }
}
