# alpm-repo-db

A library for creation and access of [alpm-repo-db] files (**A**rch **L**inux **P**ackage **M**anagement (ALPM) repository sync databases).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_repo_db/> for development version of the crate.
- <https://docs.rs/alpm-repo-db/latest/alpm_repo_db/> for released version of the crate.

## Overview

The `alpm-repo-db` crate contains a `desc` module, which provides functionality for writing and parsing of [alpm-repo-desc] files.

These `desc` files describe the metadata of single packages in an [alpm-repo-db] (aka. ALPM repository sync databases).
They contain data such as the package name, version, architecture, file name, checksums, and dependencies.

This crate also provides the command line interface (CLI) `alpm-repo-desc`, which can be used to create, parse, format and validate [alpm-repo-desc] files.

## Examples

### Library

Parsing [alpm-repo-descv1] files:

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

Parsing [alpm-repo-descv2] files:

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

### Commandline

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

The output file (`$REPODESC`) contains the desc data in [alpm-repo-descv2] format.

<!--
```bash
cat > "$REPODESC.expected" <<'EOF'
%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

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

%DEPENDS%
libfoo

%OPTDEPENDS%
libbar: Optional dependency with description

EOF

diff --ignore-trailing-space "$REPODESC" "$REPODESC.expected"
```
-->

Format repo desc data as JSON:

```bash
alpm-repo-desc format "$REPODESC" --output-format json --pretty > "$REPODESC_JSON"
```

The output file (`$REPODESC_JSON`) contains the structured JSON representation of the parsed desc data.

<!--
```bash
cat > "$REPODESC_JSON.expected" <<'EOF'
{
  "file_name": "example-1.0.0-1-any.pkg.tar.zst",
  "name": "example",
  "base": "example",
  "version": {
    "pkgver": "1.0.0",
    "epoch": null,
    "pkgrel": {
      "major": 1,
      "minor": null
    }
  },
  "description": "An example meta package",
  "groups": [],
  "compressed_size": 4634,
  "installed_size": 0,
  "sha256_checksum": "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
  "pgp_signature": null,
  "url": "https://example.org/",
  "license": [
    "GPL-3.0-or-later"
  ],
  "arch": "any",
  "build_date": 1729181726,
  "packager": {
    "name": "Foobar McFooface",
    "email": "foobar@mcfooface.org"
  },
  "replaces": [],
  "conflicts": [],
  "provides": [],
  "dependencies": [
    {
      "name": "libfoo",
      "version_requirement": null
    }
  ],
  "optional_dependencies": [
    {
      "package_relation": {
        "name": "libbar",
        "version_requirement": null
      },
      "description": "Optional dependency with description"
    }
  ],
  "make_dependencies": [],
  "check_dependencies": []
}
EOF

diff --ignore-trailing-space "$REPODESC_JSON" "$REPODESC_JSON.expected"
rm -r -- "$test_tmpdir"
```
-->

## Features

- `cli`: enables the commandline interface for the `alpm-repo-desc` binary.
- `_winnow-debug`: enables the `winnow/debug` feature for step-by-step parser debugging.

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[alpm-repo-db]: https://alpm.archlinux.page/specifications/alpm-repo-db.7.html
[alpm-repo-descv1]: https://alpm.archlinux.page/specifications/alpm-repo-descv1.5.html
[alpm-repo-descv2]: https://alpm.archlinux.page/specifications/alpm-repo-descv2.5.html
