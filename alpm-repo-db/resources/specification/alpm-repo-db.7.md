# NAME

repo-db - a database format for describing the metadata of **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package repositories.

# DESCRIPTION

Repository databases are (optionally compressed) **tar** archives that contain directories and metadata files.
The contents of such a database describes the state of an ALPM package repository (**alpm-repo**).
Here, **alpm-repo-desc** and **alpm-repo-files** files provide metadata on all specific package versions currently present in a package repository.

Repository database files are created from **alpm-package** files using package repository management software (e.g. **dbscripts**[1] which relies on **repo-add**).

Package management software (e.g. **pacman**) relies on **alpm-repo-db** files for the purpose of search, dependency resolution and download of package files.

In some contexts, repository databases may be referred to as _repository sync databases_ or _repository metadata_.

# VARIANTS

Two variants of repository databases exist: _default_ and _default with files_.

- _default_: The database contains one **alpm-repo-desc** file per **alpm-package** in the repository.
  This repository database variant allows package management software to query relevant metadata for search and download of package files from a repository.
- _default with files_: The database contains one **alpm-repo-desc** and one **alpm-repo-files** file per **alpm-package** in the repository.
  In addition to querying relevant metadata for search and download of package files from a repository, this repository database variant allows package management software to search through a simple file list per package.

# FORMAT

_Uncompressed_ repository database files in the variant _default_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.db.tar', e.g.:

- `example.db.tar`

_Uncompressed_ repository database files in the variant _default with files_ follow the following naming scheme:

An **alpm-repo-name** directly followed by the string '.files.tar', e.g.:

- `example.files.tar`

## Compression

Repository databases may optionally be compressed using a single supported compression technology.
If a package is compressed, a technology-specific suffix is appended to the file name:

- `.Z` for compression based on adaptive Lempel-Ziv coding (e.g. `example.db.tar.Z`, or `example.files.tar.Z`), see the **compress** command
- `.bz2` for **bzip2** compression (e.g. `example.db.tar.bz2`, or `example.files.tar.bz2`)
- `.gz` for **gzip** compression (e.g. `example.db.tar.gz`, or `example.files.tar.gz`)
- `.lrz` for **lrzip** compression (e.g. `example.db.tar.lrz`, or `example.files.tar.lrz`)
- `.lz4` for **lz4** compression (e.g. `example.db.tar.lz4`, or `example.files.tar.lz4`)
- `.lz` for **lzip** compression (e.g. `example.db.tar.lz`, or `example.files.tar.lz`)
- `.lzo` for **lzop** compression (e.g. `example.db.tar.lzo`, or `example.files.tar.lzo`)
- `.xz` for **xz** compression (e.g. `example.db.tar.xz`, or `example.files.tar.xz`)
- `.zst` for **zstd** compression (e.g. `example.db.tar.zst`, or `example.files.tar.zst`)

Handling of compression technologies is specific to the package repository management tool.

## Digital signatures

Digital signatures can be created for repository database files.

Currently, only **OpenPGP detached signatures** over the repository database file are supported.
Detached signatures carry the same name as the repository database file for which they are created, with an additional `.sig` suffix (e.g. `example.db.tar.zst.sig` is the digital signature for a repository database file `example.db.tar.zst`).

# CONTENTS

The contents of a repository database depend on its variant and the number of unique **alpm-package** files in an **alpm-repo** that should be described by it.
Here, each package can only be present in a single version in the repository database.
In both variants, metadata of a package is kept in a top-level directory, named after the package and its specific version, accordinng to the following schema:

An **alpm-package-name** directly followed by a `-` sign, directly followed by an **alpm-package-version** (in the _full_ or _full with epoch_ form).

## Default

The _default_ repository database variant keeps one **alpm-repo-desc** file per package in the repository database, e.g.:

```text
.
└── example-package-1.0.0-1
    └── desc
```

## Default with files

The _default with files_ repository database variant keeps one **alpm-repo-desc** and one **alpm-repo-files** file per package in the repository database, e.g.:

```text
.
└── example-package-1.0.0-1
    └── desc
    └── files
```

# CREATION

ALPM repository databases are created using one or more **alpm-package** files and their respective optional digital signatures.
For the **alpm-repo-desc** entries, the package's **PKGINFO** data, as well as the properties of the package file and its optional digital signature are used.
The **alpm-repo-files** file is directly derived from the package file's file list.

# EXAMPLES


# SEE ALSO

**bzip2**(1), **compress**(1), **gzip**(1), **lrzip**(1), **lz4**(1), **lzip**(1), **lzop**(1), **pkgctl**(1), **tar**(1), **xz**(1), **zstd**(1), **ALPM-MTREE**(5), **BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **makepkg.conf**(5), **SRCINFO**(7), **alpm-architecture**(7), **alpm-install-scriptlet**(7), **alpm-meta-package**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **alpm-package-version**(7), **alpm-split-package**(7), **devtools**(7), **makepkg**(8), **pacman**(8), **repo-add**(8)

# NOTES

1. **OpenPGP detached signatures**

   https://openpgp.dev/book/signing_data.html#detached-signatures

1. **reproducible builds**

   https://reproducible-builds.org/

1. **root directory**

   https://en.wikipedia.org/wiki/Root_directory

1. **systemd File Hierarchy Requirements**

   https://systemd.io/SYSTEMD_FILE_HIERARCHY_REQUIREMENTS/

1. **Filesystem Hierarchy Standard**

   https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard
