// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;

use alpm_types::Architecture;
use alpm_types::BuildDate;
use alpm_types::BuildDir;
use alpm_types::BuildEnv;
use alpm_types::Installed;
use alpm_types::Name;
use alpm_types::PackageOption;
use alpm_types::Packager;
use alpm_types::SchemaVersion;

use alpm_types::Version;
use clap::Parser;
use clap::Subcommand;

use crate::Error;

/// An enum describing all valid BUILDINFO schemas
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Schema {
    V1,
}

impl FromStr for Schema {
    type Err = Error;

    fn from_str(s: &str) -> Result<Schema, Self::Err> {
        match SchemaVersion::from_str(s) {
            Ok(version)
                if version >= SchemaVersion::new("1").unwrap()
                    && version < SchemaVersion::new("2").unwrap() =>
            {
                Ok(Schema::V1)
            }
            Err(_) | Ok(_) => Err(Error::InvalidBuildInfoVersion(s.to_string())),
        }
    }
}

impl Default for Schema {
    fn default() -> Self {
        Schema::V1
    }
}

impl Display for Schema {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Schema::V1 => "1",
            }
        )
    }
}

/// A type wrapping a PathBuf with a default value
///
/// This type is used in circumstances where an output file is required that defaults to ".BUILDINFO"
#[derive(Clone, Debug)]
pub struct OutputFile(pub PathBuf);

impl Default for OutputFile {
    fn default() -> Self {
        OutputFile(PathBuf::from(".BUILDINFO"))
    }
}

impl Display for OutputFile {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0.display())
    }
}

impl FromStr for OutputFile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OutputFile(PathBuf::from(s)))
    }
}

#[derive(Clone, Debug, Parser)]
#[command(about, author, name = "alpm-buildinfo", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command()]
    /// Create a BUILDINFO file according to a schema
    ///
    /// If the input can be validated according to the schema, the program exits with no output and a return code of 0.
    /// If the input can not be validated according to the schema, an error is emitted on stderr and the program exits
    /// with a non-zero exit code.
    Create {
        #[command(subcommand)]
        command: CreateCommand,
    },
    #[command()]
    /// Validate a BUILDINFO file
    ///
    /// Validate a BUILDINFO file according to a schema.
    /// If the file can be validated, the program exits with no output and a return code of 0.
    /// If the file can not be validated, an error is emitted on stderr and the program exits with a non-zero exit code.
    Validate {
        /// Provide the BUILDINFO schema version to use
        #[arg(default_value_t = Schema::default(), long, short, value_name = "VERSION")]
        schema: Schema,
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Export supplementary files
    ///
    /// Export supplementary files such as man pages and shell completions
    #[command(subcommand)]
    Export(ExportCommand),
}

#[derive(Clone, Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a BUILDINFO version 1 file
    ///
    /// If the input can be validated according to the schema, the program exits with no output and a return code of 0.
    /// If the input can not be validated according to the schema, an error is emitted on stderr and the program exits
    /// with a non-zero exit code.
    V1 {
        /// Provide a builddate
        #[arg(env = "BUILDINFO_BUILDDATE", long, value_name = "BUILDDATE")]
        builddate: BuildDate,
        /// Provide a builddir
        #[arg(env = "BUILDINFO_BUILDDIR", long, value_name = "BUILDDIR")]
        builddir: BuildDir,
        /// Provide one or more buildenv
        #[arg(
            env = "BUILDINFO_BUILDENV",
            long,
            value_delimiter = ' ',
            value_name = "BUILDENV"
        )]
        buildenv: Vec<BuildEnv>,
        /// Provide one or more installed
        #[arg(
            env = "BUILDINFO_INSTALLED",
            long,
            value_delimiter = ' ',
            value_name = "INSTALLED"
        )]
        installed: Vec<Installed>,
        /// Provide one or more options
        #[arg(
            env = "BUILDINFO_OPTIONS",
            long,
            value_delimiter = ' ',
            value_name = "OPTIONS"
        )]
        options: Vec<PackageOption>,
        /// Provide a packager
        #[arg(env = "BUILDINFO_PACKAGER", long, value_name = "PACKAGER")]
        packager: Packager,
        /// Provide a pkgarch
        #[arg(env = "BUILDINFO_PKGARCH", long, value_name = "PKGARCH")]
        pkgarch: Architecture,
        /// Provide a pkgbase
        #[arg(env = "BUILDINFO_PKGBASE", long, value_name = "PKGBASE")]
        pkgbase: Name,
        /// Provide a pkgbuild_sha256sum
        #[arg(
            env = "BUILDINFO_PKGBUILD_SHA256SUM",
            long,
            value_name = "PKGBUILD_SHA256SUM"
        )]
        pkgbuild_sha256sum: String,
        /// Provide a pkgname
        #[arg(env = "BUILDINFO_PKGNAME", long, value_name = "PKGNAME")]
        pkgname: Name,
        /// Provide a pkgver
        #[arg(env = "BUILDINFO_PKGVER", long, value_name = "PKGVER")]
        pkgver: Version,
        /// Provide a file to write to
        #[arg(default_value_t = OutputFile::default(), value_name = "FILE")]
        output: OutputFile,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum ExportCommand {
    /// Render shell completion files to a directory
    ShellCompletion {
        #[arg(value_name = "DIRECTORY")]
        output: PathBuf,
    },
    /// Render man pages to a directory
    Manpage {
        #[arg(value_name = "DIRECTORY")]
        output: PathBuf,
    },
}
