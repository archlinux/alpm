# Mission statement

The ALPM project is building a new set of resources in the domain of **A**rch **L**inux **P**ackage **M**anagement.

The project is based on a modern, Rust-based framework and comprises a common set of specifications, libraries and command line tools.

Project goals:

- Rigorous specifications
  - Formalize existing ad-hoc systems and behaviors
- Focus on reusability
  - Extensive documentation
  - Language bindings
  - Liberal license
  - Solid foundation for new tools
- Modern implementation in Rust
  - Maintainability
  - Memory safety

Project methodology:

- Document behavior of existing tools
- New Rust implementations based on documentation
- Unit tests based on documentation
- Integration tests to ensure interoperability with existing tools and artifacts

## Specifications

The work on each aspect of the ALPM domain always starts with writing detailed documentation.

These specification documents are mainly based on behavior of existing Arch packaging tools, combined with input from experienced package maintainers.

All specifications are licensed as `CC-BY-SA-4.0`.

## Reusable code

Based on these specifications, new modular library implementations are written in a bottom-up design approach.
Based on these libraries, the ALPM project also provides command-line tools to cover some basic use-cases (e.g. file format handling).

The aim is to provide both libraries and CLI tools that are easy to reuse, in as many contexts as possible.

The Rust programming language has been chosen as a modern and memory-safe language.
It offers flexibility, good runtime performance as well as good maintainability.

### Licensing

Most crates in the Rust ecosystem are liberally licensed under `MIT OR Apache-2.0`.
For maximum reusability, ALPM's code licensing conforms to this norm, wherever possible.
See the project's [reuse configuration] for all details.

### API documentation

The public interface of all ALPM libraries is documented extensively and all items cross-reference the specifications they are based on.

Developers can rely on in-code documentation to deepen their understanding of the framework's behavior.

### Language bindings

To facilitate reuse from other languages, the ALPM project produces dedicated language bindings based on the project's native Rust code.

This allows existing applications to move from unmaintained and/or less validating native language libraries to ALPM's reusable and extensively validated libraries.

Bindings for the Python language are currently in focus, other language bindings can be added in the future.

### Collaboration

The ALPM framework can be used in your project!

Already now, collaboration with the [buildbtw] project proves to be productive and enjoyable.

Additional interest in collaboration has been expressed by further parties. Please note that you don't need to be under the Arch Linux umbrella to use ALPM's libraries and tools.

If your project has a use-case for one of the ALPM components, feel free to get in touch and let us know!

[buildbtw]: https://gitlab.archlinux.org/archlinux/buildbtw/
[reuse configuration]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/blob/main/REUSE.toml
