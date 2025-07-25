# NAME

package name - package and virtual component names for ALPM based packages.

# DESCRIPTION

The **package name** format represents package and component names for ALPM based packages.
This format is used in build scripts or file formats for package metadata (e.g. in **PKGBUILD**, **PKGINFO**, **SRCINFO**, **alpm-repo-desc**, or **alpm-lib-desc**) to describe package or component names, and **package relation**s.

While the **package name** format is mostly used to describe existing packages, it can also be used to name entirely *virtual components*.
A *virtual component* is specified implicitly using an **alpm-package-relation**.

The value must be covered by the set of alphanumeric characters and '@', '.', '_', '+', '-', but it must not start with '-' or '.' (e.g. `example` is valid).

# EXAMPLES

```text
"package-name"
```

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-package-relation**(7)
