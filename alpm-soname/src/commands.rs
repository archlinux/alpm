//! Command line functions that are called by the `alpm-soname` executable.

use std::io::Write;

use alpm_package::InputDir;
use alpm_types::{Soname, SonameLookupDirectory};

use crate::{
    Error,
    cli::{OutputFormat, PackageArgs, SonameDetectionArgs},
    detection::{SonameDetection, SonameDetectionOptions},
    find_dependencies,
    find_provisions,
    lookup::extract_elf_sonames,
};

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
                writeln!(output, "{provision}").map_err(|source| Error::IoWriteError {
                    context: "writing provision to output",
                    source,
                })?;
            }
        }
        OutputFormat::Json => {
            let json = if args.pretty {
                serde_json::to_string_pretty(&provisions)?
            } else {
                serde_json::to_string(&provisions)?
            };
            writeln!(output, "{json}").map_err(|source| Error::IoWriteError {
                context: "writing JSON to output",
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
                writeln!(output, "{dependency}").map_err(|source| Error::IoWriteError {
                    context: "writing dependency to output",
                    source,
                })?;
            }
            return Ok(());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&dependencies)?;
            writeln!(output, "{json}").map_err(|source| Error::IoWriteError {
                context: "writing JSON to output",
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
                writeln!(output, "{elf_soname}").map_err(|source| Error::IoWriteError {
                    context: "writing ELF soname to output",
                    source,
                })?;
            }
        }
        OutputFormat::Json => {
            let json = if args.pretty {
                serde_json::to_string_pretty(&elf_sonames)?
            } else {
                serde_json::to_string(&elf_sonames)?
            };
            writeln!(output, "{json}").map_err(|source| Error::IoWriteError {
                context: "writing JSON to output",
                source,
            })?;
        }
    }

    Ok(())
}

/// Detects soname-based provisions and dependencies of a package and prints them to the given
/// output.
///
/// This function combines the functionality of [`find_provisions`] and [`find_dependencies`]
/// by scanning ELF files under the package directory using [`SonameDetection`].
///
/// See the [`SonameDetection`] type for more details.
///
/// # Errors
///
/// Returns an error if [`SonameDetection::new`] returns an error or if the output stream
/// can not be written to.
pub fn detect_sonames<W: Write>(
    args: SonameDetectionArgs,
    quiet: bool,
    output: &mut W,
) -> Result<(), Error> {
    let options =
        SonameDetectionOptions::new(InputDir::new(args.package_args.package)?, args.lookup_dir)
            .provides(!args.dependencies)
            .depends(!args.provisions);

    let soname_detection = SonameDetection::new(options)?;

    if args.provisions {
        let prefix = if quiet { "" } else { "provide = " };

        for provide in soname_detection.provides {
            writeln!(output, "{prefix}{provide}").map_err(|source| Error::IoWriteError {
                context: "writing provide to output",
                source,
            })?;
        }
    }
    if args.dependencies {
        let prefix = if quiet { "" } else { "depends = " };

        for dep in soname_detection.depends {
            writeln!(output, "{prefix}{dep}").map_err(|source| Error::IoWriteError {
                context: "writing dependency to output",
                source,
            })?;
        }
    }

    Ok(())
}
