# alpm-pkgbuild

A library and command line tool to interact with [PKGBUILD] files used in **A**rch **L**inux **P**ackage **M**anagement (ALPM).

A [PKGBUILD] file is a bash script, that describe all necessary steps and data for creating an [alpm-package].
It contains metadata and instructions that may describe a single [alpm-package], an [alpm-meta-package], or one or more [alpm-split-packages], built for potentially multiple architectures.

This crate contains functionality to extract relevant metadata from a [PKGBUILD] file and convert it to a [SRCINFO] file.
The [SRCINFO] file creation depends on the [`alpm-pkgbuild-bridge`] script and package.
Make sure to install it beforehand or have it somewhere in your `$PATH`.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_pkgbuild/> for development version of the crate.
- <https://docs.rs/alpm-pkgbuild/latest/alpm_pkgbuild/> for released versions of the crate.

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

cp tests/test_files/normal.pkgbuild "$PKGBUILD_IN"
```
-->

The following command takes a **PKGBUILD** file and outputs a **.SRCINFO** from the extracted metadata.

```bash
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

### Library

[PKGBUILD]: https://man.archlinux.org/man/PKGBUILD.5
[SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
[`alpm-pkgbuild-bridge`]: https://gitlab.archlinux.org/archlinux/alpm/alpm-pkgbuild-bridge
[alpm-package]: https://alpm.archlinux.page/specifications/alpm-package.7.html
[alpm-meta-package]: https://alpm.archlinux.page/specifications/alpm-meta-package.7.html
[alpm-split-packages]: https://alpm.archlinux.page/specifications/alpm-split-package.7.html
