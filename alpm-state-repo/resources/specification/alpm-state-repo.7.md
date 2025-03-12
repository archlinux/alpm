# NAME

state-repo - a directory structure to track **A**rch **L**inux **P**ackage **M**anagement (ALPM) package repository state.

# DESCRIPTION

End users gain access to ALPM based packages (see **alpm-package**) through package repositories (see **alpm-repo**).
These also contain database files describing their current state (see **alpm-repo-database**) to the end users.

An **alpm-state-repo** is used by package repository maintainers and provides a detailed view on several aspects of one or more package repositories:

- The names of all **alpm-package-bases** and their respective versions currently provided in all tracked repositories.
- A reference to the git tag, as well as the fully resolved git commit hash digest in an **alpm-source-repo**, that packages with the respective **alpm-package-base** are built from.

A state-repo is maintained using repository management software (e.g. **dbscripts** [1]).
It is usually kept in a version controlled repository (e.g. **git**) to reflect on atomic changes to the contained data in dedicated commits.

# FORMAT

An **alpm-state-repo** contains **repository directories** at its root which each may contain zero or more **state files**.

## Repository directory

A repository directory represents architecture specific information about an **alpm-repo**.
Each directory name consists of an **alpm-repo-name** directly followed by a '-' sign, directly followed by an **alpm-architecture**, e.g.:

```text
state
|-- core-any
|-- core-testing-x86_64
|-- core-x86_64
|-- extra-any
|-- extra-staging-any
|-- extra-staging-x86_64
|-- extra-testing-any
|-- extra-testing-x86_64
\-- extra-x86_64
```

Different from an **alpm-repo**, an **alpm-state-repo** distinguishes between architecture-dependent (e.g. `x86_64`) and architecture-independent (i.e. `any`) packages in a repository, which is reflected in the directory naming.
The combination of architecture-dependent and architecture-independent directories with the same **alpm-repo-name** comprise the state of an **alpm-repo** of the same name (e.g. the directories `core-any` and `core-x86_64` in an **alpm-state-repo** contain data for an **alpm-repo** named `core`).

Each **repository directory** contains zero or more **state files**.

## State file

All files in a **repository directory** are named after an **alpm-package-base** for which they provide state data.
Each **alpm-package-base** implicitly refers to one or more **alpm-packages** in the respective **alpm-repo** that the **repository directory** targets.

A state file consists of a single line with the four components _package name_, _version_, _tag_ and _digest_, each separated by a space:

1. _package name_: The **alpm-package-base** of one or more **alpm-packages** in the targeted **alpm-repo**.
   This value must be identical to the name of the **state file**!
3. _version_: The version of the **alpm-package-base**, formatted as an **alpm-package-version** in **full** or **full with epoch** variant.
4. _tag_: The git tag in the **alpm-source-repo** from which all **alpm-packages** that share _package name_ are built.
   This value is based on _version_ but is subject to **version normalization**.
6. _digest_: The SHA-1 hash digest of the commit that _tag_ points at in the **alpm-source-repo**.

### Version normalization

Git is not able to use certain characters in tags that are allowed in the **alpm-package-version** format.
To be able to relate the version encoded in a **PKGBUILD** or **SRCINFO** file that is part of an **alpm-source-repo** and by extension also the **pkgver** value found in a package's **PKGINFO** or **BUILDINFO** built from those sources, a tag undergoes normalization. 

The following modifications are applied in order to normalize the string:

1. Replace all occurrences of ':' (colon) with a '-' (dash)
2. Replace all occurrences of '~' (tilde) with a '.' (dot)

# EXAMPLES

## Single package

An **alpm-package** `example` in a repository named `core` is of architecture `x86_64` and present in version `33-1`.
The package is the sole output from a package build process using an **alpm-source-repo** at the tag `33-1`.
The tag points at the commit with the SHA-1 hash digest `0685197a7fdc13a91e1b9184c2759a5bf222210f`.

The file `core-x86_64/example` in the **alpm-state-repo** contains the following data:

```text
example 33-1 33-1 0685197a7fdc13a91e1b9184c2759a5bf222210f
```

## Single package with version normalization

The package `example` in a repository named `extra-testing` is of architecture `x86_64` and present in version `17:4.3.2-10`.
The package is the sole output from a package build process using an **alpm-source-repo** at the tag `17-4.3.2-10`.
The tag points at the commit with the SHA-1 hash digest `e4f06701c4ff5cda811a5663e83cf966a81f42e0`.

The file `extra-testing-x86_64/example` in the **alpm-state-repo** contains the following data:

```text
example 17:4.3.2-10 17-4.3.2-10 e4f06701c4ff5cda811a5663e83cf966a81f42e0
```

## Split package

The **alpm-packages** `example-a` and `example-b` in a repository named `core` are of architecture `x86_64` and are both present in version `1.0.0-1`.
The packages are both the output of a package build process using an **alpm-source-repo** at the tag `1.0.0-1`.
The tag points at the commit with the SHA-1 hash digest `0685197a7fdc13a91e1b9184c2759a5bf222210f`.
The **PKGBUILD** in the **alpm-source-repo** uses `example-base` as **pkgbase**.

The file `core-x86_64/example-base` in the **alpm-state-repo** contains the following data:

```text
example-base 1.0.0-1 1.0.0-1 0685197a7fdc13a91e1b9184c2759a5bf222210f
```

# SEE ALSO

**git**(1), **PKGBUILD**(5), **alpm-architecture**(7), **alpm-package**(7), **alpm-package-base**(7), **alpm-package-name**(7), **alpm-package-version**(7), **alpm-repo**(7), **alpm-repo-database**(7), **alpm-repo-name**(7), **alpm-source-repo**(7), **alpm-split-package**(7), **devtools**(7),

# NOTES

1. **dbscripts**

   https://gitlab.archlinux.org/archlinux/dbscripts/
