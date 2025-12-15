//! Error handling for alpm-solve.

/// alpm-solve error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`alpm_types::Error`].
    #[error(transparent)]
    AlpmTypes(#[from] alpm_types::Error),

    /// Dependencies cannot be resolved.
    // Todo, we probably want actual data here instead of just a string.
    // todo 2: i18n
    #[error("Failed to solve dependencies:\n{0}")]
    Unsatisfiable(String),
}
