use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(name = "ALPM Dev Scripts", about = "Dev scripts for the ALPM project")]
pub struct Cli {
    /// Verbose mode (-v, -vv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Tests file formats with real-world files from official repositories.
    TestFiles {
        #[clap(subcommand)]
        cmd: TestFilesCmd,
    },
}

#[derive(Parser, Debug)]
pub enum TestFilesCmd {
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

        #[clap(subcommand)]
        source: DownloadCmd,
    },
}

#[derive(Parser, Debug)]
pub enum DownloadCmd {
    /// Download all official package source repositories
    ///
    /// This is done by querying all repositories via the Gitlab API and
    /// cloning them to a local folder, which requires a Gitlab access token.
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
    },
    /// The packages contain the following file types for each package.
    /// - `.INSTALL`
    /// - `.BUILDINFO`
    /// - `.MTREE`
    /// - `.PKGINFO`
    Packages {},
}
