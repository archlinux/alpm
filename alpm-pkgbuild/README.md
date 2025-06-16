# alpm-pkgbuild

A library and commandline toolkit to interact with [`PKGBUILD`] files used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

`PKGBUILD` files are the fundamental file format that's used to actually package files.
It contains metadata and instructions on how split-/packages are to be built for potentially multiple architectures.

This crate also contains functionality to extract such metadata into the [`SRCINFO`] file format.
The `.SRCINFO` file creation depends on the [`alpm-pkgbuild-bridge`] script and package, make to install it beforehand or have it somewhere in your `$PATH`.

## Examples

<!--
```bash
# Create a temporary directory for testing.
test_tmpdir="$(mktemp --directory --suffix '.')"

# Get a random temporary file location in the created temporary directory.
PKGBUILD_IN="$test_tmpdir/PKGBUILD"
SRCINFO_OUT="$test_tmpdir/SRCINFO"
export PKGBUILD_IN
export SRCINFO_OUT
```
-->

```bash
cp tests/test_files/normal.pkgbuild "$PKGBUILD_IN"
alpm-pkgbuild srcinfo format "$PKGBUILD_IN" > "$SRCINFO_OUT"
```

<!--
Make sure the generated SRCINFO file is as expected.
```bash
cat > "$SRCINFO_OUT.expected" <<EOF
pkgbase = example
	pkgdesc = A example with all pkgbase properties set.
	pkgver = 0.1.0
	pkgrel = 1
	epoch = 1
	url = https://archlinux.org/
	install = install.sh
	changelog = changelog
	arch = x86_64
	arch = aarch64
	groups = group
	groups = group_2
	license = MIT
	depends = default_dep
	optdepends = default_optdep
	provides = default_provides
	conflicts = default_conflict
	replaces = default_replaces
	options = !lto
	backup = etc/pacman.conf
	provides_x86_64 = arch_default_provides
	conflicts_x86_64 = arch_default_conflict
	depends_x86_64 = arch_default_dep
	replaces_x86_64 = arch_default_replaces
	optdepends_x86_64 = arch_default_optdep

pkgname = example

EOF

diff --ignore-trailing-space "$SRCINFO_OUT" "$SRCINFO_OUT.expected"
```
-->

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkgbuild/> for development version of the crate.
- <https://docs.rs/alpm-pkgbuild/latest/alpm_pkgbuild/> for released versions of the crate.

## Examples

### Library

[`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
[PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
