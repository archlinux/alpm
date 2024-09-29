use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use crate::Error;

/// Trait to implement an `ASSIGN` field (`" = "` by default)
pub trait Assign {
    const ASSIGN: &'static str = " = ";
}

/// Wrap a String keyword
///
/// This struct implements the [`Assign`] trait and therefore exposes a const `ASSIGN` field.
///
/// ## Examples
///
/// ```ignore
/// use std::str::FromStr;
///
/// use alpm_buildinfo::common::KeyAssign;
///
/// let keyassign = KeyAssign::new("foo".to_string());
/// assert_eq!(keyassign.to_string(), "foo = ")
/// ```
pub struct KeyAssign(String);

impl KeyAssign {
    /// Create a new KeyAssign from String
    pub fn new(key: String) -> Self {
        KeyAssign(key)
    }

    /// Return a reference to the wrapped keyword
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Return the length of wrapped keyword and assignment
    pub fn len_all(&self) -> usize {
        self.to_string().len()
    }
}

impl Assign for KeyAssign {}

impl Display for KeyAssign {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}{}", self.0, KeyAssign::ASSIGN)
    }
}

impl FromStr for KeyAssign {
    type Err = Error;
    fn from_str(input: &str) -> Result<KeyAssign, Error> {
        Ok(KeyAssign::new(input.to_string()))
    }
}

/// Get a value of type T from a line of text, extracted using a KeyAssign
///
/// ## Errors
///
/// Returns an [`Error::InvalidValue`] if `FromStr` for `T` does not succeed.
///
/// ## Examples
///
/// ```ignore
/// use alpm_buildinfo::common::{get_multiple, KeyAssign};
/// use alpm_types::BuildEnv;
///
/// assert_eq!(
///     get_multiple::<BuildEnv>(
///         &KeyAssign::new("buildenv".to_string()),
///         "buildenv = env",
///         1,
///         "1"
///     )
///     .unwrap(),
///     BuildEnv::new("env").unwrap(),
/// );
///
/// assert!(get_multiple::<BuildEnv>(
///     &KeyAssign::new("buildenv".to_string()),
///     "buildenv = \\foo",
///     1,
///     "1"
/// )
/// .is_err());
/// ```
pub fn get_multiple<T: FromStr<Err = alpm_types::Error>>(
    keyassign: &KeyAssign,
    line: &str,
    number: usize,
    buildinfo_version: &str,
) -> Result<T, Error> {
    match T::from_str(line.split_at(keyassign.len_all()).1) {
        Ok(value) => Ok(value),
        Err(error) => Err(Error::InvalidValue(
            buildinfo_version.into(),
            keyassign.inner().to_string(),
            crate::error::ErrorLine {
                number,
                line: line.into(),
            },
            error,
        )),
    }
}

/// Get a value of type T from a line of text, extracted using a KeyAssign, ensuring to only get
/// some once
///
/// NOTE: For types returning `strum::ParseError` in their `FromStr` implementation, use
/// `get_once_strum()` instead!
///
/// ## Errors
///
/// Returns an [`Error::MultipleOccurences`] if `field` is already `Some(T)`.
/// Returns an [`Error::InvalidValue`] if `FromStr` for `T` does not succeed.
///
/// ## Examples
///
/// ```ignore
/// use alpm_buildinfo::common::{get_once, KeyAssign};
/// use alpm_types::BuildDate;
///
/// let mut field: Option<BuildDate> = None;
///
/// assert_eq!(
///     get_once(
///         &KeyAssign::new("builddate".to_string()),
///         field,
///         "builddate = 1",
///         1,
///         "1"
///     )
///     .unwrap(),
///     Some(BuildDate::new(1)),
/// );
///
/// field = Some(BuildDate::new(1));
/// assert!(get_once(
///     &KeyAssign::new("builddate".to_string()),
///     field,
///     "builddate = 1",
///     1,
///     "1"
/// )
/// .is_err());
/// ```
pub fn get_once<T: FromStr<Err = alpm_types::Error>>(
    keyassign: &KeyAssign,
    field: Option<T>,
    line: &str,
    number: usize,
    buildinfo_version: &str,
) -> Result<Option<T>, Error> {
    if field.is_some() {
        return Err(Error::MultipleOccurences(
            buildinfo_version.into(),
            keyassign.inner().to_string(),
            crate::error::ErrorLine {
                number,
                line: line.into(),
            },
        ));
    }

    match T::from_str(line.split_at(keyassign.len_all()).1) {
        Ok(value) => Ok(Some(value)),
        Err(error) => Err(Error::InvalidValue(
            buildinfo_version.into(),
            keyassign.inner().to_string(),
            crate::error::ErrorLine {
                number,
                line: line.into(),
            },
            error,
        )),
    }
}

/// Get a value of type T from a line of text, extracted using a KeyAssign, ensuring to only get
/// some once
///
/// NOTE: This is a specific implementation to deal with types that return `[strum::ParseError]` in
/// their `FromStr` implementation, as the compiler can not derive the correct `Error` type when
/// using `get_once()` otherwise.
///
/// ## Errors
///
/// Returns an [`Error::MultipleOccurences`] if `field` is already `Some(T)`.
/// Returns an [`Error::InvalidValue`] if `FromStr` for `T` does not succeed.
///
/// ## Examples
///
/// ```ignore
/// use alpm_buildinfo::common::{get_once_strum, KeyAssign};
/// use alpm_types::Architecture;
///
/// let mut field: Option<Architecture> = None;
///
/// assert_eq!(
///     get_once_strum(
///         &KeyAssign::new("pkgarch".to_string()),
///         field,
///         "pkgarch = any",
///         1,
///         "1"
///     )
///     .unwrap(),
///     Some(Architecture::Any),
/// );
///
/// field = Some(Architecture::Any);
/// assert!(get_once_strum(
///     &KeyAssign::new("pkgarch".to_string()),
///     field,
///     "pkgarch = any",
///     1,
///     "1"
/// )
/// .is_err());
/// ```
pub fn get_once_strum<T: FromStr<Err = strum::ParseError>>(
    keyassign: &KeyAssign,
    field: Option<T>,
    line: &str,
    number: usize,
    buildinfo_version: &str,
) -> Result<Option<T>, Error> {
    if field.is_some() {
        return Err(Error::MultipleOccurences(
            buildinfo_version.into(),
            keyassign.inner().to_string(),
            crate::error::ErrorLine {
                number,
                line: line.into(),
            },
        ));
    }

    match T::from_str(line.split_at(keyassign.len_all()).1) {
        Ok(value) => Ok(Some(value)),
        Err(error) => Err(Error::InvalidValue(
            buildinfo_version.into(),
            keyassign.inner().to_string(),
            crate::error::ErrorLine {
                number,
                line: line.into(),
            },
            error.into(),
        )),
    }
}

/// Ensure that a mandatory field is available
///
/// ## Errors
///
/// Returns an [`Error::MissingKeyValue`] if `field` is `None`.
///
/// ## Examples
///
/// ```ignore
/// use alpm_buildinfo::common::{ensure_mandatory_field, KeyAssign};
/// use alpm_types::BuildDate;
///
/// let mut field: Option<BuildDate> = None;
/// assert!(ensure_mandatory_field(field, "builddate", "1").is_err());
///
/// field = Some(BuildDate::new(1));
/// assert!(ensure_mandatory_field(field, "builddate", "1").is_ok());
/// ```
pub fn ensure_mandatory_field<T>(
    field: Option<T>,
    keyword: &str,
    buildinfo_version: &str,
) -> Result<T, Error> {
    if let Some(value) = field {
        Ok(value)
    } else {
        Err(Error::MissingKeyValue(
            buildinfo_version.into(),
            keyword.into(),
        ))
    }
}

/// Create a String from a keyword and a list of types implementing the Display trait
///
/// ## Examples
///
/// ```ignore
/// use alpm_buildinfo::common::{keyword_with_list_entries, KeyAssign};
/// use alpm_types::BuildEnv;
///
/// assert_eq!(
///     keyword_with_list_entries(
///         &KeyAssign::new("buildenv".to_string()),
///         &[BuildEnv::new("foo").unwrap(), BuildEnv::new("bar").unwrap()],
///     ),
///     "buildenv = foo\nbuildenv = bar",
/// );
/// ```
pub fn keyword_with_list_entries<T: Display>(keyassign: &KeyAssign, entries: &[T]) -> String {
    let output: Vec<String> = entries
        .iter()
        .map(|x| format!("{}{}", keyassign, x))
        .collect();
    output.join("\n")
}
