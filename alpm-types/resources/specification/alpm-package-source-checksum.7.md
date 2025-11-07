# NAME

package source checksum - a checksum to verify the integrity of a package source used for building an ALPM based package.

# DESCRIPTION

ALPM based packages may be built using **package sources** and for each of them at least one valid **package source checksum** must exist to be able to verify the integrity of the source.

Analogous to **package sources**, **package source checksums** may be specified in a generic or an architecture-specific way.

The value of a **package source checksum** is either the output of a **hash function**[1], or the special string 'SKIP', which indicates that no checksum verification should be done for a given source.

The following **hash functions**[1] are supported:

- **MD5**[2]
- **SHA-1**[3]
- **SHA-224** (part of the **SHA-2** [4] family)
- **SHA-256** (part of the **SHA-2** [4] family)
- **SHA-384** (part of the **SHA-2** [4] family)
- **SHA-512** (part of the **SHA-2** [4] family)
- **BLAKE2**[5]
- **CRC-32/CKSUM** (**cksum**[6] variant of the 32-bit **cyclic redundancy check (CRC)** [7])

If several **package source checksums** exist for a **package source**, they must use distinct **hash functions** (e.g. **SHA-512** and **BLAKE2**).
The number of **package source checksums** in each **hash function** category must always match the number of available **package sources**.

In **PKGBUILD** files a **package source checksum** is defined by adding a value to one of the following arrays:

- md5sums (**hash function**: **MD5**)
- sha1sums (**hash function**: **SHA-1**)
- sha224sums (**hash function**: **SHA-224**)
- sha256sums (**hash function**: **SHA-256**)
- sha384sums (**hash function**: **SHA-384**)
- sha512sums (**hash function**: **SHA-512**)
- b2sums (**hash function**: **BLAKE2**)
- cksums (**hash function**: **CRC-32/CKSUM**)

Each array exclusively accepts output of the respective **hash function** or the special string 'SKIP' as value.

Alternatively, any of the above array names, directly followed by an underscore character ("_"), directly followed by an **alpm-architecture** (all except `any`) may be used to define a source checksum for a specific architecture (e.g. `b2sums_aarch64`).

In **SRCINFO** files a package source checksum is defined by assigning one of the following keywords a value:

- md5sums (**hash function**: **MD5**)
- sha1sums (**hash function**: **SHA-1**)
- sha224sums (**hash function**: **SHA-224**)
- sha256sums (**hash function**: **SHA-256**)
- sha384sums (**hash function**: **SHA-384**)
- sha512sums (**hash function**: **SHA-512**)
- b2sums (**hash function**: **BLAKE2**)
- cksums (**hash function**: **CRC-32/CKSUM**)

Each keyword assignment exclusively accepts output of the respective **hash function** or the special string 'SKIP' as value.

Alternatively, any of the above keywords, directly followed by an underscore character ("_"), directly followed by an **alpm-architecture** (all except `any`) may be used to define a source checksum for a specific architecture (e.g. `b2sums_aarch64`).

# EXAMPLES

## Remote source with checksums

The above **PKGBUILD** example defines a **package source** setup in which a remote source is verified using a **SHA-512** and a **BLAKE2** hash.
The checksum verification for the OpenPGP signature is skipped using the `SKIP` string.

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
source=($pkgname-$pkgver.tar.gz::https://download.example.org/$pkgname-v$pkgver.tar.gz{,.sig})
sha512sums=(
  0cf9180a764aba863a67b6d72f0918bc131c6772642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6
  'SKIP'
)
b2sums=(
  d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77
  'SKIP'
)
validpgpkeys=(988881ADC9FC3655077DC2D4D757D480B5EA0E11)

build() {
  meson setup --prefix /usr $pkgname-$pkgver build
  meson compile -C build
}

package(){
  meson install -C build --destdir "$pkgdir"
}
```

The **PKGBUILD** is represented by the following **SRCINFO**:

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
    source = example-0.1.0.tar.gz::https://download.example.org/example-v0.1.0.tar.gz
    sha512sums = 0cf9180a764aba863a67b6d72f0918bc131c6772642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6
    sha512sums = SKIP
    b2sums = d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77
    b2sums = SKIP

pkgname = example
```

## Remote source with checksums for several architectures

The below **PKGBUILD** example defines a **package source** setup in which two remote sources are verified using a **SHA-512** and a **BLAKE2** hash each.
One source is exclusively used on the **x86_64** and the other exclusively on the **aarch64** architecture.

```bash
pkgname=example
pkgver=0.1.0
pkgrel=1
pkgdesc="A package example"
arch=(
  aarch64
  x86_64
)
url="https://example.org"
license=(GPL-3.0-or-later)
makedepends=(meson)
depends=(
  gcc-libs
  glibc
)
source_aarch64=(
  $pkgname-$pkgver.tar.gz::https://download.example.org/$pkgname-aarch64-v$pkgver.tar.gz
)
source_x86_64=(
  $pkgname-$pkgver.tar.gz::https://download.example.org/$pkgname-x86_64-v$pkgver.tar.gz
)
sha512sums_aarch64=(
  cc06808cbbee0510331aa97974132e8dc296aeb795be229d064bae784b0a87a5cf4281d82e8c99271b75db2148f08a026c1a60ed9cabdb8cac6d24242dac4063
)
sha512sums_x86_64=(
  0cf9180a764aba863a67b6d72f0918bc131c6772642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6
)
b2sums_aarch64=(
  a69cc58858cb37cf8da7f83f55c23f171ee3c59be76ad7edcf01dec36fd9d0104bb433cd863ee3f0b6a10a336cf2400688c57fd99392dc01c4585d8725547e8c
)
b2sums_x86_64=(
  d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77
)

build() {
  meson setup --prefix /usr $pkgname-$pkgver build
  meson compile -C build
}

package(){
  meson install -C build --destdir "$pkgdir"
}
```

The following **SRCINFO** is generated from the **PKGBUILD**:

```ini
pkgbase = example
    pkgdesc = A package example
    pkgver = 0.1.0
    pkgrel = 1
    url = https://example.org
    arch = aarch64
    arch = x86_64
    license = GPL-3.0-or-later
    makedepends = meson
    depends = gcc-libs
    depends = glibc
    source_aarch64 = example-0.1.0.tar.gz::https://download.example.org/example-aarch64-v0.1.0.tar.gz
    sha512sums_aarch64 = cc06808cbbee0510331aa97974132e8dc296aeb795be229d064bae784b0a87a5cf4281d82e8c99271b75db2148f08a026c1a60ed9cabdb8cac6d24242dac4063
    b2sums_aarch64 = a69cc58858cb37cf8da7f83f55c23f171ee3c59be76ad7edcf01dec36fd9d0104bb433cd863ee3f0b6a10a336cf2400688c57fd99392dc01c4585d8725547e8c
    source_x86_64 = example-0.1.0.tar.gz::https://download.example.org/example-x86_64-v0.1.0.tar.gz
    sha512sums_x86_64 = 0cf9180a764aba863a67b6d72f0918bc131c6772642cb2dce5a34f0a702f9470ddc2bf125c12198b1995c233c34b4afd346c54a2334c350a948a51b6e8b4e6b6
    b2sums_x86_64 = d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77

pkgname = example
```

# SEE ALSO

**PKGBUILD**(5), **SRCINFO**(5), **alpm-architecture**(7), **alpm-package-source**(7), **makepkg**(8)

# NOTES

1. **hash function**
   
   <https://en.wikipedia.org/wiki/Hash_function>
1. **MD5**
   
   <https://en.wikipedia.org/wiki/MD5>
1. **SHA-1**
   
   <https://en.wikipedia.org/wiki/SHA-1>
1. **SHA-2**
   
   <https://en.wikipedia.org/wiki/SHA-2>
1. **BLAKE2**
   
   <https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE2>
1. **cksum**
   
   <https://en.wikipedia.org/wiki/Cksum>
1. **CRC**
   
   <https://en.wikipedia.org/wiki/Cyclic_redundancy_check>
