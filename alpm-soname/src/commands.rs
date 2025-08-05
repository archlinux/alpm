//! Command line functions that are called by the `alpm-soname` executable.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::{
    Autodeps,
    Error,
    autodeps::AutodepsOptions,
    cli::{AutoDepsArgs, DependencyArgs, ProvisionArgs},
    find_dependencies,
    find_provisions,
};

/// Get the provisions of a package and print them to the given output.
///
/// See the [`find_provisions`] function for more details.
///
/// # Errors
///
/// Returns an error if [`find_provisions`] returns an error or if the output stream
/// can not be written to.
pub fn get_provisions<W: Write>(args: ProvisionArgs, output: &mut W) -> Result<(), Error> {
    let provisions = find_provisions(args.package, args.lookup_dir)?;
    for provision in provisions {
        writeln!(output, "{provision}").map_err(|source| Error::IoWriteError {
            context: "writing provision to output",
            source,
        })?;
    }
    Ok(())
}

/// Get the dependencies of a package and print them to the given output.
///
/// See the [`find_dependencies`] function for more details.
///
/// # Errors
///
/// Returns an error if [`find_dependencies`] returns an error or if the output stream
/// can not be written to.
pub fn get_dependencies<W: Write>(args: DependencyArgs, output: &mut W) -> Result<(), Error> {
    let dependencies = find_dependencies(args.package, args.lookup_dir, args.all)?;
    for dependency in dependencies {
        writeln!(output, "{dependency}").map_err(|source| Error::IoWriteError {
            context: "writing dependency to output",
            source,
        })?;
    }
    Ok(())
}

/// Get the librar provides and depends for a package by reading it's input directory.
///
/// See [`Autodeps`] for more details.
///
/// # Errors
///
/// Returns an error if [`Autodeps::with_options`] returns an error or if the output stream
/// can not be written to.
///
/// # Example
///
/// ```sh
/// % mkdir ~/pacman
/// % curl -s "$(pacman -Sp pacman)" | bsdtar -C ~/pacman -xf -
/// % alpm-soname auto-deps ~/pacman
/// depends = lib:libarchive.so.13
/// depends = lib:libc.so.6
/// depends = lib:libcrypto.so.3
/// depends = lib:libcurl.so.4
/// depends = lib:libgpgme.so.45
/// provide = lib:libalpm.so.15
/// ```
pub fn autodeps<W: Write>(args: AutoDepsArgs, output: &mut W) -> Result<(), Error> {
    let options = AutodepsOptions::new(args.package, args.lookup_dir)
        .provides(!args.depends)
        .depends(!args.provides);
    let autodeps = Autodeps::with_options(options)?;

    if !args.provides {
        let prefix = if args.quiet { "" } else { "depends = " };

        for dep in autodeps.depends {
            if !dep_satisfied(args.test_satisfied_command.as_deref(), &dep.to_string())? {
                continue;
            }
            writeln!(output, "{prefix}{dep}").map_err(|source| Error::IoWriteError {
                context: "writing dependency to output",
                source,
            })?;
        }
    }
    if !args.depends {
        let prefix = if args.quiet { "" } else { "provide = " };

        for provide in autodeps.provides {
            writeln!(output, "{prefix}{provide}").map_err(|source| Error::IoWriteError {
                context: "writing provide to output",
                source,
            })?;
        }
    }
    Ok(())
}

fn dep_satisfied(command: Option<&str>, dep: &str) -> Result<bool, Error> {
    let Some(command_str) = command else {
        return Ok(true);
    };

    let mut args = command_str.split_whitespace();

    let Some(command) = args.next() else {
        return Ok(true);
    };

    let status = Command::new(command)
        .args(args)
        .arg(dep)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| Error::CommandError {
            context: "checking if dependency is satisfied",
            command: format!("{command_str} {dep}"),
            source: e,
        })?;
    match status.code() {
        Some(0) => Ok(true),
        _ => Ok(false),
    }
}
