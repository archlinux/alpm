#![allow(dead_code)]

use alpm_types::{
    digests::{Blake2b512, Digest, Md5, Sha1, Sha224, Sha256, Sha384, Sha512},
    Architecture,
    Checksum,
    License,
    OpenPGPv4Fingerprint,
    Pkgrel,
    Pkgver,
    Url,
};
use winnow::*;

/// An arbitrary `String` attribute that's potentially specific to a certain architecture.
/// If no option is specified, it relates to the current default architectures.
///
/// If no architecture is provided `any` (the current architecture) is assumed.
struct ArchProperty {
    arch: Option<Architecture>,
    value: String,
}

/// An source file's checksum that's potentially specific to a certain architecture.
/// If no option is specified, it relates to the current default architectures.
///
/// If no architecture is provided `any` (the current architecture) is assumed.
struct ArchChecksum<D: Digest> {
    arch: Option<Architecture>,
    value: Checksum<D>,
}

/// This enum represents all lines of a SRCINFO file.
///
/// The lines have the identical order in which they appear in the SRCINFO file, which is important
/// as the file is stateful and we need to normalize the data in the next step.
///
/// Sadly we have to do it this way as the format theoretically allows comments and empty lines at
/// any given time. To produce meaningful error messages during the normalization step, we need to
/// know the line number on which the error occurred, which is why we have to encode that info into
/// the parsed data.
enum Statement {
    // Track empty/unimportant lines.
    EmptyLine,
    Comment(String),

    // ---- Shared properties between Package and PackageBase. ----
    Name(String),
    Description(String),
    URL(Url),
    Architecture(Architecture),

    // The following are the specifications of all package relations
    License(License),
    Dependencies(ArchProperty),
    OptionalDependencies(ArchProperty),
    Provides(ArchProperty),

    // ---- Package exclusive properties. ----
    // The `Clear*` prefixes are explicit values that indicate that the inherited list from
    // the PackageBase are to be ignored and set to an empty set.
    ClearLicense,
    ClearDependency,
    ClearOptionalDependency,
    ClearProvides,

    // ---- PackageBase exclusive properties. ----
    PackageVersion(Pkgver),
    PackageRelease(Pkgrel),
    ValidPgpKeys(OpenPGPv4Fingerprint),
    // These are build-time specific dependencies.
    // `makepkg` expects all dependencies for all split packages to be specified in the
    // PackageBase.
    CheckDepends(ArchProperty),
    MakeDepends(ArchProperty),

    // Sources and Checksums are highly correlated.
    // The checksums are ordered in the same way as the respective sources.
    // This will be normalized into a better representation in the next step after parsing.
    //
    // Furthermore, sources (and thereby checksums) can be architecture specific.
    Source(Url),
    B2Checksum(ArchChecksum<Blake2b512>),
    Md5Checksum(ArchChecksum<Md5>),
    Sha1Checksum(ArchChecksum<Sha1>),
    Sha256Checksum(ArchChecksum<Sha256>),
    Sha224Checksum(ArchChecksum<Sha224>),
    Sha384Checksum(ArchChecksum<Sha384>),
    Sha512Checksum(ArchChecksum<Sha512>),
}

/// Parse a given .srcinfo file.
///
/// Empty lines and comment lines are returned as `Statement::Ignored`.
/// This is to provide a proper line-based representation of the file, so we can later on provide
/// proper context in error messages during the interpretation step.
pub fn srcinfo<'s>(input: &mut &'s str) -> PResult<Vec<Statement>> {
    let (statements, _eof): (Vec<Statement>, _) = repeat_till(0.., any, eof).parse_next(input)?;

    Ok(statements)
}
