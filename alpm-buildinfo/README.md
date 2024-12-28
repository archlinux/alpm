# alpm-buildinfo

A library and commandline toolkit for the specification, writing and parsing of **A**rch **L**inux **P**ackage **M**anagement (ALPM) `BUILDINFO` files.

`BUILDINFO` files describe the build environment of a package and carry various datasets, that help in reproducing the same package bit-by-bit.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_buildinfo/> for development version of the crate
- <https://docs.rs/alpm-buildinfo/latest/alpm_buildinfo/> for released versions of the crate

## Examples

### Library

```rust
use std::str::FromStr;
use alpm_buildinfo::BuildInfoV2;
let buildinfo_data = r#"format = 2
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
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
"#;

assert!(BuildInfoV2::from_str(buildinfo_data).is_ok());
```

### Commandline

<!--
```bash
# use a custom, temporary directory for all generated files
test_tmpdir="$(mktemp --directory --suffix '.buildinfo-test')"
# set a custom, temporary output file location
BUILDINFO_OUTPUT_FILE="$(mktemp --tmpdir="$test_tmpdir" --suffix '-BUILDINFO' --dry-run)"
export BUILDINFO_OUTPUT_FILE
```
-->

Create a BUILDINFO version 1 file using `alpm-buildinfo`:

```bash
alpm-buildinfo create v2 \
    --builddate 1 \
    --builddir /build \
    --buildenv env \
    --buildenv '!otherenv' \
    --installed 'bar-1:1.0.1-15-any' \
    --installed 'beh-2.3-1-any' \
    --options something \
    --options '!else' \
    --packager 'Foobar McFooface <foobar@mcfooface.org>' \
    --pkgarch any \
    --pkgbase foo \
    --pkgbuild-sha256sum b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c \
    --pkgname foo \
    --pkgver 1.0.0-1 \
    --startdir /startdir/ \
    --buildtool devtools \
    --buildtoolver '1:1.2.1-1-any'
```

<!--

Asserts the contents of .BUILDINFO that is created above:

```bash
# set a custom, temporary file location for the expected output
BUILDINFO_OUTPUT_FILE_EXPECTED="$(mktemp --tmpdir="$test_tmpdir" --suffix '-BUILDINFO.expected' --dry-run)"

cat > "$BUILDINFO_OUTPUT_FILE_EXPECTED" <<EOF
format = 2
pkgname = foo
pkgbase = foo
pkgver = 1.0.0-1
pkgarch = any
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
packager = Foobar McFooface <foobar@mcfooface.org>
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = env
buildenv = !otherenv
options = something
options = !else
installed = bar-1:1.0.1-15-any
installed = beh-2.3-1-any
EOF

diff --ignore-trailing-space "$BUILDINFO_OUTPUT_FILE" "$BUILDINFO_OUTPUT_FILE_EXPECTED"
```

-->

All options for `alpm-buildinfo` can also be provided as environment variables. The following is equivalent to the above:

```bash
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
BUILDINFO_STARTDIR="/startdir/" \
BUILDINFO_BUILDTOOL="devtools" \
BUILDINFO_BUILDTOOLVER="1:1.2.1-1-any" \
alpm-buildinfo create v2
```

<!--

Asserts the contents of .BUILDINFO that is created above:

```bash
cat > "$BUILDINFO_OUTPUT_FILE_EXPECTED" <<EOF
format = 2
pkgname = foo
pkgbase = foo
pkgver = 1.0.0-1
pkgarch = any
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
packager = Foobar McFooface <foobar@mcfooface.org>
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
buildenv = env
buildenv = !otherenv
options = something
options = !else
installed = bar-1:1.0.1-15-any
installed = beh-2.3-1-any
EOF

diff --ignore-trailing-space "$BUILDINFO_OUTPUT_FILE" "$BUILDINFO_OUTPUT_FILE_EXPECTED"
rm -r -- "$test_tmpdir"
```

-->

## Features

- `winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
