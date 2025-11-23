# alpm-db

A library for **A**rch **L**inux **P**ackage **M**anagement (ALPM) system databases.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_db/> for development version of the crate.
- <https://docs.rs/alpm-db-desc/latest/alpm_db/> for released version of the crate.

## Overview

`alpm-db` crate contains `desc` module, which provides functionality for writing
and parsing of ALPM [DB desc] files.

These `desc` files describe the metadata of installed packages on a
system relying on ALPM.
They contain fields such as the package name, version, architecture,
and dependencies.

It also contains a commandline interface (CLI) binary `alpm-db-desc`,
which can be used to create, parse, format and validate ALPM [DB desc] files.

## Examples

### Library

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

### Commandline

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

## Features

- `cli`: enables the commandline interface for the `alpm-db-desc` binary.
- `_winnow-debug`: enables the `winnow/debug` feature for
  step-by-step parser debugging.

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
[DB desc]: https://alpm.archlinux.page/specifications/alpm-db-desc.5.html
[alpm-db-descv1]: https://alpm.archlinux.page/specifications/alpm-db-descv1.5.html
[alpm-db-descv2]: https://alpm.archlinux.page/specifications/alpm-db-descv2.5.html
