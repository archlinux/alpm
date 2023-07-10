<!--
SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
SPDX-License-Identifier: GFDL-1.3-or-later
-->

# alpm-buildinfo

A library and commandline toolkit for the specification, writing and parsing of **A**rch **L**inux **P**ackage **M**anagement (ALPM) `BUILDINFO` files.

`BUILDINFO` files describe the build environment of a package and carry various datasets, that help in reproducing the same package bit-by-bit.

## Documentation

* https://alpm-buildinfo.archlinux.page/alpm_buildinfo/ for development version of the crate
* https://docs.rs/alpm-buildinfo/latest/alpm_buildinfo/ for released versions of the crate

## Examples

### Library

```rust
use std::str::FromStr;
use alpm_buildinfo::BuildInfoV1;
let buildinfo_data = r#"format = 1
pkgname = foo
pkgbase = foo
pkgver = 1:1.0.0-1
pkgarch = any
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
packager = Foobar McFooface <foobar@mcfooface.org>
builddate = 1
builddir = /build
buildenv = envfoo
buildenv = envbar
options = some_option
options = !other_option
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
"#;

assert!(BuildInfoV1::from_str(buildinfo_data).is_ok());
```

### Commandline

Create a BUILDINFO version 1 file using `alpm-buildinfo`:

```shell
alpm-buildinfo create v1 \
    --builddate 1 \
    --builddir /build \
    --buildenv env \
    --buildenv '!otherenv' \
    --installed 'bar-1:1.0.1-15-any' \
    --installed 'beh-2.3-1-any' \
    --options something \
    --options '\!else' \
    --packager 'Foobar McFooface <foobar@mcfooface.org>' \
    --pkgarch any \
    --pkgbase foo \
    --pkgbuild-sha256sum b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c \
    --pkgname foo \
    --pkgver 1.0.0-1 \
```

All options for `alpm-buildinfo` can also be provided as environment variables. The following is equivalent to the above:

```shell
BUILDINFO_BUILDDATE="1" \
BUILDINFO_BUILDDIR="/build" \
BUILDINFO_BUILDENV='env !otherenv' \
BUILDINFO_INSTALLED="bar-1:1.0.1-15-any beh-2.3-1-any" \
BUILDINFO_OPTIONS='something !else' \
BUILDINFO_PACKAGER="Foobar McFooface <foobar@mcfooface.org>" \
BUILDINFO_PKGARCH="any" \
BUILDINFO_PKGBASE="foo" \
BUILDINFO_PKGBUILD_SHA256SUM="b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c" \
BUILDINFO_PKGNAME="foo" \
BUILDINFO_PKGVER="1.0.0-1" \
alpm-buildinfo create v1
```

## Contributing

Please refer to the [contribution guidelines](CONTRIBUTING.md) to learn how to contribute to this project.

## License

The commandline toolkit (`alpm-buildinfo`) of this project is licensed under the terms of the [GPL-3.0-or-later](https://www.gnu.org/licenses/gpl-3.0.en.html).

The library of this project is licensed under the terms of the [LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.en.html).
