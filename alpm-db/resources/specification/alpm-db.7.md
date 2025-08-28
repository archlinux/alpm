# NAME

alpm-db - a database format for describing the state of packages on an **A**rch **L**inux **P**ackage **M**anagement (ALPM) based system.

# DESCRIPTION

The databases on ALPM based systems are represented by a directory structure that contains metadata files.
The contents of such a database describes the state of all installed packages.
Here, **ALPM-MTREE**, **alpm-db-desc** and **alpm-db-files** files provide metadata on specific package versions currently installed.

Entries for databases are created from **alpm-package** files using package management software such as **pacman**.
Package management software relies on **alpm-db** structures for the purpose of search, dependency resolution and system management.

The **alpm-db** format exists in multiple versions.
The information in this document is for version `9`, which is the current version and has been introduced with the release of **pacman** 4.2.0 on 2014-12-19.

## Contents

The contents of the system database depend on the number of unique packages it describes.
Each package can only appear once, in a single version.

### Metadata

Metadata of a package is kept in a top-level directory, that is named after the package and its specific version.
The name follows this schema:

An **alpm-package-name** directly followed by a `-` sign, directly followed by an **alpm-package-version** (in the _full_ or _full with epoch_ form), e.g.:

- `example-package-1.0.0-1`
- `example-package-1:1.0.0-1`

In each of these directories, one **ALPM-MTREE**, one **alpm-db-desc** and one **alpm-db-files** file are kept to describe a package, e.g.:

```text
.
└── example-package-1.0.0-1
    ├── desc
    ├── files
    └── mtree
```

### Schema version

The current schema version of the system database is encoded in the top-level version file `ALPM_DB_VERSION`.
This ASCII text file contains a single line with a numeric version string, e.g.:

```text
9
```

When changes to the format of the system database are necessary, they are indicated by incrementing the version string intended for the version file.

Dedicated tools such as **pacman-db-upgrade** are used to upgrade the contents of the system database (e.g. metadata files, directory structure) and eventually update the version in `ALPM_DB_VERSION`.

## Creation

The data in an **alpm-db** is derived from **alpm-package** files and created using package management software such as **pacman**.
For the **alpm-db-desc** entries, the package's **PKGINFO** data, as well as the properties of the package file are used.
The **alpm-repo-files** file is directly derived from the package file's list of data files.
The **ALPM-MTREE** file is a copy of the package's **ALPM-MTREE** file.

```text
              alpm-db -----------.
              /  |  \             \
             /   |   \             \
alpm-db-files    |    alpm-db-desc  |
    |            |         |        |
    |         ALPM-MTREE   |        |
 data files      |         |        |
       \         |       PKGINFO   /
        \        |     /          /
         \       |    /          /
          alpm-package-----------
```

# EXAMPLES

## Installing a package on a system

Assuming an empty system, the installation of a package `example-package` in version `1.0.0-1` leads to the following addition in the **alpm-db**:

```text
.
└── example-package-1.0.0-1
    ├── desc
    ├── files
    └── mtree
```

## Upgrading an existing package on a system

Extending on the previous example on **installing a package on a system**, the upgrade of a package `example-package` to version `1.1.0-1` leads to the following change in the **alpm-db**:

```text
.
└── example-package-1.1.0-1
    ├── desc
    ├── files
    └── mtree
```

# SEE ALSO

**ALPM-MTREE**(5), **PKGINFO**(5), **alpm-db-desc**(5), **alpm-db-files**(5), **alpm**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-version**(7), **pacman**(8), **pacman-db-upgrade**(8)
