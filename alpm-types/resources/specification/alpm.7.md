# NAME

alpm - Arch Linux Package Management.

# DESCRIPTION

**A**rch **L**inux **P**ackage **M**anagement, short ALPM, describes how software is built from source, offered in a dedicated package format and distributed to users of the **Arch Linux**[1] distribution.

The format used for the distribution of software as prebuilt binary artifacts is openly accessible and is documented on a high level in the following sections.
As such, it can be used by other interested parties beyond the scope of what **Arch Linux**[1] offers.

## Building from source

Software is built from an **alpm-source-repo** per upstream.
This process is abstracted using dedicated package build tools such as **makepkg**, that rely on package build scripts (e.g. **PKGBUILD**) to automate the steps of downloading and verifying upstream sources, of building these sources using their respective build systems, running tests for them and installing the resulting binary artifacts.

Once the software is built successfully, available unit tests are run to ensure that the given project can be integrated with the system that it has been built against.
Finally, the software is installed to an empty output directory destination.
Here, the package build tool also creates necessary metadata files, such as **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

Generally, it is desirable to build software in secluded environments that can be setup reproducibly.
This is one cornerstone for achieving **reproducible builds**[2] of the built artifacts.
Arch Linux's canonical packaging tool **pkgctl** creates clean chroot environments with the help of **systemd-nspawn** and executes **makepkg** in them.
The **makepkg** tool is able to record relevant metadata of the current system environment in a **BUILDINFO** file, which allows to setup an identical environment again.

## Creating packages

An **alpm-package** is created from the output directory when **building from source**.
Package files are optionally compressed archives, that contain files that the upstream project installs using its build system, an optional **alpm-install-scriptlet** and the ALPM specific metadata files **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

Once a package is created, it can be digitally signed.
ALPM currently supports detached **OpenPGP signatures**[3] for this purpose.
With the help of digital signatures the authenticity of a package file can later be verified using the packager's **OpenPGP certificate**[4].

## Maintaining package repositories

An **alpm-repo** is a collection of unique **alpm-package** files in specific versions and an **alpm-repo-database** which describes this particular state.
Each package file is described by an **alpm-repo-desc** file in the **alpm-repo-database**.
This file is created from a combination of the package files' **PKGINFO** data, the optional digital signature and the metadata of the package file itself.

Package repositories are maintained with the help of dedicated tools such as **repo-add**.
To serve more complex and evolved repository setups, while allowing access to a larger set of package maintainers, Arch Linux relies on **dbscripts**[5].

## Installing packages

ALPM based packages are installed using package management software such as **pacman**.
While packages can be installed and upgraded individually, they are mostly used via package repositories.
For this, the package management software downloads the **alpm-repo-database** files of **alpm-repos** it is configured to use.
Based on their data, it can compare the state of a package repository and the package files contained in them with the state of a local system.
If newer package versions are detected in the **alpm-repo-database**, the package management software downloads these new package files and installs them.

The installation of a package file implies several things:

- The removal of all files from the filesystem, that are provided by the package in the previously installed version.
- The extraction of all files to the filesystem, that are provided by the new version of the package.
- The update of the system's metadata which tracks what version of a given package is currently installed.

# SEE ALSO

**pkgctl**(1), **systemd-nspawn**(1), **BUILDINFO**(5), **PKGBUILD**(5), **alpm-package**(7), **alpm-repo-database**(7), **alpm-repo**(7), **alpm-source-repo**(7), **makepkg**(8), **pacman**(8), **repo-add**(8)

# NOTES

1. Arch Linux

   https://archlinux.org

2. reproducible builds

   https://reproducible-builds.org

3. OpenPGP signatures

   https://openpgp.dev/book/signing_data.html#detached-signatures

4. OpenPGP certificate

   https://openpgp.dev/book/certificates.html

5. dbscripts

   https://gitlab.archlinux.org/archlinux/dbscripts
