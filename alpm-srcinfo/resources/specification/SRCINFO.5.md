# NAME

SRCINFO - Information on sources of ALPM based packages.

# DESCRIPTION

The **SCRINFO** format is a textual format that describes package source metadata.
Such files are named **.SRCINFO** and located at the root of source repositories from which ALPM based packages are built.
They are created from **PKGBUILD** files to allow safe metadata parsing in scenarios where using **bash** is not an option.
The **SRCINFO** format is used by applications such as repository management software and custom web frontends to evaluate and present the source information of packages.

## General Format

A **SRCINFO** file consists of a series of lines, each providing information on an aspect of the sources of one or several packages (see **alpm-split-package** for information on split packages).
Lines starting with a '#' sign are treated as comments and are always ignored.
Empty lines are always ignored.
Leading whitespace is always ignored.

Unless noted otherwise, the information contained in a **SRCINFO** file is considered to be covered by the set of the 95 **printable ASCII characters**[1].

The file format distinguishes between section headers introducing specific sections and keyword definitions that can be used in those sections.
Each line encodes information that either represents one section header or one keyword definition.

The **SRCINFO** format requires the definition of a single **pkgbase** and one or more **pkgname** sections.
While the **pkgbase** section contains metadata shared by all packages built from a **PKGBUILD**, the **pkgname** sections are used to declare package metadata and provide overrides that are specific for each respective package.

## Section headers

All section headers consist of a key, immediately followed by a whitespace, an '=' sign, another whitespace and a value.

The section header for a **pkgbase** section must use the word 'pkgbase' as key and an **alpm-package-name** as value.
Exactly one such section header must exist in a **SRCINFO** file at the beginning of the file.

The section header for a **pkgname** section must use the word 'pkgname' as key and an **alpm-package-name** as value.
At least one such section header must exist in a **SRCINFO** file (one for each package built from a **PKGBUILD**), but only after a **pkgbase** section header.

## Sections

The **pkgbase** section must at least contain the **pkgver** and **pkgrel** keyword definitions, and may otherwise contain all other keyword definitions as well.
Keyword definitions in this section also apply to all **pkgname** sections.

A **pkgname** section may have zero or more keyword definitions.
It must not contain the **epoch**, **pkgver**, **pkgrel**, or **validpgpkeys** keyword definitions, nor any keyword definitions representing build or test dependency declarations (see **alpm-package-relation** for details), source declarations (see **alpm-package-source** for details) or checksum declarations (see **alpm-package-source-checksum** for details).
Keyword definitions in a **pkgname** section only apply to the specific package that the section describes.
More specifically, all keyword definitions in a **pkgname** section are used to unset, extend or override the keyword definitions found in the **pkgbase** section.
The following rules apply:

- A keyword that has been previously defined in the **pkgbase** section and may be used in a **pkgname** section can be unset by assigning an empty value to it (e.g. the **pkgbase** section defines `depends = bash` and the **pkgname** section defines `depends =`).
- A keyword that has been previously defined in the **pkgbase** section and may be used in a **pkgname** section can be overridden by defining it again (e.g. the **pkgbase** section defines `depends = bash` and the **pkgname** section defines `depends = zsh`).
- All keywords that allow multiple definitions and that may be used in a **pkgname** section can be extended by first adding all keyword definitions found in the **pkgbase** section and afterwards adding further keyword definitions in a **pkgname** section (e.g. the **pkgbase** section defines `depends = bash` and the **pkgname** section defines `depends = bash` and `depends = bash-completion`).

Further rules may apply depending on keyword.

## Keywords

All keyword definitions consist of a key from the following list immediately followed by a whitespace, an '=' sign, another whitespace and a value.

By default, exactly one keyword definition may be present per keyword in a **SRCINFO**.
As exception to this rule, the keywords **arch**, **backup**, **checkdepends**, **conflicts**, **depends**, **groups**, **license**, **makedepends**, **optdepends**, **options**, **provides**, **replaces**, as well as the source related keywords **noextract**, **source**, **validpgpkeys**, **b2sums**, **md5sums**, **sha1sums**, **sha224sums**, **sha256sums**, **sha384sums** and **sha512sums** may be provided zero or more times.

Some keywords may also be provided in architecture-specific ways by appending an '_' sign, directly followed by an **alpm-architecture** (all except `any`) - see each keyword section for details.

By default keyword definitions apply to all targeted architectures.
Architecture-specific keywords only apply to the architecture they target.

### pkgdesc

The description of the package.
This keyword definition may be provided zero or one times in the **pkgbase** and/or in each **pkgname** section.
The value is a UTF-8 string, zero or more characters long (e.g. `A project used for something`).
No specific rules about the value exist, but it is suggested to be "short" and to not contain the package name (see **alpm-package-name**).

### pkgver

The pkgver package version component for the package(s) (see **alpm-pkgver** for further information on the expected value).
This keyword definition must be provided once, exclusively in the **pkgbase** section.

The value is an **alpm-pkgver** (e.g. `1.0.0` or `1:1.0.0`).

### pkgrel

The pkgrel package version component for the package(s) (see **alpm-pkgrel** for further details on the expected value).

This keyword definition must be provided once, exclusively in the **pkgbase** section.

### epoch

The optional epoch package version component for the package(s) (see **alpm-epoch** for further details on the expected value).

This keyword definition may be provided zero or one time, exclusively in the **pkgbase** section.

### url

The URL for the project of the package.
This keyword definition may be provided zero or one times in the **pkgbase** and/or in each **pkgname** section.
The value is a valid URL or an empty string (e.g. `https://example.org`).

### install

The name of an **alpm-install-scriptlet** that is used for pre and post actions when installing, upgrading or uninstalling a package.
This keyword definition may be provided zero or one times in the **pkgbase** and/or in each **pkgname** section.

The value must be a UTF-8 string, that represents a relative file path (e.g. `scriptlet.install`).

### changelog

The name of file containing changelog information for a package.
This keyword definition may be provided zero or one times in the **pkgbase** and/or in each **pkgname** section.

The value must be a UTF-8 string, that represents a relative file path (e.g. `pkg.changelog`).

### arch

An architecture that applies for a **pkgname** section.
This keyword definition may be provided one or more times in the **pkgbase** and zero or more times in each **pkgname** section.

The value is an **alpm-architecture** (e.g. `x86_64` or `any`).
The following rules apply, if this keyword is used multiple times:

- each value must be unique
- if the value `any` is used, no other value can be used alongside it, as it implies that a **pkgname** may be used _on any architecture_ (which logically excludes being used only on a specific architecture)

### groups

An arbitrary string, that denotes a distribution-wide group the package is in.
Groups are used e.g. by package managers to group packages and allow to bulk install them, or by other software to display information on these related packages.
This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is represented by a UTF-8 string.
Although it is possible to use a UTF-8 string, it is highly recommended to rely on the **alpm-package-name** format for the value instead, as package managers may use **groups** to install an entire group of packages.

### license

A license that applies for the package(s).
This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value represents a license identifier, which is a string of non-zero length (e.g. `GPL`).
Although no specific restrictions are enforced for the value aside from its length, it is highly recommended to rely on SPDX license expressions (e.g. `GPL-3.0-or-later` or `Apache-2.0 OR MIT`).
See **SPDX License List**[5] for further information.
If the keyword definition is provided in a **pkgname** section and it is also provided in the **pkgbase** section at least once, the value of the first occurrence in the **pkgname** section may be empty.
This indicates, that for the respective package none of the **license** keyword definitions defined in the **pkgbase** section apply.

### checkdepends

A dependency for running tests of the package's upstream project.
An architecture-specific **checkdepends** may be specified using a keyword consisting of 'checkdepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `checkdepends_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
The value is an **alpm-package-relation** of type **test dependency** (e.g. `example` or `example=1.0.0`).

### makedepends

A build time dependency of the package (*virtual component* or package).
An architecture-specific **makedepends** may be specified using a keyword consisting of 'makedepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `makedepends_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
The value is an **alpm-package-relation** of type **build dependency** (e.g. `example` or `example=1.0.0`).

### depends

A run-time dependency of the package (*virtual component* or package).
An architecture-specific **depends** may be specified using a keyword consisting of 'depends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `depends_aarch64`.

This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is an **alpm-package-relation** of type **run-time dependency** (e.g. `example` or `example=1.0.0`).

### optdepends

An optional dependency of the package (*virtual component* or package).
An architecture-specific **optdepends** may be specified using a keyword consisting of 'optdepends', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `optdepends_aarch64`.

This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is an **alpm-package-relation** of type **optional dependency** (e.g. `example` or `example: this is a description`).

### provides

Another *virtual component* or package, that the package provides.
An architecture-specific **provides** may be specified using a keyword consisting of 'provides', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `provides_aarch64`.

This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is an **alpm-package-relation** of type **provision** (e.g. `example` or `example=1.0.0`).

### conflicts

Another *virtual component* or package, that the package conflicts with.
An architecture-specific **conflicts** may be specified using a keyword consisting of 'conflicts', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `conflicts_aarch64`.

This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is an **alpm-package-relation** of type **conflict** (e.g. `example` or `example=1.0.0`).

### replaces

Another *virtual component* or package, that the package replaces upon installation.
An architecture-specific **replaces** may be specified using a keyword consisting of 'replaces', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `replaces_aarch64`.

This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value is an **alpm-package-relation** of type **replacement** (e.g. `example` or `example=1.0.0`).

### noextract

The filename of a **source** value, indicating that the given file is *not* automatically extracted, even if it represents a compressed file (e.g. `example.tar.gz`).
An architecture-specific **noextract** may be specified using a keyword consisting of 'noextract', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `noextract_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.

### options

An option used by the package build tool (i.e. **makepkg**, defaults are defined in **OPTIONS** of **makepkg.conf**) when building the package.
This keyword definition may be provided zero or more times either in the **pkgbase** or in a **pkgname** section.

The value must be a unique word, optionally prefixed by a single '!', which indicates the negation of the option (e.g. debug or !debug).
If the keyword definition is used in a **pkgname** section, the value of the first occurrence may be empty, which indicates, that for the respective package none of the **options** defined in the **pkgbase** section apply.

### backup

A relative file path of a file in a given package, that denotes a file for the package manager to keep backups for in case it changes or is deleted during a package update action (see **pacman** '.pacnew' and '.pacsave' files).
This keyword definition may be provided zero or more times in the **pkgbase** as well as in each **pkgname** section.
The value must be a valid relative Unix file path (e.g. `etc/package.conf`).

### source

A (remote or local) source used in the packaging process of the package(s) (see **alpm-package-source** for further details on the allowed value), that are automatically extracted if they represent compressed files (unless a **noextract** value matching the file name exists).
An architecture-specific **source** may be specified using a keyword consisting of 'source', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `source_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.

### validpgpkeys

An **OpenPGP fingerprint**[2] which may be used for the validation of **OpenPGP signatures**[3] created for one or more **alpm-package-sources** (i.e. a **source** keyword definition).

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
It must be used at least once if
- one **alpm-package-source** value makes use of the `signed` URL fragment 
- or if there is one **alpm-package-source** ending with `.sig`, that equals another without that file ending
- or if there is one **alpm-package-source** ending with `.sign`, that matches another without that file ending and all compression algorithm specific file endings (e.g. `.xz` or `.zst`) removed

The value is a 40 char long hexadecimal string (e.g `0123456789abcdef0123456789abcdef01234567`).
Although possible, it is strongly discouraged to use the non-unique, legacy **OpenPGP Key ID**[4] representation, a 16 char long hexadecimal string (e.g. `89abcdef01234567`)

### md5sums

An **MD5**[6] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **md5sums** may be specified using a keyword consisting of 'md5sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `md5sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **md5sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **md5sums** keyword definitions (i.e. the first **md5sums** keyword definition provides data for the first **source** keyword definition, etc.).

### sha1sums

A **SHA-1**[7] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **sha1sums** may be specified using a keyword consisting of 'sha1sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `sha1sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **sha1sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **sha1sums** keyword definitions (i.e. the first **sha1sums** keyword definition provides data for the first **source** keyword definition, etc.).

### sha224sums

A **SHA-224**[8] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **sha224sums** may be specified using a keyword consisting of 'sha224sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `sha224sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **sha224sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **sha224sums** keyword definitions (i.e. the first **sha224sums** keyword definition provides data for the first **source** keyword definition, etc.).

### sha256sums

A **SHA-256**[8] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **sha256sums** may be specified using a keyword consisting of 'sha256sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `sha256sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **sha256sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **sha256sums** keyword definitions (i.e. the first **sha256sums** keyword definition provides data for the first **source** keyword definition, etc.).

### sha384sums

A **SHA-384**[8] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **sha384sums** may be specified using a keyword consisting of 'sha384sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `sha384sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **sha384sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **sha384sums** keyword definitions (i.e. the first **sha384sums** keyword definition provides data for the first **source** keyword definition, etc.).

### sha512sums

A **SHA-512**[8] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **sha512sums** may be specified using a keyword consisting of 'sha512sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `sha512sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **sha512sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **sha512sums** keyword definitions (i.e. the first **sha512sums** keyword definition provides data for the first **source** keyword definition, etc.).

### b2sums

A **BLAKE2**[9] hash digest of a file provided by name in a **source** value, or the special string 'SKIP' (see **alpm-package-source-checksum** for further details on the allowed value).
An architecture-specific **b2sums** may be specified using a keyword consisting of 'b2sums', directly followed by an '_' sign, directly followed by an **alpm-architecture** (all **alpm-architectures** except `any` can be used), e.g. `b2sums_aarch64`.

This keyword definition may be provided zero or more times, exclusively in the **pkgbase** section.
If it is used, the **SRCINFO** must contain as many **b2sums** keyword definitions as there are **source** keyword definitions (the same applies for the architecture-specific variant of this keyword).
The ordering of accompanying **source** keyword definitions dictate the ordering of the **b2sums** keyword definitions (i.e. the first **b2sums** keyword definition provides data for the first **source** keyword definition, etc.).

# EXAMPLES

## Split packages, extending, overriding and unsetting of keywords

```ini
pkgbase = example
  pkgdesc = An example package
  pkgver = 1.0.0
  pkgrel = 1
  epoch = 1
  url = https://example.org
  arch = any
  license = GPL-3.0-or-later
  checkdepends = extra-test-tool
  checkdepends = other-extra-test-tool
  makedepends = cmake
  makedepends = python-sphinx
  depends = glibc
  depends = gcc-libs
  source = https://example.org/example-1.0.0.tar.gz
  sha512sums = 8b41e1b78ad11521113c52ff182a1b8e0a195754aa527fcd00a411620b46f20ffffb8088ccf85497121ad4499e0845b876f6dd6640088a2f0b2d8a600bdf4c0c
  b2sums = cb79bf658b69dff0acf721232455a461598dd26ed42047bd0362e7fbd796093145a694c1a6bcdcf5bf7f866d78f009c14bf456be0f944283829a6e33cedf2aef

pkgname = example
  pkgdesc = A project that does something
  groups = package-group
  license = GPL-3.0-or-later
  license = LGPL-3.0-or-later
  optdepends = python: for special-python-script.py
  optdepends = example-docs: for documentation
  provides = some-component
  conflicts = conflicting-package<1.0.0
  replaces = other-package>0.9.0-3
  backup = etc/example/config.toml

pkgname = example-docs
  pkgdesc = A project that does something - documentation
  license = CC-BY-SA-4.0
  depends =
```

The above example represents a **SRCINFO** file that describes two split packages (see **alpm-split-package** for more details).
Both **pkgname** sections override the `pkgdesc` keyword definition.
The first **pkgname** section additionally extends the `license` keyword specification from the **pkgbase** section and adds further package-specific keyword definitions (e.g. `replaces`, `groups`, `conflicts`, `provides`, `backup`, `optdepends`).
The second **pkgname** section overrides the `license` keyword definition and unsets the `depends` keyword definition.

## Package data, per architecture

```bash
pkgname=example
pkgver=0.1.0
pkgrel=1
pkgdesc="An example package"
arch=(x86_64 aarch64)
url="https://example.org"
license=(GPL-3.0-or-later)
depends=(bash)
depends_x86_64=(zsh)

package() {
  pkgdesc+=" - extra info"
  depends_x86_64+=(nushell)
  depends_aarch64=(sh)
}
```

The above minimal **PKGBUILD** defines a package for two **alpm-architectures**.
It is represented by the following **SRCINFO**:

```ini
pkgbase = example
  pkgdesc = An example package
  pkgver = 0.1.0
  pkgrel = 1
  url = https://example.org
  arch = x86_64
  arch = aarch64
  license = GPL-3.0-or-later
  depends = bash
  depends_x86_64 = zsh

pkgname = example
  pkgdesc = An example package - extra info
  depends_x86_64 = zsh
  depends_x86_64 = nushell
  depends_aarch64 = sh
```

Depending on targeted **alpm-architecture**, the package data will differ when fully resolving all keywords.

For `aarch64`:

- pkgdesc = An example package - extra info
- pkgver = 0.1.0
- pkgrel = 1
- url = https://example.org
- arch = aarch64
- license = GPL-3.0-or-later
- depends = bash
- depends = sh

For `x86_64`:

- pkgdesc = An example package - extra info
- pkgver = 0.1.0
- pkgrel = 1
- url = https://example.org
- arch = x86_64
- license = GPL-3.0-or-later
- depends = bash
- depends = zsh
- depends = nushell

# SEE ALSO

alpm-buildinfo(1), bash(1), makepkg.conf(5), PKGBUILD(5), alpm-architecture(7), alpm-comparison(7), alpm-epoch(7), alpm-install-scriptlet(7), alpm-package-name(7), alpm-package-relation(7), alpm-package-version(7), alpm-pkgrel(7), alpm-pkgver(7), alpm-split-package(7), makepkg(8), pacman(8)

# NOTES

1. **printable ASCII characters**

   https://en.wikipedia.org/wiki/ASCII#Printable_characters

2. **OpenPGP fingerprint**

   https://openpgp.dev/book/certificates.html#fingerprint

3. **OpenPGP signatures**

   https://openpgp.dev/book/signing_data.html

4. **OpenPGP Key ID**

   https://openpgp.dev/book/glossary.html#term-Key-ID

5. **SPDX License List**

   https://spdx.org/licenses/

6. **MD5**

  https://en.wikipedia.org/wiki/MD5

7. **SHA-1**

  https://en.wikipedia.org/wiki/SHA-1

8. **SHA-2**

  https://en.wikipedia.org/wiki/SHA-2

9. **BLAKE2**

  https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2
