# python-alpm

Python bindings for the **A**rch **L**inux **P**ackage **M**anagement (ALPM) project.

## Documentation

Latest documentation is available at <https://alpm.archlinux.page/pdoc>.

## Installation

```sh
pip install alpm
```

## Examples

### Validating and comparing versions

```python
from alpm.alpm_types import PackageVersion, SchemaVersion

package_version = PackageVersion('1.0.0alpha')

version_one = SchemaVersion.from_str('1.0.0')
version_also_one = SchemaVersion.from_str('1')
assert version_one == version_also_one
```

### Parsing SRCINFO

```python
from alpm.alpm_srcinfo import SourceInfoV1

srcinfo = SourceInfoV1.from_file("test.srcinfo")
print(srcinfo.base.name)
print(srcinfo.packages)
```

### Working with package metadata

```python
from alpm.alpm_srcinfo.source_info.v1.package import Package, Override

pkg = Package("testpkg")
pkg.description = Override("A test package")
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
