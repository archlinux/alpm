# NAME

meta package - an ALPM based package that solely defines **package relations**.

# DESCRIPTION

**Meta packages** refer to ALPM based packages that do not provide files, but instead only define **package relations**.
They are used for defining the required **package relations** of an abstract scenario or use-case (e.g. "packages for a minimum system installation" or "all packages needed for a special development environment").

**Meta packages** are handled like any other ALPM based package by a package manager and require their various **package relations** upon installation.
A `-meta` suffix may be used in the **alpm-package-name** to more easily distinguish **meta packages** from other packages.

# EXAMPLES

The following **PKGBUILD** example defines a **meta package**, that upon installation pulls in the `bash` and `gcc-libs` packages:

```bash
pkgname=example-meta
pkgver=0.1.0
pkgrel=1
pkgdesc="A meta package example"
arch=(any)
url="https://archlinux.org"
license=('GPL-3.0-or-later')
depends=(
  bash
  gcc-libs
)
```

# SEE ALSO

**PKGBUILD**(5), **PKGINFO**(5), **alpm-package-name**(7), **alpm-package-relation**(7)
