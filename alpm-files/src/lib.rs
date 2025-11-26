#![doc = include_str!("../README.md")]

pub mod files;

// Initialize i18n support.
fluent_i18n::i18n!("locales");
