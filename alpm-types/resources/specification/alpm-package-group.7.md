# NAME

package group - package grouping for ALPM based packages.

# DESCRIPTION

**Package groups** describe arbitrary grouping for ALPM based packages.
They are used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **PKGINFO** or **SRCINFO**) to describe the groups a package belongs to.
**Package groups** can be used for a number of reasons (e.g. "a group of all providers of a feature", "a group of all text editors") and no specific guidelines exist.
Software such as package managers can rely on **package groups** to list grouped packages and install as well as uninstall them in bulk.

The value is represented by a UTF-8 string (e.g. `my-group` or `my-other-group`).
Although it is possible to use a UTF-8 string, it is recommended limit the value to the **alpm-package-name** format.
Package managers may use a **package group** to install an entire group of packages and group names containing special characters or whitespaces may be confusing to users.

## Visibility of groups

**Package groups** solely exist as package metadata and any package can be in zero or more of them.
In the context of package management, **package groups** represent identifiers, that are visible across available package repositories, as they are defined by their members.

## Caveats when relying on groups for installation

**Package groups** can be used by package managers to bulk install all ALPM based packages that are currently members of a given group.
Although in effect similar to **meta packages** when it comes to the bulk installation of packages, **package groups** are not **package relations** and are solely defined as a grouping mechanism!
When further packages are added to a given group, this does not result in them being installed automatically, as package management software merely evaluates the members of a group during a given installation action.
Aside from reinstalling all members of a group, there is no mechanism to stay in sync with the current members of a **package group**.

# EXAMPLES

The following **PKGBUILD** example defines a package that belongs to the group `my-group`.
Note that the created package carries the group name in its metadata (see **PKGINFO** for details).

```bash
pkgname=group-example
pkgver=0.1.0
pkgrel=1
pkgdesc="A package example"
arch=(any)
url="https://archlinux.org"
license=('GPL-3.0-or-later')
groups=(my-group)
depends=(bash)

package() {
  install -vdm 755 "$pkgdir/usr/share/doc/$pkgname/"
  touch "$pkgdir/usr/share/doc/$pkgname/test.txt"
}
```

# SEE ALSO

**PKGBUILD**(5), **PKGINFO**(5), **alpm-meta-package**(7), **alpm-package-name**(7), **alpm-package-relation**(7)
