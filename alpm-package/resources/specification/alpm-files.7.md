# NAME

FILES - File listing format for ALPM package database entries.

# DESCRIPTION

The **FILES** format is a textual representation of the installed files of a package.

A FILES file is part of the libalpm database and lists all files installed by a package, relative to the system root.
Package managers and other tools use it to query the installed file paths for each package.

## General Format

A **FILES** file is a UTF-8 encoded, newline-delimited file.

The first non-empty line must be the literal `%FILES%`, marking the beginning of the file listing section.

All subsequent lines are interpreted as **file entries**, one per line and sorted in lexical order.

Each file entry represents either a directory or a file path that is part of the package contents.

- Directories are always listed with a trailing slash (`/`).
- All paths are relative to the system root (i.e., `/usr/bin/foo` is represented as `usr/bin/foo`).

No comment lines are allowed in this format. Empty lines are ignored.

# EXAMPLES

The following is an example of a **FILES** file for a package named `foo`:

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

The installed package will contain a binary (`usr/bin/foo`), a README file (`usr/share/doc/foo/README.md`), and a license file (`usr/share/licenses/foo/LICENSE-MIT`), among other files such as completion scripts for various shells.

# SEE ALSO

**alpm-package**(7), **alpm-desc**(7), **libalpm**(3)
