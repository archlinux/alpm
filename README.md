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

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[%1]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/milestones/1
[Apache-2.0]: LICENSES/Apache-2.0.txt
[MIT]: LICENSES/MIT.txt
[Rust]: https://www.rust-lang.org/
[alpm-types]: alpm-types/
[alpm-buildinfo]: alpm-buildinfo/
[components]: #components
[contribution guidelines]: CONTRIBUTING.md
