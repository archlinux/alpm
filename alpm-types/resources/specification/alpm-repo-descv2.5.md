# NAME

alpm-repo-desc - File format for the metadata representation of ALPM based packages in package repositories (version 2).

# DESCRIPTION

The **alpm-repo-desc** format is a textual format that represents the metadata of a single package in a specific version in a repository.
Each file describes various properties of an **alpm-package** file, including its name, version, dependencies, size, licensing information, and more.
For a full list of properties refer to its **sections**.

The accumulation of all **alpm-repo-desc** files in an **alpm-repo-db** describe the current state of the repository (i.e. which packages are available in what version).

An operating system relying on **A**rch **L**inux **P**ackage **M**anagement maintains one or more **alpm-repo-db** files which each contain one **alpm-repo-desc** file per package, each named _desc_ and located in a unique, per-package directory.
More specifically, package management software such as **pacman** and related tools use this file format e.g. to resolve dependencies and to display package information of **alpm-package** files in repositories.

The data in an **alpm-repo-desc** file is derived from an **alpm-package**.
Much of its metadata originates from the package's **PKGINFO** data.
An **alpm-repo-desc** file is usually created by software managing an **alpm-repo**, such as **repo-add**.

The file format described in this document must not be confused with the **alpm-db-desc** file format, which is used in the context of a **libalpm** database and carries a different set of metadata, but is usually also named _desc_.

The **alpm-repo-desc** format exists in multiple versions.
The information in this document is for version 2, which is the current version and has been introduced with the release of pacman 6.1.0 on 2024-03-04.

## General Format

An **alpm-repo-desc** file is a UTF-8 encoded, newline-delimited file consisting of a series of **sections**.

Each section starts with a unique _section header line_.
The section header line is followed by one or more lines with a _section-specific value_ each.
All _section-specific values_ must consist of **printable ASCII characters**[1] unless stated otherwise.
A section ends when another section header line is encountered or the end of the file is reached.

Empty lines between sections are ignored.

## Changes since the last version

The section `%MD5SUM%` has been removed.
The section `%PGPSIG%` has been made optional.

## Sections

Each _section header line_ contains the _section name_ in all capital letters, surrounded by percent signs (e.g. `%NAME%`).
_Section names_ serve as key for each _section-specific value_.

Each section allows for a single _section-specific value_, following the _section header line_.
As exemption to this rule the `%LICENSE%`, `%GROUPS%`, `%CHECKDEPENDS%`, `%DEPENDS%`, `%OPTDEPENDS%`, `%REPLACES%`, `%CONFLICTS%` and `%PROVIDES%` sections may have more than one _section-specific value_.

### %FILENAME%

The file name of the **alpm-package** (e.g. `example-1.2.3-any.pkg.tar.zst`).

### %NAME%

An **alpm-package-name**, which represents the name of a package (e.g. `example`).

### %BASE%

An **alpm-package-base** which represents the package base from which a package originates.
If a package is not an **alpm-split-package**, the value is the same as that of the `%NAME%` section.
If a package is an **alpm-split-package**, the value may be different from that in the `%NAME%` section.

### %VERSION%

An **alpm-package-version** (_full_ or _full with epoch_) which represents the version of a package (e.g. `1.0-1`).

### %DESC%

The description of the package.
The value is a UTF-8 string, zero or more characters long (e.g. `A project used for something`).

### %GROUPS%

An **alpm-package-group** that denotes a distribution-wide group the package is in.
One or more values may be present.
If the package is not in a group, the section is omitted.

The value is represented by a UTF-8 string.
Although it is possible to use a UTF-8 string, it is highly recommended to rely on the **alpm-package-name** format for the value instead, as package managers may use a package group to install an entire group of packages.

### %CSIZE%

The size of the compressed **alpm-package** in bytes.
The value is a non-negative integer (e.g. `1818463`).

### %ISIZE%

The size of the (uncompressed and unpacked) package contents in bytes.
The value is a non-negative integer representing the absolute size of the contents of the package, with multiple hardlinked files counted only once (e.g. `181849963`).

### %SHA256SUM%

The **SHA-256**[2] hash digest of the **alpm-package** file.
The string consists of 64 hexadecimal characters (e.g. `b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c`).

### %PGPSIG%

The optional **base64**[3] encoded **OpenPGP signature**[4] for the **alpm-package** file (e.g. `iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=`).
The tool for creating the **alpm-repo-desc** file may choose to not include the signature representation, in which case the section is omitted.

### %URL%

The URL for the project of the package.
The value is a valid URL or an empty string (e.g. `https://example.org`).

### %LICENSE%

An optional set of license identifiers that apply for the package.
One or more values may be present.
If there is no license identifier, the section is omitted.

Each value represents a license identifier, which is a string of non-zero length (e.g. `GPL`).
Although no specific restrictions are enforced for the value aside from its length, it is highly recommended to rely on SPDX license expressions (e.g. `GPL-3.0-or-later` or `Apache-2.0 OR MIT`).
See **SPDX License List**[5] for further information.

### %ARCH%

The architecture of the package (see **alpm-architecture** for further information).
The value must be covered by the set of alphanumeric characters and '\_' (e.g. `x86_64` or `any`).

### %BUILDDATE%

The date at which the build of the package started.
The value must be numeric and represent the seconds since the Epoch, aka. 'Unix time' (e.g. `1729181726`).

### %PACKAGER%

The User ID of the entity, that built the package.
The value is meant to be used for identity lookups and represents an **OpenPGP User ID**[6].
As such, the value is a UTF-8-encoded string, that is conventionally composed of a name and an e-mail address, which aligns with the format described in **RFC 2822**[7] (e.g. `John Doe <john@example.org>`).

### %REPLACES%

Another _virtual component_ or _package_, that the package replaces upon installation.
One or more values may be present.
If the package does not replace anything, the section is omitted.
The value is an **alpm-package-relation** of type **replacement** (e.g. `example` or `example=1.0.0`).

### %CONFLICTS%

Another _virtual component_ or _package_, that the package conflicts with.
One or more values may be present.
If the package does not conflict with anything, the section is omitted.
The value is an **alpm-package-relation** of type **conflict** (e.g. `example` or `example=1.0.0`).

### %PROVIDES%

Another _virtual component_ or _package_, that the package provides.
One or more values may be present.
If the package does not provide anything, the section is omitted.
The value is an **alpm-package-relation** of type **provision** (e.g. `example` or `example=1.0.0`).

### %DEPENDS%

A run-time dependency of the package (_virtual component_ or _package_).
One or more values may be present.
If the package has no run-time dependency, the section is omitted.
The value is an **alpm-package-relation** of type **run-time dependency** (e.g. `example` or `example=1.0.0`).

### %OPTDEPENDS%

An optional dependency of the package (`virtual component` or `package`).
One or more values may be present.
If the package has no optional dependency, the section is omitted.
The value is an **alpm-package-relation** of type **optional dependency** (e.g. `example` or `example: this is a description`).

### %MAKEDEPENDS%

A dependency for building the upstream software of the package.
One or more values may be present.
If the package has no build dependency, the section is omitted.
The value is an **alpm-package-relation** of type **build dependency** (e.g. `example` or `example=1.0.0`).

### %CHECKDEPENDS%

A dependency for running tests of the package's upstream project.
One or more values may be present.
If the package has no test dependency, the section is omitted.
The value is an **alpm-package-relation** of type **test dependency** (e.g. `example` or `example=1.0.0`).

# EXAMPLES

A full example of an **alpm-repo-desc** file for a package named `example` in version `1.0.0-1`:

```text
%FILENAME%
example-1.0.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%GROUPS%
example-group
other-group

%CSIZE%
1818463

%ISIZE%
18184634

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%
https://example.org

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%REPLACES%
other-pkg-replaced

%CONFLICTS%
other-pkg-conflicts

%PROVIDES%
example-component

%DEPENDS%
glibc
gcc-libs

%OPTDEPENDS%
bash: for a script

%MAKEDEPENDS%
cmake

%CHECKDEPENDS%
bats
```

A minimal example of an **alpm-repo-desc** file for an **alpm-meta-package** named `example-meta` in version `1.0.0-1`:

```text
%FILENAME%
example-meta-1.0.0-1-any.pkg.tar.zst

%NAME%
example-meta

%BASE%
example-meta

%VERSION%
1.0.0-1

%DESC%
An example meta package

%CSIZE%
4634

%ISIZE%
0

%SHA256SUM%
b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c

%URL%
https://example.org

%LICENSE%
GPL-3.0-or-later

%ARCH%
any

%BUILDDATE%
1729181726

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>
```

# SEE ALSO

**libalpm**(3), **BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **alpm-db-desc**(5), **alpm-architecture**(7), **alpm-meta-package**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **alpm-package-version**(7), **alpm-repo-db**(7), **alpm-split-package**(7), **pacman**(8), **repo-add**(8)

# NOTES

1. printable ASCII characters
   
   <https://en.wikipedia.org/wiki/ASCII#Printable_characters>
1. SHA-256
   
   <https://en.wikipedia.org/wiki/SHA-2>
1. Base64
   
   <https://en.wikipedia.org/wiki/Base64>
1. OpenPGP signature
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
1. SPDX License List
   
   <https://spdx.org/licenses/>
1. OpenPGP User ID
   
   <https://openpgp.dev/book/certificates.html#user-ids>
1. RFC 2822
   
   <https://www.rfc-editor.org/rfc/rfc2822>
