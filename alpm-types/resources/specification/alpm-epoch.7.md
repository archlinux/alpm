# NAME

epoch - version prefix to enforce a higher version consideration for ALPM based packages.

# DESCRIPTION

The **epoch** format is a version format, that is used as prefix to **pkgver** in a composite version string.
This format is used in build scripts or file formats for package data description or reproduction to indicate, that a composite version is to be considered newer than one without or a lower **epoch** component.
This enables package maintainers to release downgraded versions of a package if needed.

The **epoch** value is represented by a non-negative integer and is considered to default to '0' if unset.

When used in a composite version string, **epoch** is directly followed by a ':' sign and the **pkgver**.

If two composite version strings are sorted, they are first sorted based on their **epoch** component and only afterwards based on their **pkgver** component.

As general rule, the sorting precedence is: `epoch > pkgver`.

# EXAMPLES

The explicit `1` **epoch** component in the right hand composite version string overrules the implicit `0` **epoch** component of the left hand composite version string.
Since the **epoch** takes precedence, `1:0.9.0` is considered "newer" than `1.0.0` even though the upstream version represented by the **pkgver** component is older.

```text
"1.0.0" < "1:0.9.0"
```

Composite version strings with the same **pkgver** component are also sorted according to their **epoch** component first.

```text
"1:1.0.0" < "2:1.0.0"
```

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-package-version**(7), **alpm-pkgrel**(7), **alpm-pkgver**(7), **vercmp**(8)
