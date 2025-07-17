use std::{env::current_dir, path::PathBuf};

use alpm_lint::{
    Error,
    Level,
    LintScope,
    LintStore,
    Resources,
    cli::{CheckOutputFormat, OutputFormat},
};
use alpm_lint_config::{LintConfiguration, LintRuleConfiguration};
use log::debug;

/// Run a lint check on a path with a potentially explicit, but otherwise automatically detected
/// [`LintScope`].
pub fn check(
    path: Option<PathBuf>,
    scope: Option<LintScope>,
    output_format: CheckOutputFormat,
    pretty: bool,
    level: Level,
    config_path: Option<PathBuf>,
) -> Result<(), Error> {
    // Get the given path or use the cwd.
    let path = match path {
        Some(path) => path,
        None => current_dir().map_err(|source| Error::Io {
            context: "detect current working directory",
            source,
        })?,
    };
    debug!("Using path: {path:?}");

    // Load the config or fall back to the default config.
    let config = if let Some(path) = config_path {
        LintConfiguration::from_path(&path)?
    } else {
        LintConfiguration::default()
    };

    // Get or detect the scope.
    let scope = match scope {
        Some(scope) => scope,
        None => LintScope::detect(&path)?,
    };
    debug!("Detected scope: {scope:?}");

    let resources = Resources::gather(&path, scope)?;
    debug!("Resources have been gathered");

    let store = LintStore::new(config);

    let mut issues = Vec::new();
    let lint_rules = store.filtered_lint_rules(&scope);

    debug!("Start of linting.");
    for (name, rule) in lint_rules {
        // Skip any lint rules that're below the specified severity level threshold.
        if rule.level() < level {
            continue;
        }
        debug!("Running rule: '{name}'");
        rule.run(&resources, &mut issues)?;
    }

    debug!("Using output format {output_format:?}.");
    match output_format {
        CheckOutputFormat::Text => {
            for issue in &issues {
                println!("{issue:?}");
            }
        }
        CheckOutputFormat::Json => {
            println!(
                "{}",
                if pretty {
                    serde_json::to_string_pretty(&issues)?
                } else {
                    serde_json::to_string(&issues)?
                }
            );
        }
        CheckOutputFormat::Toml => {
            println!(
                "{}",
                if pretty {
                    toml::to_string_pretty(&issues)?
                } else {
                    toml::to_string(&issues)?
                }
            );
        }
    };

    // Exit with code 1 if there were any relevant lints.
    if issues.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Return the definition of all linting rules.
///
/// Returns a map of [`SerializableLintRule`]s in serialized form.
pub fn rules(output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    // Create a stub default configuration.
    let config = LintConfiguration::default();
    let store = LintStore::new(config);

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
