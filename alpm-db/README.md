# alpm-db

A library and command line interface for [alpm-db] structures used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_db/> for development version of the crate.
- <https://docs.rs/alpm-db-desc/latest/alpm_db/> for released version of the crate.

## Overview

The `alpm-db` crate provides modules and binaries for working with several components of an [alpm-db]:

- The `desc` module allows writing and parsing of [alpm-db-desc] files, which describe the metadata of an installed package.
  The `alpm-db-desc` CLI can create, format, and validate these files.
- The `files` module allows writing and parsing of [alpm-db-files] files, which provide file listings and information on files considered for backup of an installed package.
  The `alpm-db-files` CLI can create, format, and validate these files.
- The `db` module manages [alpm-db] directories (reading, writing and iterating entries stored under paths such as `/var/lib/pacman/local`).

## Examples

### Library

#### List entries in a database

```rust,no_run
use alpm_db::db::Database;

# fn main() -> Result<(), alpm_db::Error> {
let db = Database::open("/var/lib/pacman/local")?;
for entry in db.entries()? {
    println!("{:?}", entry.name.as_path_buf());
}
# Ok(())
# }
```

New entries can be produced via `DatabaseEntry::new` and stored on disk with
`Database::create_entry`.

#### Handle alpm-db-desc files programmatically

Parsing [alpm-db-descv1] files:

```rust
use std::str::FromStr;
use alpm_db::desc::DbDescFileV1;

# fn main() -> Result<(), alpm_db::Error> {
let desc_data = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%VALIDATION%
sha256
pgp

"#;

let desc = DbDescFileV1::from_str(desc_data)?;
assert_eq!(desc.name.to_string(), "foo");
assert_eq!(desc.arch.to_string(), "x86_64");
# Ok(())
# }
```

Parsing [alpm-db-descv2] files:

```rust
use std::str::FromStr;
use alpm_db::desc::DbDescFileV2;

# fn main() -> Result<(), alpm_db::Error> {
let desc_data = r#"%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%VALIDATION%
sha256
pgp

%XDATA%
pkgtype = pkg
key2 = value2
"#;

let desc = DbDescFileV2::from_str(desc_data)?;
assert_eq!(desc.name.to_string(), "foo");
assert_eq!(desc.arch.to_string(), "x86_64");
assert_eq!(desc.xdata.len(), 2);
# Ok(())
# }
```

#### Handle alpm-db-files files programmatically

```rust
use std::{path::PathBuf, str::FromStr};

use alpm_db::files::{DbFiles, DbFilesV1};

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

// Create a DbFiles from a string.
let files_from_str = DbFiles::V1(DbFilesV1::from_str(data)?);

// Create a DbFiles from list of paths.
let files_from_paths = DbFiles::V1(DbFilesV1::try_from(paths)?);

assert_eq!(files_from_str.as_ref(), files_from_paths.as_ref());
# Ok(())
# }
```

### CLI

#### alpm-db-desc

<!--
```bash
# use a temporary directory for testing
test_tmpdir="$(mktemp --directory --suffix '.dbdesc-test')"
# temporary desc input and output files
DBDESC="$(mktemp --tmpdir="$test_tmpdir" --suffix '-desc' --dry-run)"
DBDESC_JSON="$(mktemp --tmpdir="$test_tmpdir" --suffix '-desc.json' --dry-run)"
export DBDESC
export DBDESC_JSON
```
-->

Create a database desc file from CLI arguments:

```bash
alpm-db-desc create v2 \
    --name foo \
    --version 1.0.0-1 \
    --base foo \
    --description "An example package" \
    --url https://example.org/ \
    --arch x86_64 \
    --builddate 1733737242 \
    --installdate 1733737243 \
    --packager "Foobar McFooface <foobar@mcfooface.org>" \
    --size 123 \
    --validation sha256 \
    --validation pgp \
    --optdepends libfoo,libbar \
    --optdepends "libdesc: Optional dependency with description" \
    --xdata pkgtype=pkg \
    "$DBDESC"
```

The output file (`$DBDESC`) contains the desc data in [alpm-db-descv1] format.

<!--
```bash
cat > "$DBDESC.expected" <<'EOF'
%NAME%
foo

%VERSION%
1.0.0-1

%BASE%
foo

%DESC%
An example package

%URL%
https://example.org/

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
123

%VALIDATION%
sha256
pgp

%OPTDEPENDS%
libfoo
libbar
libdesc: Optional dependency with description

%XDATA%
pkgtype=pkg

EOF

diff --ignore-trailing-space "$DBDESC" "$DBDESC.expected"
```
-->

Format db desc data as JSON:

```bash
alpm-db-desc format "$DBDESC" --output-format json --pretty > "$DBDESC_JSON"
```

The output file (`$DBDESC_JSON`) contains the structured JSON representation of
the parsed desc data.

<!--
```bash
cat > "$DBDESC_JSON.expected" <<'EOF'
{
  "name": "foo",
  "version": {
    "pkgver": "1.0.0",
    "pkgrel": {
      "major": 1,
      "minor": null
    },
    "epoch": null
  },
  "base": "foo",
  "description": "An example package",
  "url": "https://example.org/",
  "arch": "x86_64",
  "builddate": 1733737242,
  "installdate": 1733737243,
  "packager": {
    "name": "Foobar McFooface",
    "email": "foobar@mcfooface.org"
  },
  "size": 123,
  "groups": [],
  "reason": "Explicit",
  "license": [],
  "validation": [
    "Sha256",
    "Pgp"
  ],
  "replaces": [],
  "depends": [],
  "optdepends": [
    {
      "package_relation": {
        "name": "libfoo",
        "version_requirement": null
      },
      "description": null
    },
    {
      "package_relation": {
        "name": "libbar",
        "version_requirement": null
      },
      "description": null
    },
    {
      "package_relation": {
        "name": "libdesc",
        "version_requirement": null
      },
      "description": "Optional dependency with description"
    }
  ],
  "conflicts": [],
  "provides": [],
  "xdata": [
    "pkgtype=pkg"
  ]
}
EOF

diff --ignore-trailing-space "$DBDESC_JSON" "$DBDESC_JSON.expected"
rm -r -- "$test_tmpdir"
```
-->

#### alpm-db-files

<!--
```bash
# Create a temporary directory for test files.
test_tempdir="$(mktemp --directory --suffix '.alpm-db-files-test')"

# Create input dir for `alpm-db-files create`.
ALPM_DB_FILES_CREATE_INPUT_DIR="${test_tempdir}/input_dir/"
mkdir --parents "${ALPM_DB_FILES_CREATE_INPUT_DIR}usr/bin/"
touch "${ALPM_DB_FILES_CREATE_INPUT_DIR}usr/bin/foo"
export ALPM_DB_FILES_CREATE_INPUT_DIR

# Create input file for `alpm-db-files format` and `alpm-db-files validate`.
input_file="${test_tempdir}/input.files"
printf "%%FILES%%\nusr/\nusr/bin/\nusr/bin/foo\n\n" > "$input_file"
export ALPM_DB_FILES_FORMAT_INPUT_FILE="$input_file"
export ALPM_DB_FILES_VALIDATE_INPUT_FILE="$input_file"

# Create comparison file for `alpm-db-files format`.
alpm_files_format_output_compare="${test_tempdir}/format-output-compare.json"
printf '{\n  "files": [\n    "usr/",\n    "usr/bin/",\n    "usr/bin/foo"\n  ]\n}\n' > "$alpm_files_format_output_compare"

# Assign output files for `alpm-db-files create` and `alpm-db-files format`.
export ALPM_DB_FILES_CREATE_OUTPUT="${test_tempdir}/create-output.files"
export ALPM_DB_FILES_FORMAT_OUTPUT="${test_tempdir}/format-output.json"
```
-->

```bash
# Create an alpm-db-files file from an input directory.
alpm-db-files create --output "$ALPM_DB_FILES_CREATE_OUTPUT" "$ALPM_DB_FILES_CREATE_INPUT_DIR"
```

<!--
```bash
cat "$ALPM_DB_FILES_CREATE_OUTPUT"
diff "$input_file" "$ALPM_DB_FILES_CREATE_OUTPUT"
```
-->

```bash
# Format an alpm-db-files file as JSON.
alpm-db-files format --input-file "$ALPM_DB_FILES_FORMAT_INPUT_FILE" --output "$ALPM_DB_FILES_FORMAT_OUTPUT" --pretty
```

<!--
```bash
cat "$ALPM_DB_FILES_FORMAT_OUTPUT"
diff "$alpm_files_format_output_compare" "$ALPM_DB_FILES_FORMAT_OUTPUT"
```
-->

```bash
# Validate an alpm-db-files file.
alpm-db-files validate --input-file "$ALPM_DB_FILES_VALIDATE_INPUT_FILE"
```

<!--
```bash
rm -r -- "$test_tempdir"
```
-->

## Features

- `cli`: adds dependencies required for the `alpm-db-desc` and `alpm-db-files` command line interfaces.
- `_winnow-debug`: enables the `winnow/debug` feature for step-by-step parser debugging.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute
to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically
licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[alpm-db]: https://alpm.archlinux.page/specifications/alpm-db.7.html
[alpm-db-desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
[alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
[alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
[alpm-db-files]: https://alpm.archlinux.page/specifications/alpm-db-files.5.html
