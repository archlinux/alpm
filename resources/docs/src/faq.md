# FAQ

## What is ALPM?

A software framework and set of documentation for working with **A**rch **L**inux **P**ackage **M**anagement.

## Is ALPM a package manager?

No, but you could build one with it.

## Can I build packages with ALPM?

Yes, but only part of the package build process is currently possible.
ALPM does not offer functionality for building a package from a [PKGBUILD(5)] file, but it allows creating a package file from a prepared input directory.

## Why or when would I use ALPM?

If your project deals with Arch Linux packages, repository sync databases, or specific metadata files (e.g. [SRCINFO(5)]).

ALPM provides a variety of libraries and command-line interfaces, that enable the creation and validation of metadata files used in the Arch Linux ecosystem.

## Under what license is ALPM released?

ALPM is free and open source and can be used under the the terms of the [Apache-2.0] or [MIT].
For details refer to the project's [reuse configuration].

## Who is funding the work on the ALPM project?

Currently no one.

In 2024 and 2025, development on the ALPM project has been funded by the [Sovereign Tech Agency] (STA).

## What are the future plans for the project?

Our plan is to:

- further improve the documentation of the Arch Linux packaging ecosystem,
- write more lint rules,
- write further libraries and command line interfaces for package creation, as well as system and repository handling,
- help integrate ALPM into further projects,
- and extend the Python bindings.

## What is the difference between the Pacman project and ALPM?

Pacman is a project that provides the [`makepkg(8)`], [`repo-add(8)`] and [`pacman(8)`] executables and the shared library [libalpm(3)].

ALPM is a highly modular "library first" framework, that provides reimagined building blocks for package management tools.

### What is the difference between the `pacman` CLI tool and ALPM?

The [`pacman(8)`] CLI tool is a system package manager written in C.
It allows to install/uninstall/upgrade/downgrade packages on a system and retrieve metadata from packages on a system or from repository metadata.

The ALPM project does not provide a tool equivalent in functionality to the [`pacman(8)`] executable.

### What is the difference between the `makepkg` CLI tool and ALPM?

The [`makepkg(8)`] CLI tool is a Bash-based script to create package files from package build scripts (i.e. [PKGBUILD(5)]).

The ALPM project does not provide a tool equivalent in functionality to the [`makepkg(8)`] executable.

### What is the difference between the shared library `libalpm` and ALPM?

The shared library [libalpm(3)] is written in C and provides functionality for managing system state based on a local package database on end-user machines.
It provides low-level access to actions such as installation/uninstallation/upgrade/downgrade of packages and the retrieval of metadata to its consumers (e.g. [`pacman(8)`]).

The ALPM project provides various building blocks in [Rust], that are required for system management, based on strong validation and strict typing.

## Is ALPM based on Pacman?

While ALPM is based on Pacman in spirit, its codebase is an independent effort.

ALPM is a project that reimagines the building blocks of Arch Linux Package Management from first principles.
ALPM attempts to provide perfect interoperability with the existing package management tooling (including the Pacman project), while relying on entirely new implementations based on specifications of the existing formats.

## Can I use ALPM with Python?

The ALPM project will create Python bindings for its library functionality.
The goal is to provide a Python API with a broad scope, that can be used in a diverse set of contexts.

## Can I use ALPM with C?

Currently, it is not a focus of the ALPM project to provide a broad C-API.
However, in the future an alternative [libalpm(3)] implementation based on ALPM libraries could be created to offer this subset of the project's functionality.

## How can I contribute?

We are happy about contributions!

If you are a first time contributor, have a look at the [contributing guidelines].
Easy first tasks are labeled as ["good first issue"].

## Who is working on this project?

The current members of the project are listed in the [GitLab organization].

## How can I get in touch?

In case you want to reach out to the project developers, use one of the several ways to [get in touch].

["good first issue"]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/?sort=closed_at_desc&state=opened&label_name%5B%5D=hint%3A%3Agood-first-issue&first_page_size=20
[Apache-2.0]: https://spdx.org/licenses/Apache-2.0.html
[GitLab organization]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/project_members
[MIT]: https://spdx.org/licenses/MIT.html
[PKGBUILD(5)]: https://man.archlinux.org/man/PKGBUILD.5
[Rust]: https://rust-lang.org/
[SRCINFO(5)]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[Sovereign Tech Agency]: https://www.sovereign.tech/tech/arch-linux-package-management
[VOA]: https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/
[`makepkg(8)`]: https://man.archlinux.org/man/makepkg.8
[`pacman(8)`]: https://man.archlinux.org/man/pacman.8
[`repo-add(8)`]: https://man.archlinux.org/man/repo-add.8
[contributing guidelines]: https://alpm.archlinux.page/CONTRIBUTING.html
[get in touch]: https://alpm.archlinux.page/community.html
[libalpm(3)]: https://man.archlinux.org/man/libalpm.3
[reuse configuration]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/blob/main/REUSE.toml
