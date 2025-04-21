//! Command line functions that are called by the `alpm-soname` executable.

use std::io::Write;

use crate::{
    Error,
    cli::{DependencyArgs, ProvisionArgs},
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
/// If `args.all` is `false`, it will only print the dependencies that are provided by the package
/// and match the lookup directory. If `args.all` is `true`, it will print all ELF sonames
/// found in the package, regardless of whether they match the lookup directory or not.
///
/// See the [`find_dependencies`] and [`extract_elf_sonames`] functions for more details.
///
/// # Errors
///
/// Returns an error if [`find_dependencies`] returns an error or if the output stream
/// can not be written to.
pub fn get_dependencies<W: Write>(args: DependencyArgs, output: &mut W) -> Result<(), Error> {
    if !args.all {
        let dependencies = find_dependencies(args.package, args.lookup_dir)?;
        for dependency in dependencies {
            writeln!(output, "{dependency}").map_err(|source| Error::IoWriteError {
                context: "writing dependency to output",
                source,
            })?;
        }
    } else {
        let elf_sonames = extract_elf_sonames(args.package)?;
        for elf_soname in elf_sonames
            .iter()
            .flat_map(|elf_soname| &elf_soname.sonames)
        {
            writeln!(output, "{elf_soname}").map_err(|source| Error::IoWriteError {
                context: "writing ELF soname to output",
                source,
            })?;
        }
    }
    Ok(())
}
