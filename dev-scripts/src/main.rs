use std::fs::remove_dir_all;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use log::LevelFilter;
use simplelog::{Config, SimpleLogger};
use strum::IntoEnumIterator;
use sync::{PackageRepositories, mirror::MirrorDownloader, pkgsrc::PkgSrcDownloader};
use testing::TestRunner;

mod cli;
mod cmd;
pub mod sync;
pub mod testing;
mod ui;

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
            cli::TestFilesCmd::Test {
                test_data_dir,
                repositories,
                file_type,
            } => {
                // Set a default download destination.
                let test_data_dir = match test_data_dir {
                    Some(test_data_dir) => test_data_dir,
                    None => dirs::cache_dir()
                        .context("Failed to determine home user cache directory.")?
                        .join("alpm/testing"),
                };
                let repositories = PackageRepositories::iter()
                    .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                    .collect();
                let runner = TestRunner {
                    test_data_dir,
                    file_type,
                    repositories,
                };
                runner.run_tests()?;
            }
            cli::TestFilesCmd::Download {
                destination,
                repositories,
                source,
            } => {
                // Set a default download destination.
                let dest = match destination {
                    Some(dest) => dest,
                    None => dirs::cache_dir()
                        .context("Failed to determine home user cache directory.")?
                        .join("alpm/testing"),
                };
                let repositories = PackageRepositories::iter()
                    .filter(|v| repositories.clone().is_none_or(|r| r.contains(v)))
                    .collect();

                match source {
                    cli::DownloadCmd::PkgSrcRepositories {} => {
                        let downloader = PkgSrcDownloader { dest };
                        downloader.download_package_source_repositories()?;
                    }
                    cli::DownloadCmd::Databases { mirror } => {
                        let downloader = MirrorDownloader {
                            dest,
                            mirror,
                            repositories,
                        };
                        downloader.sync_remote_databases()?;
                    }
                    cli::DownloadCmd::Packages { mirror } => {
                        let downloader = MirrorDownloader {
                            dest,
                            mirror,
                            repositories,
                        };
                        downloader.sync_remote_packages()?;
                    }
                };
            }
            cli::TestFilesCmd::Clean {
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
                    cli::CleanCmd::PkgSrcRepositories => {
                        remove_dir_all(dest.join("download").join("pkgsrc"))?;
                        remove_dir_all(dest.join("pkgsrc"))?;
                    }
                    cli::CleanCmd::Databases => {
                        remove_dir_all(dest.join("download").join("databases"))?;
                        remove_dir_all(dest.join("databases"))?;
                    }
                    cli::CleanCmd::Packages => {
                        remove_dir_all(dest.join("download").join("packages"))?;
                        remove_dir_all(dest.join("packages"))?;
                    }
                };
            }
        },
    }

    Ok(())
}
