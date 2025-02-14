# NAME

PKGINFO - Information on ALPM based packages (version 1).

# DESCRIPTION

The **PKGINFO** format is a textual format that describes package metadata.
Such files are located at the root of ALPM packages and are named **.PKGINFO**.
They are used e.g. by package managers to evaluate and present the context for a package within its ecosystem.
Use-cases include the representation of common metadata and the relation to other packages.

The **PKGINFO** format exists in multiple versions.
This document describes version 1, which is a legacy version and has been introduced with the release of pacman 5.1.0 on 2018-05-28.
For the latest specification, refer to **PKGINFO**.

## General Format

A **PKGINFO** file consists of a series of lines, each providing information on an aspect of a package.
Lines starting with a '#' sign are comments and are always ignored.
Leading whitespace is always ignored.

Unless noted otherwise, the information contained in a **PKGINFO** file is considered to be covered by the set of the 95 **printable ASCII characters**[1].

## Keywords

Each line encodes information that represents one keyword assignment.
All keyword assignments consist of a key from the following list immediately followed by a whitespace, an '=' sign, another whitespace and a value.

By default, exactly one keyword assignment must be present per keyword in a **PKGINFO**.
As exception to this rule, the keywords **license**, **replaces**, **group**, **conflict**, **provides**, **backup**, **depend**, **optdepend**, **makedepend** and **checkdepend** may be provided zero or more times.

### pkgname

The name of the package.
The value is an **alpm-package-name** (e.g. `example`).

### pkgbase

The base name of the package.
This keyword reflects the name of the sources from which the package is built.
If the sources of the package are used to build a single package, the value is the same as that of **pkgname**.
If the sources of the package are used to build several packages, the value may differ from that of **pkgname** (see **PKGBUILD** **PACKAGE SPLITTING** for further information).
The value is an **alpm-package-name** (e.g. `example`).

### pkgver

The full version of the package.
Note, that this is not to be confused with **alpm-pkgver**, which only represents a subset of this keyword!
The value is an **alpm-package-version**, either in *full* or in *full with epoch* form (e.g. `1.0.0-1` or `1:1.0.0-1`, respectively).

### pkgdesc

The description of the package.
The value is a UTF-8 string, zero or more characters long (e.g. `A project used for something`).
No specific rules about the value exist, but it is suggested to be "short" and to not contain the package name (see **pkgname**).

### url

The URL for the project of the package.
The value is a valid URL or an empty string (e.g. `https://example.org`).

### builddate

The date at which the build of the package started.
The value must be numeric and must represent the seconds since the Epoch, aka. 'Unix time' (e.g. `1729181726`).

### packager

The User ID of the entity that built the package.
The value is meant to be used for identity lookups and represents an **OpenPGP User ID**[2].
As such, the value is a UTF-8-encoded string, that is conventionally composed of a name and an e-mail address, which aligns with the format described in **RFC 2822**[3] (e.g. `John Doe <john@example.org>`).

### size

The size of the (uncompressed and unpacked) package contents in bytes.
The value is a non-negative integer representing the absolute size of the contents of the package, with multiple hardlinked files counted only once (e.g. `181849963`).

### arch

The architecture of the package (see **alpm-architecture** for further information).
The value must be covered by the set of alphanumeric characters and '_' (e.g. `x86_64` or `any`).

### license

A license that applies for the package.
This keyword may be assigned zero or more times.
The value represents a license identifier, which is a string of non-zero length (e.g. `GPL`).
Although no specific restrictions are enforced for the value aside from its length, it is highly recommended to rely on SPDX license expressions (e.g. `GPL-3.0-or-later` or `Apache-2.0 OR MIT`). See **SPDX License List**[4] for further information.

### replaces

Another *virtual component* or package, that the package replaces upon installation.
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **replacement** (e.g. `example` or `example=1.0.0`).

### group

An arbitrary string, that denotes a distribution-wide group the package is in.
Groups are made use of e.g. by package managers to group packages and allow to bulk install them, or by other software to display information on these related packages.
This keyword may be assigned zero or more times.
The value is represented by a UTF-8 string.
Although it is possible to use a UTF-8 string, it is highly recommended to rely on the **pkgname** format for the value instead, as package managers may use **group** to install an entire group of packages.

### conflict

Another *virtual component* or package, that the package conflicts with.
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **conflict** (e.g. `example` or `example=1.0.0`).

### provides

Another *virtual component* or package, that the package provides.
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **provision** (e.g. `example` or `example=1.0.0`).

### backup

A relative file path of a file in the package, that denotes a file for the package manager to keep backups for in case it changes or is deleted during a package update action (see **pacman** '.pacnew' and '.pacsave' files).
This keyword may be assigned zero or more times.
The value must be a valid relative Unix file path (e.g. `etc/package.conf`).

### depend

A run-time dependency of the package (*virtual component* or package).
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **run-time dependency** (e.g. `example` or `example=1.0.0`).

### optdepend

An optional dependency of the package (*virtual component* or package).
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **optional dependency** (e.g. `example` or `example: this is a description`).

### makedepend

A build time dependency of the package (*virtual component* or package).
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **build dependency** (e.g. `example` or `example=1.0.0`).

### checkdepend

A dependency for running tests of the package's upstream project.
This keyword may be assigned zero or more times.
The value is an **alpm-package-relation** of type **test dependency** (e.g. `example` or `example=1.0.0`).

# EXAMPLES

```ini
pkgname = example
pkgbase = example
pkgver = 1:1.0.0-1
pkgdesc = A project that does something
url = https://example.org
builddate = 1729181726
packager = John Doe <john@example.org>
size = 181849963
arch = any
license = GPL-3.0-or-later
license = LGPL-3.0-or-later
replaces = other-package>0.9.0-3
group = package-group
group = other-package-group
conflict = conflicting-package<1.0.0
conflict = other-conflicting-package<1.0.0
provides = some-component
provides = some-other-component=1:1.0.0-1
backup = etc/example/config.toml
backup = etc/example/other-config.txt
depend = glibc
depend = gcc-libs
optdepend = python: for special-python-script.py
optdepend = ruby: for special-ruby-script.rb
makedepend = cmake
makedepend = python-sphinx
checkdepend = extra-test-tool
checkdepend = other-extra-test-tool
```

# SEE ALSO

alpm-buildinfo(1), makepkg.conf(5), PKGBUILD(5), alpm-architecture(7), alpm-comparison(7), alpm-package-name(7), alpm-package-relation(7), alpm-package-version(7), alpm-pkgver(7), makepkg(8), pacman(8)

# NOTES

1. **printable ASCII characters**

   https://en.wikipedia.org/wiki/ASCII#Printable_characters

2. **OpenPGP User ID**

   https://openpgp.dev/book/certificates.html#user-ids

3. **RFC 2822**

   https://www.rfc-editor.org/rfc/rfc2822

4. **SPDX License List**

   https://spdx.org/licenses/
