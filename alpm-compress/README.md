# alpm-compress

A small library that provides streaming compression and decompression for multiple algorithms used in the ALPM ecosystem:

- bzip2 (`.bz2`)
- gzip (`.gz`)
- xz (`.xz`)
- zstd (`.zst`)

It offers a unified encoder/decoder API with configurable compression levels and, for zstd, optional multithreading.

In addition, it provides utilities to create and read tar archives:

- uncompressed (`.tar`)
- bzip2 (`.tar.bz2`)
- gzip (`.tar.gz`)
- xz (`.tar.xz`)
- zstd (`.tar.zst`)

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_compress/> for development version of the crate
- <https://docs.rs/alpm-compress/latest/alpm_compress/> for released versions of the crate

## Examples

### Compressing

```rust
use std::{fs::File, io::Write};

use alpm_compress::compression::{
    CompressionEncoder, 
    CompressionSettings, 
    ZstdThreads, 
    ZstdCompressionLevel
};
use tempfile::tempfile;

# fn main() -> testresult::TestResult {
// Create an encoder that writes zstd-compressed data to a file.
let file = tempfile()?;
let settings = CompressionSettings::Zstd {
    compression_level: ZstdCompressionLevel::default(),
    threads: ZstdThreads::default(),
};

let mut encoder = CompressionEncoder::new(file, &settings)?;
encoder.write_all(b"alpm-compress makes compression easy")?;
let _file = encoder.finish()?; // retrieve the inner File when done
# Ok(())
# }
```

Compression settings default to zstd compression, but you can select other algorithms and levels.
Since compression is optional via `None` variant, this library provides unified API to read and write both 
compressed and uncompressed files.

```rust
use alpm_compress::compression::{
    CompressionSettings,
    Bzip2CompressionLevel, 
    GzipCompressionLevel, 
    XzCompressionLevel, 
    ZstdCompressionLevel, 
    ZstdThreads,
};

# fn main() {
// Bzip compression
let bzip2 = CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() };
// Gzip compression
let gzip = CompressionSettings::Gzip { compression_level: GzipCompressionLevel::default() };
// Xz compression
let xz = CompressionSettings::Xz { compression_level: XzCompressionLevel::default() };
// Zstd compression
let zstd = CompressionSettings::Zstd { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::default() };
// No compression
let no_compression = CompressionSettings::None;
# }
```

### Decompressing

```rust
use std::{fs::File, io::Read};

use alpm_compress::decompression::{DecompressionSettings, CompressionDecoder};
use tempfile::tempfile;

# fn main() -> testresult::TestResult {
// Create a temporary file.
let mut file = tempfile()?;

// [..] Add zstd compressed content to the temporary file.
# // Create an encoder that writes zstd-compressed data to a file.
# use alpm_compress::compression::{CompressionEncoder, CompressionSettings, ZstdThreads, ZstdCompressionLevel};
# use std::io::{Write, Seek};
# 
# let settings = CompressionSettings::Zstd {
#     compression_level: ZstdCompressionLevel::default(),
#     threads: ZstdThreads::default(),
# };
# let mut encoder = CompressionEncoder::new(file.try_clone()?, &settings)?;
# encoder.write_all(b"alpm-compress makes compression easy")?;
# encoder.flush()?;
# encoder.finish()?;
# file.rewind()?;

// Decompress a zstd-compressed file
let mut decoder = CompressionDecoder::new(file, DecompressionSettings::Zstd)?;

let mut buf = Vec::new();
decoder.read_to_end(&mut buf)?;
assert_eq!(buf, b"alpm-compress makes compression easy");
# Ok(())
# }
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
