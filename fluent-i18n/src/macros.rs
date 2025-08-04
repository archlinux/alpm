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
///
/// [`t!`]: crate::t!
#[macro_export]
macro_rules! i18n {
    ($dir:expr, fallback = $fallback:literal) => {
        use $crate::fluent_templates;
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
        use $crate::fluent_templates;
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
/// ```rust,ignore
/// use fluent_templates::Loader;
/// use fluent_i18n::t;
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
///
/// [`ToFluentValue`]: crate::ToFluentValue
/// [`FluentValue`]: fluent_templates::fluent_bundle::FluentValue
// Allow using `crate::` in the macro definition.
//
// See <https://rust-lang.github.io/rust-clippy/master/index.html#crate_in_macro_def>
//
// This is normally not recommended for macro hygiene reasons,
// but we need it here to access the `LOCALES` static loader
// from the crate where it is defined.
//
// In other words, using `$crate` points to `alpm_common::LOCALES`
// which is not what we want. Instead, we want to access the `LOCALES`
// from the crate where the macro is invoked.
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! t {
    // t!("key")
    ($key:expr) => {{
        use $crate::fluent_templates::Loader;
        crate::LOCALES.lookup(&$crate::get_locale(), $key)
    }};

    // t!("key", { arg => val, ... })
    ($key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        use $crate::fluent_templates::Loader;
        use $crate::ToFluentValue;
        use std::borrow::Cow;
        let mut args = ::std::collections::HashMap::new();
        $(
            args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
        )+
        crate::LOCALES.lookup_with_args(&$crate::get_locale(), $key, &args)
    }};

    // t!(LOCALES, "key")
    ($locales:expr, $key:expr) => {{
        use $crate::fluent_templates::Loader;
        $locales.lookup(&$crate::get_locale(), $key)
    }};

    // t!(LOCALES, "key", { arg => val, ... })
    ($locales:expr, $key:expr, { $($arg:expr => $val:expr),+ $(,)? }) => {{
        use $crate::fluent_templates::Loader;
        use $crate::ToFluentValue;
        use std::borrow::Cow;
        let mut args = ::std::collections::HashMap::new();
        $(
            args.insert(Cow::Borrowed($arg), $val.to_fluent_value());
        )+
        $locales.lookup_with_args(&$crate::get_locale(), $key, &args)
    }};
}

#[cfg(test)]
mod tests {
    use crate::set_locale;

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
}
