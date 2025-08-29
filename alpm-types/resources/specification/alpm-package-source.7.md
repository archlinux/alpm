# NAME

package source - local or remote source data used for building an ALPM based package.

# DESCRIPTION

ALPM based packages may be built using zero or more local and/or remote sources.
Generally, package sources are architecture-independent, but may be specified in an architecture-specific way.

In **PKGBUILD** files a package source is defined by adding an entry to the **source** array.
Alternatively, an array named **source**, directly followed by an underscore character ("_"), directly followed by an **alpm-architecture** (all except `any`) - may be used to define a source for a specific architecture (e.g. `source_aarch64`).

In **SRCINFO** files a package source is defined by assigning a value to the **source** keyword.
Alternatively, an architecture specific keyword named **source**, directly followed by an underscore character ("_"), directly followed by an **alpm-architecture** (all except `any`) may be used (e.g. `source_aarch64`).

## Local

Local sources are defined using relative file paths (e.g. `my-file.txt`).

When not specifying a _host_ while using the **file URI scheme** it is possible to make use of files in absolute file paths on the current host (e.g. `file:///etc/passwd`).
However, this is strongly discouraged, because with this method source files used for packaging may change in arbitrary ways.
When packaging, it is instead advisable to rely on relative file paths of files from the same location as a **PKGBUILD**.
This way all local source files can be tracked in a Version Control System (VCS).

## Remote

Remote sources may be retrieved using various protocols, all of which are defined using valid **URL** [2] strings (e.g. `https://example.com/project-1.0.0.tar.gz`).
Aside from protocols for static remote sources (e.g. `https`), several VCS protocols such as `bzr`, `fossil`, `git`, `hg` and `svn` are understood and can be used to retrieve specific versions of remote sources.

By default, the name of the remote object defines the final local source name (e.g. `https://example.com/project-1.0.0.tar.gz` resolves to `project-1.0.0.tar.gz` and `git+https://example.org/repo#tag=1.0.0` to `repo/` - see **renaming** for details).

The VCS protocols expose differing optional functionalities for retrieving specific remote content.
This functionality is accessed using URL fragments in the source URL.

### bzr

Using bzr it is possible to rely on revision identifiers (see `bzr help revisionspec`) using the `revision` URL fragment, e.g.:

- `bzr+https://example.org/trunk#revision=123`

### fossil

Using fossil it is possible to rely on branch, commit and tag identifiers using the `branch`, `commit` and `tag` URL fragments, respectively, e.g.:

- `fossil+https://example.org/repo#branch=my-branch`
- `fossil+https://example.org/repo#commit=b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c`
- `fossil+https://example.org/repo#tag=1.0.0`

### git

Using git it is possible to rely on branch, commit and tag identifiers using the `branch`, `commit` and `tag` URL fragments, respectively, e.g.:

- `git+https://example.org/repo#branch=my-branch`
- `git+https://example.org/repo#commit=f1d2d2f924e986ac86fdf7b36c94bcdf32beec15`
- `git+https://example.org/repo#tag=1.0.0`

### hg

Using hg it is possible to rely on branch, revision and tag identifiers using the `branch`, `revision` and `tag` URL fragments, respectively, e.g.:

- `hg+https://example.org/repo#branch=my-branch`
- `hg+https://example.org/repo#revision=f1d2d2f924e986ac86fdf7b36c94bcdf32beec15`
- `hg+https://example.org/repo#tag=1.0.0`

### svn

Using svn it is possible to rely on the revision identifier using the `revision` URL fragment, e.g.:

- `svn+https://example.org/repo#revision=r123`

## Signature verification

**OpenPGP signature verification** [3] is supported when using certain types of package sources.
For package build software, that relies on a **PKGBUILD** file, to be able to verify a signature based on an **OpenPGP certificate**, there must be at least one entry in its **validpgpkeys** array.
Analogous, if a software relies on a **SRCINFO** file, it must have at least one **validpgpkeys** keyword assignment present for **signature verification** to be possible.
In both cases, **OpenPGP signature verification** [3] is attempted based on **OpenPGP certificates** [4] that match the **OpenPGP fingerprints** [5] exposed in **validpgpkeys**.

A pair of **local** or static **remote** sources, that define a file and an accompanying detached signature file (e.g. `example-1.0.0.tar.gz` with `example-1.0.0.tar.gz.sig`, or `example-1.0.0.tar.gz` with `example-1.0.0.tar.gz.asc`, or `example-1.0.0.tar.gz` with `example-1.0.0.tar.sign`) are an indication for the need of an **OpenPGP signature verification**. 

If OpenPGP signatures are available in a **git** based **remote** source, the (optional) need for **OpenPGP signature verification** [3] of git objects can be indicated using the `signed` **URL** query component (e.g. `git+https://example.org/repo?signed#tag=1.0.0` for verifying a specific git tag, or `git+https://example.org/repo?signed#commit=f1d2d2f924e986ac86fdf7b36c94bcdf32beec15` for verifying a specific commit).

If **OpenPGP signature verification** [2] is requested, the process that verifies the **package source** must fail, if

- one or more OpenPGP certificate matching a fingerprint in **validpgpkeys** is not available,
- the OpenPGP certificate used for the verification of the **package source** is revoked at signature creation time,
- the OpenPGP certificate used for the verification of the **package source** is expired at signature creation time,
- or the **package source** can not be verified with any of the OpenPGP certificates pinned by fingerprint in **validpgpkeys**.

## Renaming

In **PKGBUILD** and **SRCINFO** files, remote sources can be renamed.
Using a _target name_, directly followed by '::', directly followed by the _remote source name_ (e.g. `source-1.0.0.tar.gz::https://example.com/1.0.0.tar.gz` or `project::git+https://git.example.com/project.git`).

The renaming functionality differs between static protocols (e.g. `https`) and VCS protocols (e.g. `git`):
In the former case the _target name_ is a file and in the latter a directory (e.g. `project-1.0.0.tar.gz::https://example.org/project-v1.0.0.tar.gz` renames to the file `project-1.0.0.tar.gz` while `other-name::git+https://example.org/repo#tag=v1.0.0` renames to the directory `other-name/`).

## Extraction

By default, local and remote sources are automatically extracted by package build software such as **makepkg**, if (after **renaming**) the final source file name ending matches a known extension (e.g. `.tar.gz`).

In **PKGBUILD** files a final source file name from the **source** array can be added to the **noextract** array to indicate that the automatic extraction should be prevented for the given file.

In **SRCINFO** files a final source file name can be defined using the **noextract** keyword to indicate, that the automatic extraction should be prevented for the given file.

# EXAMPLES

## Local and static remote sources with renaming

```bash
pkgname=example
pkgver=0.1.0
pkgrel=1
pkgdesc="A package example"
arch=(x86_64)
url="https://example.org"
license=(GPL-3.0-or-later)
makedepends=(meson)
depends=(
  gcc-libs
  glibc
)
noextract=(custom-data.tar.gz)
source=(
  test.service
  custom-data.tar.gz{,.sig}
  $pkgname-$pkgver.tar.gz::https://download.example.org/$pkgname-v$pkgver.tar.gz
)
sha256sums=(
  b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
  7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730
  bf07a7fbb825fc0aae7bf4a1177b2b31fcf8a3feeaf7092761e18c859ee52a9c
  d18eca2e2e57e58a47e7dc15000d57f5180e7db9bb2a412ab2449637ab3ce3ff
)
validpgpkeys=(6d96270004515a0486bb7f76196a72b40c55a47f)

build() {
  meson setup --prefix /usr $pkgname-$pkgver build
  meson compile -C build
}

package(){
  meson install -C build --destdir "$pkgdir"
  install -vDm 644 test.service -t "$pkgname/usr/lib/systemd/system/"
  install -vDm 644 custom-data.tar.gz -t "$pkgname/usr/share/$pkgname/"
}
```

The above **PKGBUILD** example defines a **package source** setup, in which the **remote** source is renamed and a **local**, compressed source is not extracted but instead used as is.
Further, the **local** source `custom-data.tar.gz` is verified using the (assumed) detached signature file `custom-data.tar.gz.sig` using **OpenPGP signature verification** with a certificate that has the fingerprint `6d96270004515a0486bb7f76196a72b40c55a47f`.
The following **SRCINFO** file is generate from the **PKGBUILD**:

```ini
pkgbase = example
  pkgdesc = A package example
  pkgver = 0.1.0
  pkgrel = 1
  url = https://example.org
  arch = x86_64
  license = GPL-3.0-or-later
  makedepends = meson
  depends = gcc-libs
  depends = glibc
  noextract = custom-data.tar.gz
  source = test.service
  source = custom-data.tar.gz
  source = custom-data.tar.gz.sig
  source = example-0.1.0.tar.gz::https://download.example.org/example-v0.1.0.tar.gz
  validpgpkeys = 6d96270004515a0486bb7f76196a72b40c55a47f
  sha256sums = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
  sha256sums = 7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730
  sha256sums = bf07a7fbb825fc0aae7bf4a1177b2b31fcf8a3feeaf7092761e18c859ee52a9c
  sha256sums = d18eca2e2e57e58a47e7dc15000d57f5180e7db9bb2a412ab2449637ab3ce3ff

pkgname = example
```

## Local and VCS remote sources with renaming

```bash
pkgname=example-git
pkgver=0.1.0
pkgrel=1
pkgdesc="A package example"
arch=(x86_64)
url="https://example.org"
license=(GPL-3.0-or-later)
makedepends=(
  git
  meson
)
depends=(
  gcc-libs
  glibc
)
noextract=(custom-data.tar.gz)
source=(
  test.service
  custom-data.tar.gz{,.sig}
  $pkgname::git+https://git.example.org/repo?signed#tag=v$pkgver
)
sha256sums=(
  b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
  7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730
  bf07a7fbb825fc0aae7bf4a1177b2b31fcf8a3feeaf7092761e18c859ee52a9c
  1e717d3e52d72dde846f0028542d6ace456d7463fb7bc134ab9e812040758aad
)
validpgpkeys=(
  6d96270004515a0486bb7f76196a72b40c55a47f
  4cbd040533a2f43fc6691d773d510cda70f4126a
)

build() {
  meson setup --prefix /usr $pkgname build
  meson compile -C build
}

package(){
  meson install -C build --destdir "$pkgdir"
  install -vDm 644 test.service -t "$pkgname/usr/lib/systemd/system/"
  install -vDm 644 custom-data.tar.gz -t "$pkgname/usr/share/$pkgname/"
}
```

The above **PKGBUILD** example defines a **package source** setup, in which the **git** based **remote** source is renamed and a **local**, compressed source  is not extracted but instead used as is.
For the **local** source `custom-data.tar.gz` an **OpenPGP signature verification** is attempted using the (assumed) detached signature file `custom-data.tar.gz.sig` with a certificate that has the fingerprint `6d96270004515a0486bb7f76196a72b40c55a47f` and one that has the fingerprint `4cbd040533a2f43fc6691d773d510cda70f4126a`.
For the **git** based **remote** source an **OpenPGP signature verification** on the selected tag is attempted using either a certificate that has the fingerprint `6d96270004515a0486bb7f76196a72b40c55a47f` or one that has the fingerprint `4cbd040533a2f43fc6691d773d510cda70f4126a`.

The following **SRCINFO** file is generate from the **PKGBUILD**:

```ini
pkgbase = example-git
  pkgdesc = A package example
  pkgver = 0.1.0
  pkgrel = 1
  url = https://example.org
  arch = x86_64
  license = GPL-3.0-or-later
  makedepends = git
  makedepends = meson
  depends = gcc-libs
  depends = glibc
  noextract = custom-data.tar.gz
  source = test.service
  source = custom-data.tar.gz
  source = custom-data.tar.gz.sig
  source = example-git::git+https://git.example.org/repo?signed#tag=v0.1.0
  validpgpkeys = 6d96270004515a0486bb7f76196a72b40c55a47f
  validpgpkeys = 4cbd040533a2f43fc6691d773d510cda70f4126a
  sha256sums = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
  sha256sums = 7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730
  sha256sums = bf07a7fbb825fc0aae7bf4a1177b2b31fcf8a3feeaf7092761e18c859ee52a9c
  sha256sums = 1e717d3e52d72dde846f0028542d6ace456d7463fb7bc134ab9e812040758aad

pkgname = example-git
```

# SEE ALSO

**git**(1), **hg**(1), **svn**(1), **PKGBUILD**(5), **SRCINFO**(5), **alpm-architecture**(7), **makepkg**(8)

# NOTES

1. **file URI scheme**
   
   <https://en.wikipedia.org/wiki/File_URI_scheme>
1. **URL**
   
   <https://en.wikipedia.org/wiki/URL>
1. **OpenPGP signature verification**
   
   <https://openpgp.dev/book/verification.html>
1. **OpenPGP certificates**
   
   <https://openpgp.dev/book/certificates.html>
1. **OpenPGP fingerprints**
   
   <https://openpgp.dev/book/certificates.html#fingerprint>
