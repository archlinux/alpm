# NAME

alpm - Arch Linux Package Management.

# DESCRIPTION

**A**rch **L**inux **P**ackage **M**anagement (ALPM), describes how software is built from source, packaged in a dedicated package format (see **alpm-package**) and distributed to users of the **Arch Linux**[1] distribution.

The format used for the distribution of software as prebuilt binary artifacts is openly accessible and is documented on a high level in the following sections.
As such, it can be used by other interested parties beyond the scope of what **Arch Linux**[1] offers.

Metadata connections between **alpm-source-repo**, **alpm-package** and **alpm-repo-db**.

```text
     alpm-source-repo
           |
        PKGBUILD
       /        \\
  SRCINFO        \\
                  \\
                   \\
         /------- alpm-package ------\\
        /        /     |      \\       \\
       /   BUILDINFO   |    PKGINFO   |
       |               |       |      |
       |           ALPM-MTREE  |      |
       |                       |      |
       |        alpm-repo-db   |      |
       |       /            \\  |     /
     alpm-repo-files   alpm-repo-desc
```

Metadata connections between **alpm-package** and **alpm-db** (**libalpm**).

```text
      /------- alpm-package ------\\
     /        /     |      \\       \\
    /   BUILDINFO   |    PKGINFO   |
    |               |       |      |
    |           ALPM-MTREE  |      |
    |                       |     /
    |        alpm-db        |    /
    |       /        \\      |   /
  alpm-db-files     alpm-db-desc
```

## Building from source

Data files (e.g. binary or other data artifacts) are created from an **alpm-source-repo**.
This process is abstracted using dedicated package build tools such as **makepkg**, that rely on package build scripts (i.e. **PKGBUILD**).
Build tools automate the steps of downloading and verifying local or upstream source inputs, applying any required modifications, calling any respective build systems, running tests and installing the resulting binary or other data artifacts.
Package build scripts define the framework in which packages are to be created (i.e. their build, run-time and test dependencies, as well as all necessary steps) and provide specific metadata about the packages (e.g. name, version, description, groups).

Generally, it is desirable to create the package data files in secluded environments that can be setup reproducibly (e.g. chroots, containers, or virtual machines).
This is one pillar for achieving **reproducible builds**[2] of the built artifacts.
Arch Linux's canonical packaging tool **pkgctl** creates clean chroot environments with the help of **systemd-nspawn** and executes **makepkg** within them.
The **makepkg** tool is able to record relevant metadata of the current system environment in a **BUILDINFO** file, which allows to setup an identical environment again.

### Download, verification and authentication

The inputs (i.e. the sources) of a package build script may be local or remote files or data in version control systems.
After download, they are verified using locked hash digests for the respective files.
This is a fundamental building block for **reproducible builds**[2] and allows to detect **supply chain attack**[3] vectors that rely on altering source files.
In addition, each input may be authenticated using a cryptographic signature.

### Modification

Package build inputs sometimes need to be modified to fix issues with the input files themselves or to accommodate to the specific behavior of the environment they are supposed to be used in.
Applying patches is a common scenario and is usually done in a preparation step after the download, verification and authentication of the inputs.

### Building

Depending on source input and programming language a diverse set of tools may be required to build binary artifacts out of source code.
A dedicated step after the modification of source inputs is used to generate (binary) data files.

### Testing

After successfully building, any available unit tests are run to ensure that the given project can be integrated with the system that it has been built against.

### Installation

Finally, any generated (binary) data files are installed into an empty output directory (either using the project's build system or manually).
Here, the package build tool also creates necessary metadata files, such as **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

It is possible to create more than one output directory if two or more **alpm-split-package** files are to be created.

## Creating packages

One **alpm-package** file is created from each output directory after **building from source**.
Package files are optionally compressed archives, that contain any files that have been installed into the empty output directory, an optional **alpm-install-scriptlet** and the ALPM specific metadata files **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

Once a package is created, it may be digitally signed.
ALPM currently supports detached **OpenPGP signatures**[4] for this purpose.
With the help of digital signatures the authenticity of a package file can later be verified using the packager's **OpenPGP certificate**[5].

## Maintaining package repositories

An **alpm-repo** is a collection of unique **alpm-package** files in specific versions and an **alpm-repo-database** which describes this particular state.
Each package file is described by an **alpm-repo-desc** file in the **alpm-repo-database**.
This file is created from a combination of the package files' **PKGINFO** data, the optional digital signature and the metadata of the package file itself.

Package repositories are maintained with the help of dedicated tools such as **repo-add**.
To serve more complex and evolved repository setups, while allowing access to a larger set of package maintainers, Arch Linux relies on **dbscripts**[6].

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

3. supply chain attack

   https://en.wikipedia.org/wiki/Supply_chain_attack

4. OpenPGP signatures

   https://openpgp.dev/book/signing_data.html#detached-signatures

5. OpenPGP certificate

   https://openpgp.dev/book/certificates.html

6. dbscripts

   https://gitlab.archlinux.org/archlinux/dbscripts
