# alpm-soname

A library and command line toolkit for looking up [`soname`]s in ALPM context

[`soname`]: https://alpm.archlinux.page/specifications/alpm-soname.7.html

## Examples

The examples below assume that the following shared library exists:

```plaintext
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

### Command Line

You can use the subcommands to find the sonames provided by a package and the sonames required by a package.

`-v` argument can be used to set the verbosity level. (e.g. `-v` for debug and `-vv` for trace)

#### Finding Provisions

For a zstd-compressed package archive, you can retrieve the sonames provided by the package:

```bash
$ alpm-soname get-provisions --lookup-dir 'lib:/usr/lib' example-1.0.0-x86_64.pkg.tar.zst

lib:libexample.so.1
```

#### Finding Dependencies

For a zstd-compressed package archive, you can retrieve the sonames required by the package:

```bash
$ alpm-soname get-dependencies --all --lookup-dir 'lib:/usr/lib' application-1.0.0-x86_64.pkg.tar.zst

lib:libc.so.6
lib:libexample.so.1
```

The `--all` flag can be used when you want to retrieve all dependencies even though a matching provision in another package does not exist.

### Library

#### Finding Provisions

```rust no_run
use std::{path::PathBuf, str::FromStr};

use alpm_soname::find_provisions;
use alpm_types::SonameLookupDirectory;

fn main() -> Result<(), alpm_soname::Error> {
  let provisions = find_provisions(
    PathBuf::from("example-1.0.0-x86_64.pkg.tar.zst"),
    SonameLookupDirectory::from_str("lib:/usr/lib")?,
  )?;

  println!("{provisions:?}"); // [ SonameV2 { ... }, ...]
  Ok(())
}
```

### Finding Dependencies

```rust no_run
use std::{path::PathBuf, str::FromStr};

use alpm_soname::find_dependencies;
use alpm_types::SonameLookupDirectory;

fn main() -> Result<(), alpm_soname::Error> {
  let dependencies = find_dependencies(
    PathBuf::from("application-1.0.0-x86_64.pkg.tar.zst"),
    SonameLookupDirectory::from_str("lib:/usr/lib")?,
    true
  )?;

  println!("{dependencies:?}"); // [ SonameV2 { ... }, ...]
  Ok(())
}
```

## Features

- `cli` adds the commandline handling needed for the `alpm-soname` binary (enabled by default).

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
