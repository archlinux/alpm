# alpm-package

A library providing low-level functionality for **A**rch **L**inux **P**ackage **M**anagement (ALPM) based packages.

This library offers integration for creating an [alpm-package] from a prepared input directory, which contains all necessary files (i.e. metadata files, optional install scriptlets and data files).

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_package/> for development version of the crate
- <https://docs.rs/alpm-package/latest/alpm_package/> for released versions of the crate

## Examples

### Library

A package file can be created from a prepared input directory.
The input directory must contain at the very least a valid [BUILDINFO], a [PKGINFO] and an [ALPM-MTREE] file.
Then the relevant metadata/data/script files can be read from the package archive using the `PackageReader` API.

```rust
use std::fs::{File, Permissions, create_dir_all};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use alpm_compress::compression::CompressionSettings;
use alpm_mtree::create_mtree_v2_from_input_dir;
use alpm_package::{
    InputDir,
    MetadataEntry,
    OutputDir,
    Package,
    PackageCreationConfig,
    PackageInput,
};
use alpm_types::MetadataFileName;
use tempfile::TempDir;

# fn main() -> testresult::TestResult {
// Create a common temporary directory for input and output.
let temp_dir = TempDir::new()?;
let path = temp_dir.path();
let input_dir = path.join("input");
create_dir_all(&input_dir)?;
let input_dir = InputDir::new(input_dir)?;
let output_dir = OutputDir::new(path.join("output"))?;

// Create a valid, but minimal BUILDINFOv2 file.
let mut file = File::create(&input_dir.join(MetadataFileName::BuildInfo.as_ref()))?;
write!(file, r#"
format = 2
builddate = 1
builddir = /build
startdir = /startdir/
buildtool = devtools
buildtoolver = 1:1.2.1-1-any
installed = other-example-1.2.3-1-any
packager = John Doe <john@example.org>
pkgarch = any
pkgbase = example
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = example
pkgver = 1:1.0.0-1
"#)?;

// Create a valid, but minimal PKGINFOv2 file.
let mut file = File::create(&input_dir.join(MetadataFileName::PackageInfo.as_ref()))?;
write!(file, r#"
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 1:1.0.0-1
pkgdesc = A project that returns true
url = https://example.org/
builddate = 1
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
depend = bash
"#)?;

// Create a dummy script as package data.
create_dir_all(&input_dir.join("usr/bin"))?;
let mut file = File::create(&input_dir.join("usr/bin/example"))?;
write!(file, r#"!/bin/bash
true
"#)?;
file.set_permissions(Permissions::from_mode(0o755))?;

// Create a valid ALPM-MTREEv2 file from the input directory.
create_mtree_v2_from_input_dir(&input_dir)?;

// Create PackageInput and PackageCreationConfig.
let package_input: PackageInput = input_dir.try_into()?;
let config = PackageCreationConfig::new(
    package_input,
    output_dir,
    CompressionSettings::default(),
)?;
// Create package file.
let package = Package::try_from(&config)?;

// Create a reader for the package.
let mut reader = package.clone().into_reader()?;

// Read all the metadata from the package archive.
let metadata = reader.metadata()?;
let pkginfo = metadata.pkginfo;
let buildinfo = metadata.buildinfo;
let mtree = metadata.mtree;

// Or you can iterate over the metadata entries:
let mut reader = package.clone().into_reader()?;
for entry in reader.metadata_entries()? {
    let entry = entry?;
    match entry {
        MetadataEntry::PackageInfo(pkginfo) => {}
        MetadataEntry::BuildInfo(buildinfo) => {}
        MetadataEntry::Mtree(mtree) => {}
        _ => {}
    }
}

// You can also read specific metadata files directly:
let mut reader = package.clone().into_reader()?;
let pkginfo = reader.read_metadata_file(MetadataFileName::PackageInfo)?;
// let buildinfo = reader.read_metadata_file(MetadataFileName::BuildInfo)?;
// let mtree = reader.read_metadata_file(MetadataFileName::Mtree)?;

// Read the install scriptlet, if present:
let mut reader = package.clone().into_reader()?;
let install_scriptlet = reader.read_install_scriptlet()?;

// Iterate over the data entries in the package archive.
let mut reader = package.clone().into_reader()?;
for data_entry in reader.data_entries()? {
    let mut data_entry = data_entry?;
    let content = data_entry.content()?;
    // Note: data_entry also implements `Read`, so you can read from it directly.
}

// Convenience functions for reading packages without creating a reader:
let pkginfo = package.read_pkginfo()?;
let buildinfo = package.read_buildinfo()?;
let mtree = package.read_mtree()?;
let install_scriptlet = package.read_install_scriptlet()?;
# Ok(())
# }
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
[MIT]: ../LICENSES/MIT.txt
[PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
[alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
[contribution guidelines]: ../CONTRIBUTING.md
