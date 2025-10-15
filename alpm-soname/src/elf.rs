//! [ELF] file reader and helper utilities.
//!
//! [ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format

use std::io::Read;

use goblin::{Hint, Object, elf::Elf};
use log::{debug, trace};

use crate::Error;

/// Reads and parses an ELF file from disk, returning its [`Elf`] representation if valid.
///
/// Returns `Ok(None)` if the file is not an ELF binary.
///
/// # Errors
///
/// Returns an error if:
///
/// - The file cannot be opened or read.
/// - The ELF header or structure cannot be parsed.
pub fn read_elf<'a, R: Read>(
    entry: &mut R,
    buffer: &'a mut Vec<u8>,
) -> Result<Option<Elf<'a>>, Error> {
    // Read 16 bytes for checking the header
    let mut header = [0u8; 16];

    if let Err(e) = entry.read_exact(&mut header) {
        debug!("⤷ Could not read entry header ({e}), skipping...");
        return Ok(None);
    }

    // Check the header for an ELF file
    if let Ok(Hint::Elf(_)) = goblin::peek_bytes(&header) {
        trace!("⤷ File header: {header:?}");
        debug!("⤷ Found ELF file.");
    } else {
        trace!("⤷ Not an ELF file, skipping...");
        return Ok(None);
    };

    // Read the entry into the buffer
    // Also, take the header into account
    buffer.clear();
    buffer.extend_from_slice(&header);
    entry
        .read_to_end(buffer)
        .map_err(|source| Error::IoReadError {
            context: "reading entry",
            source,
        })?;

    // Parse the ELF file and return it.
    let object = Object::parse(buffer).map_err(|source| Error::ElfError {
        context: "parsing ELF file",
        source,
    })?;

    let Object::Elf(elf) = object else {
        return Ok(None);
    };

    Ok(Some(elf))
}
