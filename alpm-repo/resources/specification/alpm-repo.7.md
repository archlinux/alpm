# NAME

alpm-repo - **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package repositories.

# DESCRIPTION

Package repositories are directories containing an **alpm-repo-db** and zero or more **alpm-package** files and the respective digital signatures of each.
An **alpm-repo-db** provides metadata on the specific packages that are currently considered to be part of the package repository.
The name of the **alpm-repo-db** must be the same as that of the package repository (a valid **alpm-repo-name**).

Package repositories are usually provided centrally and exposed publicly on web servers.
As such, they may be mirrored in several different locations, synchronizing contents from the central location to distribute load and to make their contents available world wide.

The state and contents of a package repository are maintained using package repository management software (e.g. **dbscripts**[1] which relies on **repo-add**).

Package management software (e.g. **pacman**) relies on one or more package repositories to install and upgrade software on a system.

## Repository databases

The role of an **alpm-repo-db** is to define which of the **alpm-package** files in an **alpm-repo** are currently considered to be part of the package repository.
Although multiple versions of a given **alpm-package** may be present as file in an **alpm-repo** only one of them can be tracked as _the current_ in an **alpm-repo-db**.

In the following example repository named `repo`, multiple versions of a package named `example-package` exist, but only one of them is present in the **alpm-repo-db** (e.g. version `1.0.1-1`):

```text
.
├── example-package-1.0.0-1-x86_64.pkg.tar.zst
├── example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
├── example-package-1.0.0-2-x86_64.pkg.tar.zst
├── example-package-1.0.0-2-x86_64.pkg.tar.zst.sig
├── example-package-1.0.1-1-x86_64.pkg.tar.zst
├── example-package-1.0.1-1-x86_64.pkg.tar.zst.sig
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

Package management software, that downloaded the latest version of the **alpm-repo-db** is only aware of `example-package` in version `1.0.1-1`.

A package management system, that still has a copy of an older version of the **alpm-repo-db** is only aware of earlier versions of `example-package` (e.g. `1.0.0-1` or `1.0.0-2`).
For the download of those older versions to work, these specific **alpm-package** files have to be kept in the **alpm-repo** for an unspecified amount of time.
Note, that there is currently no specified default on how long package repositories are obligated to keep old versions.
However, they should always contain the packages specified in their current **alpm-repo-db**.

## Maintaining a repository

Repository management software is used to add, remove and update the contents of a package repository.
Before modifying the **alpm-repo-db** to manifest the current state, the software should e.g. ensure that:

- the targeted **alpm-package** files and digital signatures exist and can be verified,
- there is no duplicate **alpm-package** entry,
- an already existing **alpm-package** entry is updated or removed,
- required run-time dependencies (see **alpm-package-relation**) for a new or updated **alpm-package** are present (or will be present when adding several package files),
- when removing an **alpm-package**, it is no longer required as run-time dependency for another package in the repository (unless that package is also removed)
- each **alpm-package** file can be traced back to a known **alpm-source-repo**.

## Efficient handling of contents

The _current state_ of a package repository can be imagined as _a view_ onto the set of all packages (and their respective versions) present in an **alpm-repo**.
Ideally, this _view_ represents a consistent, functional subset of all **alpm-package** files present in a repository.

Several requirements can be extracted from the sections on **repository databases** and **maintaining a repository** and have to be kept in mind when maintaining a package repository:

- Old versions of package files need to be kept around for an unspecified amount of time.
- Changes to the repository must be atomic.
- All files made available by the latest version of an **alpm-repo-db** must be available for download.

For repository management software it is therefore pivotal to efficiently handle the files in a repository and only make a new version of the **alpm-repo-db** available once required files are available and necessary checks for them have passed.

To allow for files to be added and removed from a repository in a near-instantaneous manner, they can be made available via symlinks.
For this purpose, all **alpm-package** files and their respective digital signatures may be added in a pool directory.
The **alpm-package** files in a package repository are then merely symlinks to files in the pool directory, e.g.:

```text
.
├── example-package-1.0.0-1-x86_64.pkg.tar.zst -> ../../../pool/example-package-1.0.0-1-x86_64.pkg.tar.zst
├── example-package-1.0.0-1-x86_64.pkg.tar.zst.sig -> ../../../pool/example-package-1.0.0-1-x86_64.pkg.tar.zst.sig
├── example-package-1.0.0-2-x86_64.pkg.tar.zst -> ../../../pool/example-package-1.0.0-2-x86_64.pkg.tar.zst
├── example-package-1.0.0-2-x86_64.pkg.tar.zst.sig -> ../../../pool/example-package-1.0.0-2-x86_64.pkg.tar.zst.sig
├── example-package-1.0.1-1-x86_64.pkg.tar.zst -> ../../../pool/example-package-1.0.1-1-x86_64.pkg.tar.zst
├── example-package-1.0.1-1-x86_64.pkg.tar.zst.sig -> ../../../pool/example-package-1.0.1-1-x86_64.pkg.tar.zst.sig
├── repo.db -> repo.db.tar.gz
├── repo.db.sig -> repo.db.tar.gz.sig
├── repo.db.tar.gz
├── repo.db.tar.gz.sig
├── repo.files -> repo.files.tar.gz
├── repo.files.sig -> repo.files.tar.gz.sig
├── repo.files.tar.gz
└── repo.files.tar.gz.sig
```

Using this approach, the addition or removal of files in a repository or the move of files between different repositories is very light in terms of resources.
Here, the resources required for an atomic change to the state of a repository are mostly defined by the checks required for the transaction and updating the metadata of the **alpm-repo-db**.

Similarly, when updating the metadata in an **alpm-repo-db**, it is advisable to first create updated versions of the two supported database _variants_ (and optional digital signatures for them) in a temporary location in the **alpm-repo** and only then move all files to their target location simultaneously.

# SEE ALSO

**alpm-package**(7), **alpm-package-relation**(7), **alpm-repo-db**(7), **alpm-repo-name**(7), **alpm-source-repo**(7), **pacman**(8), **repo-add**(8)

# NOTES

1. **dbscripts**
   
   <https://gitlab.archlinux.org/archlinux/dbscripts>
1. **OpenPGP signatures**
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
