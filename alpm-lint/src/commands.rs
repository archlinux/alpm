use std::{env::current_dir, path::PathBuf};

use alpm_lint::{
    Error,
    LintScope,
    cli::OutputFormat,
    lint_rules::store::LintStore,
    prelude::Resources,
};
use alpm_lint_config::LintRuleConfiguration;

/// Run a lint check on a path with a potentially explicit, but otherwise automatically detected
/// [`LintScope`].
pub fn check(path: Option<PathBuf>, scope: Option<LintScope>) -> Result<(), Error> {
    // Get the given path or use the cwd.
    let path = match path {
        Some(path) => path,
        None => current_dir().map_err(|source| Error::Io {
            context: "detect current working directory",
            source,
        })?,
    };

    // Get or detect the scope.
    let scope = match scope {
        Some(scope) => scope,
        None => LintScope::detect(&path)?,
    };

    let _resources = Resources::gather(&path, scope)?;

    Ok(())
}

/// Return the definition of all linting rules.
///
/// Returns a map of [`SerializableLintRule`]s in serialized form.
pub fn rules(output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    // Create a stub default configuration.
    let config = LintRuleConfiguration::default();
    let store = LintStore::new(&config);

    let lint_rules = store.serializable_lint_rules();

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                if pretty {
                    serde_json::to_string_pretty(&lint_rules)?
                } else {
                    serde_json::to_string(&lint_rules)?
                }
            );
        }
        OutputFormat::Toml => {
            println!(
                "{}",
                if pretty {
                    toml::to_string_pretty(&lint_rules)?
                } else {
                    toml::to_string(&lint_rules)?
                }
            );
        }
    };

    Ok(())
}

/// Return the definitions of all linting options.
pub fn options(output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    let options = LintRuleConfiguration::configuration_options();

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                if pretty {
                    serde_json::to_string_pretty(&options)?
                } else {
                    serde_json::to_string(&options)?
                }
            );
        }
        OutputFormat::Toml => {
            println!(
                "{}",
                if pretty {
                    toml::to_string_pretty(&options)?
                } else {
                    toml::to_string(&options)?
                }
            );
        }
    };

    Ok(())
}
