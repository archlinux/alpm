# NAME

package - an **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package.

# DESCRIPTION

ALPM based packages refer to (optionally compressed) **tar** archives that contain **metadata**, **scripts** and **payload** files.

Package files are built from package sources (i.e. **PKGBUILD**) using package build software (e.g. **makepkg**).
For Arch Linux specific package build software refer to **devtools** and **pkgctl**.

Package files can be installed on compatible target systems using package management software (e.g. **pacman**).

# FORMAT

_Uncompressed_ ALPM based package files follow the following naming scheme:

An **alpm-package-name** directly followed by a '-' sign, directly followed by an **alpm-package-version** (in the _full_ or _full with epoch_ form), directly followed by a '-' sign, directly followed by an **alpm-architecture**, directly followed by the string '.pkg.tar' (e.g. `example-1.0.0-1-x86_64.pkg.tar`).

## Compression

ALPM based packages may optionally be compressed using a number of compression technologies.
If a package is compressed, a technology-specific suffix is appended to the file name:

- `.bz2` for bzip2 compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.bz2`), see **bzip2**
- `.gz` for gzip compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.gz`), see **gzip**
- `.lz` for lzip compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.lz`), see **lzip**
- `.lz4` for lz4 compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.lz4`), see **lz4**
- `.lrz` for lrzip compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.lrz`), see **lrzip**
- `.lzo` for lzop compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.lzo`), see **lzop**
- `.xz` for xz compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.xz`), see **xz**
- `.Z` for lzw compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.Z`), see **compress**
- `.zst` for zstandard compression (e.g. `example-1.0.0-1-x86_64.pkg.tar.zst`), see **zstd**

Handling of compression technologies is specific to the package build tool.
Refer to **COMPRESSBZ2**, **COMPRESSGZ**, **COMPRESSLRZ**, **COMPRESSLZ**, **COMPRESSLZ4**, **COMPRESSLZO**, **COMPRESSXZ**, **COMPRESSZ**, **COMPRESSZST** and **PKGEXT** in **makepkg.conf** for compression options and package extensions.

## Digital signatures

Digital signatures can be created for package files.

Currently, only **OpenPGP detached signatures** over the package file are supported.
Detached signatures carry the same name as the package file for which they are created, with an addition `.sig` suffix (e.g. `example-1.0.0-1-x86_64.pkg.tar.zst.sig`).

# CONTENTS

The contents of an ALPM based package are distinguished between **metadata**, **scripts** and **payload** files.

## Metadata

The following files must be present at the root of an ALPM based package:

- `.BUILDINFO`: a **BUILDINFO** file
- `.MTREE`: an **ALPM-MTREE** file
- `.PKGINFO`: a **PKGINFO** file

The above files provide relevant metadata for the installation, upgrade and uninstallation of packages on a target system (see **alpm-package-relation** for details), as well as reproducibly building a bit-by-bit identical package from the same sources (see **reproducible builds**[2]).

## Scripts

Optionally, a file named `.INSTALL` (an **alpm-install-scriptlet**) may be present at the root of an ALPM based package.

## Payload

Zero or more payload files may be present in an ALPM based package (refer to **alpm-meta-package** for details on why packages may not contain files).

All existing payload files of a package are extracted to the **root directory**[3] of a target system upon installation of the package.

No specific rules exist on which files and directories are allowed as contents of a package, but it is advisable to adhere to common standards such as the **systemd File Hierarchy Requirements**[4] and/or the **Filesystem Hierarchy Standard**[5] and by default have all files and directories be owned by `root`.
In conclusion, it is advisable to never package files in directories that carry user data (e.g. `/home`).

# EXAMPLES

The following example **PKGBUILD** defines the package `example` which only creates a single **payload** file:

```bash
pkgname=example
pkgver=1.0.0
pkgrel=1
pkgdesc="A simple package example"
arch=(any)
url="https://example.org"
license=(GPL-3.0-or-later)

package(){
  install -vdm 755 "$pkgdir/usr/share/$pkgname/"
  echo "example" > "$pkgdir/usr/share/$pkgname/example.txt"
}
```

Assuming **makepkg** is used to build a package from above **PKGBUILD** and is configured to use **zstd** for compression, the resulting package file is called `example-1.0.0-1-any.pkg.tar.zst`.

The package file contents can be examined as follows:
```bash
tar -tf example-1.0.0-1-any.pkg.tar.zst
.BUILDINFO
.MTREE
.PKGINFO
usr/
usr/share/
usr/share/example/
usr/share/example/example.txt
```

# SEE ALSO

bzip2(1), compress(1), gzip(1), lrzip(1), lz4(1), lzip(1), lzop(1), pkgctl(1), tar(1), xz(1), zstd(1), ALPM-MTREE(5), BUILDINFO(5), PKGBUILD(5), PKGINFO(5), makepkg.conf(5), SRCINFO(7), alpm-architecture(7), alpm-install-scriptlet(7), alpm-meta-package(7), alpm-package-name(7), alpm-package-name(7), alpm-package-relation(7), alpm-package-version(7), alpm-split-package(7), devtools(7), makepkg(8), pacman(8)

# NOTES

1. **OpenPGP detached signatures**

   https://openpgp.dev/book/signing_data.html#detached-signatures

2. **reproducible builds**

   https://reproducible-builds.org/

3. **root directory**

   https://en.wikipedia.org/wiki/Root_directory

4. **systemd File Hierarchy Requirements**

   https://systemd.io/SYSTEMD_FILE_HIERARCHY_REQUIREMENTS/

5. **Filesystem Hierarchy Standard**

   https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard
