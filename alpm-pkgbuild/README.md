# alpm-pkgbuild

A library and commandline toolkit to interact with `PKGBUILD` files used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

`PKGBUILD` files are the fundamental file format that's used to actually package files.
It contains metadata and instructions on how split-/packages are to be built for potentially mutliple architectures.

This crate also contains functionality to extract such metadata into the `.SRCINFO` file format.
The `.SRCINFO` creation depends on the [`alpm-pkgbuild-bridge.sh`] script, make to install it beforehand or have it somewhere in your `$PATH`.


## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkgbuild/> for development version of the crate.
- <https://docs.rs/alpm-pkgbuild/latest/alpm_pkgbuild/> for released versions of the crate.

## Examples

### Library



[`alpm-pkgbuild-bridge.sh`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
