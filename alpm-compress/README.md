# alpm-compress

A small library that provides streaming compression and decompression for multiple algorithms used in the ALPM ecosystem: bzip2, gzip, xz, and zstd.

It offers a unified encoder/decoder API with configurable compression levels and, for zstd, optional multithreading.

## Documentation

- <https://alpm.archlinux.page/rustdoc/alpm_compress/> for development version of the crate
- <https://docs.rs/alpm-compress/latest/alpm_compress/> for released versions of the crate

## Examples

### Compress

```rust no_run
use std::{fs::File, io::Write};
use alpm_compress::compression::{CompressionEncoder, CompressionSettings, ZstdThreads, level::ZstdCompressionLevel};

fn main() -> Result<(), alpm_compress::Error> {
    // Create an encoder that writes zstd-compressed data to a file.
    let file = File::create("example.txt.zst").expect("failed to create a file");
    let settings = CompressionSettings::Zstd {
        compression_level: ZstdCompressionLevel::default(),
        threads: ZstdThreads::default(),
    };

    let mut encoder = CompressionEncoder::new(file, &settings)?;
    encoder.write_all(b"alpm-compress makes compression easy").expect("failed to write data");
    let _file = encoder.finish()?; // retrieve the inner File when done
    Ok(())
}
```

You can select other algorithms and levels:

```rust ignore
use alpm_compress::compression::{
    CompressionSettings,
    level::{Bzip2CompressionLevel, GzipCompressionLevel, XzCompressionLevel, ZstdCompressionLevel}, 
    ZstdThreads,
};

let bzip2 = CompressionSettings::Bzip2 { compression_level: Bzip2CompressionLevel::default() };
let gzip  = CompressionSettings::Gzip  { compression_level: GzipCompressionLevel::default() };
let xz    = CompressionSettings::Xz    { compression_level: XzCompressionLevel::default() };
let zstd  = CompressionSettings::Zstd  { compression_level: ZstdCompressionLevel::default(), threads: ZstdThreads::default() };
```

### Decompress

```rust no_run
use std::{fs::File, io::Read};
use alpm_compress::decompression::{CompressionAlgorithm, CompressionDecoder};

fn main() -> Result<(), alpm_compress::Error> {
    // Decompress a zstd-compressed file
    let file = File::open("example.txt.zst").expect("failed to open a file");
    let mut decoder = CompressionDecoder::new(file, CompressionAlgorithm::Zstd)?;

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).expect("failed to read data");
    assert!(buf.starts_with(b"alpm-compress"));
    Ok(())
}
```

## Contributing

Please refer to the [contribution guidelines] to learn how to contribute to this project.

## License

This project can be used under the terms of the [Apache-2.0] or [MIT].
Contributions to this project, unless noted otherwise, are automatically licensed under the terms of both of those licenses.

[contribution guidelines]: ../CONTRIBUTING.md
[Apache-2.0]: ../LICENSES/Apache-2.0.txt
[MIT]: ../LICENSES/MIT.txt
