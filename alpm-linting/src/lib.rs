#![doc = include_str!("../README.md")]

use clap::ValueEnum;
use strum::Display;

pub mod cli;

/// This enum represents the different scopes, files and contexts that're covered by lints.
#[derive(Clone, Debug, Display, PartialEq, ValueEnum)]
pub enum LintingScopes {
    /// Run lints specific to a ArchLinux package source repository.
    SourceRepository,
    /// Lints on a `.SRCINFO` file.
    SourceInfo,
    /// Lints on a `.BUILDINFO` file.
    BuildInfo,
    /// Lints on a `.PKGINFO` file.
    PackageInfo,
    /// Lints on a ArchLinux package.
    Package,
}
