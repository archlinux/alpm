use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};
use strum::Display;

use crate::sync::PackageRepositories;

#[derive(Debug, Parser)]
#[clap(name = "ALPM Dev Scripts", about = "Dev scripts for the ALPM project")]
pub struct Cli {
    /// Verbose mode (-v, -vv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Tests file formats with real-world files from official repositories.
    TestFiles {
        #[clap(subcommand)]
        cmd: TestFilesCmd,
    },

    /// Run the `alpm-pkgbuild srcinfo format` command on a PKGBUILD and compare its output with a
    /// given .SRCINFO file.
    CompareSrcinfo {
        /// Path to the PKGBUILD file.
        #[arg(
            short,
            long = "pkgbuild",
            value_name = "PKGBUILD_PATH",
            default_value = "./PKGBUILD"
        )]
        pkgbuild_path: PathBuf,

        /// Path to the .SRCINFO file.
        #[arg(
            short,
            long = "srcinfo",
            value_name = "SRCINFO_PATH",
            default_value = "./.SRCINFO"
        )]
        srcinfo_path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Display, Eq, Parser, PartialEq, ValueEnum)]
pub enum TestFileType {
    #[strum(to_string = ".BUILDINFO")]
    BuildInfo,
    #[strum(to_string = ".SRCINFO")]
    SrcInfo,
    #[strum(to_string = ".PKGINFO")]
    PackageInfo,
    #[strum(to_string = ".MTREE")]
    MTree,
    #[strum(to_string = "desc")]
    RemoteDesc,
    #[strum(to_string = "files")]
    RemoteFiles,
    #[strum(to_string = "desc")]
    LocalDesc,
    #[strum(to_string = "files")]
    LocalFiles,
}

#[derive(Debug, Parser)]
pub enum TestFilesCmd {
    /// Download/synchronize files for testing to this machine.
    ///
    /// Each type of file can be downloaded individually.
    Test {
        // Where the local testing data is located.
        // Defaults to `~/.cache/alpm/testing`
        #[arg(short, long)]
        test_data_dir: Option<PathBuf>,

        /// Package repositories to test.
        ///
        /// If not set, all official repositories are tested.
        #[arg(short, long)]
        repositories: Option<Vec<PackageRepositories>>,

        /// The type of file that should be tested.
        file_type: TestFileType,
    },

    /// Download/synchronize files for testing to this machine.
    ///
    /// Each type of file can be downloaded individually.
    Download {
        // Where the testing data should be downloaded to.
        //
        // Defaults to `$XDG_CACHE_HOME/alpm/testing`.
        // if `$XDG_CACHE_HOME` isn't set, it falls back to to `~/.cache/alpm/testing`.
        #[arg(short, long)]
        destination: Option<PathBuf>,

        /// Package repositories to download.
        ///
        /// If not set, all official repositories are downloaded.
        #[arg(short, long)]
        repositories: Option<Vec<PackageRepositories>>,

        #[clap(subcommand)]
        source: DownloadCmd,
    },

    /// Remove or clean downloaded local testing files.
    Clean {
        // Where the testing data has been downloaded to.
        // Defaults to `~/.cache/alpm/testing`
        #[arg(short, long)]
        destination: Option<PathBuf>,

        #[clap(subcommand)]
        source: CleanCmd,
    },
}

#[derive(Debug, Parser)]
pub enum DownloadCmd {
    /// Download all official package source repositories
    ///
    /// This is done by querying all active repositories via the arch web API
    /// (<https://archlinux.org/packages/pkgbase-maintainer>) and cloning the respective
    /// package source repositories via git.
    ///
    /// This command differs from `pkgctl repo clone --universe` in so far that it
    /// also updates git repositories and removes repositories that're no longer used.
    ///
    /// The repositories contain the following file types for each package.
    /// - .SRCINFO
    PkgSrcRepositories {},

    /// Create a copy of a mirror's pacman database.
    ///
    /// The database contains the following file types for each package.
    /// - `files`
    /// - `desc`
    Databases {
        /// The domain + base path under which the mirror can be found.
        ///
        /// The mirror must support the `rsync` protocol
        #[arg(short, long, env, default_value = "mirror.pseudoform.org/packages")]
        mirror: String,

        /// Force re-extraction of the files regardless of reported changes.
        ///
        /// This is useful for if the download is cancelled halfway, in which case
        /// `rsync` will not report changes for files that it downloaded last time.
        #[arg(short, long, default_value_t = false)]
        force_extract: bool,
    },
    /// The packages contain the following file types for each package.
    /// - `.INSTALL`
    /// - `.BUILDINFO`
    /// - `.MTREE`
    /// - `.PKGINFO`
    Packages {
        /// The domain + base path under which the mirror can be found.
        ///
        /// The mirror must support the `rsync` protocol
        #[arg(short, long, env, default_value = "mirror.pseudoform.org/packages")]
        mirror: String,

        /// Force re-extraction of the files regardless of reported changes.
        ///
        /// This is useful for if the download is cancelled halfway, in which case
        /// `rsync` will not report changes for files that it downloaded last time.
        #[arg(short, long, default_value_t = false)]
        force_extract: bool,
    },
}

#[derive(Debug, Parser)]
pub enum CleanCmd {
    /// Remove all package source repositories and .SRCINFO files
    PkgSrcRepositories,

    /// Remove extracted repository sync database files and tarballs.
    Databases,

    /// Remove all downloaded packages and any other files extracted from them.
    Packages,
}
