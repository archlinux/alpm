# NAME

BUILDINFO - Information on package build environments for ALPM based packages (version 1).

# DESCRIPTION

The **BUILDINFO** format is a textual format that describes a package's build environment.
Such files are located at the root of ALPM packages, are named **.BUILDINFO** and are usually used to reproduce the environment in which a package has been build.
For further information refer to **Arch Linux's reproducible builds effort**[1].

The **BUILDINFO** format exists in multiple versions.
The document describes version 1, which is a legacy version and has been introduced with the release of pacman 5.1.0 on 2018-05-28.
For the latest specification, refer to **BUILDINFO**.

## General Format

A **BUILDINFO** file consists of a series of lines, each providing information on an aspect of the build environment of a package, or the file format itself.
Leading whitespace is always ignored.

Unless noted otherwise, the information contained in a **BUILDINFO** file is considered to be covered by the set of the 95 printable ASCII characters.

## Keywords

The information encoded on each line is represented by a single keyword definition.
Each such definition consists of a key from the following list immediately followed by a whitespace, an '=' sign, another whitespace and a value.

By default, the below keyword definitions must be used once per **BUILDINFO**.
As exception to this rule, the keywords **buildenv**, **options** and **installed** may be provided zero or more times.

### format

The **BUILDINFO** file format version.
The value must be a plain positive integer.
This must be **1** for **BUILDINFO** version 1.

### pkgname

The name of the package.
The value is an **alpm-package-name** (e.g. `example`).

### pkgbase

The base name of the package.
This keyword reflects the name of the sources from which the package is built.
If the sources of the package are used to build a single package, the value is the same as that of **pkgname**.
If the sources of the package are used to build several packages, the value may differ from that of **pkgname** (see **PKGBUILD** **PACKAGE SPLITTING** for further information).
The value is covered by the same rules as that of **pkgname** (e.g. `example`).

### pkgver

The full version of the package.
Note, that this is not to be confused with **alpm-pkgver**, which only represents a subset of this keyword!
The value is an **alpm-package-version**, either in *full* or in *full with epoch* form (e.g. `1.0.0-1` or `1:1.0.0-1`, respectively).

### pkgarch

The architecture of the package (see **alpm-architecture** for further information).
The value must be covered by the set of alphanumeric characters and '_' (e.g. `x86_64` or `any`).

### pkgbuild_sha256sum

The hex representation of the SHA-256 checksum of the **PKGBUILD** used to build the package.
The value must be covered by the set of hexadecimal characters and must be 64 characters long (e.g. `946d8362de3cebe3c86765cb36671a1dfd70993ac73e12892ac7ac5e6ff7ef95`).

### packager

The User ID of the entity, that built the package.
The value is meant to be used for identity lookups and represents an **OpenPGP User ID**[2].
As such, the value is a UTF-8-encoded string, that is conventionally composed of a name and an e-mail address, which aligns with the format described in **RFC 2822**[3] (e.g. `John Doe <john@example.org>`).

### builddate

The date at which the build of the package started.
The value must be numeric and represent the seconds since the Epoch, aka. 'Unix time' (e.g. `1729181726`).

### builddir

The absolute directory path in which the package has been built by the build tool (e.g. `makepkg`).
The value is a UTF-8-encoded string and must represent a valid absolute directory (e.g. `/builddir`).

### buildenv

A build environment used by the package build tool (i.e. `makepkg`, defined in `BUILDENV` of makepkg.conf) when building the package.
This keyword definition may be provided zero or more times.
The value must be a unique word, optionally prefixed by a single '!', which indicates the negation of the environment (e.g. `color` or `!color`).

### options

An option used by the package build tool (i.e. `makepkg`, defined in `OPTIONS` of makepkg.conf) when building the package.
This keyword definition may be provided zero or more times.
The value must be a unique word, optionally prefixed by a single '!', which indicates the negation of the option (e.g. `debug` or `!debug`).

### installed

The information about an installed package during build time of the package.
This keyword definition may be provided zero or more times.
The value represents a composite string, composed of an **alpm-package-name**, directly followed by a '-' sign, directly followed by an **alpm-package-version** (in *full* or in *full in epoch* form), directly followed by a '-' sign, followed by an **alpm-architecture** (e.g. `example-1:1.0.0-1-x86_64`).

# EXAMPLES

```ini
format = 1
pkgname = example
pkgbase = example
pkgver = 1:1.0.0-1
pkgarch = any
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
packager = John Doe <john@example.org>
builddate = 1729181726
builddir = /build
buildenv = !color
buildenv = check
options = !strip
options = staticlibs
installed = other-package-1:0.5.0-3-any
installed = package2-2.1.0-6-x86_64
```

# SEE ALSO

alpm-buildinfo(1), makepkg.conf(5), PKGBUILD(5), alpm-architecture(7), alpm-package-name(7), alpm-package-version(7), alpm-pkgver(7), makepkg(8), pacman(8)

# NOTES

1. **Arch Linux's Reproducible Builds effort**

   https://wiki.archlinux.org/title/Reproducible_builds

2. **OpenPGP User ID**

   https://openpgp.dev/book/certificates.html#user-ids

3. **RFC 2822**

   https://www.rfc-editor.org/rfc/rfc2822
