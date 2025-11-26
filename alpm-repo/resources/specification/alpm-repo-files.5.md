# NAME

alpm-repo-files - File format for listing files and directories contained in an **alpm-package**.

# DESCRIPTION

The **alpm-files** format is a textual format that lists the directories and files contained in a single **alpm-package**.

Such files are named **files** and are located in **alpm-repo-db** files.
They are created by package repository management software (such as **dbscripts** [1]).
Package management software (e.g. **pacman**) uses them to list and query the paths of directories and files contained in a package file.

It is important to not confuse **alpm-repo-files** with **alpm-db-files**, which uses the same file name (**files**) but is used in the context of an **alpm-db**.

## General Format

An **alpm-repo-files** file is a UTF-8 encoded, newline-delimited file.

The file contains a single section header and zero or more section entries.
Empty lines are ignored.
Comments are not supported.

The first line must be the string literal `%FILES%`, which represents the section header.
Packages with no files (e.g. **alpm-meta-packages**) may contain only this section header.

All subsequent lines are interpreted as section entries, one per line, sorted in lexical order.
Each section entry represents either a single file or directory path.
The following formatting rules apply to the paths:

- Directories are always listed with a trailing slash (`/`).
- All paths are relative to the system root (i.e., `/usr/bin/foo` is represented as `usr/bin/foo`).

# EXAMPLES

The following is an example of an **alpm-repo-files** file for a package named `foo`:

```text
%FILES%
usr/
usr/bin/
usr/bin/foo
usr/share/
usr/share/bash-completion/
usr/share/bash-completion/completions/
usr/share/bash-completion/completions/foo
usr/share/doc/
usr/share/doc/foo/
usr/share/doc/foo/README.md
usr/share/fish/
usr/share/fish/vendor_completions.d/
usr/share/fish/vendor_completions.d/foo.fish
usr/share/licenses/
usr/share/licenses/foo/
usr/share/licenses/foo/LICENSE-MIT
usr/share/zsh/
usr/share/zsh/site-functions/
usr/share/zsh/site-functions/_foo
```

The packaged files include a binary (`usr/bin/foo`), a README file (`usr/share/doc/foo/README.md`), and a license file (`usr/share/licenses/foo/LICENSE-MIT`), among other files such as completion scripts for various shells.

# SEE ALSO

**alpm-db-files**(5), **alpm**(7), **alpm-db**(7), **alpm-package**(7), **alpm-package-name**(7), **alpm-package-version**(7), **alpm-repo**(7), **alpm-repo-db**(7), **pacman**(8)

# NOTES

1. **dbscripts**

   <https://gitlab.archlinux.org/archlinux/dbscripts/>
