//! CLI handling for the `alpm-repo-files` executable.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use fluent_i18n::t;

/// Output format for `alpm-repo-files` commands with data output.
#[derive(Clone, Debug, strum::Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub enum OutputFormat {
    /// The JSON output format.
    #[value(help = t!("cli-output-format-json-help"))]
    Json,

    /// The alpm-repo-files output format.
    #[value(help = t!("cli-output-format-v1-help"))]
    V1,
}

/// The command line interface for `alpm-repo-files`.
#[derive(Clone, Debug, Parser)]
#[command(
    about = t!("cli-about"),
    author,
    long_about = t!("cli-long-about"),
    name = "alpm-repo-files",
    version
)]
pub struct Cli {
    /// The commands of the `alpm-repo-files` executable.
    #[command(subcommand)]
    pub command: Command,
}

/// A command of the `alpm-repo-files` executable.
#[derive(Clone, Debug, Parser)]
#[command(about, author, version)]
pub enum Command {
    /// The create command
    #[command(about = t!("cli-create-about"), long_about = t!("cli-create-long-about"))]
    Create {
        /// The directory to read from.
        #[arg(
            env = "ALPM_REPO_FILES_CREATE_INPUT_DIR",
            help = t!("cli-create-input-dir-help"),
            value_name = "INPUT_DIR"
        )]
        input_dir: PathBuf,

        /// A file path to write to instead of stdout.
        #[arg(
            env = "ALPM_REPO_FILES_CREATE_OUTPUT",
            help = t!("cli-output-help"),
            long,
            short,
            value_name = "OUTPUT"
        )]
        output: Option<PathBuf>,
    },

    /// The format command.
    #[command(about = t!("cli-format-about"), long_about = t!("cli-format-long-about"))]
    Format {
        /// An input file to read from.
        #[arg(
            env = "ALPM_REPO_FILES_FORMAT_INPUT_FILE",
            help = t!("cli-input-file-help"),
            long,
            long_help = t!("cli-input-file-long-help"),
            short,
            value_name = "INPUT_FILE"
        )]
        input_file: Option<PathBuf>,

        /// Set the output format.
        #[arg(
            env = "ALPM_REPO_FILES_FORMAT_OUTPUT_FORMAT",
            help = t!("cli-format-format-help"),
            short,
            long,
            value_name = "OUTPUT_FORMAT",
            default_value_t = OutputFormat::Json
        )]
        format: OutputFormat,

        /// A file path to write to instead of stdout.
        #[arg(
            env = "ALPM_REPO_FILES_FORMAT_OUTPUT",
            help = t!("cli-output-help"),
            long,
            short,
            value_name = "OUTPUT"
        )]
        output: Option<PathBuf>,

        /// Determines whether the output will be displayed in a pretty non-minimized fashion.
        #[arg(
            env = "ALPM_REPO_FILES_FORMAT_PRETTY",
            help = t!("cli-format-pretty-help"),
            long,
            long_help = t!("cli-format-pretty-long-help"),
            short,
        )]
        pretty: bool,
    },

    /// The validate command.
    #[command(about = t!("cli-validate-about"), long_about = t!("cli-validate-long-about"))]
    Validate {
        /// An input file to read from.
        #[arg(
            env = "ALPM_REPO_FILES_VALIDATE_INPUT_FILE",
            help = t!("cli-input-file-help"),
            long,
            long_help = t!("cli-input-file-long-help"),
            short,
            value_name = "INPUT_FILE"
        )]
        input_file: Option<PathBuf>,
    },
}
