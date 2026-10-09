//! Example for reading the current system's [alpm-db].
//!
//! [alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html

use std::{path::PathBuf, process::ExitCode};

use alpm_db::db::{Database, DatabaseConfig};
use log::{LevelFilter, debug};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

/// The error that may occur when using this example.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`alpm_db::Error`] occurred.
    #[error(transparent)]
    AlpmDb(#[from] alpm_db::Error),

    /// An I/O error occurred at a path.
    #[error("I/O error at path {path} while {context}: {source}")]
    IoPath {
        /// The path where the error occurred.
        path: PathBuf,

        /// The context in which the error occurred.
        context: String,

        /// The source of the error.
        source: std::io::Error,
    },
}

/// Initialize a logger that shows trace messages on stderr.
fn init_logger() {
    if TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .is_err()
    {
        debug!("Not initializing another logger, as one is initialized already.");
    }
}

fn run_command() -> Result<(), Error> {
    init_logger();

    let mut database = Database::new(DatabaseConfig::default())?;
    let entries = database.installed_packages()?;
    for entry in entries {
        println!("{entry}");
    }

    Ok(())
}

fn main() -> ExitCode {
    match run_command() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
