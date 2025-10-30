# python-alpm

Python bindings for the **A**rch **L**inux **P**ackage **M**anagement (ALPM) project.

## Documentation

Latest documentation is available at <https://alpm.archlinux.page/pdoc>.

## Installation

```sh
pip install python-alpm
```

## Examples

### Validating and comparing versions

```pycon
>>> from alpm.alpm_types import PackageVersion, PackageRelease, FullVersion, SchemaVersion

>>> full_version = FullVersion(pkgver=PackageVersion('0.1.0alpha'), pkgrel=PackageRelease(minor=1))
>>> full_version
FullVersion(pkgver=PackageVersion('0.1.0alpha'), pkgrel=PackageRelease(major=0, minor=1))

>>> PackageVersion('1.0.1') > full_version.pkgver
True

>>> version_one = SchemaVersion.from_str('1.0.0')
>>> version_one
SchemaVersion(major=1, minor=0, patch=0, pre='', build='')

>>> version_one == SchemaVersion(major=1)
True

```

### Parsing .SRCINFO

```pycon
>>> from alpm.alpm_srcinfo import SourceInfoV1
>>> from alpm.alpm_types import Architecture

>>> srcinfo = SourceInfoV1.from_file("../alpm-srcinfo/tests/correct/all_overrides.srcinfo")

>>> srcinfo.base
PackageBase(name='example', version=FullVersion(pkgver=PackageVersion('0.1.0'), pkgrel=PackageRelease(major=1), epoch=1))

>>> srcinfo.base.architectures
[Architecture.X86_64, Architecture.AARCH64]

>>> srcinfo.packages
[Package(name='example')]

>>> srcinfo.packages_for_architecture(Architecture.AARCH64)
[MergedPackage(architecture='AARCH64', name='example', version=FullVersion(pkgver=PackageVersion('0.1.0'), pkgrel=PackageRelease(major=1), epoch=1))]

```

### Working with package metadata

```pycon
>>> from alpm.alpm_srcinfo.source_info.v1.package import Package, Override
>>> from alpm.alpm_types import License

>>> pkg = Package("testpkg")

>>> pkg.description = Override("A test package")
>>> pkg.description
Override(value='A test package')

>>> pkg.licenses = Override([License("MIT"), License("custom-license")])
>>> pkg.licenses
Override(value=[License('MIT'), License('custom-license')])

>>> {str(lic): lic.is_spdx for lic in pkg.licenses.value}
{'MIT': True, 'custom-license': False}

```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
