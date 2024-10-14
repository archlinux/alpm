# NAME

BUILDINFO - Information on package build environments for ALPM based packages (version 1).

# DESCRIPTION

This manual page describes the format of a BUILDINFO version 1 file found in the root of an ALPM based package.
The file contains a description of the package's build environment.

The information is formatted in key-value pairs separated by a **' = '**, one value per line (e.g. *"foo = bar"*).
Arrays are represented by multiple keys of the same name.

This is a description of the allowed keys and the format of their respective values in a BUILDINFO version 1 file.
For further details see **EXAMPLES** section.

**format** Denotes the file format version, represented by a plain positive integer. This must be **1** for BUILDINFOv1.

**pkgname** The name of a package.

**pkgbase** The base name of a package, usually the same as the pkgname except for split packages.

**pkgver** The full version of a package, formatted as "$epoch:$pkgver-$pkgrel".

**pkgarch** The CPU architecture of a package.

**pkgbuild_sha256sum** The hex representation of the SHA-256 checksum of the PKGBUILD used to build a package.

**packager** The User ID of the packager, that built a package.

**builddate** The build date of a package in Unix time (seconds since the epoch).

**builddir** The absolute directory in which a package has been built.

**buildenv (array)** The build environment used by the package build tool when building the package. A buildenv may be a word, optionally prefixed by a single *!*.

**options (array)** The options used by the package build tool when building the package. An option may be a word, optionally prefixed by a single *!*.

**installed (array)** Information on the packages installed when building a package, formatted as "$pkgname-$pkgver-$pkgrel-$pkgarch".

# EXAMPLES

```
format = 1
pkgname = foo
pkgbase = foo
pkgver = 1:1.0.0-1
pkgarch = any
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
packager = Foobar McFooface <foobar@mcfooface.org>
builddate = 1
builddir = /build
buildenv = !color
buildenv = check
options = !strip
options = staticlibs
installed = bar-1:0.5.0-3-any
installed = beh-2.1.0-6-x86_64
```

# SEE ALSO

alpm-buildinfo(1), makepkg(8), pacman(8), makepkg.conf(5)
