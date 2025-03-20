# alpm-srcinfo

A library and command line tool for the specification, parsing and linting of **A**rch **L**inux **P**ackage **M**anagement (ALPM) [SRCINFO] files.

`SRCINFO` files describe a `PKGBUILD` file in a way that doesn't require an interactive shell to evaluate it.

## Documentation

- <https://alpm.archlinux.page/alpm-srcinfo/ARCHITECTURE.html> A high-level overview on how this project works.
- <https://alpm.archlinux.page/rustdoc/alpm_srcinfo/> for development version of the crate
- <https://docs.rs/alpm-srcinfo/latest/alpm_srcinfo/> for released versions of the crate

## Examples

### Commandline

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
    "architecture": "X86_64",
    "changelog": null,
    "install": null,
    "groups": [],
    "options": [],
    "backups": [],
    "package_version": "1.0.0",
    "package_release": "1",
    "epoch": 1,
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

### Library

```rust
use alpm_srcinfo::{SourceInfo, MergedPackage};
use alpm_types::{Architecture, PackageRelation, Name};

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

// Parse the file. This might already error if the file cannot be parsed.
let source_info_result = SourceInfo::from_string(source_info_data)?;

// Make sure there're aren't unrecoverable logic errors, such as missing values.
// Recoverable errors would be lints and deprecation warnings.
let source_info = source_info_result.source_info()?;

// Get all merged package representations for the x86_64 architecture.
let mut packages: Vec<MergedPackage> = source_info.packages_for_architecture(Architecture::X86_64).collect();
let package = packages.remove(0);

assert_eq!(package.name, Name::new("example")?);
assert_eq!(package.architecture, Architecture::X86_64);
assert_eq!(package.dependencies, vec![
    PackageRelation::new(Name::new("glibc")?, None),
    PackageRelation::new(Name::new("gcc-libs")?, None)
]);

# Ok(())
# }
```

## Features

- `cli` adds the commandline handling needed for the `almp-srcinfo` binary (enabled by default).
- `winnow-debug` enables the `winnow/debug` feature, which shows the exact parsing process of winnow.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
