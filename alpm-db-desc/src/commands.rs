use std::{
    fs::{File, create_dir_all},
    io::{self, IsTerminal, Read, Write},
    str::FromStr,
};

use crate::{
    DbDescFileV1,
    DbDescFileV2,
    cli::{CreateCommand, OutputFormat, ValidateArgs},
    error::Error,
};

/// Create a desc file according to the specified version command.
pub fn create_file(command: CreateCommand) -> Result<(), Error> {
    let (data, output) = match command {
        CreateCommand::V1 { args } => {
            let v1 = DbDescFileV1::new(
                args.name,
                args.version,
                args.base,
                args.description,
                args.url,
                args.arch,
                args.builddate,
                args.installdate,
                args.packager,
                args.size,
                args.groups,
                None, // reason
                args.license,
                vec![], // validation
                args.replaces,
                args.depends,
                args.optdepends,
                args.conflicts,
                args.provides,
            );
            (v1.to_string(), args.output)
        }
        CreateCommand::V2 { args, xdata } => {
            let v2 = DbDescFileV2::new(
                args.name,
                args.version,
                args.base,
                args.description,
                args.url,
                args.arch,
                args.builddate,
                args.installdate,
                args.packager,
                args.size,
                args.groups,
                None, // reason
                args.license,
                vec![], // validation
                args.replaces,
                args.depends,
                args.optdepends,
                args.conflicts,
                args.provides,
                xdata,
            );
            (v2.to_string(), args.output)
        }
    };

    // create parent directories if necessary
    if let Some(output_dir) = output.0.parent() {
        create_dir_all(output_dir).map_err(|e| {
            Error::IoPathError(output_dir.to_path_buf(), "creating output directory", e)
        })?;
    }

    let mut out = File::create(&output.0)
        .map_err(|e| Error::IoPathError(output.0.clone(), "creating output file", e))?;

    out.write_all(data.as_bytes())
        .map_err(|e| Error::IoPathError(output.0, "writing to output file", e))?;

    Ok(())
}

/// Parse a desc file and return its in-memory representation.
///
/// Reads from file if provided, or from stdin if data is piped.
pub fn parse(args: ValidateArgs) -> Result<String, Error> {
    let mut content = String::new();

    if let Some(file) = &args.file {
        File::open(file)
            .and_then(|mut f| f.read_to_string(&mut content))
            .map_err(|e| Error::IoPathError(file.clone(), "reading file", e))?;
    } else if !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| Error::Io("reading stdin", e))?;
    } else {
        return Err(Error::NoInputFile);
    }

    Ok(content)
}

/// Validate a desc file (detect version automatically and parse).
///
/// This performs basic validation by attempting to parse the file into either V2 or V1.
pub fn validate(args: ValidateArgs) -> Result<(), Error> {
    let content = parse(args)?;

    // Try parsing as v2 first; fallback to v1
    if DbDescFileV2::from_str(&content).is_ok() {
        return Ok(());
    }

    if DbDescFileV1::from_str(&content).is_ok() {
        return Ok(());
    }

    Err(Error::InvalidFormat)
}

/// Format and pretty-print a desc file.
///
/// Attempts to detect version automatically, then serialize to JSON or YAML.
pub fn format(args: ValidateArgs, output_format: OutputFormat, pretty: bool) -> Result<(), Error> {
    let content = parse(args)?;

    // Detect version by presence of XDATA
    let value = if content.contains("%XDATA%") {
        serde_json::to_value(DbDescFileV2::from_str(&content)?)?
    } else {
        serde_json::to_value(DbDescFileV1::from_str(&content)?)?
    };

    match output_format {
        OutputFormat::Json => {
            let json = if pretty {
                serde_json::to_string_pretty(&value)?
            } else {
                serde_json::to_string(&value)?
            };
            println!("{json}");
        }
    }

    Ok(())
}
