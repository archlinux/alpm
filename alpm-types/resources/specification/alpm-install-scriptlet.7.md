# NAME

alpm-install-scriptlet – a custom install script for ALPM based packages.

# DESCRIPTION

**install** specifies an optional shell script that defines custom actions to be performed during the installation, upgrade, or removal of a package.

Such files are located at the root of ALPM packages and are named **.INSTALL**.

## General Format

An install script consists of shell functions that define actions for different package lifecycle events.
These functions are optional, and you can include only those necessary for your package.

The script may be written in a shell language that supports the `-c` commandline option and calling named functions with additional arguments from the interpreter's commandline interface.

Please note that the shell interpreter used by package management software is likely globally defined (per distribution). This means that it cannot be changed on a per-package basis. The absence of a shebang in the script implies that the decision on which shell to use occurs outside the package itself, typically at the distribution or system level.

## Functions

The available functions are listed below.
They accept one or two arguments which are package versions, provided as **alpm-package-version**.

```
pre_install
```

Executed before a package is installed, with the new package version as its argument.

```
post_install
```

Executed after a package is installed, with the new package version as its argument.

```
pre_upgrade
```

Executed before a package is upgraded, with the new package version and the old package version as its arguments.

```
post_upgrade
```

Executed after a package is upgraded, with the new package version and the old package version as its arguments.

```
pre_remove
```

Executed before a package is removed, with the old package version as its argument.

```
post_remove
```

Executed after a package is removed, with the old package version as its argument.

# EXAMPLES

Example of specifying an install script in the **PKGBUILD** file:

```
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

bash(1), sh(1), PKGBUILD(5), alpm-package-version(7), pacman(8), makepkg(8)
