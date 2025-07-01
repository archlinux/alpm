//! Signature verifiers are located in directory structures described by the VOA file hierarchy.
//! The hierarchy consists of four distinct subdirectory layers, each represented by an identifier.
//!
//! See <https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/#identifiers>

mod context;
mod os;
mod purpose;
mod technology;

pub use context::*;
pub use os::Os;
pub use purpose::*;
pub use technology::*;

use crate::error::Error;

/// Allowed characters: [a-z], [0-9], "_", "." and "-"
fn check_identifier_part(s: &str) -> Result<(), Error> {
    for char in s.chars() {
        if !char.is_ascii_lowercase() && !char.is_ascii_digit() && !['_', '-', '.'].contains(&char)
        {
            return Err(Error::IllegalIdentifier);
        }
    }

    Ok(())
}

#[test]
fn legal_identifier_part_chars() {
    assert!(check_identifier_part("arch").is_ok());
    assert!(check_identifier_part("foo-0.99_1").is_ok());
    assert!(check_identifier_part("a&b").is_err());
}
