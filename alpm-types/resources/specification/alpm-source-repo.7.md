# NAME

source-repo - a repository containing sources for building one or more **A**rch **L**inux **P**ackage **M**anagement (ALPM) based packages.

# DESCRIPTION

ALPM based packages (see **alpm-package**) are created from package sources using package build software, such as **makepkg**.
For Arch Linux specific package build software refer to **devtools** and **pkgctl**.

## Contents

Package sources are represented by a single **PKGBUILD** per **alpm-source-repo**, which itself may define and require further files, such as an **alpm-install-scriptlet**, arbitrary custom local source files for the package build process and files for tooling (e.g. configuration files, etc.).
The **PKGBUILD** script is accompanied by a **SRCINFO** file which represents a parseable data format exposing relevant metadata defined by the **PKGBUILD**.

### Required

- **PKGBUILD**: The package build script.

### Optional

- **.nvchecker.toml**: A minimal configuration file for **nvchecker** (supported by **pkgctl**) to allow detection of new upstream versions.
- **LICENSES/**: A directory containing license files used by **reuse** (see **RFC 0040**[1] and **RFC 0052**[2]).
- **REUSE.toml**: A configuration file for **reuse** which covers the license information of all files in the **alpm-source-repo** (see **RFC 0040**[1] and **RFC 0052**[2]).
- **SRCINFO**: The data representation of the package build script metadata.
- **alpm-install-scriptlet**: An installation scriptlet that is added to a resulting **alpm-package** (needs to be specified in the **PKGBUILD**).
- **keys/pgp/**: A directory containing **ASCII-armored**[3] **OpenPGP certificates**[4] that represent verifiers for **OpenPGP signatures**[5] created by upstreams for their release artifacts.

## Version control

It is advisable to keep the contents of an **alpm-source-repo** in a version control system such as **git**.
In doing so the relationship between a **PKGBUILD** and a resulting **alpm-package** file can be established, which is important for **reproducible builds**[6].

Upstream sources and build artifacts (e.g. build logs, or package files) should not be included in the **git** repository.
They can be ignored using a **gitignore** file.
Alternatively **makepkg.conf** can be used to instruct the **makepkg** package build tool to store artifacts elsewhere.

## Best practices

Although a basic setup technically only requires a **PKGBUILD** file, further components are considered best practice.

- **SRCINFO**: Creating this file and keeping it in sync with the **PKGBUILD** file allows consumers of the **alpm-source-repo** to extract metadata of the **PKGBUILD** without requiring **bash**.
- **LICENSES/** and **REUSE.toml**: A **reuse** setup ensures that licensing is clearly defined for all files in the **alpm-source-repo** which enables others to use it and adapt it.
- **keys/pgp/**: If the **PKGBUILD** defines an **alpm-package-source** with **OpenPGP signature verification**[7], storing current versions of relevant **ASCII-armored**[3] **OpenPGP certificates**[4] allows users of the **alpm-source-repo** to authenticate the upstream artifacts using **OpenPGP signatures**[5].
- **.nvchecker.toml**: Using **nvchecker** (e.g. through **pkgctl**) enables users to check for new releases of an upstream project.

# EXAMPLES

The following example illustrates a basic **alpm-source-repo**:

```text
.
└── PKGBUILD
```

The following example illustrates a more complete **alpm-source-repo** with best practices applied:

```text
.
├── .SRCINFO
├── .nvchecker.toml
├── LICENSES
│   └── 0BSD.txt
├── PKGBUILD
├── REUSE.toml
├── example.install
└── keys
    └── pgp
        └── F1D2D2F924E986AC86FDF7B36C94BCDF32BEEC15.asc
```

# SEE ALSO

**bash**(1), **git**(1), **nvchecker**(1), **pkgctl**(1), **reuse**(1), **PKGBUILD**(5), **SRCINFO**(5), **alpm-install-scriptlet**(5), **gitignore**(5), **makepkg.conf**(5), **alpm-package**(7), **alpm-package-source**(7), **devtools**(7), **makepkg**(8), 

# NOTES

1. **RFC 0040**

   https://rfc.archlinux.page/0040-license-package-sources/

1. **RFC 0052**

   https://rfc.archlinux.page/0052-reuse/

1. **ASCII armored**

   https://openpgp.dev/book/armor.html

1. **OpenPGP certificates**

   https://openpgp.dev/book/certificates.html

1. **OpenPGP signatures**

   https://openpgp.dev/book/signatures.html

1. **reproducible builds**

   https://reproducible-builds.org/

1. **OpenPGP signature verification**

   https://openpgp.dev/book/verification.html
