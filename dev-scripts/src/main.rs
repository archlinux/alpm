use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use log::LevelFilter;
use simplelog::{Config, SimpleLogger};
use sync::mirror::MirrorDownloader;

mod cli;
mod cmd;
mod sync;

fn main() -> Result<()> {
    // Parse commandline options.
    let args = Cli::parse();

    // Init and set the verbosity level of the logger.
    let level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    SimpleLogger::init(level, Config::default()).context("Failed to initialize simple logger")?;

    match args.cmd {
        cli::Command::TestFiles { cmd } => match cmd {
            cli::TestFilesCmd::Download {
                destination,
                source,
            } => {
                // Set a default download destination.
                let dest = match destination {
                    Some(dest) => dest,
                    None => dirs::cache_dir()
                        .context("Failed to determine home user cache directory.")?
                        .join("alpm/testing"),
                };

                match source {
                    cli::DownloadCmd::PkgSrcRepositories {} => {
                        unimplemented!()
                    }
                    cli::DownloadCmd::Databases { mirror } => {
                        let downloader = MirrorDownloader { dest, mirror };
                        downloader.sync_remote_databases()?;
                    }
                    cli::DownloadCmd::Packages {} => {
                        unimplemented!()
                    }
                };
            }
        },
    }

    Ok(())
}
