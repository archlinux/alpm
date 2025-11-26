# NAME

alpm - Arch Linux Package Management.

# DESCRIPTION

**A**rch **L**inux **P**ackage **M**anagement (ALPM), describes the process of retrieving the sources of upstream projects, optionally building them, bundling them in a dedicated package format (see **alpm-package**) and distributing those package files to users of the **Arch Linux**[1] distribution.

The format used for the distribution of software as prebuilt binary artifacts is openly accessible and is documented on a high level in the following sections.
Aside from **Arch Linux**[1] it may find use in other distributions or on other platforms.

## Disclaimer

Throughout this document the term "repository" or "repo" appears in different contexts:

- **alpm-source-repo**: A repository (e.g. a **git** repository) containing **PKGBUILD** package build scripts, from which one or several **alpm-package** files are built.
  Aside from the package build scripts, an **alpm-source-repo** may also contain a **SRCINFO** file, which provides a collection of metadata on the data defined in the **PKGBUILD** file.
- **alpm-repo**: A repository (e.g. a directory served by a webserver) that provides **alpm-package** files, digital signatures and **alpm-repo-db** files that define the state of the **alpm-repo**.
  Each **alpm-repo-db** contains specific package-related metadata (i.e. **alpm-repo-desc** and **alpm-repo-files**).

Summarized, **alpm-source-repo** is the location from which **alpm-package** files are built, while **alpm-repo** is the location where those files are stored later on.

## Metadata connections

Metadata connections between **alpm-source-repo**, **alpm-package** and **alpm-repo-db**:

```text
     alpm-source-repo
           |
        PKGBUILD
       /        \
  SRCINFO        \
                  \
                   \
         ,------- alpm-package -----.
        /        /     |      \      \
       ,   BUILDINFO   |    PKGINFO   |
       |               |       |      |
       |           ALPM-MTREE  |      |
       |                       |      |
       |        alpm-repo-db   |      |
       |       /            \  |     /
     alpm-repo-files   alpm-repo-desc
```

Metadata connections between **alpm-package** and **alpm-db** (**libalpm**):

```text
         alpm-package ----------.-----------------.
        /     |      \           \                 \
  BUILDINFO   |    PKGINFO        \       alpm-db   \
              |       |   \        \     /       \   \
          ALPM-MTREE  `    `--- alpm-db-files     \   \
                       \                           \   \
                        `------------------------ alpm-db-desc
```

## Building from source

Package data files, such as binary or other data artifacts, are created from an **alpm-source-repo**.
This process is abstracted using dedicated package build tools such as **makepkg**, that rely on **bash** based **PKGBUILD** package build scripts.

Package build scripts define the environment in which packages are created and used.
This includes their source inputs, build, run-time and test dependencies, as well as all necessary steps for the creation of data files, their testing and installation.
Package build scripts also provide metadata about the packages such as name, version, description, groups, source or checksums.
Build tools automate the steps of downloading and verifying local or upstream source inputs, applying any required modifications (e.g. patches), calling any respective build systems, running tests and installing the resulting binary or other data artifacts to a location from which a package file is created.

The **makepkg** tool is able to record relevant metadata of the current system environment in a **BUILDINFO** file.
Based on this file an identical environment can be setup again, which is a prerequisite for **reproducible builds**[2].

Generally, it is desirable to create an **alpm-package** file in a secluded environment that can be setup reproducibly (e.g. chroots, containers, or virtual machines) and only contains the various dependencies of a package (see **alpm-package-relation**).
However, on its own **makepkg** is not able to ensure that a build environment only contains packages that satisfy the various dependencies of a package.
This is why Arch Linux's canonical packaging tool **pkgctl** creates clean chroot environments with the help of **systemd-nspawn** and executes **makepkg** within them.

### Download, verification and authentication

The inputs (i.e. the sources) of a package build script may be local or remote files or data in version control systems.
After download, they are verified using optional locked hash digests (see **alpm-package-source-checksum**) for the respective files.
This is a fundamental building block for **reproducible builds**[2] and allows to detect **supply chain attack**[3] vectors that rely on altering source files.
In addition, each input may be authenticated using a cryptographic signature.

### Modification

Package build inputs sometimes need to be modified to fix issues with the input files themselves or to accommodate to the specific behavior of the environment they are supposed to be used in.
Applying patches is a common scenario and is usually done in a preparation step after the download, verification and authentication of the inputs.

### Building

The next step after the potential modification of source inputs is to generate data files for the **alpm-package** file.
This may include compilation of binary files, translations, assets, plain data or any other type of file that might be included in a package.
Depending on source input and programming language a diverse set of tools may be required to build binary artifacts from source code.

### Testing

After successfully building, any available tests of the respective project are run to ensure that the given project can be integrated with the system that it has been built against.

### Installation

Finally, any generated package data files are installed into an empty output directory, either using the project's build system or manually.
During this step, the package build tool also creates required metadata files, such as **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

When creating more than one **alpm-package** from a **PKGBUILD**, as many output directories as there are packages are created.
For information on creating multiple packages from a single **PKGBUILD**, refer to **alpm-split-package**.

## Creating packages

One **alpm-package** file is created from each output directory after **building from source**, **testing** and **installation**.
Package files are optionally compressed **tar** archives, that contain any files that have been installed into the empty output directory, an optional **alpm-install-scriptlet** and the ALPM specific metadata files **BUILDINFO**, **PKGINFO** and **ALPM-MTREE**.

Once a package is created, it may be digitally signed.
ALPM currently supports detached **OpenPGP signatures**[4] for this purpose.
With the help of digital signatures the authenticity of a package file can later be verified using the packager's **OpenPGP certificate**[5].

## Maintaining package repositories

An **alpm-repo** is a collection of unique **alpm-package** files in specific versions and an **alpm-repo-db** which describes this particular state.
Each package file is described by an **alpm-repo-desc** file in the **alpm-repo-db**.
This file is created from a combination of the package files' **PKGINFO** data, the optional digital signature and the metadata of the package file itself.

Package repositories are maintained with the help of dedicated tools such as **repo-add**.
To serve more complex and evolved repository setups, while allowing access to a larger set of package maintainers, Arch Linux relies on **dbscripts**[6].

## Installing packages

ALPM based packages are installed using package management software such as **pacman**.
While packages can be installed and upgraded individually, they are mostly used via package repositories.
For this, the package management software downloads the **alpm-repo-db** file of each **alpm-repo** it is configured to use.
Based on their data, it can compare the state of all specified package repositories and their package files with the state of a local system.
If newer package versions are detected in the **alpm-repo-db**, the package management software downloads these new package files and installs them.

The installation of a package file implies several things:

- The removal of all files from the filesystem, that are provided by the previously installed package version.
- The addition of all files to the filesystem, that are provided by the new version of the package.
- The update of the system's metadata which tracks what version of a given package is currently installed.

# EXAMPLES

In the following, very basic example the life cycle of a package file and the related metadata is explored.

## Creating a package file from a source repository

The below **PKGBUILD** in an **alpm-source-repo** defines a package that contains a single data file:

```bash
pkgname=example
pkgver=0.1.0
pkgrel=1
pkgdesc="An example package"
arch=(any)
url="https://example.org"
license=(CC-BY-SA-4.0)
backup=(etc/example.conf)

package() {
   install -vdm 755 "$pkgdir/etc/"
   printf "Very important config\n" > "$pkgdir/etc/$pkgname.conf"
   install -vdm 755 "$pkgdir/usr/share/$pkgname/"
   printf "Hello World\!
" > "$pkgdir/usr/share/$pkgname/example.txt"
}
```

It is represented by this **SRCINFO** file:

```ini
pkgbase = example
	pkgdesc = An example package
	pkgver = 0.1.0
	pkgrel = 1
	url = https://example.org
	arch = any
	license = CC-BY-SA-4.0

pkgname = example
```

When building an **alpm-package** in a clean environment from the above **alpm-source-repo**, a package file `example-0.1.0-any.pkg.tar.zst` is created and is accompanied by the detached OpenPGP signature `example-0.1.0-any.pkg.tar.zst.sig`.
The package file contains the **ALPM-MTREE**, **BUILDINFO** and **PKGINFO** metadata files, as well as a single data file and all of its parent directories.

The **ALPM-MTREE** file may look similar to the below:

```text
#mtree
/set type=file uid=0 gid=0 mode=644
./.BUILDINFO time=1752836739.0 size=5271 sha256digest=9924a366a4ad02c31b121b22a2b285b2cae3a57873052169deb9d237936bef83
./.PKGINFO time=1752836739.0 size=297 sha256digest=0c8481c16dfc09ffdb0f518f827109795c1e07816ab7205ffffa6837b92fa4fb
./usr time=1752836739.0 mode=755 type=dir
./usr/share time=1752836739.0 mode=755 type=dir
./usr/share/example time=1752836739.0 mode=755 type=dir
./usr/share/example/example.txt time=1752836739.0 size=14 sha256digest=732c1c47a8296f4525307d28469d7ba1f3f5e4796fe55bc5625febc720a09d92
```

The **BUILDINFO** may look similar to the below (which is truncated for brevity):

```ini
format = 2
pkgname = example
pkgbase = example
pkgver = 0.1.0-1
pkgarch = any
pkgbuild_sha256sum = c2cdacc7de9ed0cb40a9177255701339f1e53f1014950c1793bb34740bfd64e9
packager = John Doe <john@example.org>
builddate = 1752836739
builddir = /build
startdir = /startdir
buildtool = devtools
buildtoolver = 1:1.3.2-1-any
buildenv = !distcc
buildenv = color
buildenv = !ccache
buildenv = check
buildenv = !sign
options = strip
options = docs
options = !libtool
options = !staticlibs
options = emptydirs
options = zipman
options = purge
options = debug
options = lto
installed = acl-2.3.2-1-x86_64
installed = archlinux-keyring-20250716-1-any
```

The **PKGINFO** of the **alpm-package** may look similar to the below:

```ini
pkgname = example
pkgbase = example
xdata = pkgtype=pkg
pkgver = 0.1.0-1
pkgdesc = An example package
url = https://example.org
builddate = 1752836739
packager = John Doe <john@example.org>
size = 14
arch = any
license = CC-BY-SA-4.0
backup = etc/example.conf
```

## Adding a package file to a repository

After adding `example-0.1.0-any.pkg.tar.zst` and `example-0.1.0-any.pkg.tar.zst.sig` to an **alpm-repo** named `example-repo`, its corresponding **alpm-repo-db** is updated.

The dedicated **alpm-repo-desc** for the package may look similar to this:

```text
%FILENAME%
example-0.1.0-1-any.pkg.tar.zst

%NAME%
example

%BASE%
example

%VERSION%
0.1.0-1

%DESC%
An example package

%CSIZE%
2274

%ISIZE%
14

%SHA256SUM%
640d6a9eaebf312273371eb7589338a3f01eb623cddf9f671ee96501d7c65ae1

%PGPSIG%
iHUEABYKAB0WIQRizHP4hOUpV7L92IObeih9mi7GCAUCaBZuVAAKCRCbeih9mi7GCIlMAP9ws/jU4f580ZRQlTQKvUiLbAZOdcB7mQQj83hD1Nc/GwD/WIHhO1/OQkpMERejUrLo3AgVmY3b4/uGhx9XufWEbgE=

%URL%
https://example.org

%LICENSE%
CC-BY-SA-4.0

%ARCH%
any

%BUILDDATE%
1752836739

%PACKAGER%
John Doe <john@example.org>
```

The **alpm-repo-files** for the package file contains:

```text
%FILES%
etc/
etc/example.conf
usr/
usr/share/
usr/share/example/
usr/share/example/example.txt
```

## Installation on a client host

On a client host, the package management software **pacman** is configured to use the **alpm-repo** `example-repo`.
Once its **alpm-repo-db** is synchronized, the system administrator of the host decides to install the `example` package from the repository.
After installation, the client host's **alpm-db** is modified to contain metadata about the `example` package.

It contains the below **alpm-db-desc** file:

```text
%NAME%
example

%VERSION%
0.1.0-1

%BASE%
example

%DESC%
An example package

%URL%
https://example.org

%ARCH%
any

%BUILDDATE%
1752836739

%INSTALLDATE%
1752836973

%PACKAGER%
John Doe <john@example.org>

%SIZE%
14

%LICENSE%
CC-BY-SA-4.0

%VALIDATION%
pgp

%XDATA%
pkgtype=pkg
```

The **alpm-db-files** entry in the system database starts from the same list of packaged paths and adds backup tracking for files that may be modified locally.
The optional `%BACKUP%` section records the MD5 checksum of each tracked file as installed on the system:

```text
%FILES%
usr/
etc/
etc/example.conf
usr/share/
usr/share/example/
usr/share/example/example.txt

%BACKUP%
etc/example.conf d41d8cd98f00b204e9800998ecf8427e
```

The **ALPM-MTREE** file also equals that contained in the **alpm-package** file:

```text
#mtree
/set type=file uid=0 gid=0 mode=644
./.BUILDINFO time=1752836739.0 size=5271 sha256digest=9924a366a4ad02c31b121b22a2b285b2cae3a57873052169deb9d237936bef83
./.PKGINFO time=1752836739.0 size=297 sha256digest=0c8481c16dfc09ffdb0f518f827109795c1e07816ab7205ffffa6837b92fa4fb
./usr time=1752836739.0 mode=755 type=dir
./usr/share time=1752836739.0 mode=755 type=dir
./usr/share/example time=1752836739.0 mode=755 type=dir
./usr/share/example/example.txt time=1752836739.0 size=14 sha256digest=732c1c47a8296f4525307d28469d7ba1f3f5e4796fe55bc5625febc720a09d92
```

# SEE ALSO

**bash**(1), **git**(1), **pkgctl**(1), **systemd-nspawn**(1), **tar**(1), **libalpm**(3), **ALPM-MTREE**(5), **BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-db**(7), **alpm-db-desc**(7), **alpm-db-files**(5), **alpm-install-scriptlet**(5), **alpm-package**(7), **alpm-package-relation**(7), **alpm-package-source-checksum**(7), **alpm-repo**(7), **alpm-repo-db**(7), **alpm-repo-desc**(7), **alpm-repo-files**(5), **alpm-source-repo**(7), **alpm-split-package**(7), **makepkg**(8), **pacman**(8), **repo-add**(8)

# NOTES

1. **Arch Linux**
   
   <https://archlinux.org>
1. **reproducible builds**
   
   <https://reproducible-builds.org>
1. **supply chain attack**
   
   <https://en.wikipedia.org/wiki/Supply_chain_attack>
1. **OpenPGP signatures**
   
   <https://openpgp.dev/book/signing_data.html#detached-signatures>
1. **OpenPGP certificate**
   
   <https://openpgp.dev/book/certificates.html>
1. **dbscripts**
   
   <https://gitlab.archlinux.org/archlinux/dbscripts>
