# NAME

soname - representation and use of soname data in ALPM-based packaging.

# DESCRIPTION

**Sonames** are a mechanism to ensure binary compatibility between shared objects and their consumers (e.g. other shared objects or executables).
More specifically, the **soname**[1] field is embedded in shared object files and usually denotes specific information on the version of the object file.

Strings representing **soname** information can be used in **alpm-package-relations** to allow for strict package dependency setups based on ABI compatibility.

The document describes version 1, which is a legacy version and has been removed in pacman 6.1.
For the latest specification, refer to **alpm-soname**.

# FORMAT

The representation of **soname** information depends on in which metadata file they occur in.

File formats, that represent static data (i.e. **PKGBUILD** and **SRCINFO**) should usually only refer to the name of the _library_ in which a **soname** is encoded (e.g. `libexample.so`).

File formats, that are able to represent dynamic data, such as that of a package's build environment (i.e. **PKGINFO**) are able to refer to **sonames** using the following format:

The name of the _library_, directly followed by an '=' sign, directly followed by a _version_ string, directly followed by a '-' sign, directly followed by the _architecture-bit_ (e.g. `libexample.so=1-64`).

Note, that in file formats that represent static data, it is also possible to use this dynamic format, but only by manually hardcoding the value, which is not desirable.

# USAGE

This section describes how to expose and use **soname** information in ALPM-based packaging.
To allow packages to rely on **soname** data in the context of an **alpm-package-relation**, it is required to expose it as an **alpm-package-relation** first.
Here, file formats that represent static package source data (i.e. **PKGBUILD** and **SRCINFO**) are distinguished from those that represent dynamic package build environment data (i.e. **PKGINFO**).

## Exposing Sonames

- Static **soname** data can be exposed by adding the static form (e.g. `libexample.so`) to the **provides** array in a **PKGBUILD** or by defining it using the **provides** keyword definition in a **SRCINFO**.
- Dynamic **soname** data can be exposed by manually adding the dynamic form (e.g. `libexample.so=1-64`) to the **provides** array in a **PKGBUILD** or by defining it using the **provides** keyword definition in a **SRCINFO**.
  This is strongly discouraged, as it requires manual handling of soname data in the package source formats.
- Dynamic **soname** data can be exposed by adding the dynamic form (e.g. `libexample.so=1-64`) using the **provides** keyword definition in a **PKGINFO**.
  This is usually added automatically by package build software (e.g. **makepkg**) based on the specific shared object file available in the build environment and the presence of the static form in **PKGBUILD** or **SRCINFO** as mentioned above.

## Using Sonames

- Static **soname** data can be relied upon by adding the static form (e.g. `libexample.so`) to the **depends** array in a **PKGBUILD** or by defining it using the **depends** keyword definition in a **SRCINFO**.
- Dynamic **soname** data can be relied upon by manually adding the dynamic form (e.g. `libexample.so=1-64`) to the **depends** array in a **PKGBUILD** or by defining it using the **depends** keyword definition in a **SRCINFO**.
  This is strongly discouraged, as it requires manual handling of **soname** data in the package source formats.
- Dynamic **soname** data can be relied upon by adding the dynamic form (e.g. `libexample.so=1-64`) using the **depend** keyword definition in a **PKGINFO**.
  This is usually added automatically by package build software (e.g. **makepkg**) based on the specific shared object file available in the build environment and the presence of the static form in **PKGBUILD** or **SRCINFO** as mentioned above.

# EXAMPLES

## Depending on a soname

In `PKGBUILD`:

```
pkgname=example
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package that depends on libexample.so"
arch=('x86_64')
url="https://example.com"
license=('MIT')
source=("https://example.com/example-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/example-$pkgver"
  make
}

package() {
  depends+=('libexample.so')
  cd "$srcdir/example-$pkgver"
  make DESTDIR="$pkgdir" install
}
```

In `.PKGINFO`:

```
pkgname = example
pkgver = 1.0.0-1
pkgdesc = An example package that depends on libexample.so
url = https://example.com
builddate = 1729181726
packager = Your Name <your@email.com>
size = 181849963
arch = x86_64
license = MIT
depend = libexample.so=1-64
```

In `.SRCINFO`:

```
pkgbase = example
    pkgdesc = An example package that depends on libexample.so
    pkgver = 1.0.0
    pkgrel = 1
    arch = x86_64
    url = https://example.com
    license = MIT
    depends = libexample.so=1-64
    source = https://example.com/example-1.0.0.tar.gz
    sha256sums = SKIP
```

## Providing a soname

In `PKGBUILD`:

```
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package that provides libexample.so"
arch=('x86_64')
url="https://example.com"
license=('MIT')
source=("https://example.com/example-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/example-$pkgver"
  make
}

package() {
  provides+=('libexample.so')
  cd "$srcdir/example-$pkgver"
  install -Dm755 libexample.so.3.0 "$pkgdir/usr/lib/libexample.so.3.0-64"
}
```

In `.PKGINFO`:

```
pkgname = example
pkgver = 1.0.0-1
pkgdesc = An example package that provides libexample.so
url = https://example.com
builddate = 1729181726
packager = Your Name <your@email.com>
size = 181849963
arch = x86_64
license = MIT
provides = libexample.so
```

In `.SRCINFO`:

```
pkgbase = example
	pkgdesc = An example package that provides libexample.so
	pkgver = 1.0.0
	pkgrel = 1
	arch = x86_64
	url = https://example.com
	license = MIT
	provides = libexample.so
	source = https://example.com/example-1.0.0.tar.gz
	sha256sums = SKIP
```

# NOTES

1. **soname**

   https://en.wikipedia.org/wiki/Soname

# SEE ALSO

PKGBUILD(5), PKGINFO(5), SRCINFO(5), alpm-package-relation(7)
