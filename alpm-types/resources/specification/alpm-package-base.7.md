# NAME

package base - an identifier that tracks from which sources an ALPM based package originates.

# DESCRIPTION

The **package base** format represents an identifier, that describes the source origin of each **alpm-package** file.
It is particularly useful in **alpm-split-package** files to track the sources from which each package file is built.

More specifically, the **alpm-package-base** format is used throughout the package management life cycle:

- in build scripts and source metadata files (i.e. **PKGBUILD** and **SRCINFO**)
- in file formats for package metadata (i.e. **BUILDINFO** and **PKGINFO**)
- in file formats for repository metadata (i.e. **alpm-repo-desc**)
- and in system state metadata (i.e. **alpm-db-desc**).

## General Format

The identifier name for **alpm-package-base** may differ depending on context.
However, the value restrictions for it are the same as those for **alpm-package-name**.

# EXAMPLES

The **PKGBUILD** of a package named `example`, that implicitly specifies its **alpm-package-base** as `example`.

```bash
pkgname=example
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package"
arch=(any)
url="https://example.org"
license=('CC0-1.0')

package() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "example\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}
```

The **PKGBUILD** of a package named `example`, that explicitly specifies its **alpm-package-base** as `something-else`.

```bash
pkgname=example
pkgbase=something-else
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package"
arch=(any)
url="https://example.org"
license=('CC0-1.0')

package() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "example\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}
```

The **PKGBUILD** of a split package setup with the packages `example` and `other-example` that both share the implicit **alpm-package-base** `example`.

```bash
pkgname=(
  example
  other-example
)
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package"
arch=(any)
url="https://example.org"
license=('CC0-1.0')

package_example() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "example\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}

package_other-example() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "other-example\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}
```

The **PKGBUILD** of a split package setup with the packages `example1` and `example2` that both share the explicit **alpm-package-base** `example`.

```bash
pkgname=(
  example1
  example2
)
pkgbase=example
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package"
arch=(any)
url="https://example.org"
license=('CC0-1.0')

package_example1() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "example1\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}

package_example2() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  printf "example2\n" > "$pkgdir/usr/share/doc/$pkgname/example.txt"
}
```

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-db-desc**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-repo-desc**(7), **alpm-split-package**(7)
