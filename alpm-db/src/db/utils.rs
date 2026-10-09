//! Utility functions for alpm-db handling.

/// The file mode of directories on disk.
pub(crate) const DIR_MODE: u32 = 0o040755;

/// Checks whether a given Unix-style `mode` is considered world-writable.
///
/// Extracts the last octet of `mode` and checks against all octals that would imply "writable".
pub(crate) fn permissions_world_writable(mode: u32) -> bool {
    let last_octal = mode % 8;
    [2, 3, 6, 7].contains(&last_octal)
}
