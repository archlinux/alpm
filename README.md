# ALPM

This project comprises specifications, as well as [Rust] libraries and tools for **A**rch **L**inux **P**ackage **M**anagement.

The ALPM project arose from the need for more clearly specifying the interfaces, as well as providing bindings and tools in a memory-safe programming language.
The specifications and implementations are based on ad-hoc implementations in the [pacman] project.
Currently, this project aims to maintain compatibility with [pacman] `5.1.0` and onwards.

The scope of this project is to provide robust integration for all relevant package creation and consumption, as well as repository management tasks.
As such, the ALPM project also aims at providing drop-in replacements or alternatives for some facilities provided by [pacman].

This project is currently supported by the [Sovereign Tech Agency].
Read the [official announcement] for more information.

## Documentation

The latest project documentation can be found at <https://alpm.archlinux.page>

## Overview

The following mindmap attempts to provide a high-level overview of the project and put file types as well as (existing and upcoming) libraries into context.

```mermaid
mindmap
  root((ALPM))
    📂 Source
      📄 PKGBUILD
      📄 .SRCINFO
      📚️ alpm-srcinfo
    📦 Package
      📄 .BUILDINFO
      📄 .PKGINFO
      📄 .INSTALL
      📄 .MTREE
      📚️ alpm-buildinfo
      📚️ alpm-pkgbuild
      📚️ alpm-pkginfo
      📚️ alpm-mtree
      📚️ alpm-package
      📚️ alpm-package-verify*
      📚️ alpm-package-verify-vda*
    🗄️ Repository
      📄 desc
      📄 files
      📚️ alpm-repo*
      📚️ alpm-repo-db*
      📚️ alpm-repo-desc*
      📚️ alpm-repo-files
      📂 alpm-state-repo
    🗄️ Package Management
      📄 desc
      📄 files
      📚️ alpm-db*
      📚️ alpm-db-desc*
      📚️ alpm-db-download*
      📚️ alpm-db-files
      📚️ alpm-db-verify*
      📚️ alpm-db-verify-vda*
```

For an overview of planned specifications and components, refer to the [milestones] of the project.

[*] Not yet implemented, or subject to change.

## Components

Currently the following components are available:

- [alpm-buildinfo]: a library and commandline interface to work with [BUILDINFO] files
- [alpm-common]: a library for common traits and functionality
- [alpm-compress]: a library for compression operations in ALPM
- [alpm-mtree]: a library and commandline interface to work with [ALPM-MTREE] files
- [alpm-package]: a library for the creation of [alpm-package][spec:alpm-package] files
- [alpm-parsers]: a library for providing various custom parsers/deserializers for file types used in ALPM
- [alpm-pkginfo]: a library and commandline interface to work with [PKGINFO] files
- [alpm-srcinfo]: a library and commandline interface to work with [SRCINFO] files
- [alpm-types]: a central library for types used by other ALPM libraries and tools
- [python-alpm]: Python bindings for ALPM crates and the python-alpm Python library

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## Releases

Releases of [components] are created by the developers of this project.

[OpenPGP certificates] with the following [OpenPGP fingerprints] can be used to verify signed tags:

- `991F6E3F0765CF6295888586139B09DA5BF0D338` (David Runge <dvzrv@archlinux.org>)
- `165E0FF7C48C226E1EC363A7F83424824B3E4B90` (Orhun Parmaksız <orhun@archlinux.org>)

The above are part of [archlinux-keyring] and certified by at least three [main signing keys] of the distribution.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[ALPM-MTREE]: https://alpm.archlinux.page/specifications/ALPM-MTREE.5.html
[Apache-2.0]: LICENSES/Apache-2.0.txt
[BUILDINFO]: https://alpm.archlinux.page/specifications/BUILDINFO.5.html
[MIT]: LICENSES/MIT.txt
[OpenPGP certificates]: https://openpgp.dev/book/certificates.html
[OpenPGP fingerprints]: https://openpgp.dev/book/certificates.html#fingerprint
[PKGINFO]: https://alpm.archlinux.page/specifications/PKGINFO.5.html
[Rust]: https://www.rust-lang.org/
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[Sovereign Tech Agency]: https://www.sovereign.tech/tech/arch-linux-package-management
[alpm-buildinfo]: alpm-buildinfo/
[alpm-common]: alpm-common/
[alpm-compress]: alpm-compress/
[alpm-mtree]: alpm-mtree/
[alpm-package]: alpm-package/
[alpm-parsers]: alpm-parsers/
[alpm-pkginfo]: alpm-pkginfo/
[alpm-srcinfo]: alpm-srcinfo/
[alpm-types]: alpm-types/
[python-alpm]: python-alpm/
[archlinux-keyring]: https://gitlab.archlinux.org/archlinux/archlinux-keyring
[components]: #components
[contribution guidelines]: CONTRIBUTING.md
[main signing keys]: https://archlinux.org/master-keys/
[milestones]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/milestones
[official announcement]: https://lists.archlinux.org/archives/list/arch-dev-public@lists.archlinux.org/thread/MZLH43574GGP7QQ7RKAAIRFT5LJPCEB4/
[pacman]: https://gitlab.archlinux.org/pacman/pacman
[spec:alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
