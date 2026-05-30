use std::{
    fmt::{Display, Formatter},
    str::FromStr,
    string::ToString,
};

use alpm_parsers::{iter_char_context, prelude::*};
use serde::{Deserialize, Serialize};
use winnow::{
    Parser,
    combinator::{Repeat, alt, eof, peek, repeat, repeat_till},
    error::{ErrMode, StrContext, StrContextValue},
    token::one_of,
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
    // Only a subset of special characters is allowed as the first character of a name.
    const SPECIAL_FIRST_CHARS: [char; 3] = ['_', '@', '+'];
    // This set of characters is allowed anywhere, **except** as first character of a name.
    const NEVER_FIRST_CHAR: [char; 5] = ['_', '@', '+', '-', '.'];

    /// Create a new `Name`
    pub fn new(name: &str) -> Result<Self, Error> {
        Self::from_str(name)
    }

    /// Return a reference to the inner type
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl Name {
    /// Recognizes a [`Name`] as part of an [`InstalledPackage`](`crate::InstalledPackage`).
    ///
    /// # Warning
    ///
    /// This parser is designed **specifically** for the internal
    /// [`InstalledPackage`](`crate::InstalledPackage`) parser.
    ///
    /// [`InstalledPackage`](`crate::InstalledPackage`) is a very special use-case, as it uses
    /// dashes (`-`) as delimiter. However, dashes are also valid characters in a [`Name`].
    /// As such, the [`Name`] parser must be aware of how many dashes are expected to be inside the
    /// input string to parse.
    ///
    /// This is a necessary, albeit cursed hack due to
    /// [`InstalledPackage`](`crate::InstalledPackage`)'s dash-based delimiter design.
    ///
    /// In contrast to [`Name::parser`], this function expects the final character to be a `-`,
    /// which it **does not consume**.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` contains an invalid [alpm-package-name].
    ///
    /// [alpm-package-name]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
    pub(crate) fn parse_name_followed_by_version<'a>(
        delimiter_count: usize,
    ) -> impl Parser<Input<'a>, Self, ErrMode<ParseStack<'a>>> {
        let never_first_char_list = ['_', '@', '+', '.'];

        let alphanum = |c: char| c.is_ascii_alphanumeric();
        let first_char = one_of((alphanum, Self::SPECIAL_FIRST_CHARS))
            .context(StrContext::Label("first character of package name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )))
            .context_with(iter_char_context!(Self::SPECIAL_FIRST_CHARS));

        let never_first_char = one_of((alphanum, never_first_char_list));

        // The following is used to parse expressions such as this:
        // `example-package-name-1:45.2.0-x86_64`
        //
        // The parser will be called with `delimiters = 3`.
        // The `part` parser consumes all valid characters, except `-`.
        // `parts` then chains `part` 2 (`3-1`) times, where each part is expected to be followed by
        // a `-`.
        // This effectively consumes: `example-package-`
        //
        // If any invalid characters are in this section, `part` will terminate, `-` will not match
        // and a respective error message is thrown that points to that specific char.
        let part: Repeat<_, _, _, (), _> = repeat(0.., never_first_char);
        let parts: Repeat<_, _, _, (), _> = repeat(
            delimiter_count - 1,
            (
                part,
                '-'.context(StrContext::Label("character in package name"))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "ASCII alphanumeric character",
                    )))
                    .context_with(iter_char_context!(Self::NEVER_FIRST_CHAR)),
            ),
        );

        // Reconstruct the `part` parser, as we need it for the final step.
        let alphanum = |c: char| c.is_ascii_alphanumeric();
        let never_first_char = one_of((alphanum, never_first_char_list));
        let part: Repeat<_, _, _, (), _> = repeat(0.., never_first_char);

        // This is the final full parser. Let's go through it piece-by-piece.
        // `example-package-name-1:45.2.0-x86_64`
        let full_parser = (
            // Extracts `e`
            // `xample-package-name-1:45.2.0-x86_64`
            first_char,
            // Extracts the first two parts (and the following delimiters)
            // `name-1:45.2.0-x86_64`
            parts,
            // Extracts the single final part
            // `-1:45.2.0-x86_64`
            part,
            // Ensures the part is followed by a delimiter and not by an invalid char.
            // `-1:45.2.0-x86_64`
            peek('-')
                .context(StrContext::Label("character in package name"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "ASCII alphanumeric character",
                )))
                .context_with(iter_char_context!(Self::NEVER_FIRST_CHAR)),
        );

        full_parser
            .take()
            .layer("alpm-package-name")
            .map(|n: &str| Name(n.to_owned()))
    }
}

impl AlpmParser for Name {
    /// Recognizes a [`Name`] in a string slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the immediate start of `input` does not a valid [alpm-package-name].
    ///
    /// [alpm-package-version]: https://alpm.archlinux.page/specifications/alpm-package-name.7.html
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        let alphanum = |c: char| c.is_ascii_alphanumeric();
        let first_char = one_of((alphanum, Self::SPECIAL_FIRST_CHARS))
            .context(StrContext::Label("first character of package name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )))
            .context_with(iter_char_context!(Self::SPECIAL_FIRST_CHARS));

        let never_first_char = one_of((alphanum, Self::NEVER_FIRST_CHAR));

        // no .context() because this is infallible due to `0..`
        // note the empty tuple collection to avoid allocation
        let remaining_chars: Repeat<_, _, _, (), _> = repeat(0.., never_first_char);

        let full_parser = (first_char, remaining_chars);

        full_parser
            .take()
            .layer("alpm-package-name")
            .map(|n: &str| Name(n.to_owned()))
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Label("character in package name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "ASCII alphanumeric character",
            )))
            .context_with(iter_char_context!(Self::NEVER_FIRST_CHAR))
            .layer("alpm-package-name")
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
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
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
pub struct SharedObjectName(pub(crate) String);

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
}

impl AlpmParser for SharedObjectName {
    /// Parses a [`SharedObjectName`] from a string slice.
    fn parser<'a>(input: &mut Input<'a>) -> PResult<'a, Self> {
        // The SharedObjectName is basically a `Name` with extra restrictions (as it requires an
        // `.so`) extension.
        // As such, we re-implement the `Name` logic to ensure proper error handling.
        let alphanum = |c: char| c.is_ascii_alphanumeric();

        let never_first_char = one_of((alphanum, Name::NEVER_FIRST_CHAR));

        (
            // The first character, which has special restrictions
            one_of((alphanum, Name::SPECIAL_FIRST_CHARS))
                .context(StrContext::Label("first character of name"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "ASCII alphanumeric character",
                )))
                .context_with(iter_char_context!(Name::SPECIAL_FIRST_CHARS)),
            // Parse the name of the shared object until an `.so`, eof or an invalid character is
            // hit.
            repeat_till::<_, _, String, _, _, _, _>(1.., never_first_char, peek(alt((".so", eof))))
                .context(StrContext::Label("name")),
            // Then make sure that there's at least one or more `.so` suffix(es).
            repeat::<_, _, String, _, _>(1.., ".so")
                .take()
                .context(StrContext::Label("suffix"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "shared object name suffix '.so'",
                ))),
        )
            .take()
            .map(|n: &str| SharedObjectName(n.to_owned()))
            .parse_next(input)
    }

    fn delimiter_error_context<'a, O, P>(
        parser: P,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>
    where
        P: Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    {
        parser
            .context(StrContext::Label("shared object name"))
            .context(StrContext::Expected(StrContextValue::Description(
                "end of input.",
            )))
    }
}

impl FromStr for SharedObjectName {
    type Err = Error;
    /// Create an [`SharedObjectName`] from a string and return it in a Result
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parser_until_eof.parse(Input::new(s))?)
    }
}

impl Display for SharedObjectName {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::configure_insta;

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
    #[case("package_name_'''")]
    #[case("-package_with_leading_hyphen")]
    fn name_parse_error(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = Name::from_str(input) else {
            panic!("'{input}' erroneously parsed as a Name")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
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
    #[case("noso")]
    #[case("example.so.1")]
    fn invalid_shared_object_name_parser(#[case] input: &str) {
        let Err(Error::ParseError(err_msg)) = SharedObjectName::from_str(input) else {
            panic!("'{input}' erroneously parsed as a SonameV2")
        };

        let (test_name, _guard) = configure_insta();
        assert_snapshot!(test_name, err_msg.to_string());
    }
}
