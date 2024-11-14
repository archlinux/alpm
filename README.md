# ALPM

This project comprises specifications, as well as [Rust] libraries and tools for **A**rch **L**inux **P**ackage **M**anagement.

The ALPM project arose from the need for more clearly specifying the interfaces, as well as providing bindings and tools in a memory-safe programming language.

The scope of this project is to provide robust integration for all relevant package creation and consumption, as well as repository management tasks.

## Components

Currently the following specifications, libraries and CLI tools are available:

- [alpm-types]: a central library for types used by other ALPM libraries and tools
- [alpm-buildinfo]: specifications, as well as a library and CLI to work with `.BUILDINFO` files

Further specifications, libraries and CLI tools for relevant file types are on the roadmap ([%1]).

## Specifications

Specifications for various formats are provided in the context of the [components] that make use of them and are located in a component's `resources/specification/` directory.
The implementations encourage robust, but lenient parsing of file formats (unknown key-value pairs are discarded with a warning).

Several versions of specifications may exist side-by-side, but only the latest version is actively supported.
Legacy versions of specifications may be deprecated and removed as needed, if they hinder further development.

## File types

- [BUILDINFO]
  - [BUILDINFOv1]
  - [BUILDINFOv2]
- [PKGINFO]
  - [PKGINFOv1]
  - [PKGINFOv2]

## Fields and composite types

- [alpm-architecture]
- [alpm-comparison]
- [alpm-epoch]
- [alpm-package-name]
- [alpm-package-relation]
- [alpm-package-version]
- [alpm-pkgrel]
- [alpm-pkgver]

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[%1]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/milestones/1
[Apache-2.0]: LICENSES/Apache-2.0.txt
[BUILDINFO]: alpm-buildinfo/resources/specification/BUILDINFO.5.md
[BUILDINFOv1]: alpm-buildinfo/resources/specification/BUILDINFOv1.5.md
[BUILDINFOv2]: alpm-buildinfo/resources/specification/BUILDINFOv2.5.md
[MIT]: LICENSES/MIT.txt
[PKGINFO]: alpm-pkginfo/resources/specification/PKGINFO.5.md
[PKGINFOv1]: alpm-pkginfo/resources/specification/PKGINFOv1.5.md
[PKGINFOv2]: alpm-pkginfo/resources/specification/PKGINFOv2.5.md
[Rust]: https://www.rust-lang.org/
[alpm-architecture]: alpm-types/resources/specification/alpm-architecture.7.md
[alpm-buildinfo]: alpm-buildinfo/
[alpm-comparison]: alpm-types/resources/specification/alpm-comparison.7.md
[alpm-epoch]: alpm-types/resources/specification/alpm-epoch.7.md
[alpm-package-name]: alpm-types/resources/specification/alpm-package-name.7.md
[alpm-package-relation]: alpm-types/resources/specification/alpm-package-relation.7.md
[alpm-package-version]: alpm-types/resources/specification/alpm-package-version.7.md
[alpm-pkgrel]: alpm-types/resources/specification/alpm-package-pkgrel.7.md
[alpm-pkgver]: alpm-types/resources/specification/alpm-package-pkgver.7.md
[alpm-types]: alpm-types/
[components]: #components
[contribution guidelines]: CONTRIBUTING.md
