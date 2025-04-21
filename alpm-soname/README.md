# alpm-soname

A library and command line toolkit for looking up [`soname`] data in an ALPM context

[`soname`]: https://alpm.archlinux.page/specifications/alpm-soname.7.html

## Examples

The examples below assume that the following shared object setup exists in a package file `example-1.0.0-1-x86_64.pkg.tar.zst`:

```plaintext
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

Here, `/usr/lib/libexample.so.1.0.0` encodes the soname `libexample.so.1` in its [ELF] dynamic section.

### Command Line

You can use the subcommands to find the sonames provided by a package and the sonames required by a package.

The `-v` option can be used to set the verbosity level. (e.g. `-v` for debug and `-vv` for trace)

#### Finding Provisions

You can retrieve the sonames provided by the package:

```bash
$ alpm-soname get-provisions --lookup-dir 'lib:/usr/lib' example-1.0.0-1-x86_64.pkg.tar.zst

lib:libexample.so.1
```

#### Finding Dependencies

You can retrieve the sonames required by the package:

```bash
$ alpm-soname get-dependencies --lookup-dir 'lib:/usr/lib' application-1.0.0-1-x86_64.pkg.tar.zst

lib:libexample.so.1
```

#### Finding Raw Dependencies

`get-dependencies` subcommand only returns the soname dependencies, that have a matching entry in the package's metadata.

If you are interested in all soname dependencies encoded in the [ELF] files of a package, you can use the `get-raw-dependencies` subcommand.

```bash
$ alpm-soname get-raw-dependencies application-1.0.0-1-x86_64.pkg.tar.zst

libc.so.6
libexample.so.1
```

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

#### Finding Dependencies

```rust no_run
use std::{path::PathBuf, str::FromStr};

use alpm_soname::find_dependencies;
use alpm_types::SonameLookupDirectory;

fn main() -> Result<(), alpm_soname::Error> {
  let dependencies = find_dependencies(
    PathBuf::from("application-1.0.0-x86_64.pkg.tar.zst"),
    SonameLookupDirectory::from_str("lib:/usr/lib")?,
  )?;

  println!("{dependencies:?}"); // [ SonameV2 { ... }, ...]
  Ok(())
}
```

### Extracting Soname Data

```rust no_run
use std::path::PathBuf;

use alpm_soname::extract_elf_sonames;

fn main() -> Result<(), alpm_soname::Error> {
  let elf_sonames = extract_elf_sonames(
    PathBuf::from("application-1.0.0-x86_64.pkg.tar.zst"),
  )?;

  println!("{elf_sonames:?}"); // [ ElfSonames { path: ..., sonames: [Soname { ... }, ...] }, ...]
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
[ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
