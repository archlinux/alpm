# FAQ

## What is ALPM?

A software framework and set of documentation for working with **A**rch **L**inux **P**ackage **M**anagement.

## Is ALPM a package manager?

No, but you could build one with it.

## Can I build packages with ALPM?

Yes, but only part of the package build process is currently possible.
ALPM does not offer functionality for building a package from a [PKGBUILD] file, but it allows creating a package file from a prepared input directory.

## Why or when would I use ALPM?

If your project deals with Arch Linux packages, repository sync databases, or specific metadata files (e.g. [SRCINFO]).

ALPM provides a variety of libraries and command-line interfaces, that enable the creation and validation of metadata files used in the Arch Linux ecosystem.

## Under what license is ALPM released?

ALPM is free and open source and for the most part can be used under the the terms of the [Apache-2.0] or [MIT].
For details refer to the project's [reuse configuration].

## What's the timeline for the current development phase?

Currently, development on the ALPM project is [supported by the Sovereing Tech Fund] (STF) until the end of 2025.

This funding encompasses a specific set of functionality.
See next question for a more detailed overview of topics.

## What is the scope of the current development phase of the project?

Until the end of 2025, we will:

- complete our set of central specification documents, covering all relevant file formats and concepts,
- complete our set of libraries that implement support for these file formats and concepts,
- create a drop-in replacement C library for libalpm,
- create Python bindings for the validation of the [SRCINFO] format,
- and implement package signature verification based on [VOA].

## What are the future plans for the project?

Our plan is to:

- further improve the documentation of the Arch Linux packaging ecosystem,
- help integrate ALPM into further projects,
- and extend the Python bindings.

## What is the difference between the Pacman project and ALPM?

Pacman is a project that provides the [`makepkg`], [`repo-add`] and [`pacman`] executables and the shared library [libalpm].

ALPM is a highly modular "library first" framework, that provides reimagined building blocks for package management tools.

### What is the difference between the `pacman` CLI tool and ALPM?

The [`pacman`] CLI tool is a system package manager written in C.
It allows to install/uninstall/upgrade/downgrade packages on a system and retrieve metadata from packages on a system or from repository metadata.

The ALPM project does not provide a tool equivalent in functionality to the [`pacman`] executable.

### What is the difference between the `makepkg` CLI tool and ALPM?

The [`makepkg`] CLI tool is a Bash-based script to create package files from package build scripts (i.e. [PKGBUILD]).

The ALPM project does not provide a tool equivalent in functionality to the [`makepkg`] executable.

### What is the difference between the shared library `libalpm` and ALPM?

The shared library `libalpm` is written in C and provides functionality for managing system state based on a local package database on end-user machines.
It provides low-level access to actions such as installation/uninstallation/upgrade/downgrade of packages and the retrieval of metadata to its consumers (e.g. [`pacman`]).

The ALPM project reimplements various building blocks that are required for system management.
It will provide an alternative implementation of the [libalpm] API, based on strong validation and strict typing.

## Is ALPM based on Pacman?

While ALPM is based on Pacman in spirit, its codebase is an independent effort.

ALPM is a project that reimagines the building blocks of Arch Linux Package Management from first principles.
ALPM attempts to provide perfect interoperability with the existing package management tooling (including the Pacman project), while relying on entirely new implementations based on specifications of the existing formats.

## Can I use ALPM with Python?

The ALPM project will create Python bindings for its library functionality.
The goal is to provide a Python API with a broad scope, that can be used in a diverse set of contexts.

## Can I use ALPM with C?

Currently, it is not a focus of the ALPM project to provide a broad C-API.
However, the alternative [libalpm] implementation based on ALPM libraries will offer a subset of its functionality.

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
[PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[VOA]: https://uapi-group.org/specifications/specs/file_hierarchy_for_the_verification_of_os_artifacts/
[`makepkg`]: https://man.archlinux.org/man/makepkg.8
[`pacman`]: https://man.archlinux.org/man/pacman.8
[`repo-add`]: https://man.archlinux.org/man/repo-add.8
[contributing guidelines]: https://alpm.archlinux.page/CONTRIBUTING.html
[get in touch]: https://alpm.archlinux.page/community.html
[libalpm]: https://man.archlinux.org/man/libalpm.3
[reuse configuration]: https://gitlab.archlinux.org/archlinux/alpm/alpm/-/blob/main/REUSE.toml
[supported by the Sovereign Tech Fund]: https://www.sovereign.tech/tech/arch-linux-package-management
