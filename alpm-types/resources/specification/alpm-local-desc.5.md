# NAME

alpm-local-desc - Metadata format for packages in the local ALPM database.

# DESCRIPTION

The **alpm-local-desc** format is a textual representation of single package's metadata. The local libalpm database, which is located in `/var/lib/pacman/local`, contains one **alpm-local-desc** file per package.

A **alpm-local-desc** file describes various properties of a package, including its name, version, dependencies, size, licensing information, and more. For a full list look at the **Keys** section.

Package managers and related tools use this metadata to display package information and resolve dependencies.

## General Format

A **DESC** file is a UTF-8 encoded, newline-delimited file made up of a series of **keys**.

Each key starts with a header line that contains the key name, wrapped in percent signs (e.g. `%NAME%`).

If a key supports multiple values, each value is listed on its own line right after the header.

A key ends when another key header is encountered or the end of the file is reached.

Empty lines between keys are ignored.

All paths, names, versions, and values must be printable ASCII characters unless stated otherwise.

## Keys

### %FILENAME%

The filename of the package archive (e.g. `example-1.0-1-x86_64.pkg.tar.zst`).

Exactly one value must be present.

### %NAME%

The name of the package (e.g. `example`). See **alpm-package-name**(7) for naming conventions.

Exactly one value must be present.

### %BASE%

The base name of the package. If the package is part of a **alpm-split-package** group, this will be the common base name; otherwise, it is the same as `%NAME%`.

Exactly one value must be present.

### %VERSION%

The version string of the package (e.g. `1.0-1`). See **alpm-package-version**(7) for versioning conventions.

Exactly one value must be present.

### %DESC%

A short description of the package.

Exactly one value must be present.

### %CSIZE%

The size in bytes of the compressed package archive.

Exactly one value must be present. The value must be a positive integer.

### %ISIZE%

The size in bytes that the package will occupy when installed.

Exactly one value must be present. The value must be a positive integer.

### %SHA256SUM%

The SHA256 checksum of the package archive contents.

Exactly one value must be present. The value must be a valid lowercase hexadecimal hash.

### %URL%

The project or upstream URL for the package.

Exactly one value must be present.

### %LICENSE%

The licenses that apply to the package.

One or more values may be present. Each line specifies one SPDX license identifier (e.g. `MIT`, `GPL-3.0-only`).

### %ARCH%

The architecture for which the package is built (e.g. `x86_64`). See **alpm-architecture** for supported architectures.

Exactly one value must be present.

### %BUILDDATE%

The UNIX timestamp representing the time the package was built.

Exactly one value must be present.

### %INSTALLDATE%

An optional UNIX timestamp representing the time the package was installed (local database only).

At most one value may be present.

### %PACKAGER%

The name and email address of the person who built the package, in the format `Name <email>`.

Exactly one value must be present.

### %REASON%

An optional reason why the package is installed. The value must be one of:

- `0`: Explicitly installed
- `1`: Installed as a dependency

At most one value may be present.

### %VALIDATION%

The validation methods used to ensure the integrity of the package.

Zero or more values may be present. Recognized values include:

- `none`
- `md5`
- `sha256`
- `pgp`

### %SIZE%

Deprecated. Use `%ISIZE%` instead.

May be present in older packages for compatibility.

### %GROUPS%

The groups to which the package belongs (e.g. `base-devel`, `gnome`).

Zero or more values may be present.

### %DEPENDS%

Runtime dependencies required by this package.

Zero or more values may be present. Each value must follow the run-time dependency format supported by **alpm-package-relation**(7).

### %OPTDEPENDS%

Optional dependencies that enhance the package’s functionality.

Zero or more values may be present. Each value must follow the optional dependency format supported by **alpm-package-relation**(7).

### %MAKEDEPENDS%

Dependencies required to build the package.

Zero or more values may be present. Each value must follow the build dependency format supported by **alpm-package-relation**(7).

### %CHECKDEPENDS%

Dependencies required to run the package's test suite.

Zero or more values may be present. Each value must follow the test dependency format supported by **alpm-package-relation**(7).

### %REPLACES%

Packages that this one can replace.

Zero or more values may be present.

### %CONFLICTS%

Packages that cannot be installed at the same time as this one.

Zero or more values may be present.

### %PROVIDES%

Virtual packages or capabilities provided by this package.

Zero or more values may be present.

### %XDATA%

The e*x*tra _data_ associated with the package.

Zero or more values may be present. The format is the same as described in **alpm-pkginfo**(7).

The value is a UTF-8-encoded string, that represents another key-value pair, delimited by a '=' sign (e.g. `key=value`).

# EXAMPLES

An example **alpm-local-desc** file for a package named `example`:

```text
%FILENAME%
example-1.0.0-1-x86_64.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
1.0.0-1

%DESC%
An example package

%CSIZE%
475255

%ISIZE%
1165163

%SHA256SUM%
b3948da79bee3aa25e1a58ee5946355b6ba822679e51a48253620dbfac510e9d

%URL%
https://gitlab.archlinux.org/archlinux/alpm

%LICENSE%
MIT
Apache-2.0

%ARCH%
x86_64

%BUILDDATE%
1733737242

%PACKAGER%
Foobar McFooface <foobar@mcfooface.org>

%DEPENDS%
gcc-libs

%MAKEDEPENDS%
cargo

%XDATA%
pkgtype=pkg
```

# SEE ALSO

**alpm-architecture**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **alpm-package-version**(7), **alpm-pkginfo**(7), **alpm-split-package**(7), **alpm-local-files**(7), **alpm**(3)
