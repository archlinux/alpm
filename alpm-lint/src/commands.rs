use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use alpm_lint::{
    Error,
    Level,
    LintScope,
    LintStore,
    Meta,
    Resources,
    cli::{CheckOutputFormat, OutputFormat},
};
use alpm_lint_config::{LintConfiguration, LintRuleConfiguration};
use log::debug;

/// Write content to either a file or stdout.
fn write_output(content: &str, output_path: Option<PathBuf>) -> Result<(), Error> {
    match output_path {
        Some(path) => {
            let mut file = File::create(&path).map_err(|source| Error::IoPath {
                path: path.clone(),
                context: "creating output file",
                source,
            })?;
            file.write_all(content.as_bytes())
                .map_err(|source| Error::IoPath {
                    path,
                    context: "writing to output file",
                    source,
                })?;
        }
        None => print!("{}", content),
    }
    Ok(())
}

/// Run a lint check on a path with a potentially explicit, but otherwise automatically detected
/// [`LintScope`].
pub fn check(
    config_path: Option<PathBuf>,
    path: Option<PathBuf>,
    scope: Option<LintScope>,
    level: Level,
    format: CheckOutputFormat,
    output: Option<PathBuf>,
    pretty: bool,
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

    debug!("Using output format {format:?}.");
    let content = match format {
        CheckOutputFormat::Text => issues
            .iter()
            .map(|issue| format!("{issue:?}"))
            .collect::<Vec<_>>()
            .join("\n"),
        CheckOutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&issues)?
            } else {
                serde_json::to_string(&issues)?
            }
        }
        CheckOutputFormat::Toml => {
            if pretty {
                toml::to_string_pretty(&issues)?
            } else {
                toml::to_string(&issues)?
            }
        }
    };

    write_output(&content, output)?;

    // Exit with code 1 if there were any relevant lints.
    if issues.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Return the definition of all linting rules.
///
/// Returns a map of [`SerializableLintRule`]s in serialized form.
pub fn rules(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    // Create a stub default configuration.
    let config = LintConfiguration::default();
    let store = LintStore::new(config);

    let lint_rules = store.serializable_lint_rules();

    let content = match output_format {
        OutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&lint_rules)?
            } else {
                serde_json::to_string(&lint_rules)?
            }
        }
        OutputFormat::Toml => {
            if pretty {
                toml::to_string_pretty(&lint_rules)?
            } else {
                toml::to_string(&lint_rules)?
            }
        }
    };

    write_output(&content, output)?;
    Ok(())
}

/// Return the definitions of all linting options.
pub fn options(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    let options = LintRuleConfiguration::configuration_options();

    let content = match output_format {
        OutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&options)?
            } else {
                serde_json::to_string(&options)?
            }
        }
        OutputFormat::Toml => {
            if pretty {
                toml::to_string_pretty(&options)?
            } else {
                toml::to_string(&options)?
            }
        }
    };

    write_output(&content, output)?;
    Ok(())
}

/// Return metadata about available lint groups, scopes, and levels.
///
/// This is primarily intended for static site generators that need to create
/// dropdown fields based on the available configuration options.
pub fn meta(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    let meta = Meta::new();

    let content = match output_format {
        OutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&meta)?
            } else {
                serde_json::to_string(&meta)?
            }
        }
        OutputFormat::Toml => {
            if pretty {
                toml::to_string_pretty(&meta)?
            } else {
                toml::to_string(&meta)?
            }
        }
    };

    write_output(&content, output)?;
    Ok(())
}
