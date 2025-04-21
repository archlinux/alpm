//! Command line functions that are called by the `alpm-soname` executable.

use std::io::Write;

use crate::{
    Error,
    cli::{DependencyArgs, ProvisionArgs},
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
