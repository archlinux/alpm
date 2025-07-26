//! Traits for rootless backends.

use std::process::Output;

/// The options for a rootless backend.
pub trait RootlessOptions {
    /// Returns the options as a [`String`] [`Vec`].
    fn to_vec(&self) -> Vec<String>;
}

/// A backend for running a command as root.
pub trait RootlessBackend<T>
where
    T: RootlessOptions,
{
    /// The Error type to use.
    type Err;

    /// Creates a new [`RootlessBackend`] impl from a [`RootlessOptions`] impl.
    fn new(options: T) -> Self
    where
        Self: Sized;

    /// Returns the specific [`RootlessOptions`] used for the [`RootlessBackend`] impl.
    fn options(&self) -> &T;

    /// Runs a command as root using the [`RootlessBackend`] impl and returns the resulting
    /// [`Output`].
    fn run(&self, command: &[&str]) -> Result<Output, Self::Err>;
}
