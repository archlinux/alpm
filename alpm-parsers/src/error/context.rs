//! The [`StringContext`] type
//!
//! This is our own `String`-capable version of the [`StrContext`] parser.

use winnow::error::{StrContext, StrContextValue};

/// A single piece of parser context that is attached to a [`ParseStack`] error.
///
/// In contrast to winnow's [`StrContext`], which only holds `&'static str`, this types allows
/// messages as [`String`]s.
///
/// This allows for setting error messages during runtime, which is a requirement for translated
/// messages and generally provides more flexibility.
///
/// [`ParseStack`]: crate::error::ParseStack
#[derive(Clone, Debug)]
pub(crate) enum StringContext {
    /// A description of what is currently being parsed.
    Label(String),

    /// A free-form description of what went wrong.
    Description(String),

    /// An expected [`char`] literal.
    ExpectedChar(char),

    /// An expected string literal.
    ///
    /// Use this for known, static syntax/keywords.
    ExpectedString(&'static str),

    /// An free-form string that explains what is expected.
    ///
    /// Use this for known, static syntax/keywords.
    ExpectedText(String),
}

impl StringContext {
    /// Returns the label text in case this context is a [`StringContext::Label`].
    pub(crate) fn label(&self) -> Option<&str> {
        match self {
            StringContext::Label(label) => Some(label),
            _ => None,
        }
    }

    /// Returns the expected literal if `Self` is of a literal variant.
    pub(crate) fn expected(&self) -> Option<String> {
        match self {
            StringContext::Label(_) => None,
            StringContext::Description(_) => None,
            StringContext::ExpectedChar(c) => Some(Self::render_char(*c)),
            StringContext::ExpectedString(s) => Some(format!("'{s}'")),
            StringContext::ExpectedText(s) => Some(s.into()),
        }
    }

    /// Renders an expected [`char`].
    ///
    /// This is copied over from winnow's [`StrContextValue`] `Display` impl.
    fn render_char(c: char) -> String {
        match c {
            '\n' => "newline".to_string(),
            '`' => "'`'".to_string(),
            c if c.is_ascii_control() => format!("`{}`", c.escape_debug()),
            c => format!("`{c}`"),
        }
    }
}

impl From<StrContext> for StringContext {
    fn from(context: StrContext) -> Self {
        match context {
            StrContext::Label(label) => StringContext::Label(label.to_string()),
            StrContext::Expected(value) => value.into(),
            // `StrContext` is non-exhaustive for some reason.
            // We just create a `Description` in case a new variant is introduced. Since we aim to
            // only use our own error type, this form of backwards compatibility will become
            // obsolete after some time anyway.
            other => StringContext::Description(other.to_string()),
        }
    }
}

impl From<StrContextValue> for StringContext {
    fn from(value: StrContextValue) -> Self {
        match value {
            StrContextValue::CharLiteral(c) => StringContext::ExpectedChar(c),
            StrContextValue::StringLiteral(s) => StringContext::ExpectedString(s),
            StrContextValue::Description(description) => {
                StringContext::ExpectedText(description.to_string())
            }
            // See the `impl From<StrContext>` equivalent for more context.
            other => StringContext::Description(other.to_string()),
        }
    }
}
