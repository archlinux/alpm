//! Command line functions that are called by the `alpm-soname` executable.

use std::{collections::BTreeMap, io::Write, path::PathBuf};

use alpm_soname::{
    ElfSonames,
    cli::{OutputFormat, PackageArgs},
    extract_elf_sonames,
    find_dependencies,
    find_provisions,
};
use alpm_types::{Name, Soname, SonameLookupDirectory, SonameV2};
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
            if args.pretty {
                for (prefix, sonames) in group_sonames_by_prefix(&provisions) {
                    writeln!(output, "{prefix}").map_err(|source| alpm_soname::Error::IoWrite {
                        context: t!("error-io-write-provision-output"),
                        source,
                    })?;
                    for soname in sonames {
                        writeln!(output, " ⤷ {soname}").map_err(|source| {
                            alpm_soname::Error::IoWrite {
                                context: t!("error-io-write-provision-output"),
                                source,
                            }
                        })?;
                    }
                }
            } else {
                for provision in &provisions {
                    writeln!(output, "{provision}").map_err(|source| {
                        alpm_soname::Error::IoWrite {
                            context: t!("error-io-write-provision-output"),
                            source,
                        }
                    })?;
                }
            }
        }
        OutputFormat::Json => {
            let json = if args.pretty {
                serde_json::to_string_pretty(&provisions)?
            } else {
                serde_json::to_string(&provisions)?
            };
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWrite {
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
            if args.pretty {
                for (prefix, sonames) in group_sonames_by_prefix(&dependencies) {
                    writeln!(output, "{prefix}").map_err(|source| alpm_soname::Error::IoWrite {
                        context: t!("error-io-write-dependency-output"),
                        source,
                    })?;
                    for soname in sonames {
                        writeln!(output, " ⤷ {soname}").map_err(|source| {
                            alpm_soname::Error::IoWrite {
                                context: t!("error-io-write-dependency-output"),
                                source,
                            }
                        })?;
                    }
                }
            } else {
                for dependency in &dependencies {
                    writeln!(output, "{dependency}").map_err(|source| {
                        alpm_soname::Error::IoWrite {
                            context: t!("error-io-write-dependency-output"),
                            source,
                        }
                    })?;
                }
            }
            return Ok(());
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&dependencies)?;
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWrite {
                context: t!("error-io-write-json"),
                source,
            })?;
        }
    }

    Ok(())
}

/// Get the raw ELF soname dependencies of a package and print them to the given output.
///
/// Unlike [`get_dependencies`], this function does not filter the dependencies by the lookup
/// directory. In other words, it prints all ELF sonames found in the package regardless of whether
/// they match the lookup directory or not.
///
/// See the [`extract_elf_sonames`] function for more details.
///
/// If `detail` is `true`, the output groups dependencies by the ELF that references them. If an
/// `elf_path` is supplied, dependencies are shown for only the ELF specified by `elf_path`.
///
/// # Errors
///
/// Returns an error if [`extract_elf_sonames`] returns an error or if the output stream
/// can not be written to.
pub fn get_raw_dependencies<W: Write>(
    args: PackageArgs,
    elf_path: Option<PathBuf>,
    detail: bool,
    output: &mut W,
) -> Result<(), Error> {
    let elf_sonames = {
        let mut elf_sonames: Vec<ElfSonames> = extract_elf_sonames(args.package)?;
        elf_sonames.sort_by(|a, b| a.path.cmp(&b.path));
        elf_sonames
    };
    let sonames = {
        let mut sonames: Vec<Soname> = elf_sonames
            .iter()
            .flat_map(|elf| elf.sonames.clone())
            .collect();
        sonames.sort();
        sonames.dedup();
        sonames
    };
    let elf_soname: Option<&ElfSonames> = elf_path
        .map(|elf_path| {
            let relpath = elf_path
                .strip_prefix(std::path::MAIN_SEPARATOR_STR)
                .unwrap_or(elf_path.as_path());
            elf_sonames
                .iter()
                .find(|elf| elf.path == relpath)
                .ok_or_else(|| Error::SonameError(alpm_soname::Error::ElfFileNotFound { elf_path }))
        })
        .transpose()?;
    match args.output_format {
        OutputFormat::Plain => {
            if detail {
                for elf_soname in elf_sonames {
                    writeln!(output, "{}", elf_soname.path.display()).map_err(|source| {
                        alpm_soname::Error::IoWrite {
                            context: t!("error-io-write-elf-soname-output"),
                            source,
                        }
                    })?;
                    for soname in &elf_soname.sonames {
                        writeln!(output, " ⤷ {soname}").map_err(|source| {
                            alpm_soname::Error::IoWrite {
                                context: t!("error-io-write-elf-soname-output"),
                                source,
                            }
                        })?;
                    }
                }
            } else if let Some(elf_soname) = elf_soname {
                writeln!(output, "{}", elf_soname.path.display()).map_err(|source| {
                    alpm_soname::Error::IoWrite {
                        context: t!("error-io-write-elf-soname-output"),
                        source,
                    }
                })?;
                for soname in &elf_soname.sonames {
                    writeln!(output, " ⤷ {soname}").map_err(|source| {
                        alpm_soname::Error::IoWrite {
                            context: t!("error-io-write-elf-soname-output"),
                            source,
                        }
                    })?;
                }
            } else {
                for soname in &sonames {
                    writeln!(output, "{soname}").map_err(|source| alpm_soname::Error::IoWrite {
                        context: t!("error-io-write-elf-soname-output"),
                        source,
                    })?;
                }
            }
        }
        OutputFormat::Json => {
            let json = match (detail, elf_soname, args.pretty) {
                (true, None, true) => serde_json::to_string_pretty(&elf_sonames)?,
                (true, None, false) => serde_json::to_string(&elf_sonames)?,
                (false, None, true) => serde_json::to_string_pretty(&sonames)?,
                (false, None, false) => serde_json::to_string(&sonames)?,
                (false, Some(elf_soname), true) => serde_json::to_string_pretty(&elf_soname)?,
                (false, Some(elf_soname), false) => serde_json::to_string(&elf_soname)?,
                (true, Some(_), _) => unreachable!("--detail conflicts with --elf <PATH>"),
            };
            writeln!(output, "{json}").map_err(|source| alpm_soname::Error::IoWrite {
                context: t!("error-io-write-json"),
                source,
            })?;
        }
    }

    Ok(())
}

/// Groups a list of [`SonameV2`] data by their shared library prefixes.
///
/// Returns a map of shared library prefixes, each with a list of raw [`Soname`] information
/// attached to them.
fn group_sonames_by_prefix(sonames: &[SonameV2]) -> BTreeMap<Name, Vec<Soname>> {
    let mut grouped_sonames: BTreeMap<Name, Vec<Soname>> = BTreeMap::new();
    for entry in sonames {
        grouped_sonames
            .entry(entry.prefix.clone())
            .or_default()
            .push(entry.soname.clone());
    }

    for sonames in grouped_sonames.values_mut() {
        sonames.sort();
        sonames.dedup();
    }

    grouped_sonames
}
