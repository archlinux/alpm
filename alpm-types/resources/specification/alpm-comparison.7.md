# NAME

comparison - comparison statements for ALPM based packages.

# DESCRIPTION

The **comparison** format is a version comparison format, that is used for representing version comparison statements for ALPM based packages in a composite comparison expression.
This format is used in build scripts or file formats for package metadata (e.g. in **PKGBUILD** or **PKGINFO**) to describe version bounds for a **package relation**.

A **comparison** statement uses one of the following **comparison** operators to specify a version bound relation:

- `<` (less than)
- `<=` (less than or equal to)
- `=` (equal to)
- `>=` (greater than or equal to)
- `>` (greater than)

## Composite comparison expressions

Composite comparison expressions consist of an **alpm-package-name**, directly followed by a **comparison** operator, directly followed by an **alpm-package-version**.

## Matching comparison expressions

Name matching is performed based on a full string match using the **alpm-package-name** component of the composite comparison expression.

Version comparison is performed based on components of a composite version string (see e.g. **alpm-epoch**, **alpm-pkgver** and **alpm-pkgrel**).
As **alpm-package-version** offers several forms, this allows for matching a variety of scenarios.

When providing the *full* or *full with epoch* form of the **alpm-package-version** format, it matches *exactly one* specific release of a package version.
When providing the *minimal* or *minimal with epoch* form of the **alpm-package-version** format, it matches *any* release of a package version.

Depending on comparison operator, the given match towards **alpm-package-name** and **alpm-package-version** is narrow (i.e. `=`), wide with lower bound (i.e. `>`, `>=`) or wide with upper bound (i.e. `<`, `<=`).

# EXAMPLES

The below composite comparison expressions can be matched by a package named `example` in specific versions:

```text
"example<1.0.0"
```

A version less than '1.0.0' (e.g. '0.8.0-1').

```text
"example<=1.0.0"
```

A version less than or equal to '1.0.0' (e.g. '0.8.0-1' or '1.0.0-3').

```text
"example<=1.0.0-1"
```

A version less than or equal to '1.0.0-1' (e.g. '0.8.0-1' or '1.0.0-1', but '1.0.0-2' does not work).

```text
"example=1.0.0"
```

Any version '1.0.0' (e.g. '1.0.0-1' or '1.0.0-2', etc.).

```text
"example=1.0.0-1"
```

The version '1.0.0-1'.

```text
"example=1:1.0.0-1"
```

The version '1:1.0.0-1'.

```text
"example>=1.0.0"
```

A version greater than or equal to '1.0.0' (e.g. '1.0.0-1' or '1.1.0-1').

```text
"example>1.0.0"
```

A version greater than '1.0.0' (e.g. '1.1.0-1' or '1:1.0.0-1').

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **alpm-epoch**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **alpm-package-version**(7), **alpm-pkgrel**(7), **alpm-pkgver**(7), **vercmp**(8)
