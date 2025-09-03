# alpm-lint

This project provides a binary for all linting purposes in the scope of the [ALPM project].

All lints and their respective documentation can be found on the central [ALPM lints website].

## Usage

The `alpm-lint` binary provides help output for all of its commands.
Documentation about individual lints and used options can be found on the [ALPM lints website].

### Project files

`alpm-lint` can be run directly in any package source repository or package directory with `alpm-lint check`, which will automatically run all applicable lints.

### Single files

You can check individual files with `alpm-lint check $PATH_TO_FILE`.
If the targeted file does not use its canonical name, you can explicitly specify the scope with `--scope`. For example: `alpm-lint check --scope source-info my.srcinfo`.

### Usage as a library

The `alpm-lint` project is not designed to be used as a library.
Hence, there're now backwards/forwards compatibility guarantees for internal types and APIs.
New lint rules may be added at any point in time and lint rules may be gracefully removed as well.

Exposed data such as JSON/TOML output, however, is of course considered a part of our external API and we'll uphold backwards compatibility and follow the best practices of semantic versioning.

## Example

<!--
```bash
# Create a temporary directory for testing.
export TEMP_TEST_DIR="$(mktemp --directory --suffix '.')"

# Prevent this script from crashing when alpm-lint exits with status code `1`.
set +e
```
-->

The following example takes a [SRCINFO] file with duplicate architecture declarations and performs a lint on it.

```bash
cat > "$TEMP_TEST_DIR/.SRCINFO" << EOF
pkgbase = example
    pkgver = 1.0.0
    epoch = 1
    pkgrel = 1
    arch = x86_64
    # A second identical architecture declaration to trigger a lint rule.
    arch = x86_64

pkgname = example
EOF

cd $TEMP_TEST_DIR;
alpm-lint check > $TEMP_TEST_DIR/output

cat > "$TEMP_TEST_DIR/expected" <<EOF
warning[source_info::duplicate_architecture]
  --> in field 'arch'
   |
   | Found duplicate architecture: x86_64
   |
help: Architecture lists for packages should always be unique.

      Duplicate architecture declarations such as \`arch=(x86_64 x86_64)\` are ignored.
   = alpm-architecture specification: https://alpm.archlinux.page/specifications/alpm-architecture.7.html
   = see: https://alpm.archlinux.page/lints/index.html#source_info::duplicate_architecture
EOF
```

<!--
```bash
diff --ignore-trailing-space "$TEMP_TEST_DIR/output" "$TEMP_TEST_DIR/expected"
```
-->

## Documentation

- <https://alpm.archlinux.page/lints/index.html/> specification of all lints that are used in this project.
- <https://alpm.archlinux.page/rustdoc/alpm_lint/> for development version of the crate
- <https://docs.rs/alpm-lint/latest/alpm_lint/> for released versions of the crate

## Contributing

Please refer to the [contribution guidelines] and the [architectural guide] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[ALPM project]: https://alpm.archlinux.page/
[ALPM lints website]: https://alpm.archlinux.page/lints/index.html
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[architectural guide]: ARCHITECTURE.md
[contribution guidelines]: CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
