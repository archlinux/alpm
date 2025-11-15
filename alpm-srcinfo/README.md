# alpm-srcinfo

A library and command line tool for the specification, parsing and linting of **A**rch **L**inux **P**ackage **M**anagement (ALPM) [SRCINFO] files.

`SRCINFO` files describe a `PKGBUILD` file in a way that doesn't require an interactive shell to evaluate it.

## Documentation

- <https://alpm.archlinux.page/alpm-srcinfo/ARCHITECTURE.html> A high-level overview on how this project works.
- <https://alpm.archlinux.page/rustdoc/alpm_srcinfo/> for development version of the crate
- <https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/> for released versions of the crate

## Examples

### Commandline

#### Inspect SRCINFO packages

<!--
```bash
# Create a temporary directory for testing.
test_tmpdir="$(mktemp --directory --suffix '.')"
# Get a random temporary file location in the created temporary directory.
SRCINFO_TEMPFILE="$(mktemp --tmpdir="$test_tmpdir" --suffix '-SRCINFO' --dry-run)"
SRCINFO_OUTPUT="$(mktemp --tmpdir="$test_tmpdir" --suffix '-SRCINFO' --dry-run)"
export SRCINFO_TEMPFILE
export SRCINFO_OUTPUT
```
-->

The following command takes a **.SRCINFO** file and outputs the merged and compiled details of all (split-)packages for a specific architecture as structured data.

```bash
cat > "$SRCINFO_TEMPFILE" << EOF
pkgbase = example
    pkgver = 1.0.0
    epoch = 1
    pkgrel = 1
    pkgdesc = A project that does something
    url = https://example.org/
    arch = x86_64
    depends = glibc
    optdepends = python: for special-python-script.py
    makedepends = cmake
    checkdepends = extra-test-tool

pkgname = example
    depends = glibc
    depends = gcc-libs
EOF

alpm-srcinfo format-packages "$SRCINFO_TEMPFILE" --architecture x86_64 --pretty > "$SRCINFO_OUTPUT"
```

<!--

Asserts that the generated JSON output is correct:

```bash
# Get a tempfile

cat > "$SRCINFO_OUTPUT.expected" <<EOF
[
  {
    "name": "example",
    "description": "A project that does something",
    "url": "https://example.org/",
    "licenses": [],
    "architecture": "x86_64",
    "changelog": null,
    "install": null,
    "groups": [],
    "options": [],
    "backups": [],
    "version": {
      "pkgver": "1.0.0",
      "pkgrel": {
        "major": 1,
        "minor": null
      },
      "epoch": 1
    },
    "pgp_fingerprints": [],
    "dependencies": [
      {
        "name": "glibc",
        "version_requirement": null
      },
      {
        "name": "gcc-libs",
        "version_requirement": null
      }
    ],
    "optional_dependencies": [
      {
        "package_relation": {
          "name": "python",
          "version_requirement": null
        },
        "description": "for special-python-script.py"
      }
    ],
    "provides": [],
    "conflicts": [],
    "replaces": [],
    "check_dependencies": [
      {
        "name": "extra-test-tool",
        "version_requirement": null
      }
    ],
    "make_dependencies": [
      {
        "name": "cmake",
        "version_requirement": null
      }
    ],
    "sources": [],
    "no_extracts": []
  }
]
EOF

diff --ignore-trailing-space "$SRCINFO_OUTPUT" "$SRCINFO_OUTPUT.expected"
```
-->

#### PKGBUILD to SRCINFO conversion

<!--
```bash
# Create a temporary directory for testing.
test_tmpdir="$(mktemp --directory --suffix '.')"

# Get a random temporary file location in the created temporary directory.
PKGBUILD_IN="$test_tmpdir/PKGBUILD"
SRCINFO_OUT="$test_tmpdir/SRCINFO"
export PKGBUILD_IN
export SRCINFO_OUT

cp tests/unit_test_files/normal.pkgbuild "$PKGBUILD_IN"
```
-->

The following command takes a **PKGBUILD** file and outputs a **.SRCINFO** from the extracted metadata.

```bash
alpm-srcinfo create "$PKGBUILD_IN" > "$SRCINFO_OUT"
```

<!--
Make sure the generated SRCINFO file is as expected.
```bash
cat > "$SRCINFO_OUT.expected" <<EOF
pkgbase = example
	pkgdesc = A example with all pkgbase properties set.
	pkgver = 0.1.0
	pkgrel = 1
	epoch = 1
	url = https://archlinux.org/
	install = install.sh
	changelog = changelog
	arch = x86_64
	arch = aarch64
	groups = group
	groups = group_2
	license = MIT
	depends = default_dep
	optdepends = default_optdep
	provides = default_provides
	conflicts = default_conflict
	replaces = default_replaces
	options = !lto
	backup = etc/pacman.conf
	provides_x86_64 = arch_default_provides
	conflicts_x86_64 = arch_default_conflict
	depends_x86_64 = arch_default_dep
	replaces_x86_64 = arch_default_replaces
	optdepends_x86_64 = arch_default_optdep

pkgname = example
EOF

diff --ignore-trailing-space "$SRCINFO_OUT" "$SRCINFO_OUT.expected"
```
-->

### Library

```rust
use alpm_srcinfo::{SourceInfoV1, MergedPackage};
use alpm_types::{SystemArchitecture, PackageRelation, Name};

# fn main() -> Result<(), alpm_srcinfo::Error> {
let source_info_data = r#"
pkgbase = example
    pkgver = 1.0.0
    epoch = 1
    pkgrel = 1
    pkgdesc = A project that does something
    url = https://example.org/
    arch = x86_64
    depends = glibc
    optdepends = python: for special-python-script.py
    makedepends = cmake
    checkdepends = extra-test-tool

pkgname = example
    depends = glibc
    depends = gcc-libs
"#;

// Parse the file. This errors if the file cannot be parsed, is missing data or contains invalid data.
let source_info = SourceInfoV1::from_string(source_info_data)?;

// Get all merged package representations for the x86_64 architecture.
let mut packages: Vec<MergedPackage> = source_info.packages_for_architecture(SystemArchitecture::X86_64).collect();
let package = packages.remove(0);

assert_eq!(package.name, Name::new("example")?);
assert_eq!(package.architecture, SystemArchitecture::X86_64.into());
assert_eq!(package.dependencies, vec![
    PackageRelation::new(Name::new("glibc")?, None),
    PackageRelation::new(Name::new("gcc-libs")?, None)
]);

# Ok(())
# }
```

## Features

- `cli` adds the commandline handling needed for the `alpm-srcinfo` binary.
- `_winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
