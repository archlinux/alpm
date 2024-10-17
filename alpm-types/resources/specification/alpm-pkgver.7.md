# NAME

pkgver - upstream version information for ALPM based packages.

# DESCRIPTION

The **pkgver** format is a version format, that is used for representing upstream version information.
This format is used in build scripts or file formats for package data description or reproduction.

A **pkgver** value is represented by a string, consisting of ASCII characters, excluding the ':', '/', '-' or any whitespace characters.
The **pkgver** value must be at least one character long, and must not start with a '.' sign.
If an upstream version contains an invalid character, it is advised to replace it with a valid one or to remove it.

# EXAMPLES

```
"1.0.0"
```

```
"1.0.0alpha"
```


# SEE ALSO

BUILDINFO(5), PKGBUILD(5), alpm-epoch(7), alpm-pkgrel(7), vercmp(8)
