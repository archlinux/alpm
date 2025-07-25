# NAME

pkgrel - version postfix to enforce a higher version consideration for ALPM based packages.

# DESCRIPTION

The **pkgrel** format is a version format, that is used as postfix to **pkgver** in a composite version string.
This format is used in build scripts or file formats for package data description or reproduction to indicate, that a composite version (e.g. **BUILDINFO**'s **pkgver**) is to be considered newer than one with a lower value **pkgrel** component.
This functionality is used in the context of distributions to release new builds of upstream software based on an upstream release of the same version.
For each rebuild, the **pkgrel** value is incremented by '1'.
Once the upstream version represented by **pkgver** is incremented, the **pkgrel** is reset to '1'.

The **pkgrel** value must consist of one or more numeric digits, optionally followed by a period (`.`) and one or more additional numeric digits.
The default value when using **pkgrel** is '1'.

When used in a composite version string, **pkgver** is directly followed by a '-' sign and the **pkgrel**.

Compositive version strings with the same **pkgver** component are sorted according to their **pkgrel**, but **epoch** may be used to set higher ordering nevertheless.
Hence, the sorting precedence is: `epoch > pkgver > pkgrel`.

# EXAMPLES

Compositive version strings with the same **pkgver** component are sorted according to their **pkgrel** component.

```text
"1.0.0-1" < "1.0.0-2"
"1.0.0-1" < "1.0.0-1.0"
"1.0.0-1.0" < "1.0.0-2.0"
```

An explicit **epoch** component is always considered before the **pkgver** and **pkgrel** components in a composite version string.

```text
"1:1.0.0-1" > "1.0.0-2"
```

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-epoch**(7), **alpm-package-version**(7), **alpm-pkgver**(7), **vercmp**(8)
