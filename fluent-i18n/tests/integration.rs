//! Integration tests for the i18n system.

use std::{env, str::FromStr, thread};

use fluent_i18n::{get_locale, i18n, set_locale, t};
use unic_langid::LanguageIdentifier;

// Test the French translations with fallback to English.
#[test]
fn i18n_french_with_fallback() -> testresult::TestResult {
    i18n!("tests/locales", fallback = "en-US");
    set_locale(Some("fr-FR"))?;

    // Lookup some French translations
    assert_eq!(t!(LOCALES, "greeting"), "Bonjour, le monde!");
    assert_eq!(
        t!(LOCALES, "welcome", { "name" => "Orhun" }),
        "Bienvenue, Orhun!"
    );
    assert_eq!(
        t!(LOCALES, "count-items", { "count" => 1 }),
        "Vous avez 1 élément"
    );
    assert_eq!(
        t!(LOCALES, "count-items", { "count" => 5 }),
        "Vous avez 5 éléments"
    );

    // Unknown key
    assert_eq!(t!(LOCALES, "unknown"), "Unknown localization unknown");

    // Fallback to English for missing key
    assert_eq!(
        t!(LOCALES, "missing-in-other"),
        "This message only exists in English"
    );

    Ok(())
}

// Test the Arabic translations without Unicode isolation characters.
#[test]
fn i18n_mixed_script_no_isolation() -> testresult::TestResult {
    i18n!("tests/locales");
    set_locale(Some("ar-SA"))?;

    let message = t!(LOCALES, "welcome", { "name" => "John Smith" });

    // Should not include Unicode isolation characters
    assert_eq!(message, "أهلاً وسهلاً، John Smith！");
    assert!(!message.contains('\u{2068}'));
    assert!(!message.contains('\u{2069}'));

    Ok(())
}

// Test the behavior of thread-local locales.
#[test]
fn test_thread_local_behavior() -> testresult::TestResult {
    i18n!("tests/locales");

    let main_msg = t!(LOCALES, "greeting");
    assert_eq!(main_msg, "Hello, world!");

    // Change the locale in another thread
    let handle = thread::spawn(|| {
        set_locale(Some("fr-FR")).expect("Failed to set locale in thread");
        t!(LOCALES, "greeting")
    });

    let child_msg = handle.join().expect("Failed to join thread");
    assert_eq!(child_msg, "Bonjour, le monde!");

    // Main thread should still be using en-US
    assert_eq!(t!(LOCALES, "greeting"), "Hello, world!");

    Ok(())
}

// Test the i18n system with a different fallback locale.
#[test]
fn i18n_with_different_fallback() -> testresult::TestResult {
    i18n!("tests/locales", fallback = "fr-FR");

    // Should trigger fallback
    set_locale(Some("non-existent"))?;

    assert_eq!(t!(LOCALES, "greeting"), "Bonjour, le monde!");

    // Using a key only available in English
    // but the fallback is set to French
    assert_eq!(
        t!(LOCALES, "missing-in-other"),
        "Unknown localization missing-in-other"
    );

    // Using a key only available in French
    // works because we set fallback to French
    assert_eq!(
        t!(LOCALES, "only-in-french"),
        "Ceci est uniquement en français"
    );

    Ok(())
}

// Test setting the locale with an environment variable.
#[test]
fn i18n_with_env() -> testresult::TestResult {
    // Set locale to Japanese
    unsafe {
        env::set_var("LANGUAGE", "ja-JP");
        env::set_var("LANG", "ja-JP");
    }

    i18n!("tests/locales", fallback = "en-US");
    set_locale(None)?;

    assert_eq!(
        get_locale().language,
        LanguageIdentifier::from_str("ja")?.language
    );
    assert_eq!(t!(LOCALES, "greeting"), "こんにちは、世界！");

    // Fallback to English for missing key
    assert_eq!(
        t!(LOCALES, "missing-in-other"),
        "This message only exists in English"
    );
    Ok(())
}
