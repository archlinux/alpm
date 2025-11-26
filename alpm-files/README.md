# alpm-files

A specification, library and command line tool for the handling of **A**rch **L**inux **P**ackage **M**anagement (ALPM) `files` files ([alpm-files]).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_files/> for development version of the crate.
- <https://docs.rs/alpm-files/latest/alpm_files/> for released version of the crate.

## Examples

### Library

```rust
use std::{path::PathBuf, str::FromStr};

use alpm_files::files::{Files, FilesV1};

# fn main() -> testresult::TestResult {
let data = r#"%FILES%
usr/
usr/bin/
usr/bin/foo
"#;
let paths = vec![
  PathBuf::from("usr/"),
  PathBuf::from("usr/bin/"),
  PathBuf::from("usr/bin/foo"),
];

// Create a Files from a string.
let files_from_str = Files::V1(FilesV1::from_str(data)?);

// Create a Files from list of paths.
let files_from_paths = Files::V1(FilesV1::try_from(paths)?);

assert_eq!(files_from_str.as_ref(), files_from_paths.as_ref());
# Ok(())
# }
```

### Command line interface

<!--
```bash
# Create a temporary directory for test files.
test_tempdir="$(mktemp --directory --suffix '.alpm-files-test')"

# Create input dir for `alpm-files create`.
ALPM_FILES_CREATE_INPUT_DIR="${test_tempdir}/input_dir/"
mkdir --parents "${ALPM_FILES_CREATE_INPUT_DIR}usr/bin/"
touch "${ALPM_FILES_CREATE_INPUT_DIR}usr/bin/foo"
export ALPM_FILES_CREATE_INPUT_DIR

# Create input file for `alpm-files format` and `alpm-files validate`.
input_file="${test_tempdir}/input.files"
printf "%%FILES%%\nusr/\nusr/bin/\nusr/bin/foo\n\n" > "$input_file"
export ALPM_FILES_FORMAT_INPUT_FILE="$input_file"
export ALPM_FILES_VALIDATE_INPUT_FILE="$input_file"

# Create comparison file for `alpm-files format`.
alpm_files_format_output_compare="${test_tempdir}/format-output-compare.json"
printf '[\n  "usr/",\n  "usr/bin/",\n  "usr/bin/foo"\n]\n' > "$alpm_files_format_output_compare"

# Assign output files for `alpm-files create` and `alpm-files format`.
export ALPM_FILES_CREATE_OUTPUT="${test_tempdir}/create-output.files"
export ALPM_FILES_FORMAT_OUTPUT="${test_tempdir}/format-output.json"
```
-->

```bash
# Create an alpm-files file from an input directory.
alpm-files create "$ALPM_FILES_CREATE_INPUT_DIR"
```

<!--
```bash
cat "$ALPM_FILES_CREATE_OUTPUT"
diff "$input_file" "$ALPM_FILES_CREATE_OUTPUT"
```
-->

```bash
# Format an alpm-files file as JSON.
alpm-files format --input-file "$ALPM_FILES_FORMAT_INPUT_FILE" --pretty
```

<!--
```bash
cat "$ALPM_FILES_FORMAT_OUTPUT"
diff "$alpm_files_format_output_compare" "$ALPM_FILES_FORMAT_OUTPUT"
```
-->

```bash
# Validate an alpm-files file.
alpm-files validate --input-file "$ALPM_FILES_VALIDATE_INPUT_FILE"
```

<!--
```bash
rm -r -- "$test_tempdir"
```
-->

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
[contribution guidelines]: ../CONTRIBUTING.md
