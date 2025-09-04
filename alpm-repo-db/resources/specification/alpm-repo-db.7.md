# NAME

alpm-repo-db - a database format for describing the metadata of **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package repositories.

# DESCRIPTION

Repository databases are (optionally compressed) **tar** archives that contain directories and metadata files.
The contents of such a database describe the state of an ALPM package repository (**alpm-repo**).
Here, **alpm-repo-desc** and **alpm-repo-files** files provide metadata on specific package versions currently considered in a package repository.

Repository database files are created from **alpm-package** files using package repository management software (e.g. **dbscripts**[1] which relies on **repo-add**).

Package management software (e.g. **pacman**) relies on **alpm-repo-db** files for the purpose of search, dependency resolution and download of package files.

In some contexts, repository databases may be referred to as _repository sync databases_ or _repository metadata_.

## Variants

Two variants of repository databases exist: _default_ and _default with files_.

1. _default_: The database contains one **alpm-repo-desc** file per **alpm-package** in the repository.
   This repository database variant allows package management software to query relevant metadata for search and download of package files from a repository.
1. _default with files_: The database contains one **alpm-repo-desc** and one **alpm-repo-files** file per **alpm-package** in the repository.
   In addition to querying relevant metadata for search and download of package files from a repository, this repository database variant allows package management software to search through a simple file list per package.

## Format

_Uncompressed_ repository database files in the variant _default_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.db.tar' (e.g. `repo.db.tar`).

_Uncompressed_ repository database files in the variant _default with files_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.files.tar' (e.g. `repo.files.tar`).

## Compression

Repository databases may optionally be compressed using a single supported compression technology.
If an **alpm-repo-db** is compressed, a technology-specific suffix is appended to the file name:

- `.Z` for compression based on adaptive Lempel-Ziv coding (e.g. `repo.db.tar.Z`, or `repo.files.tar.Z`), see the **compress** command
- `.bz2` for **bzip2** compression (e.g. `repo.db.tar.bz2`, or `repo.files.tar.bz2`)
- `.gz` for **gzip** compression (e.g. `repo.db.tar.gz`, or `repo.files.tar.gz`)
- `.lrz` for **lrzip** compression (e.g. `repo.db.tar.lrz`, or `repo.files.tar.lrz`)
- `.lz4` for **lz4** compression (e.g. `repo.db.tar.lz4`, or `repo.files.tar.lz4`)
- `.lz` for **lzip** compression (e.g. `repo.db.tar.lz`, or `repo.files.tar.lz`)
- `.lzo` for **lzop** compression (e.g. `repo.db.tar.lzo`, or `repo.files.tar.lzo`)
- `.xz` for **xz** compression (e.g. `repo.db.tar.xz`, or `repo.files.tar.xz`)
- `.zst` for **zstd** compression (e.g. `repo.db.tar.zst`, or `repo.files.tar.zst`)

Handling of compression technologies is specific to the package repository management tool.

## Digital signatures

Digital signatures can be created for repository database files.

Currently, only **OpenPGP signatures**[2] over the repository database file are supported.
Detached signatures carry the same name as the repository database file for which they are created, with an additional `.sig` suffix (e.g. `repo.db.tar.zst.sig` is the digital signature for a repository database file `repo.db.tar.zst`).

## Symlinks

For each variant of a repository database as well as for each of their digital signatures, a symlink is created, that represents an archive and compression agnostic file name.

```text
.
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

These file names are used by package management software to download package repository databases regardless of compression algorithms, which allows to change them on the fly.

## Contents

The contents of a repository database depend on the database variant and the number of unique **alpm-package** files in the **alpm-repo** it describes.
Here, each package can only be present in a single version in the repository database.
In both variants, metadata of a package is kept in a top-level directory, that is named after the package and its specific version.
The name follows this schema:

An **alpm-package-name** directly followed by a `-` sign, directly followed by an **alpm-package-version** (in the _full_ or _full with epoch_ form), e.g.:

- `example-package-1.0.0-1`
- `example-package-1:1.0.0-1`

### Default

The _default_ repository database variant keeps one **alpm-repo-desc** file per package in the repository database, e.g.:

```text
.
└── example-package-1.0.0-1
    └── desc
```

### Default with files

The _default with files_ repository database variant keeps one **alpm-repo-desc** and one **alpm-repo-files** file per package in the repository database, e.g.:

```text
.
└── example-package-1.0.0-1
    ├── desc
    └── files
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

Given a repository named `repo` and a package named `example-package`, the following example explores the use and transformation of metadata when adding a package to a package repository.

Assuming to start off with an empty package repository, the repository databases will exist already:

```text
.
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

The `repo` repository does not contain any packages yet, both `repo.db.tar.gz` and `repo.files.tar.gz` are empty.

First, the package file and its digital signature are put into the repository directory to bring it into scope:

```text
.
├── example-package-1.0.0-1-x86_64.pkg.tar.zst
├── example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

Repository management software is then used to add `example-package` to the repository database.
Here, the contents of the repository database variants are changed and their respective digital signatures updated.

The _default_ repository database will contain:

```text
.
└── example-package-1.0.0-1
    └── desc
```

The _default with files_ repository database will contain:

```text
.
└── example-package-1.0.0-1
    ├── desc
    └── files
```

Only now, package management software is made aware of the package `example-package` in version `1.0.0-1` in the package repository `repo`, after downloading the updated repository database.

## Updating a package in a repository database

Extending on the above example of **adding a package to an empty repository database**, the following example illustrates updating package metadata in a repository database.
Assuming to start off with a package repository `repo`, that already contains the package `example-package` in version `1.0.0-1`, the repository directory should look something like this:

```text
.
├── example-package-1.0.0-1-x86_64.pkg.tar.zst
├── example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

The _default_ repository database will contain:

```text
.
└── example-package-1.0.0-1
    └── desc
```

The _default with files_ repository database will contain:

```text
.
└── example-package-1.0.0-1
    ├── desc
    └── files
```

If the package `example-package` receives an upgrade (e.g. for version `1.1.0-1`), this new package file and its corresponding digital signature are copied to the repository directory:

```text
.
├── example-package-1.0.0-1-x86_64.pkg.tar.zst
├── example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
├── example-package-1.1.0-1-x86_64.pkg.tar.zst
├── example-package-1.1.0-1-x86_64.pkg.tar.zst.sig
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

At this point, both repository database variants still advertise the package `example-package` in version `1.0.0-1`.
However, the package files and digital signatures of both version `1.0.0-1` and version `1.1.0-1` are present in the repository directory.

Repository management software is then used to add `example-package` in version `1.1.0-1` to the repository database.
As only one version of `example-package` may exist in a given repository database, the metadata for `1.0.0-1` is removed.

The _default_ repository database will contain:

```text
.
└── example-package-1.1.0-1
    └── desc
```

The _default with files_ repository database will contain:

```text
.
└── example-package-1.1.0-1
    ├── desc
    └── files
```

# SEE ALSO

**bzip2**(1), **compress**(1), **gzip**(1), **lrzip**(1), **lz4**(1), **lzip**(1), **lzop**(1), **pkgctl**(1), **tar**(1), **xz**(1), **zstd**(1), **PKGINFO**(5), **alpm-repo-desc**(5), **alpm-repo-files**(5), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-version**(7), **alpm-repo**(7), **alpm-repo-name**(7), **pacman**(8), **repo-add**(8)

# NOTES

1. **dbscripts**
   
   <https://gitlab.archlinux.org/archlinux/dbscripts>
1. **OpenPGP signatures**
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
