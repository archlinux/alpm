# ALPM Scripts

This crate contains development integration for testing and interacting with **A**rch **L**inux **P**ackage **M**anagement.
Currently supported features:

- Download current live data from an Arch Linux Mirror.
- Test existing parsers against downloaded data.

## Documentation

- <https://alpm.archlinux.page/rustdoc/dev_scripts/> for development version of the crate

## Prerequisites

- `rsync` for package and database downloads.
- `tar` to extract packages.
- `git`
- A registered Arch Linux GitLab account and a configured environment to pull from it via `ssh`.
- Disk space (around ~100GB). Especially the package download is quite big:
    - `packages`: ~90GB
    - `databases`: ~1GB
    - `pkg-src-repositories`: ~3GB
    - `aur`: ~1GB

## Workflow

As an example, this is how you would test the `.BUILDINFO` validation:

1. Sync all current packages to a cache location.
   This implicitly extracts their metadata for use in tests.
   Successive calls sync the local cache directory with the remote state.
   ```sh
   cargo run -- test-files download packages
   ```
1. Validate the current metadata in the local cache:
   ```sh
   cargo run -- test-files test build-info
   ```

You can use the `--repository` flag for specifying repositories. The available package repositories are `core`, `extra` and `multilib`.

## Tests against live data

The `test-files` subcommand of the `scripts` binary downloads live data from Arch mirrors and Arch's GitLab to test the parser binaries on them.

Providing test integration for all [file types] is in scope of this crate, as soon as [components] for them exist.

Currently the following file types are supported:

- [x] ALPM-DB-DESC (local pacman database)
- [x] ALPM-DB-FILES (local pacman database)
- [x] ALPM-MTREE
- [ ] ALPM-REPO-DESC
- [ ] ALPM-REPO-FILES
- [x] BUILDINFO
- [x] PKGINFO
- [x] SRCINFO

### Local pacman database

You can validate the metadata installed on your machine by pointing `dev-scripts` at the local pacman database. These tests use the `alpm_db::db` helpers to parse each entry under `/var/lib/pacman/local` (override with `--local-db-path` if needed).

```sh
cargo run -- test-files test local-desc
cargo run -- test-files test local-files --local-db-path /some/chroot/var/lib/pacman/local
```

<!-- somewhere at the bottom of the file -->

[file types]: ./../README.md#file-types
[components]: ./../README.md#components

### Download

There are three different data sources, which can be downloaded individually:

#### Mirror Databases

Calling `test-files download databases` downloads the current repository sync databases from a given mirror and extracts them.
The default destination is `~/.cache/alpm/testing/databases`. A dedicated folder will be created for each package repository.

#### Mirror Packages

Calling `test-files download packages` downloads the current packages from a given mirror and extracts all metadata files from them.
The default destination is `~/.cache/alpm/testing/packages`. A dedicated folder will be created for each package repository.

#### Packages Source Repository

Calling `test-files download pkg-src-repositories` downloads the package source repositories for all active packages and extracts all package metadata files from them.
The default destination is `~/.cache/alpm/testing/pkgsrc`.

#### AUR

Calling `test-files download aur` downloads the current AUR packages and extracts all package metadata files from them.
The default destination is `~/.cache/alpm/testing/aur`.

### Testing

To run the parser tests for a specific file type run `test-files test $FILE_TYPE`. For instance: `test-files test build-info`.

Depending on which file type you want to test, you need to download the respective data first.

`test-files download databases` will contain the following file types:

- `desc`
- `files`

`test-files download packages` will contain the following file types:

- `.INSTALL`
- `.BUILDINFO`
- `.MTREE`
- `.PKGINFO`

`test-files download pkg-src-repositories` will contain the following file types:

- `.SRCINFO`
- `PKGBUILD`

`test-files download aur` will contain the following file types:

- `.SRCINFO`
- `PKGBUILD`
