# alpm-files

A specification, library and command line tool for the handling of **A**rch **L**inux **P**ackage **M**anagement (ALPM) `files` files ([alpm-files]).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_files/> for development version of the crate.
- <https://docs.rs/alpm-files/latest/alpm_files/> for released version of the crate.

## Examples

### Library

```rust
use std::{path::PathBuf, str::FromStr};

use alpm_files::{Files, FilesV1};

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

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
[alpm-files]: https://alpm.archlinux.page/specifications/alpm-files.5.html
[contribution guidelines]: ../CONTRIBUTING.md
