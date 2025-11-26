# NAME

alpm-db-files - File format for listing files, directories and backup metadata contained in an installed **alpm-package**.

# DESCRIPTION

The **alpm-db-files** format is a textual format that lists the directories and files contained in a single **alpm-package**.
In addition, it encodes the checksum for any file in a package that should be backed up in case it is changed on the target system (see the **backup** keyword in **PKGBUILD**, **PKGINFO** and **SRCINFO** for the relevant functionality).

Such files are named **files** and are located in **alpm-db** structures.
They are created by package management software (e.g. **pacman**) from the contents of **alpm-package** files (for the **backup** functionality with the help of **PKGINFO**).
Package management software uses them to list and query the paths of directories and files contained in a package file and to track the state of files that should be backed up in case they are changed on the target system.

It is important to not confuse **alpm-db-files** with **alpm-repo-files**, which uses the same file name (**files**) but is used in the context of an **alpm-repo-db**.

## General Format

An **alpm-db-files** file is a UTF-8 encoded, newline-delimited file.

The file may be empty, or contain one or two section headers with one or more section entries each.
Empty lines are ignored.
Comments are not supported.

## Sections

Each section header line contains the section name in all capital letters, surrounded by percent signs (e.g. `%FILES%`).
Section names serve as key for each section-specific value.

Each section allows for one or more _section-specific values_, following the _section header line_.

Note, that if a package tracks no files (e.g. **alpm-meta-package**), then none of the following sections are present, and the **alpm-db-files** file is empty.

### %FILES%

Each value in this section represents either a single file or directory path.
The following formatting rules apply to the paths:

- Directories are always listed with a trailing slash (`/`).
- All paths are relative to the system root (i.e., `/usr/bin/foo` is represented as `usr/bin/foo`).

### %BACKUP%

Each value in this section is represented by a single file path, a tab and the **MD-5**(1) hash digest of the file as found in the **alpm-package** that the **alpm-db-files** file describes (e.g. `etc/foo.conf d41d8cd98f00b204e9800998ecf8427e`.

All file paths must be relative to the system root (i.e., `/usr/bin/foo` is represented as `usr/bin/foo`)

This section can only be present, if the `%FILES%` section is also present and each file path is expected to also be present as a value in the `%FILES%` section.

# EXAMPLES

The following is an example of an **alpm-db-files** file for a package named `foo`:

```text
%FILES%
usr/
usr/bin/
usr/bin/foo
usr/share/
usr/share/doc/
usr/share/doc/foo/
usr/share/doc/foo/README.md

%BACKUP%
etc/foo.conf d41d8cd98f00b204e9800998ecf8427e
```

The `%FILES%` section lists the files and directory paths that belong to the package, while the `%BACKUP%` section captures the **MD-5** hash digest of a configuration file as found in the package file.

# SEE ALSO

**PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-repo-files**(5), **alpm-db**(7), **alpm-meta-package**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-repo-db**(7), **pacman**(8)

# NOTES

1. MD-5
   
   <https://en.wikipedia.org/wiki/MD5>
