error-create-zstd-encoder =
  Error creating a Zstandard encoder while { $context } with { $compression_settings }:
  { $source }

error-create-zstd-decoder =
  Error creating a Zstandard decoder:
  { $source }

error-finish-encoder =
  Error while finishing { $compression_type } compression encoder:
  { $source }

error-get-parallelism =
  Error while trying to get available parallelism:
  { $source }

error-integer-conversion =
  Error while trying to convert an integer:
  { $source }

error-io-read =
  I/O read error while { $context }:
  { $source }

error-io-write =
  I/O write error while { $context }:
  { $source }

error-invalid-compression-level =
  Invalid compression level { $level } (must be in the range { $min } - { $max }).

error-unknown-compression-extension =
  Unknown compression algorithm file extension:
  { $source }

error-unsupported-compression =
  Unsupported compression algorithm: { $value }
