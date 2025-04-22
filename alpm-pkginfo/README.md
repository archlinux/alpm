# alpm-pkginfo

A library and commandline toolkit for the specification, writing and parsing of `PKGINFO` files used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

`PKGINFO` files cover a package's metadata and carry various datasets that specify how a package is used in the context of a distribution.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkginfo/> for development version of the crate
- <https://docs.rs/alpm-pkginfo/latest/alpm_pkginfo/> for released versions of the crate

## Examples

### Library

Create a PKGINFO version 2 file:

```rust
use std::str::FromStr;
use alpm_pkginfo::PackageInfoV2;
let pkginfo_data = r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
"#;
assert!(PackageInfoV2::from_str(pkginfo_data).is_ok());
```

Create a PKGINFO version 1 file:

```rust
use std::str::FromStr;
use alpm_pkginfo::PackageInfoV1;
let pkginfo_data = r#"
pkgname = example
pkgbase = example
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
"#;
assert!(PackageInfoV1::from_str(pkginfo_data).is_ok());
```

### Commandline

Create a PKGINFO version 2 file using `alpm-pkginfo`:

<!--
```bash
# use a custom, temporary directory for all generated files
test_tmpdir="$(mktemp --directory --suffix '.pkginfo-test')"
# set a custom, temporary output file location
PKGINFO_OUTPUT_FILE="$(mktemp --tmpdir="$test_tmpdir" --suffix '-PKGINFO' --dry-run)"
export PKGINFO_OUTPUT_FILE
```
-->

```bash
alpm-pkginfo create v2 \
  --pkgname "example" \
  --pkgbase "example" \
  --xdata "pkgtype=pkg" \
  --pkgver "1:1.0.0-1" \
  --pkgdesc "A project that does something" \
  --url "https://example.org/" \
  --builddate "1729181726" \
  --packager "John Doe <john@example.org>" \
  --size "181849963" \
  --arch "any" \
  --license "GPL-3.0-or-later" \
  --license "LGPL-3.0-or-later" \
  --replaces "other-package>0.9.0-3" \
  --group "package-group" \
  --group "other-package-group" \
  --conflict "conflicting-package<1.0.0" \
  --conflict "other-conflicting-package<1.0.0" \
  --provides "some-component" \
  --provides "some-other-component=1:1.0.0-1" \
  --backup "etc/example/config.toml" \
  --backup "etc/example/other-config.txt" \
  --depend "glibc" \
  --depend "gcc-libs" \
  --optdepend "python: for special-python-script.py" \
  --optdepend "ruby: for special-ruby-script.rb" \
  --makedepend "cmake" \
  --makedepend "python-sphinx" \
  --checkdepend "extra-test-tool" \
  --checkdepend "other-extra-test-tool"
```

<!--

Asserts the contents of .PKGINFO that is created above:

```bash
# set a custom, temporary file location for the expected output
PKGINFO_OUTPUT_FILE_EXPECTED="$(mktemp --tmpdir="$test_tmpdir" --suffix '-PKGINFO.expected' --dry-run)"

cat > "$PKGINFO_OUTPUT_FILE_EXPECTED" <<EOF
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
EOF

diff --ignore-trailing-space "$PKGINFO_OUTPUT_FILE" "$PKGINFO_OUTPUT_FILE_EXPECTED"
```
-->

All options for `alpm-pkginfo` can also be provided as environment variables. The following is equivalent to the above:

```bash
PKGINFO_PKGNAME="example" \
PKGINFO_PKGBASE="example" \
PKGINFO_XDATA="pkgtype=pkg" \
PKGINFO_PKGVER="1:1.0.0-1" \
PKGINFO_PKGDESC="A project that does something" \
PKGINFO_URL="https://example.org/" \
PKGINFO_BUILDDATE="1729181726" \
PKGINFO_PACKAGER="John Doe <john@example.org>" \
PKGINFO_SIZE="181849963" \
PKGINFO_ARCH="any" \
PKGINFO_LICENSE="GPL-3.0-or-later LGPL-3.0-or-later" \
PKGINFO_REPLACES="other-package>0.9.0-3" \
PKGINFO_GROUP="package-group other-package-group" \
PKGINFO_CONFLICT="conflicting-package<1.0.0 other-conflicting-package<1.0.0" \
PKGINFO_PROVIDES="some-component some-other-component=1:1.0.0-1" \
PKGINFO_BACKUP="etc/example/config.toml etc/example/other-config.txt" \
PKGINFO_DEPEND="glibc gcc-libs" \
PKGINFO_OPTDEPEND="python: for special-python-script.py,ruby: for special-ruby-script.rb" \
PKGINFO_MAKEDEPEND="cmake python-sphinx" \
PKGINFO_CHECKDEPEND="extra-test-tool other-extra-test-tool" \
alpm-pkginfo create v2
```

<!--

Asserts the contents of .PKGINFO that is created above:

```bash
cat > "$PKGINFO_OUTPUT_FILE_EXPECTED" <<EOF
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org/
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
EOF

diff --ignore-trailing-space "$PKGINFO_OUTPUT_FILE" "$PKGINFO_OUTPUT_FILE_EXPECTED"
rm -r -- "$test_tmpdir"
```
-->

## Features

- `cli` adds the commandline handling needed for the `alpm-pkginfo` binary (enabled by default).
- `winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
