<!--
SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# alpm-types

Types for **A**rch **L**inux **P**ackage **M**anagement.

The provided types and the traits they implement can be used in package
management related applications (e.g. package manager, repository manager,
special purpose parsers and file specifications, etc.) which deal with
[libalpm](https://man.archlinux.org/man/libalpm.3) based packages.

This library strives to provide all underlying types for writing ALPM based
software as a leaf-crate, so that they can be shared across applications and
none of them has to implement them itself.

## Examples

### System

Known CPU architectures are represented by the `Architecture` enum.
You can create members e.g. from str:

```rust
use std::str::FromStr;
use alpm_types::Architecture;

assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
```

### Checksum

Checksums are implemented generically for a set of supported algorithms:
- `Blake2b512`
- `Md5` (**WARNING**: Use of this algorithm is highly discouraged, because it is cryptographically unsafe)
- `Sha1` (**WARNING**: Use of this algorithm is highly discouraged, because it is cryptographically unsafe)
- `Sha224`
- `Sha256`
- `Sha384`
- `Sha512`

**NOTE**: Contrary to makepkg/pacman, this crate *does not* support using cksum-style CRC-32 as it is non-standard (different implementations throughout libraries) and cryptographically unsafe.

The above algorithms are reexported in the `digests` module of this crate, so that users do not have to add the [blake2](https://crates.io/crates/blake2), [md-5](https://crates.io/crates/md-5), [sha1](https://crates.io/crates/sha1), or [sha2](https://crates.io/crates/sha2) crates themselves and can solely rely on `alpm-types`.

```rust
use std::str::FromStr;
use alpm_types::{digests::Blake2b512, Checksum};

let checksum = Checksum::<Blake2b512>::calculate_from("foo\n");

let digest = vec![
    210, 2, 215, 149, 29, 242, 196, 183, 17, 202, 68, 180, 188, 201, 215, 179, 99, 250, 66,
    82, 18, 126, 5, 140, 26, 145, 14, 192, 91, 108, 208, 56, 215, 28, 194, 18, 33, 192, 49,
    192, 53, 159, 153, 62, 116, 107, 7, 245, 150, 92, 248, 197, 195, 116, 106, 88, 51, 122,
    217, 171, 101, 39, 142, 119,
];
assert_eq!(checksum.inner(), digest);
assert_eq!(
    format!("{}", checksum),
    "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77",
);

// create checksum from hex string
let checksum = Checksum::<Blake2b512>::from_str("d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77").unwrap();
assert_eq!(checksum.inner(), digest);
```

### Date

The date when a package has been built is represented using the `BuildDate`
struct, which tracks this in seconds since the epoch.

Apart from creating BuildDate from i64 or str, it can also be created from
`DateTime<Utc>`:

```rust
use chrono::{DateTime, NaiveDateTime, Utc};
use alpm_types::BuildDate;

let datetime: BuildDate = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(1, 0).unwrap(), Utc).into();
assert_eq!(BuildDate::new(1), datetime);
```

### Env

The options available in a build environment are tracked using `BuildEnv`:

```rust
use alpm_types::BuildEnv;

let option = BuildEnv::new("foo").unwrap();
assert_eq!(option.on(), true);
assert_eq!(option.name(), "foo");
```

A package installed to an environment can be described using `Installed`:

```rust
use alpm_types::Installed;

assert!(Installed::new("foo-1:1.0.0-1-any").is_ok());
assert!(Installed::new("foo-1:1.0.0-1-foo").is_err());
assert!(Installed::new("foo-1:1.0.0-any").is_err());
```

The options used for packaging are tracked using `PackageOption`:

```rust
use alpm_types::PackageOption;

let option = PackageOption::new("foo").unwrap();
assert_eq!(option.on(), true);
assert_eq!(option.name(), "foo");
```

### Size

The compressed size of a package is represented by `CompressedSize` which
tracks the size in bytes and can also be created from str:

```rust
use alpm_types::CompressedSize;
use std::str::FromStr;

assert_eq!(CompressedSize::from_str("1"), Ok(CompressedSize::new(1)));
```

The installed size of a package is represented by `InstalledSize` which
tracks the size in bytes and can also be created from str:

```rust
use alpm_types::InstalledSize;
use std::str::FromStr;

assert_eq!(InstalledSize::from_str("1"), Ok(InstalledSize::new(1)));
```

### Name

The name for a package is restricted to a specific set of characters.
You can create `Name` directly or from str, which yields a Result:

```rust
use std::str::FromStr;
use alpm_types::{Error, Name};

assert_eq!(Name::from_str("test-123@.foo_+"), Name::new("test-123@.foo_+".to_string()));
assert_eq!(Name::from_str(".foo"), Err(Error::InvalidName(".foo".to_string())));
```

### Path

The build directory of a package build environment can be described using `BuildDir`:

```rust
use alpm_types::BuildDir;

let builddir = BuildDir::new("/build").unwrap();
assert_eq!("/build", format!("{}", builddir));
```

The start directory of a package build environment can be described using `StartDir`:

```rust
use alpm_types::StartDir;

let startdir = StartDir::new("/start").unwrap();
assert_eq!("/start", format!("{}", startdir));
```

### Pkg

The authors of packages are identified using the `Packager` type, which describes a User ID (name and valid email):

```rust
use std::str::FromStr;
use alpm_types::Packager;

let packager = Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap();
assert_eq!("Foobar McFooface", packager.name());
assert_eq!("foobar@mcfooface.org", packager.email().to_string());
```

Package types are distinguished using the `PkgType` enum. Its variants can be constructed from str:

```rust
use std::str::FromStr;
use alpm_types::PkgType;

assert_eq!(PkgType::from_str("pkg"), Ok(PkgType::Package));
```

### Version

The version and CPU architecture of a build tool is tracked using `BuildToolVer`:

```rust
use alpm_types::BuildToolVer;

let buildtoolver = BuildToolVer::new("1.0.0-1-any").unwrap();

assert_eq!("1.0.0-1-any", format!("{}", buildtoolver));
```

Schemas of compound types (e.g. those used to describe `.BUILDINFO` or `.PKGINFO` files) need a schema version to version their features. This is what `SchemaVersion` is for:

```rust
use std::str::FromStr;
use alpm_types::SchemaVersion;

let version = SchemaVersion::new("1.0.0").unwrap();

assert_eq!("1.0.0", format!("{}", version));
```

The handling of package versions is covered by the `Version` type (which consists of an optional `Epoch`, `Pkgver` and an optional `Pkgrel`).
Its `vercmp()` method implementation is compatible with that of libalpm/pacman's `vercmp`.

```rust
use std::str::FromStr;
use alpm_types::Version;

let version = Version::new("1.0.0").unwrap();

assert_eq!("1.0.0", format!("{}", version));

let version_a = Version::new("1.0.0").unwrap();
let version_b = Version::new("1.1.0").unwrap();

assert_eq!(Version::vercmp(&version_a, &version_b), -1);

// create a Version that is guaranteed to have a Pkgrel
assert!(Version::with_pkgrel("1.0.0-1").is_ok());
assert!(Version::with_pkgrel("1.0.0").is_err());
```

## Contributing

Please refer to the [contribution guidelines](CONTRIBUTING.md) to learn how to
contribute to this project.

## License

This project is licensed under the terms of the
[LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.en.html).
