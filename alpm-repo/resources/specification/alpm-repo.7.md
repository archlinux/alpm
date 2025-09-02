# NAME

alpm-repo - **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package repositories.

# DESCRIPTION

Package repositories are directories containing **alpm-repo-db**, **alpm-package** and digital signature files.
The contents of such a database describes the state of an ALPM package repository (**alpm-repo**).
Here, **alpm-repo-desc** and **alpm-repo-files** files provide metadata on specific package versions currently considered in a package repository.

Repository database files are created from **alpm-package** files using package repository management software (e.g. **dbscripts**[1] which relies on **repo-add**).

Package management software (e.g. **pacman**) relies on **alpm-repo-db** files for the purpose of search, dependency resolution and download of package files.

In some contexts, repository databases may be referred to as _repository sync databases_ or _repository metadata_.

## Variants

Two variants of repository databases exist: _default_ and _default with files_.

- _default_: The database contains one **alpm-repo-desc** file per **alpm-package** in the repository.
  This repository database variant allows package management software to query relevant metadata for search and download of package files from a repository.
- _default with files_: The database contains one **alpm-repo-desc** and one **alpm-repo-files** file per **alpm-package** in the repository.
  In addition to querying relevant metadata for search and download of package files from a repository, this repository database variant allows package management software to search through a simple file list per package.

## Format

_Uncompressed_ repository database files in the variant _default_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.db.tar' (e.g. `example.db.tar`).

_Uncompressed_ repository database files in the variant _default with files_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.files.tar' (e.g. `example.files.tar`).

## Digital signatures

Digital signatures can be created for repository database files.

Currently, only **OpenPGP signatures**[2] over the repository database file are supported.
Detached signatures carry the same name as the repository database file for which they are created, with an additional `.sig` suffix (e.g. `example.db.tar.zst.sig` is the digital signature for a repository database file `example.db.tar.zst`).

## Symlinks

For each variant of a repository database as well as for each of their digital signatures, a symlink is created, that represents an archive and compression agnostic file name.

```text
.
|-- example.db -> example.db.tar.gz
|-- example.db.sig -> example.db.tar.gz.sig
|-- example.db.tar.gz
|-- example.db.tar.gz.sig
|-- example.files -> example.files.tar.gz
|-- example.files.sig -> example.files.tar.gz.sig
|-- example.files.tar.gz
`-- example.files.tar.gz.sig
```

These file names are used by package management software to download package repository databases regardless of compression algorithms, which allows to change them on the fly.

## Contents

The contents of a repository database depend on its variant and the number of unique **alpm-package** files in an **alpm-repo** that should be described by it.
Here, each package can only be present in a single version in the repository database.
In both variants, metadata of a package is kept in a top-level directory, named after the package and its specific version, accordinng to the following schema:

An **alpm-package-name** directly followed by a `-` sign, directly followed by an **alpm-package-version** (in the _full_ or _full with epoch_ form).

### Default

The _default_ repository database variant keeps one **alpm-repo-desc** file per package in the repository database, e.g.:

```text
.
`-- example-package-1.0.0-1
    `-- desc
```

### Default with files

The _default with files_ repository database variant keeps one **alpm-repo-desc** and one **alpm-repo-files** file per package in the repository database, e.g.:

```text
.
`-- example-package-1.0.0-1
    |-- desc
    `-- files
```

## Creation

ALPM repository databases are created using one or more **alpm-package** files and their respective optional digital signatures.
For the **alpm-repo-desc** entries, the package's **PKGINFO** data, as well as the properties of the package file and its optional digital signature are used.
The **alpm-repo-files** file is directly derived from the package file's list of data files.

```text
            alpm-package - - - - digital signature
           /      |     \           /
   data files   PKGINFO  \         /
      /                \  \       /
     /                  \  \     /
    |                    \  |   /
    |     alpm-repo-db    \ |  /
    |    /            \   | | /
alpm-repo-files    alpm-repo-desc
```

# EXAMPLES

## Adding a package to an empty repository database

Given a repository named `example` and a package named `example-package`, the following example explores the use and transformation of metadata when adding a package to a package repository.

Assuming to start off with an empty package repository, the repository databases will exist already:

```text
.
|-- example.db -> example.db.tar.gz
|-- example.db.sig -> example.db.tar.gz.sig
|-- example.db.tar.gz
|-- example.db.tar.gz.sig
|-- example.files -> example.files.tar.gz
|-- example.files.sig -> example.files.tar.gz.sig
|-- example.files.tar.gz
`-- example.files.tar.gz.sig
```

The `example` repository does not contain any packages yet, both `example.db.tar.gz` and `example.files.tar.gz` are empty.

First, the package file and its digital signature are put into the repository directory to bring it into scope:

```text
.
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
|-- example.db -> example.db.tar.gz
|-- example.db.sig -> example.db.tar.gz.sig
|-- example.db.tar.gz
|-- example.db.tar.gz.sig
|-- example.files -> example.files.tar.gz
|-- example.files.sig -> example.files.tar.gz.sig
|-- example.files.tar.gz
`-- example.files.tar.gz.sig
```

Repository management software is then used to add `example-package` to the repository database.
Here, the contents of the repository database variants are changed and their respective digital signatures updated.

The _default_ repository database will contain

```text
.
`-- example-package-1.0.0-1
    `-- desc
```

The _default with files_ repository database will contain

```text
.
`-- example-package-1.0.0-1
    |-- desc
    `-- files
```

Only now, package management software is made aware of the package `example-package` in version `1.0.0-1` in the package repository `example`, after downloading the updated repository database.

## Updating a package in a repository database

Extending on the above example of **adding a package to an empty repository database**, the following example illustrates updating package metadata in a repository database.
Assuming to start off with a package repository, that already contains the package `example-package` in version `1.0.0-1`, the repository directory should look something like this:

```text
.
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
|-- example.db -> example.db.tar.gz
|-- example.db.sig -> example.db.tar.gz.sig
|-- example.db.tar.gz
|-- example.db.tar.gz.sig
|-- example.files -> example.files.tar.gz
|-- example.files.sig -> example.files.tar.gz.sig
|-- example.files.tar.gz
`-- example.files.tar.gz.sig
```

The _default_ repository database will contain

```text
.
`-- example-package-1.0.0-1
    `-- desc
```

The _default with files_ repository database will contain

```text
.
`-- example-package-1.0.0-1
    |-- desc
    `-- files
```

If the package `example-package` receives an upgrade (e.g. for version `1.1.0-1`), this new package file and its corresponding digital signature are copied to the repository directory:

```text
.
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst
|-- example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
|-- example-package-1.1.0-1-x86_64.pkg.tar.zst
|-- example-package-1.1.0-1-x86_64.pkg.tar.zst.sig
|-- example.db -> example.db.tar.gz
|-- example.db.sig -> example.db.tar.gz.sig
|-- example.db.tar.gz
|-- example.db.tar.gz.sig
|-- example.files -> example.files.tar.gz
|-- example.files.sig -> example.files.tar.gz.sig
|-- example.files.tar.gz
`-- example.files.tar.gz.sig
```

At this point, both repository database variants still advertise the package `example-package` in version `1.0.0-1`.
However, the package files and digital signatures of both version `1.0.0-1` and version `1.1.0-1` are present in the repository directory.

Repository management software is then used to add `example-package` in version `1.1.0-1` to the repository database.
As only one version of `example-package` may exist in a given repository database, the metadata for `1.0.0-1` is removed.

The _default_ repository database will contain

```text
.
`-- example-package-1.1.0-1
    `-- desc
```

The _default with files_ repository database will contain

```text
.
`-- example-package-1.1.0-1
    |-- desc
    `-- files
```

# SEE ALSO

**pkgctl**(1), **PKGINFO**(5), **pacman.conf**(5), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-version**(7), **alpm-repo-db**(7), **alpm-repo-name**(7), **pacman**(8), **repo-add**(8)

# NOTES

1. **dbscripts**
   
   <https://gitlab.archlinux.org/archlinux/dbscripts>
1. **OpenPGP signatures**
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
