# alpm-pkginfo

A library and commandline toolkit for the specification, writing and parsing of **A**rch **L**inux **P**ackage **M**anagement (ALPM) `PKGINFO` files.

`PKGINFO` files describe the build environment of a package and carry various datasets, that help in reproducing the same package bit-by-bit.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkginfo/> for development version of the crate
- <https://docs.rs/alpm-pkginfo/latest/alpm_pkginfo/> for released versions of the crate

## Examples

### Library

```rust
use std::str::FromStr;
use alpm_pkginfo::PkgInfoV1;
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
assert!(PkgInfoV1::from_str(pkginfo_data).is_ok());
```

### Commandline

Create a PKGINFO version 1 file using `alpm-pkginfo`:

```shell
alpm-pkginfo create v1 \
  --pkgname "example" \
  --pkgbase "example" \
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

All options for `alpm-pkginfo` can also be provided as environment variables. The following is equivalent to the above:

```shell
PKGINFO_PKGNAME="example" \
PKGINFO_PKGBASE="example" \
PKGINFO_PKGVER="1:1.0.0-1" \
PKGINFO_PKGDESC="A project that does something" \
PKGINFO_URL="https://example.org/" \
PKGINFO_BUILDDATE="1729181726" \
PKGINFO_PACKAGER="John Doe <john@example.org>" \
PKGINFO_SIZE="181849963" \
PKGINFO_ARCH="any" \
PKGINFO_LICENSE="GPL-3.0-or-later" \
PKGINFO_LICENSE="LGPL-3.0-or-later" \
PKGINFO_REPLACES="other-package>0.9.0-3" \
PKGINFO_GROUP="package-group" \
PKGINFO_GROUP="other-package-group" \
PKGINFO_CONFLICT="conflicting-package<1.0.0" \
PKGINFO_CONFLICT="other-conflicting-package<1.0.0" \
PKGINFO_PROVIDES="some-component" \
PKGINFO_PROVIDES="some-other-component=1:1.0.0-1" \
PKGINFO_BACKUP="etc/example/config.toml" \
PKGINFO_BACKUP="etc/example/other-config.txt" \
PKGINFO_DEPEND="glibc" \
PKGINFO_DEPEND="gcc-libs" \
PKGINFO_OPTDEPEND="python: for special-python-script.py" \
PKGINFO_OPTDEPEND="ruby: for special-ruby-script.rb" \
PKGINFO_MAKEDEPEND="cmake" \
PKGINFO_MAKEDEPEND="python-sphinx" \
PKGINFO_CHECKDEPEND="extra-test-tool" \
PKGINFO_CHECKDEPEND="other-extra-test-tool" \
alpm-pkginfo create v1
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
