# NAME

alpm-db-desc - File format for the representation of metadata of ALPM based packages (version 2).

# DESCRIPTION

The **alpm-db-desc** format is a textual format that represents the metadata of a single package.
Each file describes various properties of a package, including its name, version, dependencies, size, licensing information, and more.
For a full list of properties refer to **sections**.

An operating system relying on **A**rch **L**inux **P**ackage **M**anagement maintains a local **libalpm** database with one **alpm-db-desc** file per package, each named _desc_ and located in a unique, per-package directory.

The accumulation of all **alpm-db-desc** files in a **libalpm** database describe the current state of the system (i.e. which packages have been installed when, under what circumstances).
More specifically, package management software such as **pacman** and related tools use this file format e.g. to resolve dependencies and to display package information.

The data in an **alpm-db-desc** file is derived from an **alpm-package**.
Here, most of the metadata originates from the package's **PKGINFO** data.

The **alpm-db-desc** file format must not be confused with the **alpm-repo-desc** file format, which is used in the context of an **alpm-repo-database** and carries a different set of metadata, but is usually also named _desc_.

The **alpm-db-desc** format exists in multiple versions.
The information in this document is for version 2, which is the current version and has been introduced with the release of pacman 6.1.0 on 2024-03-04.

## General Format

An **alpm-db-desc** file is a UTF-8 encoded, newline-delimited file consisting of a series of **sections**.

Each section starts with a unique _section header line_.
The section header line is followed by one or more lines with a _section-specific value_ each.
All _section-specific values_ must consist of **printable ASCII characters**[1] unless stated otherwise.
A section ends when another section header line is encountered or the end of the file is reached.

Empty lines between sections are ignored.

## Changes since the last version

The new keyword **xdata** has been added to track extra data for a package.

## Sections

Each _section header line_ contains the _section name_ in all capital letters, surrounded by percent signs (e.g. `%NAME%`).
_Section names_ serve as key for each _section-specific value_.

Each section allows for a single _section-specific value_, following the _section header line_.
As exemption to this rule the `%LICENSE%`, `%GROUPS%`, `%DEPENDS%`, `%OPTDEPENDS%`, `%REPLACES%`, `%CONFLICTS%`, `%PROVIDES%` and `%XDATA%` sections may have more than one _section-specific value_.

### %NAME%

An **alpm-package-name**, which represents the name of a package (e.g. `example`).

### %VERSION%

An **alpm-package-version** (_full_ or _full with epoch_) which represents the version of a package (e.g. `1.0-1`).

### %BASE%

An **alpm-package-base** which represents the package base from which a package originates.
The value may be the same as that of the `%NAME%` section.

### %DESC%

The description of the package.
The value is a UTF-8 string, zero or more characters long (e.g. `A project used for something`).

### %URL%

The URL for the project of the package.
The value is a valid URL or an empty string (e.g. `https://example.org`).

### %ARCH%

The architecture of the package (see **alpm-architecture** for further information).
The value must be covered by the set of alphanumeric characters and '\_' (e.g. `x86_64` or `any`).

### %BUILDDATE%

The date at which the build of the package started.
The value must be numeric and represent the seconds since the Epoch, aka. 'Unix time' (e.g. `1729181726`).

### %INSTALLDATE%

The date at which the package has been installed on the system.
The value must be numeric and represent the seconds since the Epoch, aka. 'Unix time' (e.g. `1729181726`).

### %PACKAGER%

The User ID of the entity, that built the package.
The value is meant to be used for identity lookups and represents an **OpenPGP User ID**[2].
As such, the value is a UTF-8-encoded string, that is conventionally composed of a name and an e-mail address, which aligns with the format described in **RFC 2822**[3] (e.g. `John Doe <john@example.org>`).

### %SIZE%

The optional size of the (uncompressed and unpacked) package contents in bytes.
The value is a non-negative integer representing the absolute size of the contents of the package, with multiple hardlinked files counted only once (e.g. `181849963`).
If the package has no size (e.g. because it is an **alpm-meta-package**) the section is omitted.

### %GROUPS%

An **alpm-package-group** that denotes a distribution-wide group the package is in.
Values may be present one or more times.
If the package is not in a group, the section is omitted.

The value is represented by a UTF-8 string.
Although it is possible to use a UTF-8 string, it is highly recommended to rely on the **alpm-package-name** format for the value instead, as package managers may use a package group to install an entire group of packages.

### %REASON%

An optional value representing the reason why the package is installed.
The value must be a non-negative integer:

- `0`: Explicitly installed
- `1`: Installed as a dependency

If the package is explicitly installed (value `0`), the section is omitted.

### %LICENSE%

An optional set of license identifiers that apply for the package.
Values may be present one or more times.
If there is no license identifier, the section is omitted.

Each value represents a license identifier, which is a string of non-zero length (e.g. `GPL`).
Although no specific restrictions are enforced for the value aside from its length, it is highly recommended to rely on SPDX license expressions (e.g. `GPL-3.0-or-later` or `Apache-2.0 OR MIT`).
See **SPDX License List**[4] for further information.

### %VALIDATION%

The validation method used during installation of the package ensuring its authenticity.
The value must be one of the following:

- `none`: The package integrity and authenticity is not validated.
- `md5`: The package is validated against an accompanying MD-5 hash digest in an **alpm-repo-database** that belongs to the repository from which the package is installed.
- `sha256`: The package is validated against an accompanying SHA-256 hash digest in the **alpm-repo-database** that belongs to the repository from which the package is installed.
- `pgp`: The package's authenticity and integrity is validated using a detached **OpenPGP signature**[5] and a system-wide collection of **OpenPGP certificates**[6].

### %REPLACES%

Another _virtual component_ or _package_, that the package replaces upon installation.
Values may be present one or more times.
If the package does not replace anything, the section is omitted.
The value is an **alpm-package-relation** of type **replacement** (e.g. `example` or `example=1.0.0`).

### %DEPENDS%

A run-time dependency of the package (_virtual component_ or _package_).
Values may be present one or more times.
If the package has no run-time dependency, the section is omitted.
The value is an **alpm-package-relation** of type **run-time dependency** (e.g. `example` or `example=1.0.0`).

### %OPTDEPENDS%

An optional dependency of the package (`virtual component` or `package`).
Values may be present one or more times.
If the package has no optional dependency, the section is omitted.
The value is an **alpm-package-relation** of type **optional dependency** (e.g. `example` or `example: this is a description`).

### %CONFLICTS%

Another _virtual component_ or _package_, that the package conflicts with.
Values may be present one or more times.
If the package does not conflict with anything, the section is omitted.
The value is an **alpm-package-relation** of type **conflict** (e.g. `example` or `example=1.0.0`).

### %PROVIDES%

Another _virtual component_ or _package_, that the package provides.
Values may be present one or more times.
If the package does not provide anything, the section is omitted.
The value is an **alpm-package-relation** of type **provision** (e.g. `example` or `example=1.0.0`).

### %XDATA%

The e*x*tra _data_ associated with the package.
One value must be assigned to define a specific value, but otherwise several values may be present as well to provide further extra data.
The value is a UTF-8-encoded string, that represents a key-value pair, delimited by a '=' sign (e.g. `key=value`).

This section of the **alpm-db-desc** file format must contain a value that defines a **pkgtype** assignment (e.g. `pkgtype=pkg`).
The valid **pkgtype** values are `debug` (for debug packages), `pkg` (for single packages), `src` (for source packages) and `split` (for split packages).

Additional values may be provided following the aforementioned general rules around formatting.

# EXAMPLES

An example **alpm-db-desc** file for a package named `example` in version `1.0.0-1`:

```text
%NAME%
example

%VERSION%
1.0.0-1

%BASE%
example

%DESC%
An example package

%URL%
https://example.org

%ARCH%
x86_64

%BUILDDATE%
1733737242

%INSTALLDATE%
1733737243

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%SIZE%
4

%LICENSE%
MIT
Apache-2.0

%VALIDATION%
pgp

%DEPENDS%
gcc-libs

%XDATA%
pkgtype=pkg
```

# SEE ALSO

**libalpm**(3), **BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **alpm-repo-desc**(5), **alpm-architecture**(7), **alpm-package**(7), **alpm-package-file-name**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **alpm-package-version**(7), **alpm-repo-database**(7), **alpm-split-package**(7), **pacman**(8)

# NOTES

1. printable ASCII characters
   
   <https://en.wikipedia.org/wiki/ASCII#Printable_characters>
1. OpenPGP User ID
   
   <https://openpgp.dev/book/certificates.html#user-ids>
1. RFC 2822
   
   <https://www.rfc-editor.org/rfc/rfc2822>
1. SPDX License List
   
   <https://spdx.org/licenses/>
1. OpenPGP signature
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
1. OpenPGP certificates
   
   <https://openpgp.dev/book/certificates.html>
