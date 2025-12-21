# alpm-repo-db

A library and command line interface for creation and access of [alpm-repo-db] files (**A**rch **L**inux **P**ackage **M**anagement (ALPM) repository sync databases) and handling of their [alpm-repo-desc] and [alpm-repo-files] file formats.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_repo_db/> for development version of the crate.
- <https://docs.rs/alpm-repo-db/latest/alpm_repo_db/> for released version of the crate.

## Overview

The `alpm-repo-db` crate contains a `desc` module, which provides functionality for writing and parsing of [alpm-repo-desc] files.
These `desc` files describe the metadata of single packages in an [alpm-repo-db] (aka. ALPM repository sync databases).
They contain data such as the package name, version, architecture, file name, checksums, and dependencies.

It also contains a `files` module, which provides functionality for writing and parsing of [alpm-repo-files] files.

This crate provides the command line interfaces (CLI) `alpm-repo-desc` and `alpm-repo-files`, which can be used to create, parse, format and validate their respective file formats.

## Examples

### Library

#### Parsing [alpm-repo-descv1] files

```rust
use std::str::FromStr;
use alpm_repo_db::desc::RepoDescFileV1;

# fn main() -> Result<(), alpm_repo_db::Error> {
let desc_data = r#"%FILENAME%
example-meta-1.0.0-1-any.pkg.tar.zst

%NAME%
example-meta

%BASE%
example-meta

%VERSION%
1.0.0-1

%DESC%
An example meta package

%CSIZE%
4634

%ISIZE%
0

%MD5SUM%
d3b07384d113edec49eaa6238ad5ff00

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%
https://example.org/

%LICENSE%
GPL-3.0-or-later

%ARCH%
any

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

"#;

let desc = RepoDescFileV1::from_str(desc_data)?;
assert_eq!(desc.name.to_string(), "example-meta");
assert_eq!(desc.arch.to_string(), "any");
# Ok(())
# }
```

#### Parsing [alpm-repo-descv2] files

```rust
use std::str::FromStr;
use alpm_repo_db::desc::RepoDescFileV2;

# fn main() -> Result<(), alpm_repo_db::Error> {
let desc_data = r#"%FILENAME%
example-meta-1.0.0-1-any.pkg.tar.zst

%NAME%
example-meta

%BASE%
example-meta

%VERSION%
1.0.0-1

%DESC%
An example meta package

%CSIZE%
4634

%ISIZE%
0

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%URL%
https://example.org/

%LICENSE%
GPL-3.0-or-later

%ARCH%
any

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

"#;

let desc = RepoDescFileV2::from_str(desc_data)?;
assert_eq!(desc.name.to_string(), "example-meta");
assert_eq!(desc.arch.to_string(), "any");
# Ok(())
# }
```

#### Working with [alpm-repo-files]

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

### Command line

#### alpm-repo-desc

<!--
```bash
# use a temporary directory for testing
test_tmpdir="$(mktemp --directory --suffix '.repodesc-test')"
# temporary desc input and output files
REPODESC="$(mktemp --tmpdir="$test_tmpdir" --suffix '-desc' --dry-run)"
REPODESC_JSON="$(mktemp --tmpdir="$test_tmpdir" --suffix '-desc.json' --dry-run)"
export REPODESC
export REPODESC_JSON
```
-->

Create a repository desc file from CLI arguments:

```bash
alpm-repo-desc create v2 \
    --file-name example-1.0.0-1-any.pkg.tar.zst \
    --name example \
    --base example \
    --version 1.0.0-1 \
    --description "An example meta package" \
    --csize 4634 \
    --isize 0 \
    --sha256sum b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c \
    --url https://example.org/ \
    --license GPL-3.0-or-later \
    --arch any \
    --builddate 1729181726 \
    --packager "Foobar McFooface <foobar@mcfooface.org>" \
    --depends libfoo \
    --optdepends "libbar: Optional dependency with description" \
    "$REPODESC"
```

Format repo desc data as JSON:

```bash
alpm-repo-desc format "$REPODESC" --output-format json --pretty > "$REPODESC_JSON"
```

#### alpm-repo-files

```bash
# Create an alpm-repo-files file from an input directory.
alpm-repo-files create path/to/input/dir

# Format an alpm-repo-files file as JSON.
alpm-repo-files format --input-file path/to/repo.files --pretty

# Validate an alpm-repo-files file.
alpm-repo-files validate --input-file path/to/repo.files
```

## Features

- `cli`: adds dependencies required for the `alpm-repo-desc` and `alpm-repo-files` command line interfaces.
- `_winnow-debug`: enables the `winnow/debug` feature for step-by-step parser debugging.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html
[alpm-repo-desc]: https://alpm.archlinux.page/specifications/alpm-repo-desc.5.html
[alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
[alpm-repo-descv2]: https://alpm.archlinux.page/specifications/alpm-repo-descv2.5.html
[alpm-repo-files]: https://alpm.archlinux.page/specifications/alpm-repo-files.5.html
[contribution guidelines]: ../CONTRIBUTING.md
