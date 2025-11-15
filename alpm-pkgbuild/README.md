# alpm-pkgbuild

A library to interact with [PKGBUILD] files used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

A [PKGBUILD] file is a bash script, that describe all necessary steps and data for creating an [alpm-package].
It contains metadata and instructions that may describe a single [alpm-package], an [alpm-meta-package], or one or more [alpm-split-packages], built for potentially multiple architectures.

This crate contains functionality to extract relevant metadata from a [PKGBUILD] file and convert it to a [SRCINFO] file.
The [SRCINFO] file creation depends on the [`alpm-pkgbuild-bridge`] script and package.
Make sure to install it beforehand or have it somewhere in your `$PATH`.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkgbuild/> for development version of the crate.
- <https://docs.rs/alpm-pkgbuild/latest/alpm_pkgbuild/> for released versions of the crate.

## Where is this used?

This crate is intended solely for use by the `alpm-srcinfo` crate.
`alpm-pkgbuild` produces an intermediate representation of a PKGBUILD file, which is then handled and converted into a proper `SourceInfoV1` struct by the `alpm-srcinfo` crate.

As `alpm-pkgbuild` is designed to be used in conjunction with the `alpm-srcinfo` crate, the tests for the bridge logic of this crate also live in the `alpm-srcinfo` project.

[PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
[alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
[alpm-meta-package]: https://alpm.archlinux.page/specifications/alpm-meta-package.7.html
[alpm-split-packages]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
