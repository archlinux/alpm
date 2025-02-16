# NAME

soname - representation and use of soname data in ALPM-based packaging.

# DESCRIPTION

**Sonames** are a mechanism to ensure binary compatibility between shared objects and their consumers (e.g. other shared objects or executables).
More specifically, the **soname**[1] field is embedded in shared object files and usually denotes specific information on the version of the object file.

Strings containing **soname** information can be used in **alpm-package-relations** to allow for strict package dependency setups based on ABI compatibility.

The **alpm-soname** format exists in multiple versions.
The information in this document is for version 2 (aka. "autodeps"), which is the current version, supersedes **alpm-sonamev1** and has been introduced with the release of pacman 6.1.0 on 2024-03-04.

# FORMAT

Deriving **soname** data of shared object files reproducibly and reliably can only be achieved using dedicated build environments.

The **PKGINFO** file format may contain dynamic, build environment specific data.
In these files **sonames** may be referred to using the following format, which is derived from **soname** information encoded in a given _shared object_:
A _prefix_ string, directly followed by a ':', directly followed by a **soname** (e.g. `lib:libexample.so.1`).

Here, the _prefix_ string `lib` denotes the arbitrarily defined use name for a lookup directory (e.g. `/usr/lib`).

The **soname** represents data found in the _dynamic section_ of a _shared object_ file (see **objdump** for details).
Depending on whether the **soname** data is used in an **alpm-package-relation** of type **provision** or of type **run-time dependency** this refers to the `SONAME` or the `NEEDED` field in the _dynamic section_, respectively.

**Note**: As _prefixes_ are assigned arbitrarily by the build tool (e.g. **makepkg**), it is not possible to provide a general recommendation or overview of them in this specification (see **LIB_DIRS** in **makepkg.conf** for how _prefix_ assignments work in **makepkg**).

# USAGE

A package can depend on a specific **soname** with the help of an **alpm-package-relation** of type **run-time dependency**, if another package provides this exact **soname** in their **alpm-package-relation** of type **provision**.

More specifically, a **soname** dependency of one package is based on the **soname** data of a _shared object_ file provided by one of its dependency packages.

A package build tool (e.g. **makepkg**) automatically derives **soname** information for _shared objects_ files and executables in the build environment.
For implementers it is strongly suggested to make this behavior configurable, as **soname** detection may fail or relying on **soname** data may not be advisable in certain conditions.
See **OPTIONS** of **makepkg.conf** for details on the `autodeps` feature and **LIB_DIRS** of **makepkg.conf** for details on _prefix_ and _lookup directory_ definitions for **makepkg**.

- If the package that is built provides a _shared object_ in one of the configured _lookup directories_, the value of the `SONAME` field in the _dynamic section_ of the **ELF**[2] file is extracted.
  Together with the _prefix_ assigned to the given _lookup directory_ this data is added as a **provision** to the **PKGINFO** data for the package (e.g. `provides = lib:libexample.so.1`, if `libexample.so` exists in `/usr/lib` and the _prefix_ for `/usr/lib` is `lib`).
- If the package that is built produces a _shared object_ or application that dynamically links against another _shared object_ available in the build environment, the value of the `NEEDED` field in the _dynamic section_ of the **ELF**[2] file is extracted.
  Together with the _prefix_ assigned to the given _lookup directory_ of the _shared object_ file that provides the needed **soname**, this data is added as a **run-time dependency** to the **PKGINFO** data for the package (e.g. `depend = lib:libexample.so.1`, if `libexample.so` exists in `/usr/lib` and the _prefix_ for `/usr/lib` is `lib`).
  However, this only applies if the package containing the targeted _shared object_ file already declares the necessary **soname** data in an **alpm-package-relation** of type **provision** (i.e. `provides = lib:libexample.so.1`).

# EXAMPLES

The following examples demonstrate how to expose and use **soname** information in ALPM-based packaging.

The examples assume building with **makepkg** with the _autodeps_ feature enabled.
Further, the **LIB_DIRS** array in **makepkg.conf** is expected to _prefix_ the _lookup directory_ `/usr/lib` with `lib`.

## Providing a soname

The following example **PKGBUILD** for a package named `example` is used to build and install an upstream project that contains a shared object.

```bash
pkgname=example
pkgver=1.0.0
pkgrel=1
pkgdesc="An example library"
arch=(x86_64)
url="https://example.org/library.html"
license=(MIT)
depends=(glibc)
source=("https://example.org/$pkgname-$pkgver.tar.gz")
sha256sums=(7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730)

build() {
  make -C $pkgname-$pkgver
}

package() {
  make DESTDIR="$pkgdir" install -C $pkgname-$pkgver
}
```

This example assumes that the project results in installing the following files to the filesystem:

```
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

Here, the file `/usr/lib/libexample.so.1.0.0` encodes the **soname** `libexample.so.1`.
Further, this examples assumes that the _directory_ `/usr/lib` is assigned to the _prefix_ `lib`.

After building from source, the resulting package file for `example` contains the following **PKGINFO** file:

```ini
pkgname = example
pkgver = 1.0.0-1
pkgdesc = An example library
url = https://example.org/library.html
builddate = 1729181726
packager = Your Name <your.name@example.org>
size = 181849963
arch = x86_64
license = MIT
provides = lib:libexample.so.1
depend = glibc
```

## Depending on a soname

The following **PKGBUILD** for a package named `application` is used to build an upstream project that depends on the `example` package from the previous example.
More specifically, the resulting package depends on the shared object `libexample.so` which is provided by the `example` package.

```bash
pkgname=application
pkgver=1.0.0
pkgrel=1
pkgdesc="An example application"
arch=(x86_64)
url="https://example.org/application.html"
license=(MIT)
depends=(glibc)
depends=(example)
source=("https://example.org/$pkgname-$pkgver.tar.gz")
sha256sums=(b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c)

build() {
  make -C $pkgname-$pkgver
}

package() {
  make DESTDIR="$pkgdir" install -C $pkgname-$pkgver
}
```

After building from source, the resulting package file for `application` contains the following **PKGINFO** file:

```ini
pkgname = application
pkgver = 1.0.0-1
pkgdesc = An example application
url = https://example.org/application.html
builddate = 1729181726
packager = Your Name <your@email.com>
size = 181849963
arch = x86_64
license = MIT
depend = glibc
depend = example
depend = lib:libexample.so.1
```

# SEE ALSO

objdump(1), makepkg.conf(5), PKGBUILD(5), PKGINFO(5), alpm-sonamev1(7), alpm-package-relation(7), makepkg(8), pacman(8)

# NOTES

1. **soname**

   https://en.wikipedia.org/wiki/Soname

2. **ELF**

   https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
