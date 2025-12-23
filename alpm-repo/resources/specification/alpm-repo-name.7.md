# NAME

alpm-repo-name - names for **A**rch **L**inux **P**ackage **M**anagement (ALPM) based package repositories.

# DESCRIPTION

The **alpm-repo-name** format specifies names for ALPM based package repositories.
This format is used in configuration files for package management software (e.g. **pacman.conf** for **pacman**) and in **alpm-repo-database** file names.

A repository name serves as identifier for package management software and allows it to download and address **alpm-repo-database** files.

## Format

The **alpm-repo-name** value may be a UTF-8 string that is at least one character long.
It must not contain the characters `/`, `?`, `!` and must not start with the `-` character.

# EXAMPLES

```text
repo-name
```

# SEE ALSO

**pacman.conf**(5), **alpm-package-name**(7), **alpm-repo**(7), **alpm-repo-db**(7), **pacman**(8), **repo-add**(8)
