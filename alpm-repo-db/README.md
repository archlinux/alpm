# alpm-repo-db

A library and command line tool for handling **A**rch **L**inux **P**ackage **M**anagement (ALPM)
repository sync databases

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_repo_db/> for development version of the crate.
- <https://docs.rs/alpm-repo/latest/alpm_repo_db/> for released version of the crate.

## Examples

### Library

```rust
use std::{path::PathBuf, str::FromStr};

use alpm_repo_db::files::{RepoFiles, RepoFilesV1};

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

// Create a RepoFiles from a string.
let files_from_str = RepoFiles::V1(RepoFilesV1::from_str(data)?);

// Create a RepoFiles from list of paths.
let files_from_paths = RepoFiles::V1(RepoFilesV1::try_from(paths)?);

assert_eq!(files_from_str.as_ref(), files_from_paths.as_ref());
# Ok(())
# }
```

### Command line interface

<!--
```bash
# Create a temporary directory for test files.
test_tempdir="$(mktemp --directory --suffix '.alpm-repo-files-test')"

# Create input dir for `alpm-repo-files create`.
ALPM_REPO_FILES_CREATE_INPUT_DIR="${test_tempdir}/input_dir/"
mkdir --parents "${ALPM_REPO_FILES_CREATE_INPUT_DIR}usr/bin/"
touch "${ALPM_REPO_FILES_CREATE_INPUT_DIR}usr/bin/foo"
export ALPM_REPO_FILES_CREATE_INPUT_DIR

# Create input file for `alpm-repo-files format` and `alpm-repo-files validate`.
input_file="${test_tempdir}/input.files"
printf "%%FILES%%\nusr/\nusr/bin/\nusr/bin/foo\n" > "$input_file"
export ALPM_REPO_FILES_FORMAT_INPUT_FILE="$input_file"
export ALPM_REPO_FILES_VALIDATE_INPUT_FILE="$input_file"

# Create comparison file for `alpm-repo-files format`.
alpm_repo_files_format_output_compare="${test_tempdir}/format-output-compare.json"
printf '[\n  "usr/",\n  "usr/bin/",\n  "usr/bin/foo"\n]\n' > "$alpm_repo_files_format_output_compare"

# Assign output files for `alpm-repo-files create` and `alpm-repo-files format`.
export ALPM_REPO_FILES_CREATE_OUTPUT="${test_tempdir}/create-output.files"
export ALPM_REPO_FILES_FORMAT_OUTPUT="${test_tempdir}/format-output.json"
```
-->

```bash
# Create an alpm-repo-files file from an input directory.
alpm-repo-files create "$ALPM_REPO_FILES_CREATE_INPUT_DIR"
```

<!--
```bash
cat "$ALPM_REPO_FILES_CREATE_OUTPUT"
diff "$input_file" "$ALPM_REPO_FILES_CREATE_OUTPUT"
```
-->

```bash
# Format an alpm-repo-files file as JSON.
alpm-repo-files format --input-file "$ALPM_REPO_FILES_FORMAT_INPUT_FILE" --pretty
```

<!--
```bash
cat "$ALPM_REPO_FILES_FORMAT_OUTPUT"
diff "$alpm_repo_files_format_output_compare" "$ALPM_REPO_FILES_FORMAT_OUTPUT"
```
-->

```bash
# Validate an alpm-repo-files file.
alpm-repo-files validate --input-file "$ALPM_REPO_FILES_VALIDATE_INPUT_FILE"
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
[alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
[contribution guidelines]: ../CONTRIBUTING.md
