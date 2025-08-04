#![doc = include_str!("../README.md")]

pub mod locale;
pub use locale::{get_locale, set_locale};

mod error;
pub use error::Error;

mod value;
pub use value::{FluentValue, ToFluentValue};

mod macros;

/// Re-export the [`fluent_templates`] crate.
pub use fluent_templates;

// Crate locale.
//
// NOTE: We use the explicit initialization instead of `i18n!` macro
// here to avoid importing `fluent_templates` twice.
#[cfg(not(test))]
fluent_templates::static_loader! {
    static LOCALES = {
        locales: "locales",
        fallback_language: "en-US",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

// Test locale.
//
//
// NOTE: We use the explicit initialization instead of `i18n!` macro
// here to avoid importing `fluent_templates` twice.
#[cfg(test)]
fluent_templates::static_loader! {
    static LOCALES = {
        locales: "tests/locales",
        fallback_language: "en-US",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}
