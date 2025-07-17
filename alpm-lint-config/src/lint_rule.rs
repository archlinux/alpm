//! The linting configuration for all lint rules.
use serde::{Deserialize, Serialize};

/// Take the default value for a given option and convert it into its equivalent toml
/// representation. This is used to expose the internal default values to users.
///
/// If a second argument is provided, that value will be considered as an override.
/// This is useful for options that determine its default value based on the runtime
/// environment.
macro_rules! default_text {
    ($value:expr) => {{
        let mut text = String::new();
        $value
            .serialize(toml::ser::ValueSerializer::new(&mut text))
            .unwrap();
        text
    }};
    ($value:expr, $override:expr) => {
        $override.to_string()
    };
}

/// This macro defines the configuration struct for all available linting options.
///
/// Every line can the same rough structure:
///
/// ```text
/// /// This is a test option
/// test_option: String = "This is an option",
/// ```
///
/// - `/// This is a test option`: The documentation that'll be displayed everywhere for this
///   option.
/// - `test_option`: name of the option.
/// - `: String`: the type of the option.
/// - `= "This is an option"`: The default value for this option. Everything that converts via
///   `Into` into the expected type is accepted.
///
/// # Options
///
/// The macro accepts additional options. These have the form of `#[option_name = option_value]`
/// and are positioned above the option declaration, such as:
///
/// ```text
/// /// This is a test option
/// #[default_text = "my default documentation text"]
/// test_option: String = "This is an option",
/// ```
///
/// - `#[default_text = "some default"]`: Use this, if your default value depends on runtime data.
///   This is used to set the human readable text in documentation contexts. For example set
///   `#[default_text = "current architecture"]` for lints that default to the system's architecture
///   that is detected during linting runtime.
#[macro_export]
macro_rules! linting_config {
    ($(
        $(#[doc = $doc:literal])+
        $(#[default_text = $default_text:expr])?
        $name:ident: $type:ty = $default:expr,
    )*) => {
        use std::collections::BTreeMap;

        /// Configuration struct that contains all options to adjust ALPM-related linting rules.
        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
        pub struct LintRuleConfiguration {
            $(
                $(#[doc = $doc])+
                pub $name: $type
            )*
        }

        /// This module contains the default value functions for every configuration option.
        mod defaults {
            $(#[inline] pub fn $name() -> $type { $default.into() })*
        }

        impl Default for LintRuleConfiguration {
            fn default() -> Self {
                Self {
                    $($name: defaults::$name(),)*
                }
            }
        }

        impl LintRuleConfiguration {
            /// Returns the map of all configuration options with their respective name, default value
            /// and documentation.
            ///
            /// This function is mainly designed to be used to generate the public documentation of
            /// alpm-linting and for development integration.
            ///
            /// ```
            /// use alpm_lint_config::LintRuleConfiguration;
            ///
            /// println!("{:?}", LintRuleConfiguration::configuration_options());
            /// ```
            pub fn configuration_options() -> BTreeMap<&'static str, LintRuleConfigurationOption> {
                let mut map = BTreeMap::new();
                $(
                    map.insert(stringify!($name), LintRuleConfigurationOption {
                        name: stringify!($name).to_string(),
                        default: default_text!(defaults::$name() $(, $default_text)?),
                        doc: concat!($($doc, '\n',)*),
                    });
                )*

                map
            }

        }

        /// An enum with variants representing the literal field names of [`LintRuleConfiguration`].
        ///
        /// The purpose of this enum is to allow lint rules to point to specific options that
        /// they require, as we need some form of identifier for that. We cannot point to the
        /// [`LintRuleConfiguration`] fields directly, so this is the next best thing.
        #[derive(Debug, strum::Display)]
        // To convert the names to CamelCase, we would have to write a custom proc-macro, so we can
        // runs that logic during compile time. As this enum is only used for inter-linking inside our own
        // crate, having a proc-macro would be a bit overkill.
        #[allow(non_camel_case_types)]
        #[allow(missing_docs)]
        pub enum LintRuleConfigurationOptionName {
            $($name)*,
        }
    }
}

/// Represents a single configuration option.
///
/// This struct is mainly used to do automatic
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LintRuleConfigurationOption {
    /// The name of the configuration option.
    pub name: String,
    /// The stringified `toml` value of the default value for this option.
    pub default: String,
    /// The documentation for this option.
    pub doc: &'static str,
}

linting_config! {
    /// This is a test option
    test_option: String = "This is an option",
}
