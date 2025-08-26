//! The linting configuration for all lint rules.
use serde::{Deserialize, Serialize};

/// Returns the serialized TOML representation of a default value.
///
/// Takes the default value for a given option and converts it into its equivalent TOML
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
            .expect(&format!(
                "Failed to serialize default text: '{:?}' to toml",
                $value
            ));
        text
    }};
    ($value:expr, $override:expr) => {
        $override.to_string()
    };
}

/// Creates the [`LintRuleConfiguration`] struct that defines all available lint rules.
///
/// Every lint rule is defined using the same structure, e.g.:
///
/// ```text
/// /// This is a test option
/// test_option: String = "This is a default",
/// ```
///
/// - `/// This is a test option`: The documentation that is associated with the option.
/// - `test_option`: The name of the option.
/// - `: String`: The data type of the option value.
/// - `= "This is a default"`: The default value for the option. Anything that implements [`Into`]
///   for the target data type is accepted.
///
/// # Parameters
///
/// The macro accepts additional parameters. These have the form of `#[param_name = param_value]`
/// and are positioned above the option declaration, for example:
///
/// This list of parameters is currently supported:
///
/// - `#[default_text = "some default"]`: Use this, if your default value depends on runtime data.
///   This is used to set the human readable text in documentation contexts. For example set
///   `#[default_text = "current architecture"]` for lints that default to the system's architecture
///   that is detected during linting runtime.
///
///   Example:
///   ```text
///   /// This is a test option
///   #[default_text = "current architecture"]
///   architecture: Option<Architecture> = None,
///   ```
#[macro_export]
macro_rules! create_lint_rule_config {
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
            /// This function is mainly designed to generate the public documentation of
            /// alpm-lint and for development tooling.
            ///
            /// # Examples
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
        // NOTE: To convert the names to CamelCase, we would have to write a custom proc-macro, so we can
        // run that logic during compile time. As this enum is only used for inter-linking inside our own
        // crate, having a proc-macro would be a bit overkill.
        #[allow(non_camel_case_types)]
        pub enum LintRuleConfigurationOptionName {
            $(
                $(#[doc = $doc])+
                $name
            )*,
        }
    }
}

/// Represents a single configuration option.
///
/// This struct is used to do expose information about LintRule options as structure data via
/// `LintRuleConfiguration::configuration_options`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LintRuleConfigurationOption {
    /// The name of the configuration option.
    pub name: String,
    /// The stringified `toml` value of the default value for this option.
    pub default: String,
    /// The documentation for this option.
    pub doc: &'static str,
}

create_lint_rule_config! {
    /// This is a test option
    test_option: String = "This is an option",
}
