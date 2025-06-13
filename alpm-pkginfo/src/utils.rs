//! Utilities used by some or all PackageInfo versions.
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use alpm_types::{PackageRelation, SonameV1, SonameV2};
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
use crate::{PackageInfoV1, PackageInfoV2};

/// Provides either a [`PackageRelation`] or a [`SonameV1`].
///
/// This enum is used for [alpm-package-relations] of type _run-time dependency_ and _provision_ in
/// [`PackageInfoV1`] or [`PackageInfoV2`].
///
/// [alpm-package-relations]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationOrSoname {
    /// An [alpm-package-relation].
    ///
    /// [alpm-package-relation]: https://alpm.archlinux.page/specifications/alpm-package-relation.7.html
    Relation(PackageRelation),
    /// An [alpm-sonamev1].
    ///
    /// [alpm-sonamev1]: https://alpm.archlinux.page/specifications/alpm-sonamev1.7.html
    SonameV1(SonameV1),
    /// An [alpm-sonamev2].
    ///
    /// [alpm-sonamev2]: https://alpm.archlinux.page/specifications/alpm-sonamev2.7.html
    SonameV2(SonameV2),
}

impl RelationOrSoname {
    /// Recognizes a [`SonameV2`], a [`SonameV1`] or a [`PackageRelation`] in a string slice.
    ///
    /// First attempts to recognize a [`SonameV2`], then a [`SonameV1`] and if that fails, falls
    /// back to recognizing a [`PackageRelation`].
    /// Depending on recognized type, a [`RelationOrSoname`] is created accordingly.
    pub fn parser(input: &mut &str) -> ModalResult<Self> {
        // Implement a custom `winnow::combinator::alt`, as all type parsers are built in
        // such a way that they return errors on unexpected input instead of backtracking.
        let checkpoint = input.checkpoint();
        let sonamev2_result = SonameV2::parser.parse_next(input);
        if sonamev2_result.is_ok() {
            let sonamev2 = sonamev2_result?;
            return Ok(RelationOrSoname::SonameV2(sonamev2));
        }

        input.reset(&checkpoint);
        let sonamev1_result = SonameV1::parser.parse_next(input);
        if sonamev1_result.is_ok() {
            let sonamev1 = sonamev1_result?;
            return Ok(RelationOrSoname::SonameV1(sonamev1));
        }

        input.reset(&checkpoint);
        let relation_result = rest.and_then(PackageRelation::parser).parse_next(input);
        if relation_result.is_ok() {
            let relation = relation_result?;
            return Ok(RelationOrSoname::Relation(relation));
        }

        cut_err(fail)
            .context(StrContext::Expected(StrContextValue::Description(
                "alpm-sonamev2, alpm-sonamev1 or alpm-package-relation",
            )))
            .parse_next(input)
    }
}

impl Display for RelationOrSoname {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationOrSoname::Relation(version) => write!(f, "{version}"),
            RelationOrSoname::SonameV1(soname) => write!(f, "{soname}"),
            RelationOrSoname::SonameV2(soname) => write!(f, "{soname}"),
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
    /// use alpm_pkginfo::RelationOrSoname;
    /// use alpm_types::{PackageRelation, SonameV1, SonameV2};
    ///
    /// # fn main() -> Result<(), alpm_pkginfo::Error> {
    /// let relation: RelationOrSoname = "example=1.0.0".parse()?;
    /// assert_eq!(
    ///     relation,
    ///     RelationOrSoname::Relation(PackageRelation::new(
    ///         "example".parse()?,
    ///         Some("=1.0.0".parse()?)
    ///     ))
    /// );
    ///
    /// let sonamev2: RelationOrSoname = "lib:example.so.1".parse()?;
    /// assert_eq!(
    ///     sonamev2,
    ///     RelationOrSoname::SonameV2(SonameV2::new("lib".parse()?, "example.so.1".parse()?))
    /// );
    ///
    /// let sonamev1: RelationOrSoname = "example.so".parse()?;
    /// assert_eq!(
    ///     sonamev1,
    ///     RelationOrSoname::SonameV1(SonameV1::new("example.so".parse()?, None, None)?)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parser
            .parse(s)
            .map_err(|error| Error::AlpmType(alpm_types::Error::ParseError(error.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use alpm_types::{ElfArchitectureFormat, VersionOrSoname};
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
        "lib:example.so.1",
        RelationOrSoname::SonameV2(
            SonameV2::new(
                "lib".parse().unwrap(),
                "example.so.1".parse().unwrap(),
            )
        )
    )]
    #[case(
        "lib:example.so",
        RelationOrSoname::SonameV2(
            SonameV2::new(
                "lib".parse().unwrap(),
                "example.so".parse().unwrap(),
            )
        )
    )]
    #[case(
        "example.so",
        RelationOrSoname::SonameV1(
            SonameV1::new(
                "example.so".parse().unwrap(),
                None,
                None,
            ).unwrap()
        )
    )]
    #[case(
        "example.so=1.0.0-64",
        RelationOrSoname::SonameV1(
            SonameV1::new(
                "example.so".parse().unwrap(),
                Some(VersionOrSoname::Version("1.0.0".parse().unwrap())),
                Some(ElfArchitectureFormat::Bit64),
            ).unwrap()
        )
    )]
    #[case(
        "libexample.so=otherlibexample.so-64",
        RelationOrSoname::SonameV1(
            SonameV1::new(
                "libexample.so".parse().unwrap(),
                Some(VersionOrSoname::Soname("otherlibexample.so".parse().unwrap())),
                Some(ElfArchitectureFormat::Bit64),
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
