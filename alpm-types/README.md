<!--
SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# alpm-types

Types for **A**rch **L**inux **P**ackage **M**anagement.

The provided types and the traits they implement can be used in package
management related applications (e.g. package manager, repository manager,
special purpose parsers and file specifications, etc.) which deal with
[libalpm](https://man.archlinux.org/man/libalpm.3) based packages.

This library strives to provide all underlying types for writing ALPM based
software as a leaf-crate, so that they can be shared across applications and
none of them has to implement them itself.

## Contributing

Please refer to the [contribution guidelines](CONTRIBUTING.md) to learn how to
contribute to this project.

## License

This project is licensed under the terms of the
[LGPL-3.0-or-later](https://www.gnu.org/licenses/lgpl-3.0.en.html).
