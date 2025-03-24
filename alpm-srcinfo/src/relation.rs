//! Custom package relationship types.
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{PackageRelation, SharedObjectName, SonameV1};
use serde::{Deserialize, Serialize};
use winnow::{
    ModalResult,
    Parser,
    combinator::{cut_err, fail},
    error::{StrContext, StrContextValue},
    stream::Stream,
    token::rest,
};

use crate::Error;
#[cfg(doc)]
use crate::SourceInfoV1;

/// Provides either a [`PackageRelation`] or a [`SonameV1::Basic`].
///
/// This enum is used for [alpm-package-relations] of type _run-time dependency_ and _provision_ in
/// [`SourceInfoV1`].
///
/// [alpm-package-relations]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RelationOrSoname {
    /// A shared object name (as [`SonameV1::Basic`]).
    BasicSonameV1(SonameV1),
    /// A package relation (as [`PackageRelation`]).
    Relation(PackageRelation),
}

impl PartialEq<PackageRelation> for RelationOrSoname {
    fn eq(&self, other: &PackageRelation) -> bool {
        self.to_string() == other.to_string()
    }
}

impl PartialEq<SonameV1> for RelationOrSoname {
    fn eq(&self, other: &SonameV1) -> bool {
        self.to_string() == other.to_string()
    }
}

impl RelationOrSoname {
    /// Recognizes a [`SonameV1::Basic`] or a [`PackageRelation`] in a string slice.
    ///
    /// First attempts to recognize a [`SonameV1::Basic`] and if that fails, falls back to
    /// recognizing a [`PackageRelation`].
    /// Depending on recognized type, a [`RelationOrSoname`] is created accordingly.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Implement a custom `winnow::combinator::alt`, as all type parsers are built in
        // such a way that they return errors on unexpected input instead of backtracking.
        let checkpoint = input.checkpoint();
        let shared_object_name_result = SharedObjectName::parser.parse_next(input);
        if shared_object_name_result.is_ok() {
            let shared_object_name = shared_object_name_result?;
            return Ok(RelationOrSoname::BasicSonameV1(SonameV1::Basic(
                shared_object_name,
            )));
        }

        input.reset(&checkpoint);
        let relation_result = rest.and_then(PackageRelation::parser).parse_next(input);
        if relation_result.is_ok() {
            let relation = relation_result?;
            return Ok(RelationOrSoname::Relation(relation));
        }

        cut_err(fail)
            .context(StrContext::Expected(StrContextValue::Description(
                "alpm-sonamev1 or alpm-package-relation",
            )))
            .parse_next(input)
    }
}

impl Display for RelationOrSoname {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationOrSoname::Relation(version) => write!(f, "{version}"),
            RelationOrSoname::BasicSonameV1(soname) => write!(f, "{soname}"),
        }
    }
}

impl FromStr for RelationOrSoname {
    type Err = Error;

    /// Creates a [`RelationOrSoname`] from a string slice.
    ///
    /// Relies on [`RelationOrSoname::parser`] to recognize types in `input` and create a
    /// [`RelationOrSoname`] accordingly.
    ///
    /// # Errors
    ///
    /// Returns an error if no [`RelationOrSoname`] can be created from `input`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpm_srcinfo::RelationOrSoname;
    /// use alpm_types::{PackageRelation, SonameV1};
    ///
    /// # fn main() -> Result<(), alpm_srcinfo::Error> {
    /// let relation: RelationOrSoname = "example=1.0.0".parse()?;
    /// assert_eq!(
    ///     relation,
    ///     RelationOrSoname::Relation(PackageRelation::new(
    ///         "example".parse()?,
    ///         Some("=1.0.0".parse()?)
    ///     ))
    /// );
    ///
    /// let soname: RelationOrSoname = "example.so".parse()?;
    /// assert_eq!(
    ///     soname,
    ///     RelationOrSoname::BasicSonameV1(SonameV1::new("example.so".parse()?, None, None)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parser
            .parse(s)
            .map_err(|error| Error::ParseError(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    #[rstest]
    #[case(
        "example",
        RelationOrSoname::Relation(PackageRelation::new("example".parse().unwrap(), None))
    )]
    #[case(
        "example=1.0.0",
        RelationOrSoname::Relation(PackageRelation::new("example".parse().unwrap(), "=1.0.0".parse().ok())) 
    )]
    #[case(
        "example>=1.0.0",
        RelationOrSoname::Relation(PackageRelation::new("example".parse().unwrap(), ">=1.0.0".parse().ok()))
    )]
    #[case(
        "example.so",
        RelationOrSoname::BasicSonameV1(
            SonameV1::new(
                "example.so".parse().unwrap(),
                None,
                None,
            ).unwrap()
        )
    )]
    fn test_relation_or_soname_parser(
        #[case] mut input: &str,
        #[case] expected: RelationOrSoname,
    ) -> TestResult {
        let input_str = input.to_string();
        let result = RelationOrSoname::parser(&mut input)?;
        assert_eq!(result, expected);
        assert_eq!(result.to_string(), input_str);
        Ok(())
    }
}
