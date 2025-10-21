//! Command line functions that are called by the `alpm-soname` executable.

use std::io::Write;

use alpm_soname::{
    cli::{OutputFormat, PackageArgs},
    extract_elf_sonames,
    find_dependencies,
    find_provisions,
};
use alpm_types::{Soname, SonameLookupDirectory};
use fluent_i18n::t;
use thiserror::Error;

/// A high-level error wrapper around [`alpm_soname::Error`] to add CLI error cases.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// JSON error
    #[error("{msg}", msg = t!("error-json", { "source" => .0.to_string() }))]
    Json(#[from] serde_json::Error),

    /// An [alpm_soname::Error]
    #[error(transparent)]
    SonameError(#[from] alpm_soname::Error),
}

/// Get the provisions of a package and print them to the given output.
///
/// See the [`find_provisions`] function for more details.
///
/// # Errors
///
/// Returns an error if [`find_provisions`] returns an error or if the output stream
/// can not be written to.
pub fn get_provisions<W: Write>(
    args: PackageArgs,
    lookup_dir: SonameLookupDirectory,
    output: &mut W,
) -> Result<(), Error> {
    let provisions = find_provisions(args.package, lookup_dir)?;

    match args.output_format {
        OutputFormat::Plain => {
            for provision in provisions {
                writeln!(output, "{provision}").map_err(|source| {
                    alpm_soname::Error::IoWriteError {
                        context: t!("error-io-write-provision-output"),
                        source,
                    }
                })?;
            }
        }
        OutputFormat::Json => {
            let json = if args.pretty {
                serde_json::to_string_pretty(&provisions)?
            } else {
                serde_json::to_string(&provisions)?
            };
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWriteError {
                context: t!("error-io-write-json"),
                source,
            })?;
            return Ok(());
        }
    }

    Ok(())
}

/// Get the dependencies of a package and print them to the given output.
///
/// See the [`find_dependencies`] functions for more details.
///
/// # Errors
///
/// Returns an error if [`find_dependencies`] returns an error or if the output stream
/// can not be written to.
pub fn get_dependencies<W: Write>(
    args: PackageArgs,
    lookup_dir: SonameLookupDirectory,
    output: &mut W,
) -> Result<(), Error> {
    let dependencies = find_dependencies(args.package, lookup_dir)?;

    match args.output_format {
        OutputFormat::Plain => {
            for dependency in &dependencies {
                writeln!(output, "{dependency}").map_err(|source| {
                    alpm_soname::Error::IoWriteError {
                        context: t!("error-io-write-dependency-output"),
                        source,
                    }
                })?;
            }
            return Ok(());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&dependencies)?;
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWriteError {
                context: t!("error-io-write-json"),
                source,
            })?;
        }
    }

    Ok(())
}

/// Get the raw ELF soname dependencies of a package and print them to the given output.
///
///
/// Unlike [`get_dependencies`], this function does not filter the dependencies by the lookup
/// directory. In other words, it prints all ELF sonames found in the package regardless of whether
/// they match the lookup directory or not.
///
/// See the [`extract_elf_sonames`] function for more details.
///
/// # Errors
///
/// Returns an error if [`extract_elf_sonames`] returns an error or if the output stream
/// can not be written to.
pub fn get_raw_dependencies<W: Write>(args: PackageArgs, output: &mut W) -> Result<(), Error> {
    let mut elf_sonames: Vec<Soname> = extract_elf_sonames(args.package)?
        .into_iter()
        .flat_map(|elf_soname| elf_soname.sonames)
        .collect();
    elf_sonames.sort();
    elf_sonames.dedup();

    match args.output_format {
        OutputFormat::Plain => {
            for elf_soname in elf_sonames {
                writeln!(output, "{elf_soname}").map_err(|source| {
                    alpm_soname::Error::IoWriteError {
                        context: t!("error-io-write-elf-soname-output"),
                        source,
                    }
                })?;
            }
        }
        OutputFormat::Json => {
            let json = if args.pretty {
                serde_json::to_string_pretty(&elf_sonames)?
            } else {
                serde_json::to_string(&elf_sonames)?
            };
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWriteError {
                context: t!("error-io-write-json"),
                source,
            })?;
        }
    }

    Ok(())
}
