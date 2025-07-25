# NAME

package version - package versions for ALPM based packages.

# DESCRIPTION

The **package version** format represents version information for ALPM based packages.
This format is used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **PKGINFO**, **SRCINFO**, **alpm-repo-desc**, or **alpm-lib-desc**) to describe package versions, or component versions as part of a **package relation**.

The value is represented by a composite version string, which may consist of several components (**alpm-epoch**, **alpm-pkgver**, **alpm-pkgrel**).
Various forms of composite version strings exist, but they are only used in specific contexts.

## Full

The value for this form consists of an **alpm-pkgver**, directly followed by a '-' sign, directly followed by an **alpm-pkgrel** (e.g. `1.0.0-1`).
This **alpm-package-version** form is used in various scenarios, such as:

- package filenames
- **package-relation** expressions
    - **replaces**, **conflict**, **provides**, **depend**, **makedepend** and **checkdepend** in **PKGINFO** files
- **pkgver** in **BUILDINFO** files
- **pkgver** in **PKGINFO** files
- as part of **buildtoolver** in **BUILDINFOv2** files
- as part of **installed** in **BUILDINFO** files

## Full with epoch

The value for this form consists of an **alpm-epoch**, directly followed by a ':' sign, directly followed by an **alpm-pkgver**, directly followed by a '-' sign, directly followed by an **alpm-pkgrel** (e.g. `1:1.0.0-1`).
This **alpm-package-version** form is used in various scenarios, such as:

- package filenames
- **package-relation** expressions
    - **replaces**, **conflict**, **provides**, **depend**, **makedepend** and **checkdepend** in **PKGINFO** files
- **pkgver** in **BUILDINFO** files
- **pkgver** in **PKGINFO** files
- as part of **buildtoolver** in **BUILDINFOv2** files
- as part of **installed** in **BUILDINFO** files
- as part of the **alpm-state-repo** contents

## Minimal

The value for this form consists of an **alpm-pkgver** (e.g. `1.0.0`).
This **alpm-package-version** form is used in various scenarios, such as:

- **package-relation** expressions
    - **replaces**, **conflicts**, **provides**, **depends**, **makedepends** and **checkdepends** in **PKGBUILD** files
    - **replaces**, **conflict**, **provides**, **depend**, **makedepend** and **checkdepend** in **PKGINFO** files
- **pkgver** in **PKGBUILD** files
- **pkgver** in **SRCINFO** files
- as part of **buildtoolver** in **BUILDINFOv2** files

## Minimal with epoch

The value for this form consists of an **alpm-epoch**, directly followed by a ':' sign, directly followed by an **alpm-pkgver** (e.g. `1:1.0.0`).
This **alpm-package-version** form is used in various scenarios, such as:

- as part of **package-relation** expressions
    - **replaces**, **conflicts**, **provides**, **depends**, **makedepends** and **checkdepends** in **PKGBUILD** files
    - **replaces**, **conflict**, **provides**, **depend**, **makedepend** and **checkdepend** in **PKGINFO** files
- **pkgver** in **PKGBUILD** files
- **pkgver** in **SRCINFO** files
- as part of **buildtoolver** in **BUILDINFOv2** files

# EXAMPLES

```text
"1.0.0-1"
```

A full package version.

```text
"1:1.0.0-1"
```

A full package version with epoch.

```text
"1.0.0"
```

A minimal package version.

```text
"1:1.0.0"
```

A minimal package version with epoch.

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-epoch**(7), **alpm-package-relation**(7), **alpm-pkgrel**(7), **alpm-pkgver**(7), **vercmp**(8)
