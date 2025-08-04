//! Custom value handling compatible with [`FluentValue`].

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

/// Re-export the type.
pub use fluent_templates::fluent_bundle::FluentValue;

/// Helper trait for converting various types to a [`FluentValue`].
///
/// This trait is a wrapper for the `From` implementation of [`FluentValue`]
/// for providing more methods to convert different types.
///
/// One example is converting [`PathBuf`] which is originally not supported
/// but we support it by converting it to a string.
pub trait ToFluentValue {
    /// Converts the value to a [`FluentValue`].
    fn to_fluent_value(&self) -> FluentValue<'static>;
}

impl ToFluentValue for Path {
    /// Converts a [`Path`] to a [`FluentValue`] by converting it to a string.
    ///
    /// # Note
    ///
    /// This transforms the [`Path`] using [`Path::to_string_lossy`].
    fn to_fluent_value(&self) -> FluentValue<'static> {
        FluentValue::from(self.to_string_lossy().into_owned())
    }
}

impl ToFluentValue for PathBuf {
    /// Converts a [`PathBuf`] to a [`FluentValue`] by converting it to a string.
    ///
    /// Calls the [`ToFluentValue`] implementation for [`Path`].
    fn to_fluent_value(&self) -> FluentValue<'static> {
        self.as_path().to_fluent_value()
    }
}

impl<T> ToFluentValue for Option<T>
where
    T: ToFluentValue,
{
    /// Blanket implementation for `Option<T>`
    fn to_fluent_value(&self) -> FluentValue<'static> {
        match self {
            Some(value) => value.to_fluent_value(),
            None => FluentValue::None,
        }
    }
}

/// Helper macro to implement the [`ToFluentValue`] trait for a list of types.
///
/// This macro calls [`FluentValue::from`] for the given type under the hood.
///
/// See [trait implementations] for the list of types that implement [`From`] for [`FluentValue`].
///
/// [trait implementations]: https://docs.rs/fluent-bundle/latest/fluent_bundle/enum.FluentValue.html#trait-implementations
macro_rules! impl_fluent_for {
    ( $( $t:ty ),+ $(,)? ) => {
        $(
            impl ToFluentValue for $t {
                fn to_fluent_value(&self) -> FluentValue<'static> {
                    FluentValue::from(self.clone())
                }
            }
        )+
    };
}

// Implement for most common types.
impl_fluent_for!(
    String,
    Cow<'static, str>,
    usize,
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    &'static str
);

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use crate::{set_locale, t};

    /// Asserts that the custom [`FluentValue`] conversions
    /// work as expected (e.g. for [`PathBuf`]).
    #[test]
    fn test_custom_fluent_type() -> TestResult<()> {
        set_locale(Some("en-US"))?;

        let path = std::path::PathBuf::from("/some/path/to/file.txt");
        let message = t!("error-io-path", { "context" => "reading", "path" => path });
        assert_eq!(message, "I/O error while reading: /some/path/to/file.txt");

        let opt_path: Option<std::path::PathBuf> = Some(path);
        let message = t!("error-io-path", { "context" => "writing", "path" => opt_path });
        assert_eq!(message, "I/O error while writing: /some/path/to/file.txt");

        Ok(())
    }
}
