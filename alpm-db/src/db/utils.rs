//! Utility functions for alpm-db handling.

/// The file mode of directories on disk.
pub(crate) const DIR_MODE: u32 = 0o040755;

/// Checks whether a given Unix-style `mode` is considered world-writable.
///
/// This ensures, that the last octal of `mode` is not `2`, `3`, `6` or `7`.
pub(crate) fn permissions_world_writable(mode: u32) -> bool {
    mode & 0o002 != 0
}
