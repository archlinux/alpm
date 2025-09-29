//! Compression handling.

mod encoder;
pub use encoder::CompressionEncoder;

mod level;
pub use level::{
    Bzip2CompressionLevel,
    GzipCompressionLevel,
    XzCompressionLevel,
    ZstdCompressionLevel,
};

mod settings;
pub use settings::{CompressionSettings, ZstdThreads};
