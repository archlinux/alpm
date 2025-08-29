# NAME

soname - representation and use of soname data in ALPM based packaging.

# DESCRIPTION

**Sonames**[1] are a mechanism to ensure binary compatibility between _shared objects_ and their consumers (i.e. other **ELF**[2] files).
More specifically, **soname**[1] data is encoded in the `SONAME` field of the `dynamic section` in _shared object_ files and usually denotes specific information on the version of the object file.
**ELF**[2] files may dynamically link against specific versions of _shared objects_ and declare this dependency with the help of the `NEEDED` field in their _dynamic section_.

Strings representing **soname** information can be used in **alpm-package-relations** of type **provision** and **run-time dependency** to allow for strict package dependency setups based on ABI compatibility.

The **alpm-soname** format exists in multiple versions.
This document describes version 1, which is a legacy version and has been removed with the release of pacman 6.1 on 2024-03-04.
For the latest specification, refer to **alpm-soname**.

# FORMAT

The representation of **soname** information depends on in which ALPM file format they occur in.

File formats, that represent static package source data (i.e. **PKGBUILD** and **SRCINFO**) are distinguished from those that represent dynamic package build environment data (i.e. **PKGINFO**).

The **SRCINFO** (and by extension also the **PKGBUILD**) file format is used to represent static data about package sources and should therefore only refer to the _basic form_ of **alpm-sonamev1**.
The _basic form_ is defined as the name of the _shared object_ file in which **soname** data is encoded (e.g. `libexample.so`).

The **PKGINFO** file format may contain dynamic, build environment specific data.
In these files **soname** data may be referred to using the _unversioned_ or the _explicit form_ (both contain information derived from the targeted _shared object_ file).

The _unversioned form_ is defined as the name of a _shared object_ file, directly followed by an '=' sign, directly followed by a _shared object_'s (unversioned) **soname** string, directly followed by a '-' sign, directly followed by the _ELF architecture format_ of the _shared object_ file (e.g. `libexample.so=libexample.so-64`).
Refer to **readelf** for further information on how to extract relevant fields such as `SONAME` or `NEEDED` from the _dynamic section_ of an **ELF**[2] file.

The _explicit form_ is defined as the name of a _shared object_ file, directly followed by an '=' sign, directly followed by a _version_ string, directly followed by a '-' sign, directly followed by the _ELF architecture format_ (e.g. `libexample.so=1-64`).
The _version_ string represents the `current` interface number of the **library interface version**[3] that is encoded as part of the **soname** by the developer of the _shared object_ (e.g. `1` in `libexample.so.1`).
Refer to **readelf** for further information on how to extract relevant fields such as `SONAME` or `NEEDED` from the _dynamic section_ of an **ELF**[2] file.

The _ELF architecture format_ is a number derived from the `Class` field of the _ELF header_ of an **ELF**[2] file.
For the 32-bit format it is `32`, for the 64-bit format it is `64`.
Refer to **readelf** for further information on how to extract the _ELF header_ from an **ELF**[2] file.

When using the name of a _shared object_ file, this always refers to the least specific file name.
If for example the following files exist on a system:

```text
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

Then the symlink `libexample.so` is the least specific file name, while `libexample.so.1.0.0` would refer to the actual **ELF**[2] file.

**Note**: While technically possible, it is strongly discouraged to use _unversioned_ or _explicit form_ in **PKGBUILD** or **SRCINFO**!
This is because it requires manual handling of dynamically changing **soname** data in the static package source file formats and those may be out of sync with the actual, explicit data derived from the build environment.

**Note**: Both _basic_ and _explicit forms_ of the **alpm-sonamev1** format overlap with the **alpm-package-name** and **composite comparison expressions** as defined in the **alpm-comparison** format, respectively.
Due to this ambiguity it is essential to use the **alpm-sonamev1** forms in the respective correct file formats or ideally switch to a later version of the **alpm-soname** format altogether.

# USAGE

A package can depend on a specific **soname** with the help of an **alpm-package-relation** of type **run-time dependency**, if another package provides this exact **soname** in their **alpm-package-relation** of type **provision**.

More specifically, a **soname** dependency of one package is based on the **soname** data of a _shared object_ file provided by one of its dependency packages.

A package build tool (e.g. **makepkg**) automatically derives **soname** information for **ELF**[2] files in the build environment based on the following rules:

- If the package that is built contains a _shared object_ file and the name of that _shared object_ file (i.e. the _basic form_) is present as an **alpm-package-relation** of type **provision** in the package's **PKGBUILD** file (e.g. `provides=(libexample.so)`), then relevant data from the `SONAME` field in the _dynamic section_ of the targeted **ELF**[2] file is extracted.

    - If the **soname** data contains a version, it is extracted together with the _ELF architecture format_ of the file to construct the _explicit form_.
      It is added instead of the _basic form_ to the **alpm-package-relation** of type **provision** in the **PKGINFO** file of the package (e.g. `provides = libexample.so=1-64`).
    - If the **soname** data does not contain a version, the entire **soname** together with the _ELF architecture format_ of the file is used to construct the _unversioned form_.
      It is added instead of the _basic form_ to the **alpm-package-relation** of type **provision** in the **PKGINFO** file of the package (e.g. `provides = libexample.so=libexample.so-64`).

- If the package that is built contains an **ELF**[2] file that dynamically links against a _shared object_ available in the build environment and the name of that _shared object_ file (i.e. the _basic form_) is present as an **alpm-package-relation** of type **run-time dependency** in the package's **PKGBUILD** file (e.g. `depends=(libexample.so)`), then relevant **soname** data from the `NEEDED` fields in the _dynamic section_ of the **ELF**[2] file is extracted for those specific _shared objects_.
    - All **soname** data that contains a version is extracted together with the _ELF architecture format_ of the **ELF**[2] file and is used to construct the _explicit form_.
      It is matched against the **alpm-package-relation** of type **provision** in the **PKGINFO** data of the packages available in the build environment.
      If one matches, the _explicit form_ instead of the _basic form_ is added to the **alpm-package-relation** of type **run-time dependency** in the **PKGINFO** file of the package that is being built (e.g. `depend = libexample.so=1-64`).
    - All **soname** data that does not contain a version is extracted in its entirety, together with the _ELF architecture format_ of the **ELF**[2] file and is used to construct the _unversioned form_.
      It is matched against the **alpm-package-relation** of type **provision** in the **PKGINFO** data of the packages available in the build environment.
      If one matches, the _unversioned form_ instead of the _basic form_ is added to the **alpm-package-relation** of type **run-time dependency** in the **PKGINFO** file of the package that is being built (e.g. `depend = libexample.so=1-64`).

# EXAMPLES

## Providing a soname

The following example **PKGBUILD** for a package named `example` is used to build and install an upstream project, which provides a shared object named `libexample.so`.

```bash
pkgname=example
pkgver=1.0.0
pkgrel=1
pkgdesc="An example library"
arch=(x86_64)
url="https://example.org/library.html"
license=(MIT)
depends=(glibc)
provides=(libexample.so)
source=("https://example.org/$pkgname-$pkgver.tar.gz")
sha256sums=(7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730)

build() {
  make -C $pkgname-$pkgver
}

package() {
  make DESTDIR="$pkgdir" install -C $pkgname-$pkgver
}
```

The following **SRCINFO** file can be generated from the above **PKGBUILD** file:

```ini
pkgbase = example
	pkgdesc = An example library
	pkgver = 1.0.0
	pkgrel = 1
	arch = x86_64
	url = https://example.org/library.html
	license = MIT
  depends = glibc
	provides = libexample.so
	source = https://example.org/example-1.0.0.tar.gz
	sha256sums = 7d865e959b2466918c9863afca942d0fb89d7c9ac0c99bafc3749504ded97730
pkgname = example
```

This example assumes that the project results in installing the following files to the filesystem:

```text
/usr/lib/libexample.so -> libexample.so.1
/usr/lib/libexample.so.1 -> libexample.so.1.0.0
/usr/lib/libexample.so.1.0.0
```

Here, the file `/usr/lib/libexample.so.1.0.0` encodes the **soname** `libexample.so.1`.

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
provides = libexample.so=1-64
depend = glibc
```

## Depending on a soname

The following **PKGBUILD** for a package named `application` is used to build an upstream project that depends on the `example` package from the previous example.
More specifically, the resulting package depends on the _shared object_ `libexample.so` which is provided by the `example` package.

In the **PKGBUILD** the **run-time dependency** on the `libexample.so` _shared object_ is added in the `package` function instead of in the global **run-time dependency** declaration, while the `example` package is declared as global **build dependency**.
This is because multiple **sonames** of the same name but of different _ELF architecture format_ may exist (consider e.g. the explicit forms `libexample.so=1-32` and `libexample.so=1-64`).
If `libexample.so` was exposed as global **run-time dependency**, a build tool may accidentally pull in the wrong package containing a `libexample.so` due to the existing structural overlap between **alpm-sonamev1** and **alpm-package-relation**!

```bash
pkgname=application
pkgver=1.0.0
pkgrel=1
pkgdesc="An example application"
arch=(x86_64)
url="https://example.org/application.html"
license=(MIT)
depends=(glibc)
makedepends=(example)
source=("https://example.org/$pkgname-$pkgver.tar.gz")
sha256sums=(b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c)

build() {
  make -C $pkgname-$pkgver
}

package() {
  depends+=(example libexample.so)
  make DESTDIR="$pkgdir" install -C $pkgname-$pkgver
}
```

The following **SRCINFO** file can be generated from the above **PKGBUILD** file:

```ini
pkgbase = application
    pkgdesc = An example application
    pkgver = 1.0.0
    pkgrel = 1
    arch = x86_64
    url = https://example.org/application.html
    license = MIT
    depends = glibc
    depends = example
    depends = libexample.so
    makedepends = example
    source = https://example.org/application-1.0.0.tar.gz
    sha256sums = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = application
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
depend = libexample.so=1-64
```

# SEE ALSO

**readelf**(1), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **makepkg.conf**(5), **alpm-comparison**(7), **alpm-package-name**(7), **alpm-package-relation**(7), **makepkg**(8)

# NOTES

1. **soname**
   
   <https://en.wikipedia.org/wiki/Soname>
1. **ELF**
   
   <https://en.wikipedia.org/wiki/Executable_and_Linkable_Format>
1. **library interface version**
   
   <https://www.gnu.org/software/libtool/manual/html_node/Versioning.html>
