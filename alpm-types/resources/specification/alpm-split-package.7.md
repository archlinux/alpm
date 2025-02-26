# NAME

split package - an ALPM based package that is built along other packages from the same package sources.

# DESCRIPTION

Split packages refer to two or more ALPM based packages that are built from the same package sources.
These types of packages can be created for arbitrary reasons, e.g.:

- split different types of functionality (e.g. one package providing the graphical user interface and another the terminal user interface of a project)
- split out specific documentation (e.g. developer documentation is not useful for all users)
- various functionalities of a project are optional, are e.g. provided as plugins and can be enabled by installing additional split packages

Split packages built from the same package sources share some **PKGINFO** package metadata (i.e. **pkgbase**, **pkgver**, **makedepends**), while other metadata may be set specifically per individual split package.

All **PKGINFOv2** files for split packages define one **xdata** value of `pkgtype=split`, indicating that the metadata describes a split package.

Split packages are handled like any other ALPM based package by a package manager and require their various package relations upon installation.

# EXAMPLES

The following **PKGBUILD** example defines two split packages, that share `example-split` as (virtual - without any actual package of that name) **pkgbase**:

```bash
pkgbase=example-split
pkgname=(example-a example-b)
pkgver=0.1.0
pkgrel=1
pkgdesc="A split package example"
arch=(any)
url="https://archlinux.org"
license=(GPL-3.0-or-later)
makedepends=(
  meson
)
depends=(
  bash
  gcc-libs
)

package_example-a(){
  pkgdesc+=" - A"
  license+=(LGPL-2.1-or-later)
  depends+=(bash-completions)
}

package_example-b(){
  pkgdesc+=" - B"
  depends=(zsh)
}
```

# SEE ALSO

**PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-package-name**(7), **alpm-package-relation**(7)

