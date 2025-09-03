use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use alpm_lint::{Error, Level, LintScope, LintStore, Resources, cli::OutputFormat};
use alpm_lint_config::{LintConfiguration, LintGroup, LintRuleConfiguration};
use log::debug;
use serde::Serialize;
use strum::VariantArray;

/// Writes `content` to stdout or a file.
///
/// If `output_path` is a file path, `content` is written to it, else `content` is written to
/// stdout.
///
/// # Errors
///
/// Returns an error if an output file cannot be created or written to.
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
        None => print!("{content}"),
    }
    Ok(())
}

/// Takes any serializable object and serializes it into the given [`OutputFormat`].
///
/// # Errors
///
/// Returns an error if serialization fails.
fn serialize_output<T: Serialize>(
    object: T,
    output_format: OutputFormat,
    pretty: bool,
    context: &str,
) -> Result<String, Error> {
    let output = match output_format {
        OutputFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(&object).map_err(|error| Error::Json {
                    error,
                    context: context.into(),
                })?
            } else {
                serde_json::to_string(&object).map_err(|error| Error::Json {
                    error,
                    context: context.into(),
                })?
            }
        }
    };

    Ok(output)
}

/// Runs a lint check.
///
/// If not provided, the `path` and `scope` are automatically detected.
/// Defaults to the current working directory if no `path` is provided.
pub fn check(
    config_path: Option<PathBuf>,
    path: Option<PathBuf>,
    scope: Option<LintScope>,
    level: Level,
    format: LintOutputFormat,
    output: Option<PathBuf>,
    pretty: bool,
) -> Result<(), Error> {
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
    debug!("Resources have been gathered.");

    let store = LintStore::new(config);

    let mut issues = Vec::new();
    let lint_rules = store.filtered_lint_rules(&scope, level);

    debug!("Start of linting.");
    for (name, rule) in lint_rules {
        debug!("Running rule: '{name}'");
        rule.run(&resources, &mut issues)?;
    }

    let found_issues = !issues.is_empty();

    debug!("Using output format {format:?}.");
    let content = match format {
        None => unimplemented!(),
        Some(output_format) => serialize_output(issues, output_format, pretty)?,
    };

    write_output(&content, output)?;

    // Exit with code 1 if there were any relevant lints.
    if found_issues {
        std::process::exit(1);
    }

    Ok(())
}

/// Writes the definition of all linting rules to output.
///
/// Writes a map of [`SerializableLintRule`]s in serialized form.
pub fn rules(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    // Create a stub default configuration.
    let config = LintConfiguration::default();
    let store = LintStore::new(config);

    let lint_rules = store.serializable_lint_rules();
    let content = serialize_output(lint_rules, output_format, pretty, "lint rules")?;

    write_output(&content, output)?;
    Ok(())
}

/// Writes the definitions of all linting options to output.
pub fn options(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    let options = LintRuleConfiguration::configuration_options();
    let content = serialize_output(options, output_format, pretty, "lint options")?;

    write_output(&content, output)?;
    Ok(())
}

/// Metadata information for static site generator integration.
///
/// Contains all available lint groups, scopes, and levels that can be used
/// to create dropdown fields in a static site generator.
#[derive(Debug, Serialize)]
pub struct Meta {
    /// All available lint groups.
    pub groups: Vec<LintGroup>,
    /// All available lint scopes.
    pub scopes: Vec<LintScope>,
    /// All available lint levels.
    pub levels: Vec<Level>,
}

impl Default for Meta {
    fn default() -> Self {
        Self::new()
    }
}

impl Meta {
    /// Creates a new [`Meta`] instance with all available groups, scopes, and levels.
    ///
    /// This function collects all enum variants for lint groups, scopes, and levels
    /// that can be used in configuration and CLI options.
    pub fn new() -> Self {
        Self {
            groups: LintGroup::VARIANTS.to_vec(),
            scopes: LintScope::VARIANTS.to_vec(),
            levels: Level::VARIANTS.to_vec(),
        }
    }
}

/// Writes metadata about available lint groups, scopes, and levels to output.
///
/// This is primarily intended for static site generators that need to create
/// dropdown fields based on the available configuration options.
pub fn meta(
    output_format: OutputFormat,
    pretty: bool,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    let meta = Meta::new();
    let content = serialize_output(meta, output_format, pretty, "lint meta types")?;

    write_output(&content, output)?;
    Ok(())
}
