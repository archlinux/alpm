# alpm-types

Types for **A**rch **L**inux **P**ackage **M**anagement.

The provided types and the traits they implement can be used in package management related applications (e.g. package manager, repository manager, special purpose parsers and file specifications, etc.) which deal with [libalpm](https://man.archlinux.org/man/libalpm.3) based packages.

This library strives to provide all underlying types for writing ALPM based software as a leaf-crate, so that they can be shared across applications and none of them has to implement them itself.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_types/> for development version of the crate
- <https://docs.rs/alpm-types/latest/alpm_types/> for released versions of the crate

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[reuse configuration]: ../REUSE.toml
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
