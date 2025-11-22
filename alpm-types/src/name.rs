use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use alpm_parsers::iter_char_context;
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{Repeat, alt, cut_err, eof, peek, repeat, repeat_till},
    error::{StrContext, StrContextValue},
    stream::Stream,
    token::{any, one_of, rest},
};

use crate::Error;

/// A build tool name
///
/// The same character restrictions as with `Name` apply.
/// Further name restrictions may be enforced on an existing instances using
/// `matches_restriction()`.
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{BuildTool, Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create BuildTool from &str
/// assert!(BuildTool::from_str("test-123@.foo_+").is_ok());
/// assert!(BuildTool::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", BuildTool::from_str("foo")?));
///
/// // validate that BuildTool follows naming restrictions
/// let buildtool = BuildTool::from_str("foo")?;
/// let restrictions = vec![Name::from_str("foo")?, Name::from_str("bar")?];
/// assert!(buildtool.matches_restriction(&restrictions));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildTool(Name);

impl BuildTool {
    /// Create a new BuildTool
    pub fn new(name: Name) -> Self {
        BuildTool(name)
    }

    /// Create a new BuildTool in a Result, which matches one Name in a list of restrictions
    ///
    /// ## Examples
    /// ```
    /// use alpm_types::{BuildTool, Error, Name};
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("foo")?]).is_ok());
    /// assert!(BuildTool::new_with_restriction("foo", &[Name::new("bar")?]).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_restriction(name: &str, restrictions: &[Name]) -> Result<Self, Error> {
        let buildtool = BuildTool::from_str(name)?;
        if buildtool.matches_restriction(restrictions) {
            Ok(buildtool)
        } else {
            Err(Error::ValueDoesNotMatchRestrictions {
                restrictions: restrictions.iter().map(ToString::to_string).collect(),
            })
        }
    }

    /// Validate that the BuildTool has a name matching one Name in a list of restrictions
    pub fn matches_restriction(&self, restrictions: &[Name]) -> bool {
        restrictions
            .iter()
            .any(|restriction| restriction.eq(self.inner()))
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &Name {
        &self.0
    }
}

impl FromStr for BuildTool {
    type Err = Error;
    /// Create a BuildTool from a string
    fn from_str(s: &str) -> Result<BuildTool, Self::Err> {
        Name::new(s).map(BuildTool)
    }
}

impl Display for BuildTool {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

/// A package name
///
/// Package names may contain the characters `[a-zA-Z0-9\-._@+]`, but must not
/// start with `[-.]` (see [alpm-package-name]).
///
/// ## Examples
/// ```
/// use std::str::FromStr;
///
/// use alpm_types::{Error, Name};
///
/// # fn main() -> Result<(), alpm_types::Error> {
/// // create Name from &str
/// assert_eq!(
///     Name::from_str("test-123@.foo_+"),
///     Ok(Name::new("test-123@.foo_+")?)
/// );
/// assert!(Name::from_str(".test").is_err());
///
/// // format as String
/// assert_eq!("foo", format!("{}", Name::new("foo")?));
/// # Ok(())
/// # }
/// ```
///
/// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Name(String);

impl Name {
    /// Create a new `Name`
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::from_str(name)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Recognizes a [`Name`] in a string slice.
    ///
    /// Consumes all of its input.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` contains an invalid _alpm-package-name_.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        let alphanum = |c: char| c.is_ascii_alphanumeric();
        let special_first_chars = ['_', '@', '+'];
        let first_char = one_of((alphanum, special_first_chars))
            .context(StrContext::Label("first character of package name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )))
            .context_with(iter_char_context!(special_first_chars));

        let never_first_special_chars = ['_', '@', '+', '-', '.'];
        let never_first_char = one_of((alphanum, never_first_special_chars));

        // no .context() because this is infallible due to `0..`
        // note the empty tuple collection to avoid allocation
        let remaining_chars: Repeat<_, _, _, (), _> = repeat(0.., never_first_char);

        let full_parser = (
            first_char,
            remaining_chars,
            // bad characters fall through to eof so we insert that context here
            eof.context(StrContext::Label("character in package name"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "ASCII alphanumeric character",
                )))
                .context_with(iter_char_context!(never_first_special_chars)),
        );

        full_parser
            .take()
            .map(|n: &str| Name(n.to_owned()))
            .parse_next(input)
    }
}

impl FromStr for Name {
    type Err = Error;

    /// Creates a [`Name`] from a string slice.
    ///
    /// Delegates to [`Name::parser`].
    ///
    /// # Errors
    ///
    /// Returns an error if [`Name::parser`] fails.
    fn from_str(s: &str) -> Result<Name, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.inner())
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.inner()
    }
}

/// A shared object name.
///
/// This type wraps a [`Name`] and is used to represent the name of a shared object file
/// that ends with the `.so` suffix.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SharedObjectName(pub(crate) Name);

impl SharedObjectName {
    /// Creates a new [`SharedObjectName`].
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not end with `.so`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_types::SharedObjectName;
    ///
    /// # fn main() -> Result<(), alpm_types::Error> {
    /// let shared_object_name = SharedObjectName::new("example.so")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::from_str(name)
    }

    /// Returns the name of the shared object as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Parses a [`SharedObjectName`] from a string slice.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Make a checkpoint for parsing the full name in one go later on.
        // The full name will later on include the `.so` extension, but we have to make sure first
        // that the name has the correct structure.
        // (a filename followed by one or more `.so` suffixes)
        let checkpoint = input.checkpoint();

        // Parse the name of the shared object until eof or the `.so` is hit.
        repeat_till::<_, _, String, _, _, _, _>(1.., any, peek(alt((".so", eof))))
            .context(StrContext::Label("name"))
            .parse_next(input)?;

        // Parse at least one or more `.so` suffix(es).
        cut_err(repeat::<_, _, String, _, _>(1.., ".so").take())
            .context(StrContext::Label("suffix"))
            .context(StrContext::Expected(StrContextValue::Description(
                "shared object name suffix '.so'",
            )))
            .parse_next(input)?;

        // Ensure that there is no trailing content
        cut_err(eof)
            .context(StrContext::Label(
                "unexpected trailing content after shared object name.",
            ))
            .context(StrContext::Expected(StrContextValue::Description(
                "end of input.",
            )))
            .parse_next(input)?;

        input.reset(&checkpoint);
        let name = rest
            .and_then(Name::parser)
            .context(StrContext::Label("name"))
            .parse_next(input)?;

        Ok(SharedObjectName(name))
    }
}

impl FromStr for SharedObjectName {
    type Err = Error;
    /// Create an [`SharedObjectName`] from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser.parse(s)?)
    }
}

impl Display for SharedObjectName {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "bar",
        ["foo".parse(), "bar".parse()].into_iter().flatten().collect::<Vec<Name>>(),
        Ok(BuildTool::from_str("bar").unwrap()),
    )]
    #[case(
        "bar",
        ["foo".parse(), "foo".parse()].into_iter().flatten().collect::<Vec<Name>>(),
        Err(Error::ValueDoesNotMatchRestrictions {
            restrictions: vec!["foo".to_string(), "foo".to_string()],
        }),
    )]
    fn buildtool_new_with_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: Result<BuildTool, Error>,
    ) {
        assert_eq!(
            BuildTool::new_with_restriction(buildtool, &restrictions),
            result
        );
    }

    #[rstest]
    #[case("bar", ["foo".parse(), "bar".parse()].into_iter().flatten().collect::<Vec<Name>>(), true)]
    #[case("bar", ["foo".parse(), "foo".parse()].into_iter().flatten().collect::<Vec<Name>>(), false)]
    fn buildtool_matches_restriction(
        #[case] buildtool: &str,
        #[case] restrictions: Vec<Name>,
        #[case] result: bool,
    ) {
        let buildtool = BuildTool::from_str(buildtool).unwrap();
        assert_eq!(buildtool.matches_restriction(&restrictions), result);
    }

    #[rstest]
    #[case("package_name_'''", "invalid character in package name")]
    #[case("-package_with_leading_hyphen", "invalid first character")]
    fn name_parse_error(#[case] input: &str, #[case] err_snippet: &str) {
        let Err(Error::ParseError(err_msg)) = Name::from_str(input) else {
            panic!("'{input}' erroneously parsed as a Name")
        };
        assert!(
            err_msg.contains(err_snippet),
            "Error:\n=====\n{err_msg}\n=====\nshould contain snippet:\n\n{err_snippet}"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn valid_name_from_string(name_str in r"[a-zA-Z0-9_@+]+[a-zA-Z0-9\-._@+]*") {
            let name = Name::from_str(&name_str).unwrap();
            prop_assert_eq!(name_str, format!("{}", name));
        }

        #[test]
        fn invalid_name_from_string_start(name_str in r"[-.][a-zA-Z0-9@._+-]*") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert!(matches!(error, Error::ParseError(_)));
        }

        #[test]
        fn invalid_name_with_invalid_characters(name_str in r"[^\w@._+-]+") {
            let error = Name::from_str(&name_str).unwrap_err();
            assert!(matches!(error, Error::ParseError(_)));
        }
    }

    #[rstest]
    #[case("example.so", SharedObjectName("example.so".parse().unwrap()))]
    #[case("example.so.so", SharedObjectName("example.so.so".parse().unwrap()))]
    #[case("libexample.1.so", SharedObjectName("libexample.1.so".parse().unwrap()))]
    fn shared_object_name_parser(
        #[case] input: &str,
        #[case] expected_result: SharedObjectName,
    ) -> testresult::TestResult<()> {
        let shared_object_name = SharedObjectName::new(input)?;
        assert_eq!(expected_result, shared_object_name);
        assert_eq!(input, shared_object_name.as_str());
        Ok(())
    }

    #[rstest]
    #[case("noso", "expected shared object name suffix '.so'")]
    #[case("example.so.1", "unexpected trailing content after shared object name")]
    fn invalid_shared_object_name_parser(#[case] input: &str, #[case] error_snippet: &str) {
        let result = SharedObjectName::from_str(input);
        assert!(result.is_err(), "Expected SharedObjectName parsing to fail");
        let err = result.unwrap_err();
        let pretty_error = err.to_string();
        assert!(
            pretty_error.contains(error_snippet),
            "Error:\n=====\n{pretty_error}\n=====\nshould contain snippet:\n\n{error_snippet}"
        );
    }
}
