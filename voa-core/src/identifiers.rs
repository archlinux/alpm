//! Identifier types for each layer of the VOA filesystem hierarchy.
//!
//! VOA signature verifiers are located in a directory structure described by the VOA file
//! hierarchy. The hierarchy consists of distinct subdirectory layers, each represented by a
//! specific identifier type.
//!
//! This module provides types for each of the four VOA identifier layers:
//! [`Os`], [`Purpose`], [`Context`], [`Technology`].
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

/// Check if `value` contains a valid identifier part.
///
/// Identifier parts must only contain a set of legal characters: [a-z], [0-9], "_", "." and "-".
/// A valid identifier part may not be an empty string.
fn check_identifier_part(value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::IllegalIdentifier);
    }

    for char in value.chars() {
        if !char.is_ascii_lowercase() && !char.is_ascii_digit() && !['_', '-', '.'].contains(&char)
        {
            return Err(Error::IllegalIdentifier);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::identifiers::check_identifier_part;

    #[test]
    fn legal_identifier_parts() {
        assert!(check_identifier_part("arch").is_ok());
        assert!(check_identifier_part("foo-0.99_1").is_ok());
        assert!(check_identifier_part("a&b").is_err());

        assert!(check_identifier_part("").is_err());
    }
}
