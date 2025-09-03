# alpm-lint

This project provides a binary for all linting purposes in the scope of the ALPM project.

All lints and their respective documentation can be found on the official [ALPM lints website].

## Usage

The `alpm-lint` binary provides help texts for all of it's commands.
Documentation about individual lints and used options can be found on the 

### Project files

`alpm-lint` is designed to be simply run in either package source repositories or in package folders.
Just run `alpm-lint check` in any package directory or package source repository and it will run all applicable lints.

### Single files

Individual files can be checked via `alpm-lint check $PATH_TO_FILE`.
If a file doesn't have the canonical name for it's file, the scope can also be supplied via `--scope`:
E.g. `alpm-lint check --scope source-info my.srcinfo`

## Example

<!--
```bash
# Create a temporary directory for testing.
export MY_TEST_DIR="$(mktemp --directory --suffix '.')"
```
-->

The following example takes a **.SRCINFO** file with duplicate architecture declarations and performs a lint on it.

```bash
cat > "$MY_TEST_DIR/.SRCINFO" << EOF
pkgbase = example
    pkgver = 1.0.0
    epoch = 1
    pkgrel = 1
    arch = x86_64
    # A second identical architecture declaration to trigger a lint rule.
    arch = x86_64

pkgname = example
EOF

# Prevent this script from crashing when alpm-lint exits with status code `1`.
set +e

cd $MY_TEST_DIR;
alpm-lint check > $MY_TEST_DIR/output

cat > "$MY_TEST_DIR/expected" <<EOF
warning[source_info::duplicate_architecture]
  --> in field 'arch'
   |
   | Found duplicate architecture: x86_64
   |
help: Architecture lists for packages should always be unique.

      Duplicate architecture declarations such as \`arch=(x86_64 x86_64)\` are ignored.
   = see: https://alpm.archlinux.page/lints/index.html#source_info::duplicate_architecture
EOF

diff --ignore-trailing-space "$MY_TEST_DIR/output" "$MY_TEST_DIR/expected"
```

## Documentation

- <https://alpm.archlinux.page/lints/index.html/> specification of all lints that're used in this project.
- <https://alpm.archlinux.page/rustdoc/alpm_lint/> for development version of the crate
- <https://docs.rs/alpm-lint/latest/alpm_lint/> for released versions of the crate

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[ALPM lints website]: https://alpm.archlinux.page/lints/index.html
[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
