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

Known CPU architectures are represented by the `Architecture` enum.
You can create members e.g. from str:

```rust
use std::str::FromStr;
use alpm_types::Architecture;

assert_eq!(Architecture::from_str("aarch64"), Ok(Architecture::Aarch64));
```

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

The options available in a build environment are tracked using `BuildEnv`:

```rust
use alpm_types::BuildEnv;

let option = BuildEnv::new("foo").unwrap();
assert_eq!(option.on(), true);
assert_eq!(option.name(), "foo");
```

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

The name for a package is restricted to a specific set of characters.
You can create `Name` directly or from str, which yields a Result:

```rust
use std::str::FromStr;
use alpm_types::{Error, Name};

assert_eq!(Name::from_str("test-123@.foo_+"), Ok(Name::new("test-123@.foo_+")));
assert_eq!(Name::from_str(".foo"), Err(Error::InvalidName(".foo".to_string())));
```

The authors of packages are identified using the `Packager` type, which describes a User ID (name and valid email):

```rust
use std::str::FromStr;
use alpm_types::Packager;

let packager = Packager::new("Foobar McFooface <foobar@mcfooface.org>").unwrap();
assert_eq!("Foobar McFooface", packager.name());
assert_eq!("foobar@mcfooface.org", packager.email().to_string());
```

The options used for packaging are tracked using `PackageOption`:

```rust
use alpm_types::PackageOption;

let option = PackageOption::new("foo").unwrap();
assert_eq!(option.on(), true);
assert_eq!(option.name(), "foo");
```

Package types are distinguished using the `PkgType` enum. Its variants can be constructed from str:

```rust
use std::str::FromStr;
use alpm_types::PkgType;

assert_eq!(PkgType::from_str("pkg"), Ok(PkgType::Package));
```

Schemas of compound types (e.g. those used to describe `.BUILDINFO` or `.PKGINFO` files) need a schema version to version their features. This is what `SchemaVersion` is for:

```rust
use std::str::FromStr;
use alpm_types::SchemaVersion;

let version = SchemaVersion::new("1.0.0").unwrap();

assert_eq!("1.0.0", format!("{}", version));
```

## Contributing

Please refer to the [contribution guidelines](CONTRIBUTING.md) to learn how to
contribute to this project.

## License

This project is licensed under the terms of the
[LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.en.html).
