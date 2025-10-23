error-create-zstd-encoder =
  Error creating a Zstandard encoder while { $context } with { $compression_settings }:
  { $source }

error-create-zstd-decoder =
  Error creating a Zstandard decoder:
  { $source }

error-create-zstd-encoder-init = initializing

error-create-zstd-encoder-set-checksum = setting checksums to be added

error-create-zstd-encoder-set-threads = setting multithreading

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

error-io-write-archive = writing the archive

error-io-read-archive-entries = reading archive entries

error-io-open-archive = opening archive for reading

error-io-read-archive-entry-content = reading archive entry content

error-io-read-archive-entry-mode = retrieving permissions of archive entry

error-io-read-archive-entry = reading archive entry

error-io-read-archive-entry-path = retrieving path of archive entry

error-invalid-compression-level =
  Invalid compression level { $level } (must be in the range { $min } - { $max }).

error-unknown-compression-extension =
  Unknown compression algorithm file extension:
  { $source }

error-unsupported-compression =
  Unsupported compression algorithm: { $value }
