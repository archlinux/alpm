//! Locale management methods and macros.

use std::{
    borrow::Cow,
    cell::RefCell,
    sync::{LazyLock, OnceLock},
};

use fluent_templates::{LanguageIdentifier, fluent_bundle::FluentValue};
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
    static CURRENT_LOCALE: RefCell<Option<LanguageIdentifier>> = RefCell::new(None);
}

/// The fallback locale used when no other locale is set or detected.
///
/// This is set during the static loader initialization and used when
/// the current locale is not available.
///
/// Note: This is public to allow access in the `i18n!` macro for setting the fallback locale.
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
/// # Errors
///
/// Returns an error if:
///
/// - The provided `locale` string cannot be parsed into a [`LanguageIdentifier`].
/// - The system locale cannot be parsed into a [`LanguageIdentifier`].
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
/// 1. Checking if a locale has been set using [`set_locale`].
/// 2. If not set, it checks the fallback locale.
/// 3. If neither is set, it returns the default locale (`"en-US"`).
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

/// Macro to initialize the i18n system with a specified directory for locale files.
///
/// It should be called at the start of the application (either in `main.rs` or `lib.rs`).
///
/// It supports two forms of usage:
///
/// 1. `i18n!("locales", fallback = "en-US")`
///
///  Initializes the i18n system with the specified directory ("locales") and
///  sets the fallback locale to the provided value.
///
/// 2. `i18n!("locales")`
///
///  Initializes the i18n system with the specified directory ("locales") and
///  uses the default fallback locale ("en-US").
///
/// Then you can use the [`t!`] macro to look up translations.
///
/// # Note
///
/// Under the hood, this macro calls [`fluent_templates::static_loader!`] macro to
/// create a static loader named `LOCALES` and sets it up with the provided directory
/// and fallback locale. The loader can be accessed in the scope where this macro is called,
/// which is usually the root module of your application (e.g. `crate::LOCALES`).
#[macro_export]
macro_rules! i18n {
    ($dir:expr, fallback = $fallback:literal) => {
        fluent_templates::static_loader! {
            static LOCALES = {
                locales: $dir,
                fallback_language: $fallback,
                customise: |bundle| {
                    // Disable Unicode directional isolate characters (U+2068, U+2069)
                    // By default, Fluent wraps variables for security
                    // and proper text rendering in mixed-script environments (Arabic + English).
                    // Disabling gives cleaner output: "Welcome, Alice!" but reduces protection
                    // against bidirectional text attacks. Safe for English-only applications.
                    bundle.set_use_isolating(false);

                    // Set the fallback locale.
                    // It is a bit hacky to call this here, but `static_loader!`
                    // is a declarative macro and this is the only available place.
                    $crate::locale::FALLBACK_LOCALE
                        .set($fallback.parse().expect("Invalid fallback locale"))
                        .ok();
                }
            };
        }
    };
    ($dir:expr) => {
        fluent_templates::static_loader! {
            static LOCALES = {
                locales: $dir,
                fallback_language: "en-US",
                customise: |bundle| {
                    bundle.set_use_isolating(false);
                }
            };
        }
    };
}

/// Macro to lookup a translation for a given key.
///
/// This macro can be used in two ways:
///
/// 1. `t!("key")`
///
///   Looks up the translation for the given key in the current locale
///
/// 2. `t!("key", { arg1 => value1, arg2 => value2, ... })`
///
///   Looks up the translation for the given key in the current locale
///   and replaces the placeholders in the translation with the provided values.
///
///   The argument values must implement the [`ToFluentValue`] trait, which allows
///   converting various types to a [`FluentValue`].
///
/// # Note
///
/// Call [`i18n!`] macro to initialize the i18n system with default static loader
/// before using this macro.
///
/// # Using a custom static loader
///
/// If you don't want to use the [`i18n!`] macro or have a custom static loader,
/// you can use it with this macro by passing it as the first argument as follows:
///
/// ```rust
/// use fluent_templates::Loader;
/// use fluent_templates::FluentValue;
///
/// let custom_loader = fluent_templates::static_loader! {
///     static CUSTOM_LOCALES = {
///         locales: "path/to/locales",
///         fallback_language: "en-US",
///         customise: |bundle| {
///             bundle.set_use_isolating(false);
///         }
///     };
/// };
///
/// let translation = t!(CUSTOM_LOCALES, "key");
/// let translation_args = t!(CUSTOM_LOCALES, "key", { arg1 => value1, arg2 => value2 });
/// ```
#[macro_export]
macro_rules! t {
    // t!("key")
    ($key:expr) => {{
        use fluent_templates::Loader;
        crate::LOCALES.lookup(&$crate::locale::get_locale(), $key)
    }};

    // t!("key", { arg => val, ... })
    ($key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        use fluent_templates::Loader;
        use $crate::locale::ToFluentValue;
        use std::borrow::Cow;
        let mut args = ::std::collections::HashMap::new();
        $(
            args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
        )+
        crate::LOCALES.lookup_with_args(&$crate::locale::get_locale(), $key, &args)
    }};

    // t!(LOCALES, "key")
    ($locales:expr, $key:expr) => {{
        use fluent_templates::Loader;
        $locales.lookup(&$crate::locale::get_locale(), $key)
    }};

    // t!(LOCALES, "key", { arg => val, ... })
    ($locales:expr, $key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        use fluent_templates::Loader;
        use $crate::locale::ToFluentValue;
        use std::borrow::Cow;
        let mut args = ::std::collections::HashMap::new();
        $(
            args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
        )+
        $locales.lookup_with_args(&$crate::locale::get_locale(), $key, &args)
    }};
}

/// Helper trait for converting various types to a [`FluentValue`].
///
/// This trait is a wrapper for the `From` implementation of [`FluentValue`]
/// for providing more methods to convert different types.
///
/// One example is converting [`PathBuf`](std::path::PathBuf) which is
/// originally not supported but we support it by converting it to a string.
pub trait ToFluentValue {
    /// Converts the value to a [`FluentValue`].
    fn to_fluent_value(&self) -> FluentValue<'static>;
}

impl ToFluentValue for std::path::Path {
    fn to_fluent_value(&self) -> FluentValue<'static> {
        FluentValue::from(self.to_string_lossy().into_owned())
    }
}

impl ToFluentValue for std::path::PathBuf {
    fn to_fluent_value(&self) -> FluentValue<'static> {
        self.as_path().to_fluent_value()
    }
}

impl<'a, T> ToFluentValue for Option<T>
where
    T: ToFluentValue,
{
    fn to_fluent_value(&self) -> FluentValue<'static> {
        match self {
            Some(value) => value.to_fluent_value(),
            None => FluentValue::None,
        }
    }
}

/// Helper macro to implement the [`ToFluentValue`] trait for a list of types.
macro_rules! impl_fluent_for {
    ( $( $t:ty ),+ $(,)? ) => {
        $(
            impl ToFluentValue for $t {
                fn to_fluent_value(&self) -> FluentValue<'static> {
                    FluentValue::from(self.clone())
                }
            }
        )+
    };
}

// See <https://docs.rs/fluent-bundle/latest/fluent_bundle/enum.FluentValue.html#trait-implementations>
// for all the types that implement `From` for `FluentValue`.
impl_fluent_for!(
    String,
    Cow<'static, str>,
    usize,
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    &'static str
);

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr};

    use unic_langid::subtags::Language;

    use super::*;

    // Ensures that the value lookups with/without parameters
    // work correctly for the English locale.
    #[test]
    fn test_localization_lookup() -> testresult::TestResult<()> {
        set_locale(Some("en-US"))?;

        assert_eq!(t!("greeting"), "Hello, world!");
        assert_eq!(t!("welcome", { "name" => "Orhun" }), "Welcome, Orhun!");
        assert_eq!(t!("count-items", { "count" => 1 }), "You have 1 item");
        assert_eq!(t!("count-items", { "count" => 5 }), "You have 5 items");
        assert_eq!(t!("unknown"), "Unknown localization unknown");

        Ok(())
    }

    // Ensures that the missing keys fallback to the English locale.
    #[test]
    fn test_localization_fallback_to_english() -> testresult::TestResult<()> {
        set_locale(Some("fr-FR"))?;

        assert_eq!(t!("greeting"), "Bonjour, le monde!");
        assert_eq!(
            t!("missing-in-other"),
            "This message only exists in English"
        );

        Ok(())
    }

    // Asserts the erroneous lookups return the expected value.
    #[test]
    fn test_localization_message_not_found() -> testresult::TestResult<()> {
        set_locale(Some("en-US"))?;
        let result = t!("nonexistent-key");
        assert_eq!(result, "Unknown localization nonexistent-key");

        set_locale(Some("fr-FR"))?;
        let result = t!("nonexistent-key");
        assert_eq!(result, "Unknown localization nonexistent-key");

        Ok(())
    }

    /// Ensures that the locale can be set to a valid language code
    /// and that the translations are correctly applied with the
    /// default fallback locale.
    #[test]
    fn test_localization_invalid_locale_english_fallback() -> testresult::TestResult<()> {
        set_locale(Some("invalid-locale"))?;

        assert_eq!(get_locale().language, Language::from_str("invalid")?);
        assert_eq!(t!("greeting"), "Hello, world!");
        Ok(())
    }

    /// Ensure that setting the locale with an environment variable
    /// works as expected.
    #[test]
    fn test_localization_via_lang_env() -> testresult::TestResult<()> {
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

    // Ensures that Latin script names are NOT isolated in RTL context
    // since the Unicode directional isolation is disabled.
    #[test]
    fn test_unicode_directional_isolation_disabled() -> testresult::TestResult<()> {
        set_locale(Some("ar-SA"))?;

        let message = t!("welcome", { "name" => "John Smith" });
        assert!(!message.contains("\u{2068}John Smith\u{2069}"));
        assert_eq!(message, "أهلاً وسهلاً، John Smith！");

        Ok(())
    }

    // Asserts that the custom [`FluentValue`] conversions
    // work as expected (e.g. PathBuf)
    #[test]
    fn test_custom_fluent_type() -> testresult::TestResult<()> {
        set_locale(Some("en-US"))?;

        let path = std::path::PathBuf::from("/some/path/to/file.txt");
        let message = t!("error-io-path", { "context" => "reading", "path" => path });
        assert_eq!(message, "I/O error while reading: /some/path/to/file.txt");

        let opt_path: Option<std::path::PathBuf> = Some(path);
        let message = t!("error-io-path", { "context" => "writing", "path" => opt_path });
        assert_eq!(message, "I/O error while writing: /some/path/to/file.txt");

        Ok(())
    }

    // Ensures that setting an unknown locale
    // falls back to the default locale and
    // the translations are still available.
    #[test]
    fn test_unknown_locale() -> testresult::TestResult<()> {
        assert!(set_locale(Some("???")).is_err());

        set_locale(Some("unknown-locale"))?;
        assert_eq!(
            get_locale(),
            LanguageIdentifier::from_str("unknown-locale")?,
        );
        assert_eq!(t!("greeting"), "Hello, world!");
        Ok(())
    }
}
