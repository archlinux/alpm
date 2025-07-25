# NAME

alpm-install-scriptlet – a custom install script for ALPM based packages.

# DESCRIPTION

An optional shell script that defines custom actions to be performed during the installation, upgrade, or removal of a package.

The shell functions are executed in the root (`/`) of the system, so all paths inside of it must be absolute.

Such files are located at the root of ALPM packages and are named **.INSTALL**.

## General Format

An **alpm-install-scriptlet** may contain zero or more specific shell functions that define actions for different package lifecycle events.
All of the functions are optional and only those necessary for a given package may be included.

The script may be written in a shell language that supports the `-c` commandline option and calling named functions with additional arguments from the interpreter's commandline interface.

Note that the shell interpreter used by package management software to run the script is defined globally (e.g. per distribution).
No shebang is used in the script and the decision on which shell to use for running it is not based on a package's metadata, but instead typically set at the distribution or system level.
This implies that the shell interpreter cannot be changed on a per-package basis.

## Functions

The available functions are listed below.
They accept one or two arguments which are package versions, provided as **alpm-package-version**.

```text
pre_install
```

Executed before a package is installed, with the new package version as its argument.

```text
post_install
```

Executed after a package is installed, with the new package version as its argument.

```text
pre_upgrade
```

Executed before a package is upgraded, with the following arguments:

1. New package version
1. Old package version

```text
post_upgrade
```

Executed after a package is upgraded, with the following arguments:

1. New package version
1. Old package version

```text
pre_remove
```

Executed before a package is removed, with the old package version as its argument.

```text
post_remove
```

Executed after a package is removed, with the old package version as its argument.

# EXAMPLES

Example of specifying an install script in the **PKGBUILD** file:

```text
install=example.install
```

Example of a basic example.install script:

```bash
pre_install() {
    echo "Preparing to install package version $1"
}

post_install() {
    echo "Package version $1 installed"
}

pre_upgrade() {
    echo "Preparing to upgrade from version $2 to $1"
}

post_upgrade() {
    echo "Upgraded from version $2 to $1"
}

pre_remove() {
    echo "Preparing to remove package version $1"
}

post_remove() {
    echo "Package version $1 removed"
}
```

# SEE ALSO

**bash**(1), **sh**(1), **PKGBUILD**(5), **alpm-package-version**(7), **pacman**(8), **makepkg**(8)
